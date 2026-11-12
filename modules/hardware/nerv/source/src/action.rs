//! NERV <PART> <N> ENABLE | DISABLE | INFO | RESET
//!
//! Every change is runtime-only: rfkill, USB authorization, driver
//! bind/unbind and input inhibit are all forgotten at reboot (the radio
//! switches are remembered by systemd-rfkill, the same as the KDE toggle).

use crate::model::{Control, Device, Part, State};
use crate::report::{self, Paint, Role};
use crate::scan::{self, Policy, Scan};
use crate::state;
use crate::sys::{self, p};
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Action {
    Enable,
    Disable,
    Info,
    Reset,
}

impl Action {
    pub fn parse(s: &str) -> Option<Action> {
        match s.to_ascii_uppercase().as_str() {
            "ENABLE" => Some(Action::Enable),
            "DISABLE" => Some(Action::Disable),
            "INFO" => Some(Action::Info),
            "RESET" => Some(Action::Reset),
            _ => None,
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Action::Enable => "ENABLE",
            Action::Disable => "DISABLE",
            Action::Info => "INFO",
            Action::Reset => "RESET",
        }
    }
}

// ---------------------------------------------------------------- binding ---

pub enum Bind {
    Bound(String),
    NoDriver,
    Blocked(String),
    Failed(String),
}

/// The bus "drivers_probe" file and the name to write into it.
fn probe_target(dev: &Path) -> Option<(PathBuf, String)> {
    let name = sys::file_name(dev);
    if dev.join("config").exists() {
        Some((p("/sys/bus/pci/drivers_probe"), name))
    } else if name.contains(':') && dev.join("bInterfaceClass").exists() {
        Some((p("/sys/bus/usb/drivers_probe"), name))
    } else {
        None
    }
}

/// Loads the driver for one PCI function or USB interface and attaches it.
/// modprobe -b respects the blacklist (hardening.nix), so a blocked driver
/// is never forced in.
pub fn try_bind(policy: &Policy, dev: &Path) -> Bind {
    if let Some(d) = sys::link_name(&dev.join("driver")) {
        return Bind::Bound(d);
    }
    let Some(alias) = sys::read_at(dev, "modalias") else { return Bind::NoDriver };
    let mods = policy.modules_for(&alias);
    if mods.is_empty() {
        return Bind::NoDriver;
    }
    if let Some(m) = policy.blocked(&mods) {
        return Bind::Blocked(m.clone());
    }
    let _ = sys::run("modprobe", &["-b", "-q", &alias], 30);
    for _ in 0..10 {
        if let Some(d) = sys::link_name(&dev.join("driver")) {
            return Bind::Bound(d);
        }
        if let Some((probe, name)) = probe_target(dev) {
            let _ = sys::write(&probe, &name);
        }
        sys::sleep_ms(200);
    }
    Bind::Failed(format!("driver {} did not attach (journalctl -k)", mods.join(", ")))
}

/// What to bind for a device: the PCI function itself, or every USB
/// interface that has no driver.
fn bind_targets(dev: &Path) -> Vec<PathBuf> {
    if dev.join("config").exists() {
        return vec![dev.to_path_buf()];
    }
    let port = sys::file_name(dev);
    sys::list(dev)
        .into_iter()
        .filter(|c| sys::file_name(c).starts_with(&format!("{port}:")) && !c.join("driver").exists())
        .collect()
}

// ----------------------------------------------------------------- safety ---

/// Disabling this would leave no real keyboard (receivers and gaming mice
/// do not count). Allowed, but only after typing DISABLE.
fn last_keyboard(scan: &Scan, d: &Device) -> bool {
    d.part == Part::Keyboard
        && !scan.of(Part::Keyboard).iter().any(|k| k.sys != d.sys && k.state != State::Off && !k.mixed)
}

fn confirm_last_keyboard(paint: &Paint, n: usize) -> bool {
    println!("   {}", paint.c(Role::Rec, "⚠  THIS IS YOUR LAST KEYBOARD"));
    println!("   {}", paint.c(Role::Dim, &format!("Back on: paste  sudo NERV KB {n} ENABLE  with the mouse, or reboot.")));
    print!("   {} ", paint.c(Role::Rec, "Type DISABLE to go on:"));
    let _ = std::io::Write::flush(&mut std::io::stdout());
    let mut line = String::new();
    let _ = std::io::stdin().read_line(&mut line);
    line.trim().eq_ignore_ascii_case("DISABLE")
}

fn refusal(scan: &Scan, d: &Device, action: Action) -> Option<&'static str> {
    let harmful = matches!(action, Action::Disable | Action::Reset);
    if harmful && scan.root_paths.iter().any(|r| r.starts_with(&d.sys)) {
        return Some("THORNS ROOT DISK");
    }
    if d.part == Part::Gpu && harmful {
        return Some("GPUS ARE NOT SWITCHED HERE ── pick a GRUB entry instead");
    }
    if d.part == Part::Fingerprint && d.state == State::NotBound && action == Action::Enable {
        return Some("FINGERPRINT SUPPORT (fprintd) IS NOT ENABLED IN THORNS");
    }
    if d.part == Part::Disk && harmful && d.control == Control::None {
        return Some("INTERNAL DISK ── only USB drives are powered off");
    }
    if d.state == State::Blocked && action == Action::Enable {
        return Some("BLOCKED BY POLICY ── its driver is blacklisted in hardening.nix");
    }
    None
}

