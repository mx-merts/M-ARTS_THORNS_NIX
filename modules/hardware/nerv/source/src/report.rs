//! Everything NERV prints: colors, the title, the report, INFO, step lines.

use crate::model::{Device, Part, State};
use crate::scan::Scan;
use crate::sys;
use std::env;
use std::io::IsTerminal;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
const RULE: usize = 68;
const NAME_W: usize = 36;
const DRIVER_W: usize = 11;

#[derive(Clone, Copy)]
pub enum Role {
    Title,
    Strong,
    Dim,
    Good,
    Off,
    Nerv,
    Bad,
    Rec,
}

pub struct Paint {
    color: bool,
    truecolor: bool,
    /// Linux console (TTY): no Unicode title, 16 colors.
    console: bool,
}

impl Paint {
    pub fn detect() -> Paint {
        let term = env::var("TERM").unwrap_or_default();
        let color = std::io::stdout().is_terminal() && env::var_os("NO_COLOR").is_none() && term != "dumb";
        let ct = env::var("COLORTERM").unwrap_or_default();
        Paint {
            color,
            truecolor: ct == "truecolor" || ct == "24bit" || env::var_os("KITTY_WINDOW_ID").is_some(),
            console: term == "linux",
        }
    }

    fn code(&self, role: Role) -> &'static str {
        if self.truecolor {
            match role {
                Role::Title => "\x1b[1;38;2;200;30;30m",
                Role::Strong => "\x1b[1;38;2;255;255;255m",
                Role::Dim => "\x1b[38;2;108;112;134m",
                Role::Good => "\x1b[38;2;126;194;126m",
                Role::Off => "\x1b[38;2;138;143;163m",
                Role::Nerv => "\x1b[38;2;95;179;200m",
                Role::Bad => "\x1b[38;2;224;85;85m",
                Role::Rec => "\x1b[38;2;224;180;80m",
            }
        } else {
            match role {
                Role::Title => "\x1b[1;31m",
                Role::Strong => "\x1b[1;97m",
                Role::Dim => "\x1b[90m",
                Role::Good => "\x1b[32m",
                Role::Off => "\x1b[37m",
                Role::Nerv => "\x1b[36m",
                Role::Bad => "\x1b[91m",
                Role::Rec => "\x1b[33m",
            }
        }
    }

    pub fn c(&self, role: Role, text: &str) -> String {
        if self.color && !text.is_empty() {
            format!("{}{}\x1b[0m", self.code(role), text)
        } else {
            text.to_string()
        }
    }

    /// ƝΞᚱꝞ in a graphical terminal (Junicode via kitty symbol_map, italic Ɲ),
    /// plain NERV on the Linux console and in pipes.
    pub fn title(&self) -> String {
        if self.console || !self.color {
            return self.c(Role::Title, "NERV");
        }
        format!("{}\x1b[3mƝ\x1b[23mΞᚱꝞ\x1b[0m", self.code(Role::Title))
    }

    fn state_role(state: State) -> Role {
        match state {
            State::On => Role::Good,
            State::Off | State::Blocked => Role::Off,
            State::BoundByNerv => Role::Nerv,
            State::NotBound => Role::Bad,
        }
    }

    pub fn symbol(&self, state: State) -> String {
        self.c(Self::state_role(state), state.symbol())
    }
}

fn pad(s: &str, w: usize) -> String {
    let s = sys::fit(s, w);
    let n = s.chars().count();
    format!("{s}{}", " ".repeat(w.saturating_sub(n)))
}

pub fn section_header(paint: &Paint, part: Part) -> String {
    let tag = part.short().map(|s| format!("[{s}] ")).unwrap_or_default();
    let used = 2 + part.name().len() + 1 + tag.chars().count();
    format!(
        " {} {}",
        paint.c(Role::Strong, &format!("▌{}", part.name())),
        paint.c(Role::Dim, &format!("{tag}{}", "─".repeat(RULE.saturating_sub(used))))
    )
}

