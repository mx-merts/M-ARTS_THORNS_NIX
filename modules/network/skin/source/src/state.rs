//! Runtime state in /run/skin (tmpfs: every boot starts SEALED).
//!
//!   state    one line per layer: "DNS SEAL 1731420000", plus "timer SEVER <deadline>"
//!   events   one notification per line, read by the user's notifier
//!   rules    the nftables ruleset applied last (for SKIN KS / SKIN LAN)
//!
//! /etc/skin/skin.conf (written by the NixOS module): "known=<SSID>" lines.
//! /var/lib/skin/known (sudo SKIN TRUST, not in git): one SSID per line.

use crate::model::{Layer, Mode};
use crate::sys::{self, p};
use std::fs::OpenOptions;
use std::io::Write;

pub const DIR: &str = "/run/skin";

#[derive(Clone, Debug, PartialEq)]
pub struct State {
    pub modes: [Mode; 5],
    /// When each layer entered its mode (0 = since boot).
    pub since: [u64; 5],
    /// Whole-membrane timer: back to SEAL at this time.
    pub timer: Option<(Mode, u64)>,
}

impl Default for State {
    fn default() -> State {
        State { modes: [Mode::Seal; 5], since: [0; 5], timer: None }
    }
}

impl State {
    pub fn mode(&self, l: Layer) -> Mode {
        self.modes[l.index()]
    }

    pub fn set(&mut self, l: Layer, m: Mode) {
        if self.modes[l.index()] != m {
            self.modes[l.index()] = m;
            self.since[l.index()] = sys::now();
        }
    }

    /// Some(mode) when every layer is in the same mode.
    pub fn whole(&self) -> Option<Mode> {
        let first = self.modes[0];
        self.modes.iter().all(|m| *m == first).then_some(first)
    }
}

pub fn parse(text: &str) -> State {
    let mut s = State::default();
    for line in text.lines() {
        let f: Vec<&str> = line.split_whitespace().collect();
        match f.as_slice() {
            ["timer", m, t] => {
                if let (Some(m), Ok(t)) = (Mode::parse(m), t.parse()) {
                    s.timer = Some((m, t));
                }
            }
            [l, m, t] => {
                if let (Some(l), Some(m)) = (Layer::parse(l), Mode::parse(m)) {
                    s.modes[l.index()] = m;
                    s.since[l.index()] = t.parse().unwrap_or(0);
                }
            }
            _ => {}
        }
    }
    s
}

pub fn format(s: &State) -> String {
    let mut out = String::new();
    for l in Layer::ALL {
        out += &format!("{} {} {}\n", l.short(), s.mode(l).name(), s.since[l.index()]);
    }
    if let Some((m, t)) = s.timer {
        out += &format!("timer {} {t}\n", m.name());
    }
    out
}

pub fn load() -> State {
    parse(&sys::read(&p(&format!("{DIR}/state"))).unwrap_or_default())
}

pub fn save(s: &State) -> Result<(), String> {
    sys::write_atomic(&p(&format!("{DIR}/state")), &format(s))
}

/// Queues a notification for the user's notifier ("TUNNEL SEVERED", "...").
pub fn event(title: &str, body: &str, sticky: bool) {
    let line = format!(
        "{}\t{}\t{}\t{}\n",
        sys::now(),
        if sticky { "sticky" } else { "normal" },
        title.replace(['\t', '\n'], " "),
        body.replace(['\t', '\n'], " ")
    );
    let path = p(&format!("{DIR}/events"));
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(&path) {
        let _ = f.write_all(line.as_bytes());
    }
}

/// Networks trusted on this machine only (sudo SKIN TRUST). Kept out of
/// git on purpose: a home SSID is enough to find the house on a Wi-Fi map.
pub const TRUSTED: &str = "/var/lib/skin/known";

fn lines_of(text: &str) -> Vec<String> {
    text.lines().map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect()
}

/// SSIDs trusted with `sudo SKIN TRUST` (one per line).
pub fn trusted_local() -> Vec<String> {
    lines_of(&std::fs::read_to_string(p(TRUSTED)).unwrap_or_default())
}

/// SSIDs from the NixOS option programs.skin.knownNetworks.
pub fn known_nix() -> Vec<String> {
    let conf = sys::read(&p("/etc/skin/skin.conf")).unwrap_or_default();
    lines_of(&conf).iter().filter_map(|l| l.strip_prefix("known=")).map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect()
}

/// Every known network (home): LAN stays open there while sealed.
pub fn known_networks() -> Vec<String> {
    let mut all = known_nix();
    for s in trusted_local() {
        if !all.contains(&s) {
            all.push(s);
        }
    }
    all
}

/// Adds or removes an SSID in the local list. Ok(false): nothing to change.
pub fn set_trusted(ssid: &str, on: bool) -> Result<bool, String> {
    let mut list = trusted_local();
    let has = list.iter().any(|s| s == ssid);
    if has == on {
        return Ok(false);
    }
    if on {
        list.push(ssid.to_string());
    } else {
        list.retain(|s| s != ssid);
    }
    let path = p(TRUSTED);
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    let mut text = list.join("\n");
    if !text.is_empty() {
        text.push('\n');
    }
    sys::write_atomic(&path, &text)?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_round_trips() {
        let mut s = State::default();
        s.modes[Layer::Ks.index()] = Mode::Rot;
        s.since[Layer::Ks.index()] = 1731420000;
        s.timer = Some((Mode::Sever, 1731420600));
        let back = parse(&format(&s));
        assert_eq!(back, s);
        assert_eq!(back.whole(), None);
        assert_eq!(State::default().whole(), Some(Mode::Seal));
        assert_eq!(parse("").whole(), Some(Mode::Seal));
    }

    #[test]
    fn known_lines_are_trimmed() {
        assert_eq!(lines_of(" Home \n\n  \nCafe:5G\n"), vec!["Home", "Cafe:5G"]);
    }
}