// ---------------------------------------------------------------- actions ---

enum Outcome {
    /// Finished; the word to show ("ENABLED", "RESET COMPLETE"...).
    Done(&'static str),
    /// A step failed (already printed).
    Failed,
    /// This part has no such switch.
    Unsupported,
    /// A powered-off drive only comes back when replugged.
    Replug,
}

struct Out<'a> {
    paint: &'a Paint,
    ok: bool,
}

impl Out<'_> {
    fn step(&mut self, what: &str, r: Result<(), String>) {
        println!("{}", report::step(self.paint, what, &r));
        self.ok &= r.is_ok();
    }

    fn end(&self, word: &'static str) -> Outcome {
        if self.ok { Outcome::Done(word) } else { Outcome::Failed }
    }
}

fn bt_power(on: bool) -> Result<(), String> {
    sys::run("bluetoothctl", &["power", if on { "on" } else { "off" }], 5).map(|_| ())
}

fn usb_auth(path: &Path, on: bool) -> Result<(), String> {
    sys::write(&path.join("authorized"), if on { "1" } else { "0" })
}

fn pci_unbind(path: &Path) -> Result<String, String> {
    let drv = sys::link_name(&path.join("driver")).ok_or("no driver attached")?;
    sys::write(&path.join("driver/unbind"), &sys::file_name(path))?;
    Ok(drv)
}

fn pci_probe(path: &Path) -> Result<(), String> {
    let _ = sys::write(&p("/sys/bus/pci/drivers_probe"), &sys::file_name(path));
    for _ in 0..15 {
        if path.join("driver").exists() {
            return Ok(());
        }
        sys::sleep_ms(200);
    }
    Err("driver did not attach (journalctl -k)".into())
}

fn mounted_partitions(disk: &str) -> Vec<String> {
    let mountinfo = sys::read(&p("/proc/self/mountinfo")).unwrap_or_default();
    sys::list(&p(&format!("/sys/block/{disk}")))
        .into_iter()
        .map(|c| sys::file_name(&c))
        .filter(|n| n.starts_with(disk) && n != disk)
        .filter(|n| mountinfo.contains(&format!(" /dev/{n} ")))
        .collect()
}

fn enable(o: &mut Out, policy: &Policy, d: &Device) -> Outcome {
    // Seen but not bound: load and attach the driver.
    if d.state == State::NotBound && !matches!(d.control, Control::Disk(_) | Control::Inhibit(_)) && d.part != Part::Fingerprint {
        let targets = bind_targets(&d.sys);
        if targets.is_empty() {
            o.step("bind driver", Err("nothing to bind".into()));
        }
        for t in targets {
            let r = match try_bind(policy, &t) {
                Bind::Bound(drv) => Ok(drv),
                Bind::NoDriver => Err("no Linux driver matches this device".to_string()),
                Bind::Blocked(m) => Err(format!("{m} is blacklisted")),
                Bind::Failed(e) => Err(e),
            };
            let label = match &r {
                Ok(drv) => format!("bind {drv}"),
                Err(_) => "bind driver".into(),
            };
            o.step(&label, r.map(|_| ()));
        }
        return o.end("BOUND");
    }
    match &d.control {
        Control::Rfkill(rf) => {
            o.step("rfkill unblock", sys::write(&rf.join("soft"), "0"));
            if d.part == Part::Bluetooth && o.ok {
                sys::sleep_ms(800);
                o.step("power on", bt_power(true));
            }
        }
        Control::UsbAuth(path) => {
            o.step("usb authorize", usb_auth(path, true));
        }
        Control::PciBind(path) => {
            o.step("bind driver", pci_probe(path));
        }
        Control::Inhibit(paths) => {
            for path in paths {
                o.step("input uninhibit", sys::write(&path.join("inhibited"), "0"));
            }
        }
        Control::Disk(_) if d.state == State::On => return Outcome::Done("ALREADY ON"),
        Control::Disk(_) => return Outcome::Replug,
        Control::ResetOnly(_) | Control::None => return Outcome::Unsupported,
    }
    o.end("ENABLED")
}

