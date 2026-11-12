//! Colors and the title, the same family as NERV.

use crate::model::Mode;
use std::env;
use std::io::IsTerminal;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Clone, Copy)]
pub enum Role {
    Title,
    Strong,
    Dim,
    Good,
    Warn,
    Bad,
    Custom,
}

pub struct Paint {
    color: bool,
    truecolor: bool,
    console: bool,
}

impl Paint {
    pub fn detect() -> Paint {
        let term = env::var("TERM").unwrap_or_default();
        let ct = env::var("COLORTERM").unwrap_or_default();
        Paint {
            color: std::io::stdout().is_terminal() && env::var_os("NO_COLOR").is_none() && term != "dumb",
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
                Role::Warn => "\x1b[38;2;224;180;80m",
                Role::Bad => "\x1b[38;2;224;85;85m",
                Role::Custom => "\x1b[38;2;95;179;200m",
            }
        } else {
            match role {
                Role::Title => "\x1b[1;31m",
                Role::Strong => "\x1b[1;97m",
                Role::Dim => "\x1b[90m",
                Role::Good => "\x1b[32m",
                Role::Warn => "\x1b[33m",
                Role::Bad => "\x1b[91m",
                Role::Custom => "\x1b[36m",
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

    /// ΣꝀᛁƝ in a graphical terminal (Junicode via kitty symbol_map, italic Ɲ
    /// like NERV), plain SKIN on the Linux console and in pipes.
    pub fn title(&self) -> String {
        if self.console || !self.color {
            return self.c(Role::Title, "SKIN");
        }
        format!("{}ΣꝀᛁ\x1b[3mƝ\x1b[23m\x1b[0m", self.code(Role::Title))
    }

    pub fn mode_role(m: Mode) -> Role {
        match m {
            Mode::Seal => Role::Good,
            Mode::Sever => Role::Warn,
            Mode::Rot => Role::Bad,
        }
    }

    /// A link kitty opens with Ctrl+click (OSC 8).
    pub fn link(&self, url: &str) -> String {
        if self.color && !self.console {
            format!("\x1b]8;;{url}\x1b\\{}\x1b]8;;\x1b\\", self.c(Role::Strong, url))
        } else {
            url.to_string()
        }
    }

    /// "   DNS   → encrypted ............ OK"
    pub fn step(&self, layer: &str, what: &str, r: &Result<String, String>) -> String {
        let label = format!("{layer:<5} → {what}");
        let dots = ".".repeat(28usize.saturating_sub(label.chars().count()).max(3));
        match r {
            Ok(note) if note.is_empty() => format!("   {label} {} {}", self.c(Role::Dim, &dots), self.c(Role::Good, "OK")),
            Ok(note) => format!("   {label} {} {}   {}", self.c(Role::Dim, &dots), self.c(Role::Good, "OK"), self.c(Role::Dim, note)),
            Err(e) => format!("   {label} {} {}  {}", self.c(Role::Dim, &dots), self.c(Role::Bad, "FAILED"), self.c(Role::Dim, e)),
        }
    }
}
