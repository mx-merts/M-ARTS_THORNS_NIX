//! Reads /var/log/audit/audit.log and turns audit events into SPINE events.
//!
//! The kernel audit rules (security.audit.rules) tag interesting events with a
//! key; this module only classifies them. It never blocks anything.

use crate::event::Hub;
use crate::util::{dec, uname};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, Read, Seek, SeekFrom};
use std::os::unix::fs::MetadataExt;
use std::path::PathBuf;

/// Splits "type=SYSCALL msg=audit(1729.038:231): rest" into (type, serial, rest).
pub fn parse_record(line: &str) -> Option<(&str, &str, &str)> {
    let rest = line.strip_prefix("type=")?;
    let (rtype, rest) = rest.split_once(' ')?;
    let rest = rest.strip_prefix("msg=audit(")?;
    let (stamp, rest) = rest.split_once("): ")?;
    let (_ts, serial) = stamp.split_once(':')?;
    let word = |s: &str| !s.is_empty() && s.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_');
    if !word(rtype) || serial.is_empty() || !serial.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    Some((rtype, serial, rest))
}

/// key=value pairs; values keep their quotes (see `util::dec`). First wins.
pub fn kv(text: &str) -> HashMap<String, String> {
    let b = text.as_bytes();
    let key_char = |c: u8| c.is_ascii_alphanumeric() || c == b'_';
    let mut out = HashMap::new();
    let mut i = 0;
    while i < b.len() {
        if !key_char(b[i]) {
            i += 1;
            continue;
        }
        let ks = i;
        while i < b.len() && key_char(b[i]) {
            i += 1;
        }
        if i >= b.len() || b[i] != b'=' {
            continue;
        }
        let key = &text[ks..i];
        i += 1;
        let vs = i;
        if i < b.len() && b[i] == b'"' {
            i += 1;
            while i < b.len() && b[i] != b'"' {
                i += 1;
            }
            if i < b.len() {
                i += 1;
            }
        } else {
            // Stop at ANY control byte or space. auditd's ENRICHED format puts a
            // 0x1d separator right after the last raw field ("key=(null)<GS>ARCH=...").
            while i < b.len() && b[i] > b' ' {
                i += 1;
            }
        }
        out.entry(key.to_string()).or_insert_with(|| text[vs..i].to_string());
    }
    out
}

/// Title and severity for an audit key.
fn classify(key: &str, from_store: bool, failed: bool) -> (String, &'static str) {
    let (mut title, mut sev) = match key {
        "ssh" => ("/root/.ssh changed".to_string(), if from_store { "warn" } else { "crit" }),
        "boot" => ("Boot partition changed".to_string(), if from_store { "notice" } else { "crit" }),
        "nocer" => ("NOCER state changed".to_string(), "notice"),
        "autostart" => ("Autostart entry changed".to_string(), "warn"),
        "user-systemd" => ("User systemd unit changed".to_string(), "warn"),
        other => (format!("Audit event: {other}"), "notice"),
    };
    if failed {
        title.push_str(" (failed)");
        if sev == "crit" {
            sev = "warn";
        }
    }
    (title, sev)
}

pub struct AuditEvent {
    pub records: Vec<(String, String)>,
}

