//! NERV BOOT: runs once per boot as root (nerv-boot.service).
//!
//! Looks for PCI functions and USB interfaces that have a matching driver but
//! no driver attached, and binds them. It only ever ADDS drivers: nothing is
//! unbound, nothing blacklisted is forced in. The result goes to
//! /run/nerv/boot for the report and the login notifier.

use crate::action::{try_bind, Bind};
use crate::scan::{self, Policy};
use crate::state::{self, Boot};
use crate::sys::{self, p};
use std::path::{Path, PathBuf};

/// PCI classes that are normal without a driver (bridges, RAM controllers,
/// signal processing helpers...). Only the classes that matter are checked.
fn pci_matters(class: u32) -> bool {
    let base = class >> 16;
    let sub = class >> 8;
    matches!(base, 0x01 | 0x02 | 0x03 | 0x04 | 0x0d) || sub == 0x0c03 || sub == 0x0c80 || sub == 0x0c05
}

fn usb_matters(class: u32) -> bool {
    matches!(class, 0x01 | 0x02 | 0x03 | 0x06 | 0x07 | 0x08 | 0x0a | 0x0b | 0x0e | 0xe0)
}

fn local_time() -> String {
    sys::run("date", &["+%H:%M:%S"], 3).map(|s| s.trim().to_string()).unwrap_or_else(|_| "?".into())
}

pub fn run() -> i32 {
    if !sys::is_root() {
        eprintln!("NERV BOOT must run as root (it is started by nerv-boot.service)");
        return 1;
    }
    let policy = Policy::new();
    let mut boot = Boot { time: Some(local_time()), ..Boot::default() };

    // (path to bind, name for the report, does a missing driver matter)
    let mut candidates: Vec<(PathBuf, String, bool)> = Vec::new();
    for d in scan::pci_devices().into_iter().filter(|d| d.driver.is_none()) {
        let matters = pci_matters(d.class);
        candidates.push((d.path.clone(), d.name(), matters));
    }
    for u in scan::usb_devices().into_iter().filter(|u| u.authorized) {
        for i in u.ifaces.iter().filter(|i| i.driver.is_none()) {
            candidates.push((i.path.clone(), u.name.clone(), usb_matters(i.class)));
        }
    }

    for (path, name, matters) in candidates {
        match try_bind(&policy, &path) {
            Bind::Bound(drv) => {
                println!("bound    {name} ({drv})");
                boot.bound.push((owner(&path), name));
            }
            Bind::Blocked(m) => {
                println!("blocked  {name} ({m})");
                boot.blocked.push((owner(&path), name, m));
            }
            Bind::Failed(e) => {
                // A driver exists for it, so it is real hardware that should work.
                println!("failed   {name}: {e}");
                boot.failed.push((owner(&path), name, e));
            }
            Bind::NoDriver if matters => {
                println!("nodriver {name}");
                boot.failed.push((owner(&path), name, "no Linux driver matches this device".into()));
            }
            Bind::NoDriver => {}
        }
    }
    // A USB device with several interfaces is one record.
    boot.bound.dedup_by(|a, b| a.0 == b.0);
    boot.failed.dedup_by(|a, b| a.0 == b.0);

    let dir = p(state::DIR);
    if let Err(e) = std::fs::create_dir_all(&dir) {
        eprintln!("NERV BOOT: cannot create {}: {e}", dir.display());
        return 1;
    }
    match sys::write(&dir.join("boot"), &state::format_boot(&boot)) {
        Ok(()) => {
            println!(
                "NERV boot scan: {} bound, {} not bound, {} blocked by policy",
                boot.bound.len(),
                boot.failed.len(),
                boot.blocked.len()
            );
            0
        }
        Err(e) => {
            eprintln!("NERV BOOT: cannot write the boot report: {e}");
            1
        }
    }
}

/// USB interface -> its USB device (what the report lists); PCI stays as is.
fn owner(path: &Path) -> PathBuf {
    if path.join("bInterfaceClass").exists() {
        sys::usb_parent(path.parent().unwrap_or(path)).unwrap_or_else(|| path.to_path_buf())
    } else {
        path.to_path_buf()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn picks_the_classes_that_matter() {
        assert!(pci_matters(0x030000)); // VGA
        assert!(pci_matters(0x028000)); // Wi-Fi
        assert!(pci_matters(0x040380)); // HD audio
        assert!(pci_matters(0x0c0330)); // xHCI USB
        assert!(!pci_matters(0x060400)); // PCI bridge
        assert!(!pci_matters(0x050000)); // RAM controller
        assert!(!pci_matters(0x118000)); // signal processing
        assert!(usb_matters(0x0e));
        assert!(!usb_matters(0xff));
    }
}
