//! One full look at the machine: header facts, every part, boot results.
//! Reads sysfs/procfs only (works without root); the per-part probes are in
//! probe.rs.

use crate::gpu::{self, Conf, Rec};
use crate::model::{Control, Device, Part, State};
use crate::probe;
use crate::state::{self, Boot};
use crate::sys::{self, p};
use std::cell::OnceCell;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub struct Scan {
    pub devices: Vec<Device>,
    pub machine: String,
    pub kernel: String,
    pub secure_boot: String,
    pub conf: Conf,
    pub rec: Rec,
    pub boot: Boot,
    /// Canonical sysfs paths of the disks THORNS runs from (/, /boot, /nix).
    pub root_paths: Vec<PathBuf>,
}

impl Scan {
    pub fn of(&self, part: Part) -> Vec<&Device> {
        self.devices.iter().filter(|d| d.part == part).collect()
    }
}

// ------------------------------------------------------------ PCI / USB ----

pub struct Pci {
    pub path: PathBuf,
    pub addr: String,
    pub class: u32,
    pub vendor: u32,
    pub device: u32,
    pub sub: Option<String>,
    /// pci.ids name of the exact card (subsystem), when listed.
    pub sub_name: Option<String>,
    pub driver: Option<String>,
    pub vendor_name: Option<String>,
    pub device_name: Option<String>,
}

impl Pci {
    pub fn base_class(&self) -> u32 {
        self.class >> 16
    }
    pub fn sub_class(&self) -> u32 {
        self.class >> 8
    }
    pub fn id(&self) -> String {
        format!("{:04x}:{:04x}", self.vendor, self.device)
    }
    pub fn name(&self) -> String {
        let v = self.vendor_name.as_deref().map(sys::short_vendor).unwrap_or_default();
        match (&self.device_name, v.is_empty()) {
            (Some(d), _) => sys::join_name(&v, &sys::short_device(d)),
            (None, false) => format!("{v} device {:04x}", self.device),
            (None, true) => format!("PCI device {}", self.id()),
        }
    }
    /// Standard INFO lines for a PCI function.
    pub fn describe(&self, d: &mut Device) {
        d.add("VENDOR", self.vendor_name.clone().unwrap_or_default());
        d.add("DEVICE", self.device_name.clone().unwrap_or_default());
        d.add("ID", self.id());
        let sub = match (&self.sub, &self.sub_name) {
            (Some(id), Some(n)) => format!("{id}  {n}"),
            (Some(id), None) => id.clone(),
            _ => String::new(),
        };
        d.add("SUBSYSTEM", sub);
        d.add("BUS", format!("PCI {}", self.addr));
        d.id = Some(self.id());
    }
}

pub fn pci_devices() -> Vec<Pci> {
    sys::list(&p("/sys/bus/pci/devices"))
        .into_iter()
        .filter_map(|link| {
            let path = sys::real(&link);
            let vendor = sys::hex_at(&path, "vendor")?;
            let device = sys::hex_at(&path, "device")?;
            let (vendor_name, device_name) = sys::pci_names(vendor, device);
            let (sv, sd) = (sys::hex_at(&path, "subsystem_vendor"), sys::hex_at(&path, "subsystem_device"));
            let (sub, sub_name) = match (sv, sd) {
                (Some(a), Some(b)) if a != 0 => (Some(format!("{a:04x}:{b:04x}")), sys::pci_subsystem_name(vendor, device, a, b)),
                _ => (None, None),
            };
            Some(Pci {
                addr: sys::file_name(&link),
                class: sys::hex_at(&path, "class").unwrap_or(0),
                driver: sys::link_name(&path.join("driver")),
                path,
                vendor,
                device,
                sub,
                sub_name,
                vendor_name,
                device_name,
            })
        })
        .collect()
}

pub struct UsbIf {
    pub path: PathBuf,
    pub class: u32,
    pub driver: Option<String>,
}

