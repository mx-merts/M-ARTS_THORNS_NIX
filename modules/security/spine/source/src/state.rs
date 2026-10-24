//! state.json: a heartbeat plus a few counters, read by `spine status`.

use crate::event::Hub;
use crate::kernel::KernelInfo;
use crate::paths::Paths;
use crate::util;
use serde_json::{json, Value};
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::os::unix::fs::{chown, OpenOptionsExt};

pub fn write(paths: &Paths, hub: &Hub, kernel: &Option<KernelInfo>, started: u64) -> io::Result<()> {
    let (suppressed, module_loads) = hub.stats();
    let kernel_json = match kernel {
        Some(k) => json!({"running": k.running, "installed": k.installed, "pending": k.pending}),
        None => Value::Null,
    };
    let st = json!({
        "version": crate::VERSION,
        "started": started,
        "heartbeat": util::now(),
        "kernel": kernel_json,
        "module_loads": module_loads,
        "suppressed": suppressed,
        "audit_log": paths.audit_log.exists(),
    });
    let tmp = paths.state.with_extension("json.tmp");
    let mut f = OpenOptions::new().create(true).write(true).truncate(true).mode(0o640).open(&tmp)?;
    f.write_all(st.to_string().as_bytes())?;
    drop(f);
    let _ = chown(&tmp, Some(0), util::wheel_gid()); // readable by the wheel group
    fs::rename(&tmp, &paths.state)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_readable_state() {
        let p = Paths::temp("state");
        let hub = Hub::new(p.clone());
        let k = Some(KernelInfo { running: "1".into(), installed: "2".into(), pending: true });
        write(&p, &hub, &k, 42).unwrap();
        let v: Value = serde_json::from_slice(&fs::read(&p.state).unwrap()).unwrap();
        assert_eq!(v["started"], 42);
        assert_eq!(v["kernel"]["pending"], true);
        assert_eq!(v["audit_log"], false);
        write(&p, &hub, &None, 42).unwrap();
        let v: Value = serde_json::from_slice(&fs::read(&p.state).unwrap()).unwrap();
        assert!(v["kernel"].is_null());
    }
}
