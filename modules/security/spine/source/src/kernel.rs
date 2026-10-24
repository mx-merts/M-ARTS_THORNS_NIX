//! "Reboot needed" detection: compares the running kernel with the newest
//! generation (what the next boot will run).

use crate::event::Hub;
use crate::paths::Paths;
use crate::util::kernel_version;
use std::fs;

pub struct KernelInfo {
    pub running: String,
    pub installed: String,
    pub pending: bool,
}

pub fn check(paths: &Paths, hub: &Hub) -> Option<KernelInfo> {
    if !(paths.booted_kernel.exists() && paths.latest_kernel.exists()) {
        return None;
    }
    let booted = fs::canonicalize(&paths.booted_kernel).ok()?;
    let latest = fs::canonicalize(&paths.latest_kernel).ok()?;
    let info = KernelInfo {
        running: kernel_version(&booted.to_string_lossy()),
        installed: kernel_version(&latest.to_string_lossy()),
        pending: booted != latest,
    };
    if info.pending {
        let dk = format!("kernel|{}", latest.display());
        hub.emit(
            "warn",
            "kernel",
            "Kernel update pending: reboot to apply",
            &format!("running {}, installed {}", info.running, info.installed),
            Some((&dk, 86_400)),
        );
    }
    Some(info)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::read_events;
    use std::os::unix::fs::symlink;

    fn kernel(dir: &std::path::Path, ver: &str) -> std::path::PathBuf {
        let d = dir.join(format!("x-linux-{ver}"));
        fs::create_dir_all(&d).unwrap();
        fs::write(d.join("bzImage"), "").unwrap();
        d.join("bzImage")
    }

    #[test]
    fn detects_pending_kernel_once_per_day() {
        let p = Paths::temp("kernel");
        let dir = p.booted_kernel.parent().unwrap().to_path_buf();
        symlink(kernel(&dir, "6.18.56"), &p.booted_kernel).unwrap();
        symlink(kernel(&dir, "6.18.57"), &p.latest_kernel).unwrap();
        let hub = Hub::new(p.clone());
        let info = check(&p, &hub).unwrap();
        assert!(info.pending);
        assert_eq!((info.running.as_str(), info.installed.as_str()), ("6.18.56", "6.18.57"));
        check(&p, &hub);
        let ev = read_events(&p.events).unwrap();
        assert_eq!(ev.len(), 1);
        assert_eq!(ev[0]["detail"], "running 6.18.56, installed 6.18.57");
    }

    #[test]
    fn up_to_date_and_missing() {
        let p = Paths::temp("kernel2");
        let hub = Hub::new(p.clone());
        assert!(check(&p, &hub).is_none()); // nothing to compare
        let dir = p.booted_kernel.parent().unwrap().to_path_buf();
        let k = kernel(&dir, "6.18.56");
        symlink(&k, &p.booted_kernel).unwrap();
        symlink(&k, &p.latest_kernel).unwrap();
        assert!(!check(&p, &hub).unwrap().pending);
        assert!(read_events(&p.events).is_err());
    }
}