pub struct Usb {
    pub path: PathBuf,
    pub port: String,
    pub vendor: u32,
    pub product: u32,
    pub name: String,
    pub vendor_name: Option<String>,
    pub product_name: Option<String>,
    pub authorized: bool,
    pub speed: Option<String>,
    pub ifaces: Vec<UsbIf>,
}

impl Usb {
    pub fn id(&self) -> String {
        format!("{:04x}:{:04x}", self.vendor, self.product)
    }
    pub fn drivers(&self) -> Option<String> {
        let mut seen: Vec<String> = Vec::new();
        for d in self.ifaces.iter().filter_map(|i| i.driver.clone()) {
            if !seen.contains(&d) {
                seen.push(d);
            }
        }
        (!seen.is_empty()).then(|| seen.join(","))
    }
    pub fn describe(&self, d: &mut Device) {
        d.add("VENDOR", self.vendor_name.clone().unwrap_or_default());
        d.add("DEVICE", self.product_name.clone().unwrap_or_default());
        d.add("ID", self.id());
        d.add("BUS", format!("USB {}{}", self.port, self.speed.as_deref().map(|s| format!(" · {s}")).unwrap_or_default()));
        d.id = Some(self.id());
    }
}

/// "10000" -> "USB 10G", "480" -> "USB 2.0".
pub fn usb_speed(mbps: &str) -> Option<String> {
    let v: f64 = mbps.trim().parse().ok()?;
    Some(match v as u64 {
        0..=12 => "USB 1.1".to_string(),
        13..=480 => "USB 2.0".to_string(),
        n => format!("USB {}G", n / 1000),
    })
}

pub fn usb_devices() -> Vec<Usb> {
    let base = p("/sys/bus/usb/devices");
    let entries = sys::list(&base);
    entries
        .iter()
        .filter_map(|link| {
            let port = sys::file_name(link);
            if port.contains(':') || port.starts_with("usb") {
                return None; // interfaces and root hubs
            }
            let path = sys::real(link);
            let vendor = sys::hex_at(&path, "idVendor")?;
            let product = sys::hex_at(&path, "idProduct")?;
            if sys::hex_at(&path, "bDeviceClass") == Some(0x09) {
                return None; // hubs
            }
            let (db_vendor, db_product) = sys::usb_names(vendor, product);
            let vendor_name = sys::read_at(&path, "manufacturer").or(db_vendor);
            let product_name = sys::read_at(&path, "product").or(db_product);
            let v = vendor_name.as_deref().map(sys::short_vendor).unwrap_or_default();
            let name = match &product_name {
                Some(pn) => sys::join_name(&v, pn),
                None if !v.is_empty() => format!("{v} device {product:04x}"),
                None => format!("USB device {vendor:04x}:{product:04x}"),
            };
            let prefix = format!("{port}:");
            let ifaces = entries
                .iter()
                .filter(|e| sys::file_name(e).starts_with(&prefix))
                .map(|e| {
                    let ip = sys::real(e);
                    UsbIf {
                        class: sys::hex_at(&ip, "bInterfaceClass").unwrap_or(0),
                        driver: sys::link_name(&ip.join("driver")),
                        path: ip,
                    }
                })
                .collect();
            Some(Usb {
                authorized: sys::read_at(&path, "authorized").as_deref() != Some("0"),
                speed: sys::read_at(&path, "speed").and_then(|s| usb_speed(&s)),
                path,
                port,
                vendor,
                product,
                name,
                vendor_name,
                product_name,
                ifaces,
            })
        })
        .collect()
}

// ----------------------------------------------------------- driver policy ---

/// Answers "is there a driver for this, and is it blacklisted?" with modprobe.
pub struct Policy {
    blacklist: OnceCell<HashSet<String>>,
}

fn norm(m: &str) -> String {
    m.trim().replace('-', "_")
}

