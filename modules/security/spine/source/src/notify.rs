//! `spine notify`: the session-side service that turns new events into desktop
//! notifications. It remembers how far it has read (an offset file), so each
//! event is shown once and history is not replayed on first start.

use crate::paths::Paths;
use crate::util::html_escape;
use serde_json::Value;
use std::env;
use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

/// notify-send arguments for an event, or None if it is not shown (info).
pub fn build_args(rec: &Value) -> Option<Vec<String>> {
    let sev = rec.get("sev")?.as_str()?;
    let (urgency, icon) = match sev {
        "notice" => ("low", "dialog-information"),
        "warn" => ("normal", "dialog-warning"),
        "crit" => ("critical", "dialog-error"),
        _ => return None,
    };
    let title = rec.get("title").and_then(Value::as_str).unwrap_or("");
    let detail = rec.get("detail").and_then(Value::as_str).unwrap_or("");
    Some(vec![
        "-a".into(),
        "SPINE".into(),
        "-u".into(),
        urgency.into(),
        "-i".into(),
        icon.into(),
        format!("SPINE [{}] {}", sev.to_uppercase(), title),
        html_escape(detail),
    ])
}

fn deliver(rec: &Value) {
    if let Some(args) = build_args(rec) {
        let _ = Command::new("notify-send")
            .args(args)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }
}

fn offset_file() -> Option<PathBuf> {
    let base = match env::var_os("XDG_STATE_HOME").filter(|v| !v.is_empty()) {
        Some(d) => PathBuf::from(d),
        None => PathBuf::from(env::var_os("HOME")?).join(".local/state"),
    };
    Some(base.join("spine").join("offset"))
}

/// New complete lines in `path` after `offset`; returns (lines, new offset).
pub fn read_new(path: &std::path::Path, offset: u64, size: u64) -> std::io::Result<(Vec<String>, u64)> {
    let mut f = File::open(path)?;
    f.seek(SeekFrom::Start(offset))?;
    let mut data = Vec::new();
    f.take(size.saturating_sub(offset)).read_to_end(&mut data)?;
    let cut = data.iter().rposition(|&b| b == b'\n').map_or(0, |i| i + 1);
    let text = String::from_utf8_lossy(&data[..cut]).into_owned();
    Ok((text.lines().map(str::to_string).collect(), offset + cut as u64))
}

pub fn run() -> i32 {
    let paths = Paths::system();
    let Some(off_path) = offset_file() else {
        eprintln!("spine notify: cannot find a state directory (HOME not set)");
        return 1;
    };
    if let Some(dir) = off_path.parent() {
        let _ = fs::create_dir_all(dir);
    }
    let mut offset: Option<u64> = fs::read_to_string(&off_path).ok().and_then(|t| t.trim().parse().ok());
    loop {
        let Ok(md) = fs::metadata(&paths.events) else {
            thread::sleep(Duration::from_secs(5));
            continue;
        };
        let size = md.len();
        let mut off = match offset {
            Some(o) if o <= size => o,
            _ => {
                // First run, or the log was trimmed: do not replay history.
                let _ = fs::write(&off_path, size.to_string());
                size
            }
        };
        if size > off {
            if let Ok((lines, new_off)) = read_new(&paths.events, off, size) {
                for line in lines {
                    if let Ok(rec) = serde_json::from_str::<Value>(&line) {
                        deliver(&rec);
                    }
                }
                if new_off != off {
                    off = new_off;
                    let _ = fs::write(&off_path, off.to_string());
                }
            }
        }
        offset = Some(off);
        thread::sleep(Duration::from_secs(3));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn builds_escaped_arguments_and_skips_info() {
        let a = build_args(&json!({"sev":"crit","title":"T","detail":"a <b> & c"})).unwrap();
        assert_eq!(a[3], "critical");
        assert_eq!(a[6], "SPINE [CRIT] T");
        assert_eq!(a[7], "a &lt;b&gt; &amp; c");
        assert_eq!(build_args(&json!({"sev":"warn","title":"T"})).unwrap()[3], "normal");
        assert!(build_args(&json!({"sev":"info","title":"x"})).is_none());
        assert!(build_args(&json!({"nope":1})).is_none());
    }

    #[test]
    fn reads_only_complete_new_lines() {
        let p = Paths::temp("notify");
        fs::write(&p.events, "{\"a\":1}\n{\"b\":2}\n{\"c\"").unwrap();
        let size = fs::metadata(&p.events).unwrap().len();
        let (lines, off) = read_new(&p.events, 0, size).unwrap();
        assert_eq!(lines, vec!["{\"a\":1}", "{\"b\":2}"]);
        assert_eq!(off, 16);
        let (lines, off2) = read_new(&p.events, off, size).unwrap();
        assert!(lines.is_empty());
        assert_eq!(off2, off); // partial line stays unread
    }
}
