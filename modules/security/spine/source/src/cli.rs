//! Command-line views: status, log, test, event.

use crate::event::{read_events, Hub, SEVERITIES, SEV_ORDER};
use crate::paths::Paths;
use crate::util::{local_time, now};
use crate::VERSION;
use serde_json::Value;
use std::fs;
use std::io::ErrorKind;

fn load_events(paths: &Paths) -> Vec<Value> {
    match read_events(&paths.events) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("spine: cannot read {}: {e}", paths.events.display());
            Vec::new()
        }
    }
}

fn str_of<'a>(e: &'a Value, k: &str) -> &'a str {
    e.get(k).and_then(Value::as_str).unwrap_or("")
}

fn print_event(e: &Value) {
    let ts = e.get("ts").and_then(Value::as_u64).unwrap_or(0);
    println!(
        "{}  {:<6} {:<20} {}",
        local_time(ts),
        str_of(e, "sev"),
        str_of(e, "src"),
        str_of(e, "title")
    );
    let detail = str_of(e, "detail");
    if !detail.is_empty() {
        println!("{}{}", " ".repeat(46), detail);
    }
}

pub fn status() -> i32 {
    let paths = Paths::system();
    let st: Value = fs::read(&paths.state)
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or(Value::Null);
    println!("SPINE {VERSION}");

    match st.get("heartbeat").and_then(Value::as_u64).map(|h| now().saturating_sub(h)) {
        Some(age) if age < 180 => println!("daemon     : running (heartbeat {age}s ago)"),
        _ => println!("daemon     : NOT RUNNING (check: systemctl status spined)"),
    }
    let audit_ok = st.get("audit_log").and_then(Value::as_bool).unwrap_or(false);
    println!("audit log  : {}", if audit_ok { "readable by daemon" } else { "not found" });
    if let Some(k) = st.get("kernel").filter(|k| k.is_object()) {
        let (run, inst) = (str_of(k, "running"), str_of(k, "installed"));
        if k.get("pending").and_then(Value::as_bool).unwrap_or(false) {
            println!("kernel     : REBOOT NEEDED (running {run}, installed {inst})");
        } else {
            println!("kernel     : up to date ({run})");
        }
    }

    let events = load_events(&paths);
    let day_ago = now().saturating_sub(86_400);
    let counts: Vec<String> = SEV_ORDER
        .iter()
        .map(|s| {
            let n = events
                .iter()
                .filter(|e| {
                    e.get("ts").and_then(Value::as_u64).unwrap_or(0) >= day_ago && str_of(e, "sev") == *s
                })
                .count();
            format!("{s} {n}")
        })
        .collect();
    println!("last 24h   : {}", counts.join("  "));
    println!(
        "counters   : {} module loads seen, {} duplicate events suppressed",
        st.get("module_loads").and_then(Value::as_u64).unwrap_or(0),
        st.get("suppressed").and_then(Value::as_u64).unwrap_or(0)
    );

    let important: Vec<&Value> = events.iter().filter(|e| str_of(e, "sev") != "info").collect();
    if !important.is_empty() {
        println!("\nrecent events:");
        for e in &important[important.len().saturating_sub(5)..] {
            print_event(e);
        }
    }
    0
}

pub fn log(args: &[String]) -> i32 {
    let mut n = 20usize;
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "-n" => match it.next().and_then(|v| v.parse().ok()) {
                Some(v) => n = v,
                None => {
                    eprintln!("spine: -n needs a number");
                    return 2;
                }
            },
            other => {
                eprintln!("spine: unknown option '{other}' (usage: spine log [-n N])");
                return 2;
            }
        }
    }
    let events = load_events(&Paths::system());
    for e in &events[events.len().saturating_sub(n)..] {
        print_event(e);
    }
    0
}

fn write_event(sev: &str, src: &str, title: &str, detail: &str, what: &str) -> i32 {
    let hub = Hub::new(Paths::system());
    match hub.try_emit(sev, src, title, detail, None) {
        Ok(()) => 0,
        Err(e) if e.kind() == ErrorKind::PermissionDenied => {
            eprintln!("spine: run with sudo: sudo spine {what}");
            1
        }
        Err(e) => {
            eprintln!("spine: cannot write event: {e}");
            1
        }
    }
}

pub fn test() -> i32 {
    let code = write_event("notice", "test", "SPINE test event", "Pipeline check: log -> notification", "test");
    if code == 0 {
        println!("test event written; a notification should appear within a few seconds");
    }
    code
}

/// `spine event <sev> <src> <title> [detail]`: lets other tools report here.
pub fn event(args: &[String]) -> i32 {
    if args.len() < 3 || args.len() > 4 || !SEVERITIES.contains(&args[0].as_str()) {
        eprintln!("usage: spine event <info|notice|warn|crit> <src> <title> [detail]");
        return 2;
    }
    let detail = args.get(3).map(String::as_str).unwrap_or("");
    write_event(&args[0], &args[1], &args[2], detail, "event ...")
}
