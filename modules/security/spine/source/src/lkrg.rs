//! LKRG (Linux Kernel Runtime Guard) messages: follows the kernel log in the
//! journal and reports LKRG detections. LKRG runs in log-only mode on THORNS
//! (modules/security/lkrg.nix), so these events are the only place a detected
//! kernel exploit attempt becomes visible.
//!
//! Message prefixes (lkrg src/modules/print_log/p_lkrg_print_log.h):
//!   "LKRG: ALERT: ..."  a detected violation or attack     -> crit
//!   "LKRG: FATAL: ..."  LKRG itself failed fatally          -> warn
//!   "LKRG: FAULT: ..."  an LKRG error                       -> warn
//!   "LKRG: DYING: ..."  LKRG is being unloaded              -> warn
//!   "LKRG: STATE: ..."  a setting was changed at runtime   -> notice
//!                       (e.g. "Changing 'kint_enforce' from 0 to 2")
//! Everything else (ALIVE/ISSUE/WATCH/DEBUG) is ignored.

use crate::event::Hub;
use serde_json::Value;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

/// (severity, title, detail) for a kernel journal JSON line, if it is an
/// LKRG message worth reporting.
pub fn classify(line: &str) -> Option<(&'static str, &'static str, String)> {
    let v: Value = serde_json::from_str(line).ok()?;
    let msg = v.get("MESSAGE")?.as_str()?.trim(); // binary messages are arrays: skipped
    let rest = msg.strip_prefix("LKRG: ")?;
    let (kind, detail) = rest.split_once(": ").unwrap_or((rest, ""));
    let detail = detail.trim().to_string();
    match kind {
        "ALERT" => Some(("crit", "LKRG: kernel integrity violation / exploit attempt", detail)),
        "FATAL" => Some(("warn", "LKRG fatal error", detail)),
        "FAULT" => Some(("warn", "LKRG error", detail)),
        "DYING" => Some(("warn", "LKRG is being unloaded", detail)),
        "STATE" => Some(("notice", "LKRG setting changed", detail)),
        _ => None,
    }
}

/// Never returns: restarts journalctl if it ever exits.
pub fn run(hub: &Hub) {
    loop {
        let mut cmd = Command::new("journalctl");
        cmd.args(["-k", "-f", "-n", "0", "-o", "json"]);
        cmd.stdout(Stdio::piped()).stderr(Stdio::null());
        if let Ok(mut child) = cmd.spawn() {
            if let Some(out) = child.stdout.take() {
                for line in BufReader::new(out).lines().map_while(Result::ok) {
                    if let Some((sev, title, detail)) = classify(&line) {
                        let dk = format!("lkrg|{sev}|{detail}");
                        hub.emit(sev, "lkrg", title, &detail, Some((&dk, 600)));
                    }
                }
            }
            let _ = child.wait();
        }
        thread::sleep(Duration::from_secs(30));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn j(msg: &str) -> String {
        serde_json::json!({ "MESSAGE": msg, "_TRANSPORT": "kernel" }).to_string()
    }

    #[test]
    fn classifies_lkrg_messages() {
        let (sev, _, d) = classify(&j("LKRG: ALERT: DETECT: Task: Credentials corruption for pid 1234")).unwrap();
        assert_eq!(sev, "crit");
        assert_eq!(d, "DETECT: Task: Credentials corruption for pid 1234");
        assert_eq!(classify(&j("LKRG: DYING: LKRG unloaded")).unwrap().0, "warn");
        assert_eq!(classify(&j("LKRG: FAULT: something")).unwrap().0, "warn");
        assert_eq!(classify(&j("LKRG: STATE: Changing 'kint_enforce' from 0 (log) to 2 (panic)")).unwrap().0, "notice");
    }

    #[test]
    fn ignores_everything_else() {
        assert!(classify(&j("LKRG: ALIVE: System is clean")).is_none());
        assert!(classify(&j("LKRG: ISSUE: minor")).is_none());
        assert!(classify(&j("usb 1-1: new high-speed USB device")).is_none());
        assert!(classify("not json").is_none());
        assert!(classify(r#"{"MESSAGE":[76,75]}"#).is_none());
    }
}
