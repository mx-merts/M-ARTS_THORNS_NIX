//! NERV NOTIFY: runs once at login (nerv-notify user service). Waits for the
//! boot scan, then shows what is worth knowing. Silent when all is well.
//!
//! NERV NOTIFY --DRY prints the notifications instead of sending them.

use crate::model::{Part, State};
use crate::scan::{self, Scan};
use crate::state;
use crate::sys::{self, p};
use std::process::{Command, Stdio};

pub struct Note {
    pub title: String,
    pub body: String,
    /// Stays on screen until closed.
    pub sticky: bool,
    pub icon: &'static str,
}

pub fn build(scan: &Scan) -> Vec<Note> {
    let mut notes = Vec::new();
    let r = &scan.rec;
    if !r.ok() {
        notes.push(Note {
            title: format!("{} DETECTED", r.detected),
            body: format!("{} RECOMMENDED (current: {}) ── reboot and pick it in GRUB", r.recommended, r.current),
            sticky: true,
            icon: "video-display",
        });
    }
    let bound: Vec<&str> = scan.boot.bound.iter().map(|(_, n)| n.as_str()).collect();
    if !bound.is_empty() {
        notes.push(Note {
            title: format!("{} DEVICE{} BOUND AT BOOT", bound.len(), plural(bound.len())),
            body: format!("{} ── run NERV", bound.join(", ")),
            sticky: false,
            icon: "preferences-system",
        });
    }
    let unbound: Vec<String> = scan
        .devices
        .iter()
        .filter(|d| d.state == State::NotBound && d.part != Part::Fingerprint)
        .map(|d| d.name.clone())
        .chain(
            scan.boot
                .failed
                .iter()
                .filter(|(path, _, _)| !scan.devices.iter().any(|d| &d.sys == path))
                .map(|(_, n, _)| n.clone()),
        )
        .collect();
    if !unbound.is_empty() {
        notes.push(Note {
            title: format!("{} DEVICE{} SEEN BUT NOT BOUND", unbound.len(), plural(unbound.len())),
            body: format!("{} ── run NERV", unbound.join(", ")),
            sticky: true,
            icon: "dialog-warning",
        });
    }
    for fp in scan.of(Part::Fingerprint) {
        notes.push(Note {
            title: "FINGERPRINT READER DETECTED".into(),
            body: format!("{} ── {}", fp.name, fp.note.to_lowercase()),
            sticky: false,
            icon: "fingerprint",
        });
    }
    notes
}

fn plural(n: usize) -> &'static str {
    if n == 1 { "" } else { "S" }
}

fn send(n: &Note) {
    let expire = if n.sticky { "0" } else { "-1" };
    let _ = Command::new("notify-send")
        .args(["-a", "ƝΞᚱꝞ", "-i", n.icon, "-t", expire, &n.title, &escape(&n.body)])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
}

pub fn escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

pub fn run(dry: bool) -> i32 {
    // The boot scan may still be running right after login.
    let boot_file = p(&format!("{}/boot", state::DIR));
    for _ in 0..45 {
        if boot_file.exists() {
            break;
        }
        sys::sleep_ms(2000);
    }
    let scan = scan::scan();
    let notes = build(&scan);
    for n in &notes {
        if dry {
            println!("[{}] {}\n    {}", if n.sticky { "sticky" } else { "normal" }, n.title, n.body);
        } else {
            send(n);
        }
    }
    if dry && notes.is_empty() {
        println!("(nothing to say: right session, everything bound)");
    }
    0
}