impl Policy {
    pub fn new() -> Policy {
        Policy { blacklist: OnceCell::new() }
    }

    fn blacklist(&self) -> &HashSet<String> {
        self.blacklist.get_or_init(|| {
            let text = sys::run("modprobe", &["-c"], 10).unwrap_or_default();
            let mut set = HashSet::new();
            for line in text.lines() {
                let mut w = line.split_whitespace();
                match (w.next(), w.next(), w.next()) {
                    (Some("blacklist"), Some(m), _) => {
                        set.insert(norm(m));
                    }
                    // "install foo /bin/false": hard block
                    (Some("install"), Some(m), Some(cmd)) if cmd.ends_with("/false") || cmd.ends_with("/true") => {
                        set.insert(norm(m));
                    }
                    _ => {}
                }
            }
            set
        })
    }

    /// Kernel modules that claim this modalias.
    pub fn modules_for(&self, alias: &str) -> Vec<String> {
        sys::run("modprobe", &["-R", alias], 5)
            .unwrap_or_default()
            .lines()
            .map(norm)
            .filter(|m| !m.is_empty())
            .collect()
    }

    pub fn blocked<'a>(&self, modules: &'a [String]) -> Option<&'a String> {
        modules.iter().find(|m| self.blacklist().contains(*m))
    }

    /// State, note and reason for a device that has no driver attached.
    pub fn unbound(&self, sys_path: &Path) -> (State, String, String) {
        let Some(alias) = sys::read_at(sys_path, "modalias") else {
            return (State::NotBound, "NO DRIVER".into(), "no Linux driver matches this device".into());
        };
        let mods = self.modules_for(&alias);
        if mods.is_empty() {
            return (State::NotBound, "NO DRIVER".into(), "no Linux driver matches this device".into());
        }
        if let Some(m) = self.blocked(&mods) {
            return (State::Blocked, "BLOCKED BY POLICY".into(), format!("{m} is blacklisted (hardening.nix)"));
        }
        (
            State::NotBound,
            "NOT BOUND".into(),
            format!("driver {} exists but did not attach (journalctl -k)", mods.join(", ")),
        )
    }
}

// ----------------------------------------------------------------- header ----

fn junk(s: &str) -> bool {
    let l = s.to_ascii_lowercase();
    ["to be filled", "default string", "system product", "system version", "not applicable", "none", "o.e.m", "oem", "123456789", "type1"]
        .iter()
        .any(|j| l.contains(j))
        || l == "1.0"
        || l == "0"
}

fn machine() -> String {
    let dmi = p("/sys/class/dmi/id");
    let get = |k: &str| sys::read_at(&dmi, k).filter(|v| !junk(v));
    // "CASPER BILGISAYAR SISTEMLERI" -> "CASPER": long company names keep their first word.
    let vendor = get("sys_vendor")
        .map(|v| sys::short_vendor(&v))
        .map(|v| if v.len() > 14 { v.split_whitespace().next().unwrap_or("").to_string() } else { v })
        .unwrap_or_default();
    let product = get("product_name");
    let version = get("product_version");
    let board = get("board_name");
    let mut out = match (product, version) {
        // Lenovo: product_name is a part number, product_version the model.
        (Some(p), Some(v)) if vendor.eq_ignore_ascii_case("lenovo") => format!("{v} ({p})"),
        (Some(p), Some(v)) if !p.contains(&v) && v.len() <= 16 => format!("{p} ({v})"),
        (Some(p), _) => p,
        (None, _) => board.unwrap_or_else(|| "UNKNOWN MACHINE".into()),
    };
    if !vendor.is_empty() && !out.to_lowercase().starts_with(&vendor.to_lowercase()) {
        out = format!("{vendor} {out}");
    }
    out
}

