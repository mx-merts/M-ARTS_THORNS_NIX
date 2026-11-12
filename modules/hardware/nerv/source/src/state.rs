//! Runtime files in /run/nerv (tmpfs: gone at reboot, no machine memory).
//!
//!   boot       written by the boot scan: what it bound, what failed
//!   disabled   parts switched off with NERV ... DISABLE (until reboot)
//!
//! Both are tab-separated text, one record per line.

use crate::model::Part;
use crate::sys;
use std::path::{Path, PathBuf};

pub const DIR: &str = "/run/nerv";

#[derive(Default, Debug)]
pub struct Boot {
    /// Local time of the boot scan, "14:02:11"; None if it has not run.
    pub time: Option<String>,
    /// (sysfs path, name)
    pub bound: Vec<(PathBuf, String)>,
    /// (sysfs path, name, reason)
    pub failed: Vec<(PathBuf, String, String)>,
    /// (sysfs path, name, module)
    pub blocked: Vec<(PathBuf, String, String)>,
}

impl Boot {
    pub fn was_bound(&self, path: &Path) -> bool {
        self.bound.iter().any(|(p, _)| p == path)
    }
}

pub fn load_boot() -> Boot {
    let text = sys::read(&sys::p(&format!("{DIR}/boot"))).unwrap_or_default();
    parse_boot(&text)
}

pub fn parse_boot(text: &str) -> Boot {
    let mut b = Boot::default();
    for line in text.lines() {
        if let Some(t) = line.strip_prefix("time=") {
            b.time = Some(t.trim().to_string());
            continue;
        }
        let f: Vec<&str> = line.split('\t').collect();
        match (f.first().copied(), f.len()) {
            (Some("bound"), 3) => b.bound.push((PathBuf::from(f[1]), f[2].into())),
            (Some("failed"), 4) => b.failed.push((PathBuf::from(f[1]), f[2].into(), f[3].into())),
            (Some("blocked"), 4) => b.blocked.push((PathBuf::from(f[1]), f[2].into(), f[3].into())),
            _ => {}
        }
    }
    b
}

pub fn format_boot(b: &Boot) -> String {
    let mut out = format!("time={}\n", b.time.as_deref().unwrap_or("?"));
    for (p, n) in &b.bound {
        out += &format!("bound\t{}\t{}\n", p.display(), clean(n));
    }
    for (p, n, r) in &b.failed {
        out += &format!("failed\t{}\t{}\t{}\n", p.display(), clean(n), clean(r));
    }
    for (p, n, m) in &b.blocked {
        out += &format!("blocked\t{}\t{}\t{}\n", p.display(), clean(n), clean(m));
    }
    out
}

fn clean(s: &str) -> String {
    s.replace(['\t', '\n'], " ")
}

#[derive(Clone, Debug, PartialEq)]
pub struct Disabled {
    pub part: Part,
    pub path: PathBuf,
    pub name: String,
}

pub fn load_disabled() -> Vec<Disabled> {
    let text = sys::read(&sys::p(&format!("{DIR}/disabled"))).unwrap_or_default();
    text.lines()
        .filter_map(|l| {
            let f: Vec<&str> = l.split('\t').collect();
            if f.len() != 3 {
                return None;
            }
            Some(Disabled { part: Part::parse(f[0])?, path: PathBuf::from(f[1]), name: f[2].to_string() })
        })
        .collect()
}

pub fn save_disabled(list: &[Disabled]) -> Result<(), String> {
    let dir = sys::p(DIR);
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let text: String = list
        .iter()
        .map(|d| format!("{}\t{}\t{}\n", d.part.name(), d.path.display(), clean(&d.name)))
        .collect();
    sys::write(&dir.join("disabled"), &text)
}

pub fn mark_disabled(part: Part, path: &Path, name: &str, disabled: bool) -> Result<(), String> {
    let mut list = load_disabled();
    list.retain(|d| d.path != path);
    if disabled {
        list.push(Disabled { part, path: path.to_path_buf(), name: name.to_string() });
    }
    save_disabled(&list)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boot_file_round_trips() {
        let mut b = Boot { time: Some("14:02:11".into()), ..Boot::default() };
        b.bound.push(("/sys/devices/x".into(), "Realtek RTL8153".into()));
        b.failed.push(("/sys/devices/y".into(), "Goodix".into(), "no\tdriver".into()));
        b.blocked.push(("/sys/devices/z".into(), "FireWire".into(), "firewire_ohci".into()));
        let back = parse_boot(&format_boot(&b));
        assert_eq!(back.time.as_deref(), Some("14:02:11"));
        assert!(back.was_bound(Path::new("/sys/devices/x")));
        assert_eq!(back.failed[0].2, "no driver");
        assert_eq!(back.blocked[0].2, "firewire_ohci");
    }
}