pub fn row(paint: &Paint, n: usize, d: &Device) -> String {
    let name_role = if d.state == State::Off { Role::Off } else { Role::Strong };
    let name = pad(&d.name, NAME_W);
    let name = if matches!(name_role, Role::Strong) { name } else { paint.c(name_role, &name) };
    let driver = pad(d.driver.as_deref().unwrap_or("──"), DRIVER_W);
    let note = match d.state {
        State::On if d.note.starts_with("THORNS ROOT") => {
            let rest = &d.note["THORNS ROOT".len()..];
            format!("{}{}", paint.c(Role::Strong, "THORNS ROOT"), rest)
        }
        State::On => d.note.clone(),
        s => paint.c(Paint::state_role(s), &d.note),
    };
    format!("  {:>2} {}  {} {} {}", n, paint.symbol(d.state), name, paint.c(Role::Dim, &driver), note)
}

pub fn section(paint: &Paint, scan: &Scan, part: Part) -> Vec<String> {
    let devs = scan.of(part);
    if devs.is_empty() {
        return Vec::new();
    }
    let mut out = vec![section_header(paint, part)];
    for (i, d) in devs.iter().enumerate() {
        out.push(row(paint, i + 1, d));
    }
    if part == Part::Gpu {
        let r = &scan.rec;
        let mark = if r.ok() {
            format!("{}{}{}", paint.c(Role::Dim, "(CURRENT: "), paint.c(Role::Dim, &r.current), paint.c(Role::Good, " ✓)"))
        } else {
            paint.c(Role::Bad, &format!("(CURRENT: {})", r.current))
        };
        out.push(format!(
            "    {}   {}",
            paint.c(Role::Rec, &format!("» {} DETECTED ── {} RECOMMENDED", r.detected, r.recommended)),
            mark
        ));
        if !r.ok() {
            out.push(paint.c(Role::Dim, &format!("      {}. Reboot and pick the {} entry in GRUB.", r.why, r.recommended)));
        }
        out.push(String::new());
    }
    out
}

pub fn header(paint: &Paint, scan: &Scan) -> Vec<String> {
    let total = scan.devices.len();
    let nerv = scan.devices.iter().filter(|d| d.state == State::BoundByNerv).count();
    let bad = scan.devices.iter().filter(|d| d.state == State::NotBound).count();
    let when = match &scan.boot.time {
        Some(t) => format!("BOOT {t}"),
        None => "LIVE (boot scan has not run)".into(),
    };
    let mut scan_line = format!("{when} {} {total} DEVICES", paint.c(Role::Dim, "·"));
    if nerv > 0 {
        scan_line += &format!(" {} {}", paint.c(Role::Dim, "·"), paint.c(Role::Nerv, &format!("{nerv} BOUND BY NERV")));
    }
    if bad > 0 {
        scan_line += &format!(" {} {}", paint.c(Role::Dim, "·"), paint.c(Role::Bad, &format!("{bad} NOT BOUND")));
    }
    let left = |k: &str, v: &str| format!(" {}  {}", paint.c(Role::Dim, &format!("{k:<7}")), pad(v, 40));
    vec![
        format!(" {} {} {} {}", paint.title(), paint.c(Role::Strong, VERSION), paint.c(Role::Dim, "──"), paint.c(Role::Dim, "HARDWARE NERVOUS SYSTEM")),
        String::new(),
        format!("{}  {}  {}", left("MACHINE", &scan.machine), paint.c(Role::Dim, "SESSION"), paint.c(Role::Strong, &scan.conf.session)),
        format!("{}  {}  {}", left("KERNEL", &scan.kernel), paint.c(Role::Dim, "SECURE BOOT"), scan.secure_boot),
        format!(" {}  {}", paint.c(Role::Dim, "SCAN   "), scan_line),
        String::new(),
    ]
}