pub fn handle_event(hub: &Hub, ev: &AuditEvent) {
    let mut syscall = None;
    let (mut paths, mut parents) = (Vec::new(), Vec::new());
    for (rtype, rest) in &ev.records {
        let f = kv(rest);
        match rtype.as_str() {
            "SYSCALL" => syscall = Some(f),
            "PATH" => {
                let name = dec(f.get("name").map(String::as_str));
                if f.get("nametype").map(String::as_str) == Some("PARENT") {
                    parents.push(name);
                } else {
                    paths.push(name);
                }
            }
            _ => {}
        }
    }
    let Some(sc) = syscall else { return };
    let key = sc.get("key").map(|k| k.trim_matches('"')).unwrap_or("");
    if key.is_empty() || key == "(null)" {
        return;
    }

    let comm = dec(sc.get("comm").map(String::as_str));
    let exe = dec(sc.get("exe").map(String::as_str));
    let who = format!(
        "{}/{}",
        uname(sc.get("auid").map(String::as_str)),
        uname(sc.get("uid").map(String::as_str))
    );
    let target = paths.first().or(parents.first()).cloned().unwrap_or_else(|| "?".to_string());
    let failed = sc.get("success").map(String::as_str) == Some("no");
    let from_store = exe.starts_with("/nix/store/");
    let detail = format!("{comm} ({exe}) by {who}: {target}");
    let src = format!("audit:{key}");

    if key == "kernel-modules" {
        // udev/modprobe loads are expected and numerous: only count them.
        hub.count_module_load();
        if !from_store {
            let dk = format!("kernel-modules|{exe}");
            hub.emit("crit", &src, "Kernel module loaded outside the Nix store", &detail, Some((&dk, 3600)));
        }
        return;
    }

    let (title, sev) = classify(key, from_store, failed);
    let dk = format!("audit|{key}|{sev}|{}", if key == "boot" { "" } else { target.as_str() });
    hub.emit(sev, &src, &title, &detail, Some((&dk, 600)));
}

struct Pending {
    records: Vec<(String, String)>,
    cycle: u64,
}

/// Follows audit.log. Records are grouped into events by serial number; an
/// event is complete once a whole poll cycle passed without new records for it.
pub struct AuditTail {
    path: PathBuf,
    file: Option<File>,
    ino: Option<u64>,
    buf: Vec<u8>,
    pending: HashMap<String, Pending>,
    cycle: u64,
}

impl AuditTail {
    pub fn new(path: PathBuf) -> Self {
        AuditTail { path, file: None, ino: None, buf: Vec::new(), pending: HashMap::new(), cycle: 0 }
    }

    fn read_new(&mut self) -> io::Result<Vec<u8>> {
        let Ok(md) = fs::metadata(&self.path) else { return Ok(Vec::new()) };
        let reopen = self.file.is_none() || self.ino != Some(md.ino());
        if reopen {
            let first = self.ino.is_none();
            let mut f = File::open(&self.path)?;
            if first {
                f.seek(SeekFrom::End(0))?; // skip history on first start
            }
            self.file = Some(f);
            self.ino = Some(md.ino());
        } else if let Some(f) = self.file.as_mut() {
            if md.len() < f.stream_position()? {
                f.seek(SeekFrom::Start(0))?; // truncated in place
            }
        }
        let mut data = Vec::new();
        if let Some(f) = self.file.as_mut() {
            f.read_to_end(&mut data)?;
        }
        Ok(data)
    }

    pub fn poll(&mut self, hub: &Hub) -> io::Result<()> {
        self.cycle += 1;
        let data = self.read_new()?;
        self.buf.extend_from_slice(&data);
        if let Some(last_nl) = self.buf.iter().rposition(|&b| b == b'\n') {
            let complete: Vec<u8> = self.buf.drain(..=last_nl).collect();
            for line in String::from_utf8_lossy(&complete).lines() {
                if let Some((rtype, serial, rest)) = parse_record(line) {
                    let p = self
                        .pending
                        .entry(serial.to_string())
                        .or_insert_with(|| Pending { records: Vec::new(), cycle: 0 });
                    p.records.push((rtype.to_string(), rest.to_string()));
                    p.cycle = self.cycle;
                }
            }
        }
        // Always flush: events finish in a LATER cycle than the one that read them.
        self.flush(hub);
        Ok(())
    }

