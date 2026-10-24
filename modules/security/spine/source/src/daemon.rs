//! `spine daemon`: the root-side collector (runs as the spined service).

use crate::audit::AuditTail;
use crate::event::{self, Hub};
use crate::paths::Paths;
use crate::{clam, kernel, state, util, VERSION};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

const POLL: Duration = Duration::from_secs(5);
const SLOW: Duration = Duration::from_secs(60);

pub fn run() -> i32 {
    let paths = Paths::system();
    let hub = Arc::new(Hub::new(paths.clone()));
    event::trim_events(&paths.events);
    hub.emit("info", "spine", "SPINE daemon started", &format!("version {VERSION}"), None);

    {
        let hub = Arc::clone(&hub);
        thread::spawn(move || clam::run(&hub));
    }

    let started = util::now();
    let mut tail = AuditTail::new(paths.audit_log.clone());
    let mut last_slow: Option<Instant> = None;
    loop {
        if let Err(e) = tail.poll(&hub) {
            hub.emit("warn", "spine", "Audit reader error", &e.to_string(), Some(("audit-error", 3600)));
        }
        if last_slow.map_or(true, |t| t.elapsed() >= SLOW) {
            last_slow = Some(Instant::now());
            let k = kernel::check(&paths, &hub);
            if let Err(e) = state::write(&paths, &hub, &k, started) {
                hub.emit("warn", "spine", "State update error", &e.to_string(), Some(("state-error", 3600)));
            }
        }
        thread::sleep(POLL);
    }
}
