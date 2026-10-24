//! The event log: one JSON object per line in /var/lib/spine/events.jsonl.
//! `Hub` is the single place events are written, with de-duplication.

use crate::paths::Paths;
use crate::util;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;
use std::sync::Mutex;

pub const SEV_ORDER: [&str; 4] = ["crit", "warn", "notice", "info"];
pub const SEVERITIES: [&str; 4] = ["info", "notice", "warn", "crit"];

#[derive(Default)]
struct Inner {
    last_seen: HashMap<String, u64>,
    suppressed: u64,
    module_loads: u64,
}

pub struct Hub {
    paths: Paths,
    inner: Mutex<Inner>,
}

impl Hub {
    pub fn new(paths: Paths) -> Self {
        Hub { paths, inner: Mutex::new(Inner::default()) }
    }

    /// Appends one event. `dedupe` is (key, window in seconds): an event with
    /// the same key inside the window is dropped and only counted.
    pub fn try_emit(
        &self,
        sev: &str,
        src: &str,
        title: &str,
        detail: &str,
        dedupe: Option<(&str, u64)>,
    ) -> io::Result<()> {
        let now = util::now();
        let mut inner = self.inner.lock().unwrap_or_else(|p| p.into_inner());
        if let Some((key, window)) = dedupe {
            if let Some(prev) = inner.last_seen.get(key) {
                if now.saturating_sub(*prev) < window {
                    inner.suppressed += 1;
                    return Ok(());
                }
            }
            inner.last_seen.insert(key.to_string(), now);
            if inner.last_seen.len() > 2000 {
                inner.last_seen.retain(|_, v| now.saturating_sub(*v) <= 86400);
            }
        }
        let line = json!({"ts": now, "sev": sev, "src": src, "title": title, "detail": detail})
            .to_string();
        let mut f = OpenOptions::new()
            .create(true)
            .append(true)
            .mode(0o640)
            .open(&self.paths.events)?;
        writeln!(f, "{line}")
    }

    /// Like `try_emit`, but a write failure is only logged (daemon use).
    pub fn emit(&self, sev: &str, src: &str, title: &str, detail: &str, dedupe: Option<(&str, u64)>) {
        if let Err(e) = self.try_emit(sev, src, title, detail, dedupe) {
            eprintln!("spine: cannot write event: {e}");
        }
    }

    pub fn count_module_load(&self) {
        self.inner.lock().unwrap_or_else(|p| p.into_inner()).module_loads += 1;
    }

    /// (duplicates suppressed, kernel module loads seen)
    pub fn stats(&self) -> (u64, u64) {
        let inner = self.inner.lock().unwrap_or_else(|p| p.into_inner());
        (inner.suppressed, inner.module_loads)
    }
}

pub fn read_events(path: &Path) -> io::Result<Vec<Value>> {
    let bytes = fs::read(path)?;
    Ok(String::from_utf8_lossy(&bytes)
        .lines()
        .filter_map(|l| serde_json::from_str(l).ok())
        .collect())
}

/// Keeps the log bounded: above 10000 lines, keep the newest 5000.
pub fn trim_events(path: &Path) {
    let Ok(bytes) = fs::read(path) else { return };
    let text = String::from_utf8_lossy(&bytes);
    let lines: Vec<&str> = text.lines().collect();
    if lines.len() > 10_000 {
        let keep = lines[lines.len() - 5_000..].join("\n") + "\n";
        // fs::write truncates in place, so owner and mode are preserved.
        let _ = fs::write(path, keep);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_valid_json_lines_and_dedupes() {
        let p = Paths::temp("event");
        let hub = Hub::new(p.clone());
        hub.try_emit("warn", "t", "Title \"q\"", "détail <x>", Some(("k", 600))).unwrap();
        hub.try_emit("warn", "t", "Title", "again", Some(("k", 600))).unwrap();
        hub.try_emit("warn", "t", "Other", "x", Some(("k2", 600))).unwrap();
        hub.try_emit("info", "t", "No dedupe", "x", None).unwrap();
        hub.try_emit("info", "t", "No dedupe", "x", None).unwrap();
        let ev = read_events(&p.events).unwrap();
        assert_eq!(ev.len(), 4);
        assert_eq!(ev[0]["title"], "Title \"q\"");
        assert_eq!(ev[0]["detail"], "détail <x>");
        assert_eq!(hub.stats().0, 1);
    }

    #[test]
    fn trims_only_above_the_limit() {
        let p = Paths::temp("trim");
        let body: String = (0..10_001).map(|i| format!("{{\"n\":{i}}}\n")).collect();
        fs::write(&p.events, body).unwrap();
        trim_events(&p.events);
        let ev = read_events(&p.events).unwrap();
        assert_eq!(ev.len(), 5_000);
        assert_eq!(ev[4_999]["n"], 10_000);
        trim_events(&p.events); // now small: untouched
        assert_eq!(read_events(&p.events).unwrap().len(), 5_000);
    }
}
