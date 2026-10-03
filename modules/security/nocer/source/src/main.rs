// NOCER — NixOS runtime certificate manager
// TUI + CLI. Runtime sertifika aç/kapa, rebuild yok, sudo yok.

use std::env;
use std::fs;
use std::io;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use clap::{Parser, Subcommand};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
    Terminal,
};
use serde::{Deserialize, Serialize};

// ───────────────────────── Yollar ─────────────────────────

fn state_dir() -> PathBuf {
    env::var("NOCER_STATE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/var/lib/nocer"))
}
fn certs_dir() -> PathBuf      { state_dir().join("certs") }
fn enabled_dir() -> PathBuf    { state_dir().join("enabled") }
fn index_file() -> PathBuf     { state_dir().join("index.json") }
fn firefox_policy_file() -> PathBuf { state_dir().join("firefox-policies.json") }
fn bundle_path() -> PathBuf {
    env::var("NOCER_BUNDLE_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/run/nocer-ca-bundle.crt"))
}

// ───────────────────────── Veri ─────────────────────────

#[derive(Serialize, Deserialize, Clone)]
struct CertEntry {
    id: u32,
    name: String,
    file: String,
    enabled: bool,
}

#[derive(Serialize, Deserialize)]
struct Index {
    next_id: u32,
    certs: Vec<CertEntry>,
}

impl Default for Index {
    fn default() -> Self {
        Self { next_id: 1, certs: Vec::new() }
    }
}

impl Index {
    fn load() -> io::Result<Self> {
        let p = index_file();
        if !p.exists() { return Ok(Self::default()); }
        let raw = fs::read_to_string(&p)?;
        Ok(serde_json::from_str(&raw).unwrap_or_default())
    }

    fn save(&self) -> io::Result<()> {
        let raw = serde_json::to_string_pretty(self)?;
        fs::write(index_file(), raw)
    }

    fn find(&self, name: &str) -> Option<&CertEntry> {
        self.certs.iter().find(|c| c.name == name)
    }

    fn position(&self, name: &str) -> Option<usize> {
        self.certs.iter().position(|c| c.name == name)
    }
}

// ───────────────────────── Kurulum ─────────────────────────

fn ensure_dirs() -> io::Result<()> {
    fs::create_dir_all(state_dir())?;
    fs::create_dir_all(certs_dir())?;
    fs::create_dir_all(enabled_dir())?;
    if !index_file().exists() {
        Index::default().save()?;
    }
    Ok(())
}

/// Firefox policies.json'ı aktif sertifikalara göre yeniden yazar.
fn write_firefox_policy(idx: &Index) -> io::Result<()> {
    let install: Vec<String> = idx.certs.iter()
        .filter(|c| c.enabled)
        .map(|c| certs_dir().join(&c.file).to_string_lossy().into_owned())
        .collect();

    let policy = serde_json::json!({
        "policies": {
            "Certificates": {
                "ImportEnterpriseRoots": true,
                "Install": install
            }
        }
    });

    fs::write(firefox_policy_file(), serde_json::to_string_pretty(&policy)?)?;
    Ok(())
}

