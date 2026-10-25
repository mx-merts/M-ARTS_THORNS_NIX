// Copyright (c) 2026 M-ARTS (https://github.com/mx-merts)
// SPDX-License-Identifier: CC-BY-NC-SA-4.0

//! SPINE - local security event hub.
//!
//! SPINE only observes and reports. It never blocks, kills or deletes anything.
//!
//!   spine daemon   root service: audit log, ClamAV journal, kernel state
//!   spine notify   user service: desktop notifications
//!   spine status   overview (default)
//!   spine log      recent events
//!   spine test     (sudo) write a test event
//!   spine event    (sudo) write an event; used by other tools (thorns-aide)

mod audit;
mod clam;
mod cli;
mod daemon;
mod event;
mod kernel;
mod notify;
mod paths;
mod state;
mod util;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

const USAGE: &str = "\
usage: spine <command>

  status          overview (default)
  log [-n N]      recent events (default 20)
  test            write a test event (needs sudo)
  event <sev> <src> <title> [detail]
                  write an event (needs sudo); sev: info|notice|warn|crit
  daemon          run the collector (root service)
  notify          run desktop notifications (user service)
  version";

/// Rust ignores SIGPIPE, so `spine status | head -n 5` would panic when `head`
/// closes the pipe (and with panic = "abort" that dumps core and triggers the
/// desktop crash reporter). One-shot commands restore the Unix default instead:
/// the process just ends quietly, like any other command-line tool.
fn restore_sigpipe() {
    // SAFETY: signal() with SIG_DFL has no preconditions; this runs once at
    // startup, before any thread is created.
    unsafe { libc::signal(libc::SIGPIPE, libc::SIG_DFL); }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    // The two services keep Rust's default (write errors are handled, not fatal).
    if !matches!(args.first().map(String::as_str), Some("daemon") | Some("notify")) {
        restore_sigpipe();
    }
    let code = match args.first().map(String::as_str) {
        None | Some("status") => cli::status(),
        Some("log") => cli::log(&args[1..]),
        Some("test") => cli::test(),
        Some("event") => cli::event(&args[1..]),
        Some("daemon") => daemon::run(),
        Some("notify") => notify::run(),
        Some("version") | Some("--version") | Some("-V") => {
            println!("spine {VERSION}");
            0
        }
        Some("help") | Some("--help") | Some("-h") => {
            println!("{USAGE}");
            0
        }
        Some(other) => {
            eprintln!("spine: unknown command '{other}'\n\n{USAGE}");
            2
        }
    };
    std::process::exit(code);
}
