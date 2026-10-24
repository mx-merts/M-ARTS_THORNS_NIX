//! ClamAV detections: follows the journal of the ClamAV units and reports
//! every "<path>: <signature> FOUND" line.

use crate::event::Hub;
use serde_json::Value;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

const UNITS: [&str; 3] = [
    "clamav-daemon.service",
    "clamav-clamonacc.service",
    "clamav-weekly-scan.service",
];

/// Returns the message if this journal JSON line is a ClamAV detection.
pub fn detection(line: &str) -> Option<String> {
    let v: Value = serde_json::from_str(line).ok()?;
    let msg = v.get("MESSAGE")?.as_str()?.trim(); // binary messages are arrays: skipped
    (msg.ends_with(" FOUND") && msg.contains(": ")).then(|| msg.to_string())
}

/// Never returns: restarts journalctl if it ever exits.
pub fn run(hub: &Hub) {
    loop {
        let mut cmd = Command::new("journalctl");
        cmd.args(["-f", "-n", "0", "-o", "json"]);
        for u in UNITS {
            cmd.args(["-u", u]);
        }
        cmd.stdout(Stdio::piped()).stderr(Stdio::null());
        if let Ok(mut child) = cmd.spawn() {
            if let Some(out) = child.stdout.take() {
                for line in BufReader::new(out).lines().map_while(Result::ok) {
                    if let Some(msg) = detection(&line) {
                        let dk = format!("clam|{msg}");
                        hub.emit("crit", "clamav", "Malware detected", &msg, Some((&dk, 3600)));
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

    #[test]
    fn recognises_detections_only() {
        let hit = r#"{"MESSAGE":"/home/m-arts/Downloads/eicar.test: Eicar-Test-Signature FOUND","_PID":"1"}"#;
        assert_eq!(
            detection(hit).unwrap(),
            "/home/m-arts/Downloads/eicar.test: Eicar-Test-Signature FOUND"
        );
        assert!(detection(r#"{"MESSAGE":"SelfCheck: Database status OK."}"#).is_none());
        assert!(detection(r#"{"MESSAGE":[1,2,3]}"#).is_none());
        assert!(detection("not json").is_none());
        assert!(detection(r#"{"MESSAGE":"Infected files: 1"}"#).is_none());
    }
}