/// systemd servisi varsa onu tetikle, yoksa/başarısızsa elle birleştir.
/// Hiçbir koşulda stdout/stderr'e yazmaz — TUI bozulmasın.
fn trigger_rebuild() {
    let systemd_ok = if Path::new("/run/systemd/system").exists() {
        Command::new("systemctl")
            .args(["start", "--no-block", "nocer-rebuild.service"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    } else {
        false
    };

    if !systemd_ok {
        let out = bundle_path();
        let mut buf = Vec::new();
        if let Ok(system) = fs::read("/etc/ssl/certs/ca-certificates.crt") {
            buf.extend_from_slice(&system);
        }
        if let Ok(entries) = fs::read_dir(enabled_dir()) {
            for e in entries.flatten() {
                if let Ok(data) = fs::read(e.path()) {
                    buf.push(b'\n');
                    buf.extend_from_slice(&data);
                }
            }
        }
        let _ = fs::write(&out, buf);
    }
}

// ───────────────────────── fzf ─────────────────────────

fn run_fzf() -> io::Result<Option<PathBuf>> {
    let home = env::var("HOME").unwrap_or_default();
    let dirs = [
        format!("{home}/Downloads"),
        format!("{home}/Desktop"),
        format!("{home}/Documents"),
        home.clone(),
    ];
    let dirs_joined = dirs.join(" ");

    let cmd = format!(
        "find {dirs_joined} -maxdepth 3 -type f \
         \\( -iname '*.crt' -o -iname '*.pem' -o -iname '*.cer' \\) 2>/dev/null \
         | fzf --prompt='Select certificate> ' --height=60% --reverse"
    );

    let out = Command::new("sh").args(["-c", &cmd]).output()?;
    if !out.status.success() { return Ok(None); }
    let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if s.is_empty() { Ok(None) } else { Ok(Some(PathBuf::from(s))) }
}

/// Dosya adını uzantısıyla birlikte döndürür: "meb.crt" → "meb.crt"
fn name_from_path(p: &Path) -> String {
    p.file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "cert.crt".to_string())
}

/// Dosya PEM formatında mı? (ASCII + BEGIN CERTIFICATE marker)
fn is_pem(path: &Path) -> io::Result<bool> {
    let data = fs::read(path)?;
    let head = &data[..data.len().min(1024)];
    Ok(head.windows(27).any(|w| w == b"-----BEGIN CERTIFICATE-----"))
}

/// DER formatındaki sertifikayı PEM'e çevirir.
fn convert_der_to_pem(src: &Path, dest: &Path) -> io::Result<()> {
    let status = Command::new("openssl")
        .args(["x509", "-inform", "DER", "-in"])
        .arg(src)
        .args(["-outform", "PEM", "-out"])
        .arg(dest)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()?;
    if !status.success() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "openssl could not convert DER file to PEM",
        ));
    }
    Ok(())
}

// ───────────────────────── İşlemler ─────────────────────────

fn add_cert(source: &Path) -> io::Result<()> {
    if !source.exists() {
        return Err(io::Error::new(io::ErrorKind::NotFound,
            format!("file not found: {}", source.display())));
    }

    let mut idx = Index::load()?;
    let name = name_from_path(source);
    let filename = name.clone();

    if idx.find(&name).is_some() {
        return Err(io::Error::new(io::ErrorKind::AlreadyExists,
            format!("a certificate named '{name}' already exists")));
    }

    let dest = certs_dir().join(&filename);
    if is_pem(source)? {
        // Zaten PEM — olduğu gibi kopyala
        fs::copy(source, &dest)?;
    } else {
        // DER/binary — PEM'e çevirerek kaydet
        convert_der_to_pem(source, &dest)?;
    }

    let id = idx.next_id;
    idx.next_id += 1;
    idx.certs.push(CertEntry {
        id,
        name: name.clone(),
        file: filename,
        enabled: false,
    });
    idx.save()?;

    Ok(())
}

fn set_enabled(name: &str, enabled: bool) -> io::Result<()> {
    let mut idx = Index::load()?;
    let pos = idx.position(name)
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound,
            format!("'{name}' not found")))?;

    idx.certs[pos].enabled = enabled;
    let file = idx.certs[pos].file.clone();

    let src = certs_dir().join(&file);
    let link = enabled_dir().join(&file);

    if enabled {
        if link.symlink_metadata().is_ok() { let _ = fs::remove_file(&link); }
        symlink(&src, &link)?;
    } else if link.symlink_metadata().is_ok() {
        fs::remove_file(&link)?;
    }

    idx.save()?;
    write_firefox_policy(&idx)?;
    trigger_rebuild();

    Ok(())
}

fn remove_cert(name: &str) -> io::Result<()> {
    let mut idx = Index::load()?;
    let pos = idx.position(name)
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound,
            format!("'{name}' not found")))?;

    let entry = idx.certs.remove(pos);
    let _ = fs::remove_file(certs_dir().join(&entry.file));
    let _ = fs::remove_file(enabled_dir().join(&entry.file));

    idx.save()?;
    write_firefox_policy(&idx)?;
    trigger_rebuild();

    Ok(())
}