    /// Handles events whose records stopped arriving in an earlier cycle.
    fn flush(&mut self, hub: &Hub) {
        let mut done: Vec<String> = self
            .pending
            .iter()
            .filter(|(_, p)| p.cycle < self.cycle)
            .map(|(s, _)| s.clone())
            .collect();
        done.sort_by_key(|s| s.parse::<u64>().unwrap_or(0));
        for serial in done {
            if let Some(p) = self.pending.remove(&serial) {
                handle_event(hub, &AuditEvent { records: p.records });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::read_events;
    use crate::paths::Paths;
    use std::io::Write;

    const STORE: &str =
        "/nix/store/lnirw7a51gnlhnrakivzg11vh4h5kszy-coreutils-full-9.11/bin/coreutils";

    /// Records of one audit event, in the format auditd writes them.
    fn event_lines(serial: u32, key: &str, exe: &str, comm: &str, name: &str, nametype: &str, success: &str) -> String {
        format!(
            "type=SYSCALL msg=audit(1729600823.038:{serial}): arch=c000003e syscall=257 success={success} exit=3 a0=ffffff9c items=2 ppid=3460 pid=11004 auid=1000 uid=1000 gid=100 euid=1000 tty=pts0 ses=3 comm=\"{comm}\" exe=\"{exe}\" subj=unconfined key=\"{key}\"\n\
             type=CWD msg=audit(1729600823.038:{serial}): cwd=\"/home/m-arts\"\n\
             type=PATH msg=audit(1729600823.038:{serial}): item=0 name=\"/parent/\" inode=1 dev=08:02 nametype=PARENT\n\
             type=PATH msg=audit(1729600823.038:{serial}): item=1 name={name} inode=2 dev=08:02 nametype={nametype}\n\
             type=PROCTITLE msg=audit(1729600823.038:{serial}): proctitle=746F756368\n"
        )
    }

    fn quoted(s: &str) -> String {
        format!("\"{s}\"")
    }

    fn append(path: &std::path::Path, text: &str) {
        let mut f = fs::OpenOptions::new().create(true).append(true).open(path).unwrap();
        f.write_all(text.as_bytes()).unwrap();
    }

    #[test]
    fn parses_records_and_fields() {
        let l = "type=PATH msg=audit(1729600823.038:231): item=1 name=\"/a b/c\" nametype=CREATE";
        let (t, serial, rest) = parse_record(l).unwrap();
        assert_eq!((t, serial), ("PATH", "231"));
        let f = kv(rest);
        assert_eq!(f["name"], "\"/a b/c\"");
        assert_eq!(f["nametype"], "CREATE");
        assert!(parse_record("garbage").is_none());
        assert!(parse_record("type=X msg=audit(1.0:abc): y").is_none());
    }

    #[test]
    fn skips_history_flushes_late_and_dedupes() {
        let p = Paths::temp("tail1");
        append(&p.audit_log, "type=SYSCALL msg=audit(1.000:1): key=\"autostart\" comm=\"old\" exe=\"/x\"\n");
        let hub = Hub::new(p.clone());
        let mut tail = AuditTail::new(p.audit_log.clone());
        tail.poll(&hub).unwrap(); // first start: history is skipped
        tail.poll(&hub).unwrap();
        assert!(read_events(&p.events).is_err() || read_events(&p.events).unwrap().is_empty());

        let path = quoted("/home/m-arts/.config/autostart/.test");
        append(&p.audit_log, &event_lines(231, "autostart", STORE, "touch", &path, "CREATE", "yes"));
        append(&p.audit_log, &event_lines(232, "autostart", STORE, "rm", &path, "DELETE", "yes"));
        tail.poll(&hub).unwrap(); // records read, event not complete yet
        assert!(read_events(&p.events).is_err() || read_events(&p.events).unwrap().is_empty());
        tail.poll(&hub).unwrap(); // NO new data this cycle: must still flush
        let ev = read_events(&p.events).unwrap();
        assert_eq!(ev.len(), 1, "second event has the same path: deduped");
        assert_eq!(ev[0]["sev"], "warn");
        assert_eq!(ev[0]["src"], "audit:autostart");
        assert!(ev[0]["detail"].as_str().unwrap().contains("/home/m-arts/.config/autostart/.test"));
        assert_eq!(hub.stats().0, 1);
    }

    fn run(hub: &Hub, serial: u32, key: &str, exe: &str, name: &str, success: &str) {
        let text = event_lines(serial, key, exe, "x", name, "CREATE", success);
        let records = text
            .lines()
            .filter_map(parse_record)
            .map(|(t, _, r)| (t.to_string(), r.to_string()))
            .collect();
        handle_event(hub, &AuditEvent { records });
    }

    #[test]
    fn classifies_boot_ssh_and_kernel_modules() {
        let p = Paths::temp("classify");
        let hub = Hub::new(p.clone());
        run(&hub, 300, "boot", "/tmp/evil", "\"/boot/grub/grub.cfg\"", "yes");
        run(&hub, 301, "boot", STORE, "\"/boot/grub/grub.cfg\"", "yes"); // other severity: reported too
        run(&hub, 305, "boot", STORE, "\"/boot/grub/grub.cfg\"", "yes"); // same key and severity: deduped
        run(&hub, 302, "kernel-modules", STORE, "\"x\"", "yes"); // counted only
        run(&hub, 303, "kernel-modules", "/home/m-arts/insmod", "\"x\"", "yes");
        // hex encoded name with a space; denied write from outside the store
        run(&hub, 304, "ssh", "/usr/bin/x", "2F726F6F742F2E7373682F6D79206B6579", "no");
        let ev = read_events(&p.events).unwrap();
        let got: Vec<(&str, &str)> = ev.iter().map(|e| (e["sev"].as_str().unwrap(), e["title"].as_str().unwrap())).collect();
        assert_eq!(got[0], ("crit", "Boot partition changed"));
        assert_eq!(got[1], ("notice", "Boot partition changed"));
        assert_eq!(got[2], ("crit", "Kernel module loaded outside the Nix store"));
        assert_eq!(got[3], ("warn", "/root/.ssh changed (failed)"));
        assert!(ev[3]["detail"].as_str().unwrap().contains("/root/.ssh/my key"));
        assert_eq!(ev.len(), 4);
        assert_eq!(hub.stats().1, 2);
    }

    #[test]
    fn boot_from_the_store_is_only_a_notice() {
        let p = Paths::temp("boot");
        let hub = Hub::new(p.clone());
        run(&hub, 1, "boot", STORE, "\"/boot/kernels/x\"", "yes");
        assert_eq!(read_events(&p.events).unwrap()[0]["sev"], "notice");
    }

    #[test]
    fn follows_log_rotation() {
        let p = Paths::temp("rotate");
        append(&p.audit_log, "");
        let hub = Hub::new(p.clone());
        let mut tail = AuditTail::new(p.audit_log.clone());
        tail.poll(&hub).unwrap();
        fs::rename(&p.audit_log, p.audit_log.with_extension("1")).unwrap();
        append(&p.audit_log, &event_lines(600, "nocer", STORE, "nocer", "\"/var/lib/nocer/index.json\"", "CREATE", "yes"));
        tail.poll(&hub).unwrap();
        tail.poll(&hub).unwrap();
        assert_eq!(read_events(&p.events).unwrap()[0]["src"], "audit:nocer");
    }

    #[test]
    fn partial_line_waits_for_its_newline() {
        let p = Paths::temp("partial");
        append(&p.audit_log, "");
        let hub = Hub::new(p.clone());
        let mut tail = AuditTail::new(p.audit_log.clone());
        tail.poll(&hub).unwrap();
        let full = event_lines(700, "ssh", STORE, "nano", "\"/root/.ssh/authorized_keys\"", "CREATE", "yes");
        let (a, b) = full.split_at(full.len() / 2);
        append(&p.audit_log, a);
        tail.poll(&hub).unwrap();
        append(&p.audit_log, b);
        for _ in 0..3 {
            tail.poll(&hub).unwrap();
        }
        let ev = read_events(&p.events).unwrap();
        assert_eq!(ev.len(), 1);
        assert_eq!(ev[0]["sev"], "warn");
    }

    #[test]
    fn finished_events_are_handled_in_serial_order() {
        let p = Paths::temp("order");
        let hub = Hub::new(p.clone());
        let mut tail = AuditTail::new(p.audit_log.clone());
        append(&p.audit_log, "");
        tail.poll(&hub).unwrap();
        let path = quoted("/home/m-arts/.config/autostart/.x");
        for i in 0..20u32 {
            let comm = format!("c{i}");
            append(&p.audit_log, &event_lines(100 + i, "autostart", STORE, &comm, &path, "CREATE", "yes"));
        }
        tail.poll(&hub).unwrap();
        tail.poll(&hub).unwrap();
        let ev = read_events(&p.events).unwrap();
        assert_eq!(ev.len(), 1);
        assert!(ev[0]["detail"].as_str().unwrap().starts_with("c0 "), "the FIRST event must be kept");
    }

    #[test]
    fn a_louder_event_is_not_hidden_by_an_earlier_quieter_one() {
        let p = Paths::temp("masking");
        let hub = Hub::new(p.clone());
        // A normal rebuild write from the Nix store, then (inside the 10 minute
        // window) a write from an unknown binary: both must be reported.
        run(&hub, 1, "boot", STORE, "\"/boot/grub/grub.cfg\"", "yes");
        run(&hub, 2, "boot", "/tmp/evil", "\"/boot/EFI/BOOT/BOOTX64.EFI\"", "yes");
        let ev = read_events(&p.events).unwrap();
        let sevs: Vec<&str> = ev.iter().map(|e| e["sev"].as_str().unwrap()).collect();
        assert_eq!(sevs, vec!["notice", "crit"]);
    }

    #[test]
    fn enriched_extras_do_not_leak_into_values() {
        // ENRICHED log format: "<raw fields><0x1d>ARCH=x86_64 SYSCALL=bpf AUID=..."
        let f = kv("success=yes key=(null)\x1dARCH=x86_64 SYSCALL=bpf AUID=\"unset\"");
        assert_eq!(f["key"], "(null)");
        assert_eq!(f["ARCH"], "x86_64");
        assert_eq!(f["AUID"], "\"unset\"");
        let f = kv("comm=\"touch\" key=\"autostart\"\x1dARCH=x86_64");
        assert_eq!(f["key"], "\"autostart\"");
        assert_eq!(f["ARCH"], "x86_64");
    }

    #[test]
    fn enriched_events_without_a_rule_key_are_ignored() {
        let p = Paths::temp("enriched-nokey");
        let hub = Hub::new(p.clone());
        let sys = "arch=c000003e syscall=321 success=yes exit=3 auid=4294967295 uid=0 comm=\"systemd\" exe=\"/nix/store/x-systemd-261.3/lib/systemd/systemd\" subj=unconfined key=(null)\x1dARCH=x86_64 SYSCALL=bpf AUID=\"unset\" UID=\"root\"";
        handle_event(&hub, &AuditEvent { records: vec![("SYSCALL".to_string(), sys.to_string())] });
        assert!(read_events(&p.events).is_err(), "no event may be written for a record without a rule key");
    }

    #[test]
    fn enriched_events_with_a_rule_key_still_work() {
        let p = Paths::temp("enriched-key");
        let hub = Hub::new(p.clone());
        let text = event_lines(900, "autostart", STORE, "touch", "\"/home/m-arts/.config/autostart/.t\"", "CREATE", "yes")
            .replace("key=\"autostart\"\n", "key=\"autostart\"\x1dARCH=x86_64 SYSCALL=openat AUID=\"m-arts\"\n");
        assert!(text.contains('\x1d'));
        let records = text
            .lines()
            .filter_map(parse_record)
            .map(|(t, _, r)| (t.to_string(), r.to_string()))
            .collect();
        handle_event(&hub, &AuditEvent { records });
        let ev = read_events(&p.events).unwrap();
        assert_eq!(ev.len(), 1);
        assert_eq!(ev[0]["src"], "audit:autostart");
        assert_eq!(ev[0]["title"], "Autostart entry changed");
    }
}