fn secure_boot() -> String {
    if !p("/sys/firmware/efi").exists() {
        return "LEGACY BIOS".into();
    }
    let vars = p("/sys/firmware/efi/efivars");
    let file = sys::list(&vars).into_iter().find(|f| sys::file_name(f).starts_with("SecureBoot-"));
    match file.and_then(|f| std::fs::read(f).ok()) {
        Some(b) if b.len() >= 5 && b[4] == 1 => "ON".into(),
        Some(_) => "OFF".into(),
        None => "UNKNOWN".into(),
    }
}

// ------------------------------------------------------------------- scan ----

pub fn scan() -> Scan {
    let conf = gpu::load_conf();
    let boot = state::load_boot();
    let disabled = state::load_disabled();
    let policy = Policy::new();
    let pci = pci_devices();
    let usb = usb_devices();
    let mut ctx = probe::Ctx::new(&pci, &usb, &policy, &boot);

    // Parts that a disabled entry belongs to are not re-listed as plain USB.
    for d in &disabled {
        if d.part != Part::Usb {
            ctx.claimed.insert(d.path.clone());
        }
    }

    let mut devices = Vec::new();
    let gpus = probe::gpus(&mut ctx, &mut devices);
    probe::audio(&mut ctx, &mut devices);
    probe::network(&mut ctx, &mut devices);
    probe::bluetooth(&mut ctx, &mut devices);
    probe::cameras(&mut ctx, &mut devices);
    probe::inputs(&mut ctx, &mut devices);
    probe::fingerprints(&mut ctx, &mut devices);
    let root_paths = probe::disks(&mut ctx, &mut devices);
    probe::usb_rest(&mut ctx, &mut devices);
    probe::thunderbolt(&mut ctx, &mut devices);
    probe::batteries(&mut devices);

    // Bound at boot by NERV.
    for d in devices.iter_mut() {
        if d.state == State::On && boot.was_bound(&d.sys) {
            d.state = State::BoundByNerv;
            d.note = "BOUND BY NERV".into();
        }
    }
    // Switched off with NERV ... DISABLE.
    for dis in &disabled {
        match devices.iter_mut().find(|d| d.sys == dis.path) {
            Some(d) => {
                d.state = State::Off;
                d.note = "DISABLED (until reboot)".into();
                d.reason = None;
            }
            None if dis.path.exists() || dis.part == Part::Disk => {
                let mut d = Device::new(dis.part, dis.name.clone(), dis.path.clone());
                d.state = State::Off;
                d.note = if dis.part == Part::Disk { "POWERED OFF (replug)".into() } else { "DISABLED (until reboot)".into() };
                d.control = if dis.path.join("authorized").exists() {
                    Control::UsbAuth(dis.path.clone())
                } else if dis.path.join("config").exists() {
                    Control::PciBind(dis.path.clone())
                } else {
                    Control::None
                };
                devices.push(d);
            }
            None => {}
        }
    }
    devices.sort_by_key(|d| Part::ALL.iter().position(|p| *p == d.part));
    for d in devices.iter_mut() {
        probe::finish_info(d);
    }

    let rec = gpu::recommend(&gpus, &conf);
    Scan {
        devices,
        machine: machine(),
        kernel: sys::read(&p("/proc/sys/kernel/osrelease")).unwrap_or_else(|| "?".into()),
        secure_boot: secure_boot(),
        conf,
        rec,
        boot,
        root_paths,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn usb_speeds() {
        assert_eq!(usb_speed("480").as_deref(), Some("USB 2.0"));
        assert_eq!(usb_speed("5000").as_deref(), Some("USB 5G"));
        assert_eq!(usb_speed("10000").as_deref(), Some("USB 10G"));
        assert_eq!(usb_speed("12").as_deref(), Some("USB 1.1"));
        assert_eq!(usb_speed("1.5").as_deref(), Some("USB 1.1"));
    }

    #[test]
    fn junk_dmi_values() {
        assert!(junk("To Be Filled By O.E.M."));
        assert!(junk("Default string"));
        assert!(!junk("TUF Gaming F15"));
    }
}