fn list_certs() -> io::Result<()> {
    let idx = Index::load()?;
    if idx.certs.is_empty() {
        println!("(no certificates)");
        return Ok(());
    }
    println!("{:<6} {:<10} {}", "ID", "STATUS", "NAME");
    println!("{}", "─".repeat(50));
    for c in &idx.certs {
        let s = if c.enabled { "● enabled" } else { "○ disabled" };
        println!("{:<6} {:<10} {}", c.id, s, c.name);
    }
    Ok(())
}

// ───────────────────────── CLI ─────────────────────────

#[derive(Parser)]
#[command(name = "nocer", version, about = "NixOS runtime certificate manager")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a certificate file
    Add { path: PathBuf },
    /// List all certificates
    List,
    /// Enable a certificate
    Enable { name: String },
    /// Disable a certificate
    Disable { name: String },
    /// Remove a certificate
    Remove { name: String },
}

// ───────────────────────── TUI ─────────────────────────

enum Mode {
    Normal,
    ConfirmDelete { idx: usize },
}

struct App {
    selected: usize,
    mode: Mode,
    status: String,
}

impl App {
    fn new() -> Self {
        Self { selected: 0, mode: Mode::Normal, status: String::new() }
    }
}

fn tui() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let res = tui_loop(&mut terminal);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    res
}

fn tui_loop<B: Backend + io::Write>(terminal: &mut Terminal<B>) -> io::Result<()> {
    let mut app = App::new();

    loop {
        let idx = Index::load()?;

        terminal.draw(|f| draw(f, &idx, &mut app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press { continue; }

            match app.mode {
                Mode::Normal => match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(()),

                    KeyCode::Up => {
                        if app.selected > 0 { app.selected -= 1; }
                    }
                    KeyCode::Down => {
                        if !idx.certs.is_empty() && app.selected + 1 < idx.certs.len() {
                            app.selected += 1;
                        }
                    }

                    KeyCode::Enter => {
                        if let Some(c) = idx.certs.get(app.selected) {
                            let name = c.name.clone();
                            let new_state = !c.enabled;
                            let r = set_enabled(&name, new_state);
                            app.status = match r {
                                Ok(_) => format!(
                                    "{} '{}'",
                                    if new_state { "enabled" } else { "disabled" },
                                    name
                                ),
                                Err(e) => format!("error: {e}"),
                            };
                        }
                    }

                    KeyCode::Char('a') | KeyCode::Char('A') => {
                        disable_raw_mode()?;
                        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

                        let pick = run_fzf();

                        enable_raw_mode()?;
                        execute!(terminal.backend_mut(), EnterAlternateScreen)?;
                        terminal.clear()?;

                        match pick {
                            Ok(Some(path)) => {
                                let r = add_cert(&path);
                                app.status = match r {
                                    Ok(_) => format!("added '{}'", name_from_path(&path)),
                                    Err(e) => format!("error: {e}"),
                                };
                            }
                            Ok(None) => { app.status = "add cancelled".into(); }
                            Err(e)   => { app.status = format!("fzf error: {e}"); }
                        }
                    }

                    KeyCode::Char('d') | KeyCode::Char('D') => {
                        if !idx.certs.is_empty() {
                            app.mode = Mode::ConfirmDelete { idx: app.selected };
                        }
                    }

                    _ => {}
                },

                Mode::ConfirmDelete { idx: i } => match key.code {
                    KeyCode::Char('y') | KeyCode::Char('Y') => {
                        if let Some(c) = idx.certs.get(i) {
                            let name = c.name.clone();
                            let r = remove_cert(&name);
                            app.status = match r {
                                Ok(_) => format!("removed '{name}'"),
                                Err(e) => format!("error: {e}"),
                            };
                            if app.selected > 0 && app.selected >= i {
                                app.selected -= 1;
                            }
                        }
                        app.mode = Mode::Normal;
                    }
                    KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                        app.mode = Mode::Normal;
                        app.status = "delete cancelled".into();
                    }
                    _ => {}
                },
            }
        }
    }
}