/// Problems worth a line under the report: not bound, plus boot failures
/// for devices no part shows.
pub fn issues(scan: &Scan) -> Vec<String> {
    let mut out = Vec::new();
    for part in Part::ALL {
        for (i, d) in scan.of(part).iter().enumerate() {
            if d.state == State::NotBound {
                let tag = part.short().unwrap_or(part.name());
                let reason = d.reason.clone().unwrap_or_else(|| d.note.clone());
                out.push(format!("{tag} {}  {} ── {reason}", i + 1, d.name));
            }
        }
    }
    for (path, name, reason) in &scan.boot.failed {
        if !scan.devices.iter().any(|d| &d.sys == path) {
            out.push(format!("{name} ── {reason}"));
        }
    }
    out
}

pub fn report(paint: &Paint, scan: &Scan, only: Option<Part>) -> String {
    let mut lines = Vec::new();
    if only.is_none() {
        lines.extend(header(paint, scan));
    }
    for part in Part::ALL {
        if only.is_none() || only == Some(part) {
            lines.extend(section(paint, scan, part));
        }
    }
    while only.is_some() && lines.last().is_some_and(|l| l.is_empty()) {
        lines.pop();
    }
    if only.is_some_and(|p| scan.of(p).is_empty()) {
        lines.push(format!("   {}", paint.c(Role::Dim, &format!("NO {} FOUND", only.unwrap().name()))));
    }
    if only.is_none() {
        if lines.last().is_some_and(|l| !l.is_empty()) {
            lines.push(String::new());
        }
        lines.push(format!(
            " {} ON   {} OFF   {} BOUND BY NERV   {} NOT BOUND   {} BLOCKED BY POLICY",
            paint.symbol(State::On),
            paint.symbol(State::Off),
            paint.symbol(State::BoundByNerv),
            paint.symbol(State::NotBound),
            paint.symbol(State::Blocked)
        ));
        lines.push(format!(" {}", paint.c(Role::Dim, &"─".repeat(RULE + 2))));
        let issues = issues(scan);
        if issues.is_empty() {
            lines.push(format!(" {}", paint.c(Role::Good, "NO ISSUES")));
        } else {
            let word = if issues.len() == 1 { "ISSUE" } else { "ISSUES" };
            for (i, text) in issues.iter().enumerate() {
                let head = if i == 0 { format!("{} {word}", issues.len()) } else { String::new() };
                lines.push(format!(" {}   {}", paint.c(Role::Bad, &format!("{head:<9}")), text));
            }
        }
        lines.push(paint.c(Role::Dim, " NERV <PART> <N> INFO | ENABLE | DISABLE | RESET    NERV HELP"));
    }
    lines.join("\n")
}

pub fn info(paint: &Paint, d: &Device, n: usize) -> String {
    let mut out = vec![format!(" {}   {}", paint.c(Role::Strong, &format!("▌{} {n}", d.part.name())), d.name)];
    for (k, v) in &d.info {
        out.push(format!("   {}  {v}", paint.c(Role::Dim, &format!("{k:<11}"))));
    }
    out.join("\n")
}

/// "   usb deauthorize ........... OK"
pub fn step(paint: &Paint, what: &str, result: &Result<(), String>) -> String {
    let dots = ".".repeat(26usize.saturating_sub(what.chars().count()).max(3));
    match result {
        Ok(()) => format!("   {what} {} {}", paint.c(Role::Dim, &dots), paint.c(Role::Good, "OK")),
        Err(e) => format!("   {what} {} {}  {}", paint.c(Role::Dim, &dots), paint.c(Role::Bad, "FAILED"), paint.c(Role::Dim, e)),
    }
}

pub fn part_head(paint: &Paint, d: &Device, n: usize) -> String {
    format!(" {}   {}", paint.c(Role::Strong, &format!("▌{} {n}", d.part.name())), d.name)
}

pub fn verdict(paint: &Paint, state: State, text: &str, dim: &str) -> String {
    let dim = if dim.is_empty() { String::new() } else { format!("   {}", paint.c(Role::Dim, dim)) };
    format!("   {}  {}{dim}", paint.symbol(state), paint.c(Paint::state_role(state), text))
}

pub fn refused(paint: &Paint, text: &str) -> String {
    format!("   {}  {}", paint.c(Role::Bad, "✖"), paint.c(Role::Bad, text))
}