fn disable(o: &mut Out, d: &Device) -> Outcome {
    match &d.control {
        Control::Rfkill(rf) => {
            if d.part == Part::Bluetooth {
                let _ = bt_power(false);
            }
            o.step("rfkill block", sys::write(&rf.join("soft"), "1"));
        }
        Control::UsbAuth(path) => {
            o.step("usb deauthorize", usb_auth(path, false));
        }
        Control::PciBind(path) => {
            let r = pci_unbind(path);
            let label = r.as_ref().map(|drv| format!("unbind {drv}")).unwrap_or_else(|_| "unbind driver".into());
            o.step(&label, r.map(|_| ()));
        }
        Control::Inhibit(paths) => {
            for path in paths {
                o.step("input inhibit", sys::write(&path.join("inhibited"), "1"));
            }
        }
        Control::Disk(name) => {
            for part in mounted_partitions(name) {
                let dev = format!("/dev/{part}");
                o.step(&format!("unmount {part}"), sys::run("udisksctl", &["unmount", "-b", &dev, "--no-user-interaction"], 60).map(|_| ()));
            }
            if o.ok {
                let dev = format!("/dev/{name}");
                o.step("power off", sys::run("udisksctl", &["power-off", "-b", &dev, "--no-user-interaction"], 60).map(|_| ()));
            }
        }
        Control::ResetOnly(_) | Control::None => return Outcome::Unsupported,
    }
    o.end("DISABLED")
}

fn reset(o: &mut Out, d: &Device) -> Outcome {
    if let Control::Inhibit(paths) = &d.control {
        for path in paths {
            o.step("input inhibit", sys::write(&path.join("inhibited"), "1"));
        }
        sys::sleep_ms(300);
        for path in paths {
            o.step("input uninhibit", sys::write(&path.join("inhibited"), "0"));
        }
    } else if matches!(d.control, Control::Disk(_) | Control::None) {
        return Outcome::Unsupported;
    } else if d.sys.join("idVendor").exists() {
        o.step("usb deauthorize", usb_auth(&d.sys, false));
        sys::sleep_ms(700);
        o.step("usb authorize", usb_auth(&d.sys, true));
    } else if d.sys.join("config").exists() {
        match pci_unbind(&d.sys) {
            Ok(drv) => {
                o.step(&format!("unbind {drv}"), Ok(()));
                sys::sleep_ms(500);
                o.step(&format!("bind {drv}"), pci_probe(&d.sys));
            }
            Err(e) => {
                o.step("unbind driver", Err(e));
            }
        }
    } else {
        return Outcome::Unsupported;
    }
    o.end("RESET COMPLETE")
}

pub fn run(paint: &Paint, part: Part, n: usize, action: Action) -> i32 {
    let scan = scan::scan();
    let devs = scan.of(part);
    let Some(d) = (n >= 1).then(|| devs.get(n - 1)).flatten() else {
        let have = if devs.is_empty() { format!("no {} found", part.name()) } else { format!("{} has {}", part.name(), devs.len()) };
        println!("{}", report::refused(paint, &format!("NO {} {n} ── {have}", part.name())));
        return 1;
    };
    if action == Action::Info {
        println!("{}", report::info(paint, d, n));
        return 0;
    }
    println!("{}", report::part_head(paint, d, n));
    if !sys::is_root() {
        let short = part.short().unwrap_or(part.name());
        println!("{}   {}", report::refused(paint, "PERMISSION DENIED"), paint.c(Role::Dim, &format!("── run: sudo NERV {short} {n} {}", action.name())));
        return 1;
    }
    if let Some(why) = refusal(&scan, d, action) {
        println!("{}", report::refused(paint, &format!("REFUSED ── {why}")));
        return 1;
    }
    let policy = Policy::new();
    let mut o = Out { paint, ok: true };
    if action == Action::Disable && last_keyboard(&scan, d) && !confirm_last_keyboard(paint, n) {
        println!("{}", report::verdict(paint, State::On, "CANCELLED", "nothing was changed"));
        return 1;
    }
    let result = match action {
        Action::Enable => enable(&mut o, &policy, d),
        Action::Disable => disable(&mut o, d),
        Action::Reset => reset(&mut o, d),
        Action::Info => unreachable!(),
    };
    let runtime_switch = !matches!(d.control, Control::Rfkill(_));
    match result {
        Outcome::Unsupported => {
            println!("{}", report::refused(paint, &format!("{} IS NOT SUPPORTED FOR {} ── INFO only", action.name(), part.name())));
            1
        }
        Outcome::Replug => {
            println!("{}", report::verdict(paint, State::Off, "POWERED OFF", "unplug and replug the drive to turn it back on"));
            1
        }
        Outcome::Done(word) => {
            match action {
                Action::Disable => {
                    if runtime_switch {
                        let _ = state::mark_disabled(part, &d.sys, &d.name, true);
                    }
                    let dim = if runtime_switch { "(until reboot)" } else { "(radio switch)" };
                    println!("{}", report::verdict(paint, State::Off, word, dim));
                }
                Action::Enable | Action::Reset => {
                    let _ = state::mark_disabled(part, &d.sys, &d.name, false);
                    let dim = if part == Part::Wifi && action == Action::Reset { "reconnecting..." } else { "" };
                    println!("{}", report::verdict(paint, State::On, word, dim));
                }
                Action::Info => {}
            }
            0
        }
        Outcome::Failed => {
            println!("{}", report::verdict(paint, State::NotBound, &format!("{} FAILED", action.name()), ""));
            1
        }
    }
}
