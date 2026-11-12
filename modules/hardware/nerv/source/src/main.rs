//! NERV: hardware nervous system for THORNS.
//!
//!   NERV                          full report
//!   NERV <PART>                   one section          (NERV BT)
//!   NERV <PART> <N> <ACTION>      INFO | ENABLE | DISABLE | RESET
//!   NERV CONSOLE                  interactive prompt
//!   NERV BOOT / NERV NOTIFY       used by the systemd services
//!
//! Part and action names are not case sensitive.

mod action;
mod boot;
mod gpu;
mod model;
mod notify;
mod probe;
mod report;
mod scan;
mod state;
mod sys;

use action::Action;
use model::Part;
use report::{Paint, Role};
use std::env;
use std::io::{self, BufRead, Write};

fn help(paint: &Paint) -> String {
    let d = |s: &str| paint.c(Role::Dim, s);
    let rows = [
        ("NERV", "full report", ""),
        ("NERV <PART>", "one section", "NERV BT"),
        ("NERV <PART> <N> INFO", "details of one part", "NERV GPU 2 INFO"),
        ("NERV <PART> <N> ENABLE", "switch on / bind driver", "sudo NERV BT 1 ENABLE"),
        ("NERV <PART> <N> DISABLE", "switch off", "sudo NERV CAM 1 DISABLE"),
        ("NERV <PART> <N> RESET", "unbind + rebind", "sudo NERV WF 1 RESET"),
        ("NERV CONSOLE", "interactive prompt", "sudo NERV CONSOLE"),
    ];
    let mut out = vec![
        format!(" {} {} {} {}", paint.title(), paint.c(Role::Strong, report::VERSION), d("──"), d("HARDWARE NERVOUS SYSTEM")),
        String::new(),
    ];
    for (cmd, what, example) in rows {
        out.push(format!(" {}  {:<24} {}", paint.c(Role::Strong, &format!("{cmd:<24}")), what, d(example)).trim_end().to_string());
    }
    out.push(String::new());
    let parts: Vec<String> = Part::ALL
        .iter()
        .map(|p| match p.short() {
            Some(s) => format!("{} {s}", p.name()),
            None => p.name().to_string(),
        })
        .collect();
    out.push(format!(" {}  {}", d("PARTS"), parts[..7].join(" · ")));
    out.push(format!("        {}", parts[7..].join(" · ")));
    out.push(String::new());
    out.push(d(" <N> is the number in the list. Changes need sudo and last until reboot."));
    out.push(d(" The THORNS root disk, the display GPU and the only keyboard are never switched off."));
    out.join("\n")
}

/// Runs one command line (without the leading "NERV"). Returns the exit code.
fn dispatch(paint: &Paint, words: &[String]) -> i32 {
    let w: Vec<String> = words.iter().map(|s| s.to_ascii_uppercase()).collect();
    let w: Vec<&str> = w.iter().map(String::as_str).collect();
    match w.as_slice() {
        [] | ["LIST"] => {
            println!("{}", report::report(paint, &scan::scan(), None));
            0
        }
        ["HELP"] | ["--HELP"] | ["-H"] => {
            println!("{}", help(paint));
            0
        }
        ["--VERSION"] | ["-V"] | ["VERSION"] => {
            println!("NERV-{}", report::VERSION);
            0
        }
        [part] | ["LIST", part] if Part::parse(part).is_some() => {
            let part = Part::parse(part).unwrap();
            println!("{}", report::report(paint, &scan::scan(), Some(part)));
            0
        }
        [part, n, act] => {
            let Some(part) = Part::parse(part) else {
                return bad(paint, &format!("UNKNOWN PART: {part}"));
            };
            let Ok(n) = n.parse::<usize>() else {
                return bad(paint, &format!("NOT A NUMBER: {n}"));
            };
            let Some(act) = Action::parse(act) else {
                return bad(paint, &format!("UNKNOWN ACTION: {act} (INFO, ENABLE, DISABLE, RESET)"));
            };
            action::run(paint, part, n, act)
        }
        [first, ..] if Part::parse(first).is_none() => bad(paint, &format!("UNKNOWN COMMAND: {}", words.join(" "))),
        _ => bad(paint, &format!("USAGE: NERV <PART> <N> <ACTION>   ({})", words.join(" "))),
    }
}

fn bad(paint: &Paint, text: &str) -> i32 {
    println!("{}   {}", report::refused(paint, text), paint.c(Role::Dim, "NERV HELP"));
    2
}

fn console(paint: &Paint) -> i32 {
    println!(
        " {} {} CONSOLE {}",
        paint.title(),
        paint.c(Role::Strong, report::VERSION),
        paint.c(Role::Dim, "── type HELP, EXIT to leave")
    );
    let stdin = io::stdin();
    let mut line = String::new();
    loop {
        print!(" {} ", paint.c(Role::Title, "NERV ▸"));
        let _ = io::stdout().flush();
        line.clear();
        match stdin.lock().read_line(&mut line) {
            Ok(0) | Err(_) => {
                println!();
                return 0;
            }
            Ok(_) => {}
        }
        let mut words: Vec<String> = line.split_whitespace().map(str::to_string).collect();
        if words.first().is_some_and(|w| w.eq_ignore_ascii_case("NERV")) {
            words.remove(0);
        }
        match words.first().map(|w| w.to_ascii_uppercase()).as_deref() {
            None => continue,
            Some("EXIT") | Some("QUIT") | Some("Q") => return 0,
            Some("CONSOLE") => continue,
            _ => {
                dispatch(paint, &words);
            }
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let paint = Paint::detect();
    let code = match args.first().map(|a| a.to_ascii_uppercase()).as_deref() {
        Some("BOOT") => boot::run(),
        Some("NOTIFY") => notify::run(args.get(1).is_some_and(|a| a.eq_ignore_ascii_case("--dry"))),
        Some("CONSOLE") => console(&paint),
        _ => dispatch(&paint, &args),
    };
    std::process::exit(code);
}
