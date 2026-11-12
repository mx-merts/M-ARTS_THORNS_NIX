//! System access: files, external commands with time limits, time.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::OnceLock;
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// Every path SKIN touches goes through `p()`. SKIN_ROOT (tests only) points
/// it at a fake tree instead of "/".
fn root() -> &'static Path {
    static ROOT: OnceLock<PathBuf> = OnceLock::new();
    ROOT.get_or_init(|| match env::var_os("SKIN_ROOT") {
        Some(r) if !r.is_empty() => PathBuf::from(r),
        _ => PathBuf::from("/"),
    })
}

pub fn p(path: &str) -> PathBuf {
    root().join(path.trim_start_matches('/'))
}

/// File contents without surrounding whitespace; None if missing or empty.
pub fn read(path: &Path) -> Option<String> {
    let text = fs::read_to_string(path).ok()?;
    let text = text.trim();
    (!text.is_empty()).then(|| text.to_string())
}

/// Writes through a temporary file, so a reader never sees half a file.
pub fn write_atomic(path: &Path, text: &str) -> Result<(), String> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, text).map_err(|e| e.to_string())?;
    fs::rename(&tmp, path).map_err(|e| e.to_string())
}

pub fn is_root() -> bool {
    let Ok(status) = fs::read_to_string("/proc/self/status") else {
        return false;
    };
    status
        .lines()
        .find(|l| l.starts_with("Uid:"))
        .and_then(|l| l.split_whitespace().nth(2))
        == Some("0")
}

/// Runs a command and returns its stdout. Gives up after `secs` seconds:
/// no step of SKIN may hang.
pub fn run(cmd: &str, args: &[&str], secs: u64) -> Result<String, String> {
    run_input(cmd, args, None, secs)
}

/// Same as `run`, with text fed to the command's stdin (nft -f -).
pub fn run_input(cmd: &str, args: &[&str], input: Option<&str>, secs: u64) -> Result<String, String> {
    let mut child = Command::new(cmd)
        .args(args)
        // systemd gives skin-watch RUNTIME_DIRECTORY=/run/skin (and
        // STATE_DIRECTORY). warp-cli reads RUNTIME_DIRECTORY as the place of
        // warp-svc's socket and then cannot find the daemon: keep them away
        // from every command SKIN runs.
        .env_remove("RUNTIME_DIRECTORY")
        .env_remove("STATE_DIRECTORY")
        .stdin(if input.is_some() { Stdio::piped() } else { Stdio::null() })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("{cmd}: {e}"))?;
    if let (Some(text), Some(mut stdin)) = (input, child.stdin.take()) {
        use std::io::Write;
        let _ = stdin.write_all(text.as_bytes());
    }
    let start = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) if start.elapsed() > Duration::from_secs(secs) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!("no answer in {secs}s"));
            }
            Ok(None) => thread::sleep(Duration::from_millis(20)),
            Err(e) => return Err(e.to_string()),
        }
    }
    let out = child.wait_with_output().map_err(|e| e.to_string())?;
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    if out.status.success() {
        Ok(stdout)
    } else {
        let err = String::from_utf8_lossy(&out.stderr).trim().lines().last().unwrap_or("").to_string();
        Err(if err.is_empty() { format!("{cmd} failed") } else { err })
    }
}

pub fn sleep_ms(ms: u64) {
    thread::sleep(Duration::from_millis(ms));
}

pub fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

/// 133 -> "2:13", 3725 -> "1h 02m".
pub fn duration(secs: u64) -> String {
    if secs >= 3600 {
        format!("{}h {:02}m", secs / 3600, (secs % 3600) / 60)
    } else {
        format!("{}:{:02}", secs / 60, secs % 60)
    }
}

/// Local wall clock "08:42" for an epoch time (via date: no libc needed).
pub fn clock(epoch: u64) -> String {
    run("date", &["-d", &format!("@{epoch}"), "+%H:%M"], 3)
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|_| "?".into())
}

/// Tells systemd the service is ready (Type=notify). Silent if not started
/// by systemd. Abstract sockets start with '@'.
pub fn notify_ready() {
    use std::os::unix::net::UnixDatagram;
    let Some(path) = env::var_os("NOTIFY_SOCKET") else { return };
    let Ok(sock) = UnixDatagram::unbound() else { return };
    let path = path.to_string_lossy().into_owned();
    if let Some(name) = path.strip_prefix('@') {
        use std::os::linux::net::SocketAddrExt;
        if let Ok(addr) = std::os::unix::net::SocketAddr::from_abstract_name(name) {
            let _ = sock.send_to_addr(b"READY=1", &addr);
        }
    } else {
        let _ = sock.send_to(b"READY=1", &path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn durations() {
        assert_eq!(duration(133), "2:13");
        assert_eq!(duration(600), "10:00");
        assert_eq!(duration(3725), "1h 02m");
    }
}