fn draw(f: &mut ratatui::Frame, idx: &Index, app: &mut App) {
    let area = f.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(6),
            Constraint::Length(3),
        ])
        .split(area);

    // Başlık
    let title = Paragraph::new(Line::from(vec![
        Span::styled(" NOCER ", Style::default()
            .fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::styled("· NixOS Certificate Manager ",
            Style::default().fg(Color::DarkGray)),
    ]))
    .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    // Liste
    let items: Vec<ListItem> = if idx.certs.is_empty() {
        vec![ListItem::new(Line::from(Span::styled(
            "  (no certificates — press A to add)",
            Style::default().fg(Color::DarkGray),
        )))]
    } else {
        idx.certs.iter().map(|c| {
            let (tag, col) = if c.enabled {
                ("● ENABLED ", Color::Green)
            } else {
                ("○ DISABLED", Color::DarkGray)
            };
            ListItem::new(Line::from(vec![
                Span::styled(tag, Style::default().fg(col).add_modifier(Modifier::BOLD)),
                Span::raw("  "),
                Span::styled(format!("[{:>2}]", c.id),
                    Style::default().fg(Color::Yellow)),
                Span::raw("  "),
                Span::raw(&c.name),
            ]))
        }).collect()
    };

    let mut state = ListState::default();
    if !idx.certs.is_empty() {
        let sel = app.selected.min(idx.certs.len() - 1);
        state.select(Some(sel));
    }

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" Certificates "))
        .highlight_style(Style::default()
            .bg(Color::Blue).fg(Color::White).add_modifier(Modifier::BOLD))
        .highlight_symbol("▶ ");
    f.render_stateful_widget(list, chunks[1], &mut state);

    // Alt bilgi
    let help_line = if app.status.is_empty() {
        Line::from(Span::styled(
            "↑/↓ navigate   ENTER toggle   A add   D delete   Q quit",
            Style::default().fg(Color::Gray),
        ))
    } else {
        Line::from(Span::styled(
            app.status.clone(),
            Style::default().fg(Color::Yellow),
        ))
    };
    let help = Paragraph::new(help_line)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(help, chunks[2]);

    // Silme onayı popup
    if let Mode::ConfirmDelete { idx: i } = app.mode {
        if let Some(c) = idx.certs.get(i) {
            let text = format!("Delete '{}'?", c.name);
            let popup = centered_rect(50, 20, area);
            f.render_widget(Clear, popup);
            let p = Paragraph::new(vec![
                Line::from(""),
                Line::from(Span::styled(text,
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))),
                Line::from(""),
                Line::from(Span::styled("(y/N)",
                    Style::default().fg(Color::DarkGray))),
            ])
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).title(" Confirm "));
            f.render_widget(p, popup);
        }
    }
}

fn centered_rect(pct_x: u16, pct_y: u16, area: ratatui::layout::Rect) -> ratatui::layout::Rect {
    let v = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - pct_y) / 2),
            Constraint::Percentage(pct_y),
            Constraint::Percentage((100 - pct_y) / 2),
        ])
        .split(area);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - pct_x) / 2),
            Constraint::Percentage(pct_x),
            Constraint::Percentage((100 - pct_x) / 2),
        ])
        .split(v[1])[1]
}

// ───────────────────────── main ─────────────────────────

fn main() -> io::Result<()> {
    ensure_dirs()?;

    // Firefox policy'yi her başlatmada güncelle (boş/eksik olmasın)
    if let Ok(idx) = Index::load() {
        let _ = write_firefox_policy(&idx);
    }

    let cli = Cli::parse();
    let result = match cli.command {
        Some(Commands::Add { path }) => {
            add_cert(&path).map(|_| {
                println!("✔ added '{}'", name_from_path(&path));
            })
        }
        Some(Commands::List) => list_certs(),
        Some(Commands::Enable { name }) => {
            set_enabled(&name, true).map(|_| println!("● enabled '{name}'"))
        }
        Some(Commands::Disable { name }) => {
            set_enabled(&name, false).map(|_| println!("○ disabled '{name}'"))
        }
        Some(Commands::Remove { name }) => {
            remove_cert(&name).map(|_| println!("✖ removed '{name}'"))
        }
        None => tui(),
    };

    if let Err(e) = result {
        eprintln!("nocer: {e}");
        std::process::exit(1);
    }
    Ok(())
}
