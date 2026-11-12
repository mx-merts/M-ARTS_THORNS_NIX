//! SKIN NOTIFY: the user's notifier (skin-notify user service). Follows
//! /run/skin/events and turns every new line into a notification.
//! SKIN NOTIFY --DRY prints them instead.

use crate::state;
use crate::sys::{self, p};
use std::process::{Command, Stdio};

fn send(title: &str, body: &str, sticky: bool) {
    let esc = body.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;");
    let _ = Command::new("notify-send")
        .args(["-a", "ᛊᛕᛁȠ", "-i", "network-wireless", "-t", if sticky { "0" } else { "-1" }, title, &esc])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
}

pub fn run(dry: bool) -> i32 {
    let path = p(&format!("{}/events", state::DIR));
    // Start at the end: only what happens from now on.
    let mut seen = std::fs::read_to_string(&path).map(|t| t.lines().count()).unwrap_or(0);
    loop {
        let text = std::fs::read_to_string(&path).unwrap_or_default();
        let lines: Vec<&str> = text.lines().collect();
        if lines.len() < seen {
            seen = 0; // file was recreated (reboot of the daemon)
        }
        for line in &lines[seen..] {
            let f: Vec<&str> = line.splitn(4, '\t').collect();
            if f.len() == 4 {
                let sticky = f[1] == "sticky";
                if dry {
                    println!("[{}] {}\n    {}", f[1], f[2], f[3]);
                } else {
                    send(f[2], f[3], sticky);
                }
            }
        }
        seen = lines.len();
        sys::sleep_ms(1500);
    }
}
