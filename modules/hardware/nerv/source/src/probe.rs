//! One function per part. Each finds its devices in sysfs, fills in name,
//! driver, state, note, INFO lines and how NERV can switch it, and "claims"
//! the USB devices it used so they are not listed again under USB.

use crate::gpu::{self, Gpu};
use crate::model::{Control, Device, Part, State};
use crate::scan::{Pci, Policy, Usb};
use crate::state::Boot;
use crate::sys::{self, p};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

pub struct Ctx<'a> {
    pub pci: &'a [Pci],
    pub usb: &'a [Usb],
    pub policy: &'a Policy,
    pub boot: &'a Boot,
    /// USB devices already listed under a part.
    pub claimed: HashSet<PathBuf>,
    /// USB device -> input kinds found under it ("MOUSE", "GAMEPAD"...).
    pub input_kinds: HashMap<PathBuf, Vec<&'static str>>,
}

impl<'a> Ctx<'a> {
    pub fn new(pci: &'a [Pci], usb: &'a [Usb], policy: &'a Policy, boot: &'a Boot) -> Ctx<'a> {
        Ctx { pci, usb, policy, boot, claimed: HashSet::new(), input_kinds: HashMap::new() }
    }

    fn usb_at(&self, path: &Path) -> Option<&'a Usb> {
        self.usb.iter().find(|u| u.path == path)
    }

    fn pci_at(&self, path: &Path) -> Option<&'a Pci> {
        self.pci.iter().find(|d| d.path == path)
    }

    /// Marks an unbound device: boot scan's reason first, live check second.
    fn mark_unbound(&self, d: &mut Device, probe_path: &Path) {
        if let Some((_, _, reason)) = self.boot.failed.iter().find(|(p, _, _)| p == probe_path || p == &d.sys) {
            d.state = State::NotBound;
            d.note = "NOT BOUND".into();
            d.reason = Some(reason.clone());
            return;
        }
        let (state, note, reason) = self.policy.unbound(probe_path);
        d.state = state;
        d.note = note;
        d.reason = Some(reason);
    }

    /// Standard device for a USB device that a part owns.
    fn usb_device(&mut self, part: Part, u: &Usb) -> Device {
        let mut d = Device::new(part, u.name.clone(), u.path.clone());
        u.describe(&mut d);
        d.driver = u.drivers();
        d.control = Control::UsbAuth(u.path.clone());
        if !u.authorized {
            d.state = State::Off;
            d.note = "DISABLED".into();
        }
        self.claimed.insert(u.path.clone());
        d
    }

    /// Standard device for a PCI function that a part owns.
    fn pci_device(&self, part: Part, pci: &Pci) -> Device {
        let mut d = Device::new(part, pci.name(), pci.path.clone());
        pci.describe(&mut d);
        d.driver = pci.driver.clone();
        d.control = Control::PciBind(pci.path.clone());
        if d.driver.is_none() {
            self.mark_unbound(&mut d, &pci.path);
        }
        d
    }

    /// USB devices with an interface of `class` and no driver on it.
    fn unbound_usb_with_class(&self, class: u32) -> Vec<&'a Usb> {
        self.usb
            .iter()
            .filter(|u| !self.claimed.contains(&u.path))
            .filter(|u| u.ifaces.iter().any(|i| i.class == class && i.driver.is_none()))
            .filter(|u| u.ifaces.iter().all(|i| i.driver.is_none()))
            .collect()
    }

    /// A USB device with no driver on any interface, as a NotBound part.
    fn unbound_usb(&mut self, part: Part, u: &Usb, class: u32) -> Device {
        let mut d = self.usb_device(part, u);
        let iface = u.ifaces.iter().find(|i| i.class == class).map(|i| i.path.clone());
        if let Some(i) = iface {
            self.mark_unbound(&mut d, &i);
        }
        d
    }
}

fn driver_version(driver: &str) -> Option<String> {
    sys::read(&p(&format!("/sys/module/{}/version", driver.replace('-', "_"))))
}

fn driver_line(d: &Device) -> String {
    match &d.driver {
        Some(drv) => match driver_version(drv) {
            Some(v) => format!("{drv} {v}"),
            None => drv.clone(),
        },
        None => "──".into(),
    }
}

/// Fills the DRIVER / STATE / REASON lines every part shares (call last).
pub fn finish_info(d: &mut Device) {
    let mut head = vec![("STATE".to_string(), format!("{} {}", d.state.symbol(), d.note))];
    head.push(("DRIVER".to_string(), driver_line(d)));
    if let Some(r) = &d.reason {
        head.push(("REASON".to_string(), r.clone()));
    }
    head.append(&mut d.info);
    d.info = head;
}

// --------------------------------------------------------------------- GPU ---

pub fn gpus(ctx: &mut Ctx, out: &mut Vec<Device>) -> Vec<Gpu> {
    // Which GPU has a connected screen.
    let mut connected: HashSet<PathBuf> = HashSet::new();
    let mut any_connector = false;
    for e in sys::list(&p("/sys/class/drm")) {
        let name = sys::file_name(&e);
        let Some((card, _)) = name.split_once('-') else { continue };
        any_connector = true;
        if sys::read_at(&e, "status").as_deref() == Some("connected") {
            connected.insert(sys::real(&p(&format!("/sys/class/drm/{card}/device"))));
        }
    }
    let mut gpus = Vec::new();
    for pci in ctx.pci.iter().filter(|d| d.base_class() == 0x03) {
        let mut d = ctx.pci_device(Part::Gpu, pci);
        d.control = Control::None; // never switched off from here
        let display = if any_connector {
            connected.contains(&pci.path)
        } else {
            sys::read_at(&pci.path, "boot_vga").as_deref() == Some("1")
        };
        let runtime = sys::read_at(&pci.path, "power/runtime_status");
        let sleeping = runtime.as_deref() == Some("suspended");
        let arch = (pci.vendor == 0x10de).then(|| gpu::nvidia_arch(pci.device, pci.device_name.as_deref()));
        if d.driver.is_some() {
            d.note = if display {
                "DISPLAY".into()
            } else if d.driver.as_deref() == Some("nvidia") {
                "OFFLOAD".into()
            } else {
                "SECONDARY".into()
            };
            if sleeping && !display {
                d.note += " · SLEEPING";
            }
        }
        if let Some(a) = arch {
            let entries = match a.family() {
                gpu::Family::Desk => "DESK (nouveau)",
                gpu::Family::Legacy => "NVIDIA-L",
                gpu::Family::Modern => "NVIDIA, PRIME",
            };
            d.add("ARCH", format!("{} ── supported by {entries}", a.name()));
        }
        let power = sys::read_at(&pci.path, "power_state").unwrap_or_default();
        let rt = runtime.map(|r| format!(" · runtime {r}")).unwrap_or_default();
        d.add("POWER", format!("{power}{rt}").trim_start_matches(" · ").to_string());
        if d.driver.as_deref() == Some("nvidia") && !display {
            d.add("RUN WITH", "nvidia-offload <program>");
        }
        gpus.push(Gpu { addr: pci.addr.clone(), vendor: pci.vendor, arch });
        out.push(d);
    }
    gpus
}

// ------------------------------------------------------------------- AUDIO ---

struct Card {
    num: String,
    id: String,
    long: Option<String>,
    dev: PathBuf,
}

fn sound_cards() -> Vec<Card> {
    let longs = sys::read(&p("/proc/asound/cards")).unwrap_or_default();
    sys::list(&p("/sys/class/sound"))
        .into_iter()
        .filter_map(|e| {
            let name = sys::file_name(&e);
            let num = name.strip_prefix("card")?;
            if num.is_empty() || !num.chars().all(|c| c.is_ascii_digit()) {
                return None;
            }
            // /proc/asound/cards: " 0 [PCH            ]: HDA-Intel - HDA Intel PCH"
            let long = longs.lines().find_map(|l| {
                let t = l.trim_start();
                let (n, rest) = t.split_once(' ')?;
                (n == num).then(|| rest.split_once(" - ").map(|(_, x)| x.trim().to_string()))?
            });
            Some(Card {
                num: num.to_string(),
                id: sys::read_at(&e, "id").unwrap_or_default(),
                long,
                dev: sys::real(&e.join("device")),
            })
        })
        .collect()
}

pub fn audio(ctx: &mut Ctx, out: &mut Vec<Device>) {
    let cards = sound_cards();
    for pci in ctx.pci.iter().filter(|d| d.base_class() == 0x04 && matches!(d.sub_class() & 0xff, 0x01 | 0x03)) {
        let mut d = ctx.pci_device(Part::Audio, pci);
        let mine: Vec<&Card> = cards.iter().filter(|c| c.dev == pci.path).collect();
        if d.driver.is_some() {
            d.control = Control::ResetOnly(pci.path.clone());
            if mine.is_empty() {
                d.state = State::NotBound;
                d.note = "NO SOUND CARD".into();
                d.reason = Some(format!(
                    "driver {} loaded but made no sound card: firmware or SOF topology missing? (journalctl -k | grep -i sof)",
                    d.driver.clone().unwrap_or_default()
                ));
            } else {
                let hdmi = pci.vendor == 0x10de
                    || pci.vendor == 0x1002
                    || pci.device_name.as_deref().is_some_and(|n| n.contains("HDMI") || n.contains("DisplayPort"));
                let nums: Vec<&str> = mine.iter().map(|c| c.num.as_str()).collect();
                d.note = format!("CARD {} · {}", nums.join(","), if hdmi { "HDMI" } else { "BUILT-IN" });
            }
        }
        for c in &mine {
            d.add(&format!("CARD {}", c.num), c.long.clone().unwrap_or_else(|| c.id.clone()));
        }
        out.push(d);
    }
    // USB sound cards (headsets, DACs).
    for c in &cards {
        let Some(up) = sys::usb_parent(&c.dev) else { continue };
        let Some(u) = ctx.usb_at(&up) else { continue };
        if ctx.claimed.contains(&up) {
            continue;
        }
        let mut d = ctx.usb_device(Part::Audio, u);
        if d.state == State::On {
            d.note = format!("CARD {} · USB", c.num);
        }
        d.add(&format!("CARD {}", c.num), c.long.clone().unwrap_or_else(|| c.id.clone()));
        out.push(d);
    }
    for u in ctx.unbound_usb_with_class(0x01) {
        let d = ctx.unbound_usb(Part::Audio, u, 0x01);
        out.push(d);
    }
}

// ----------------------------------------------------------------- NETWORK ---

fn pci_vendor_short(pci: Option<&Pci>) -> String {
    pci.and_then(|p| p.vendor_name.as_deref()).map(sys::short_vendor).unwrap_or_default()
}

fn rfkill_in(dir: &Path) -> Option<PathBuf> {
    sys::list(dir).into_iter().find(|e| sys::file_name(e).starts_with("rfkill")).map(|e| sys::real(&e))
}

/// (soft blocked, hard blocked)
fn rfkill_state(rf: &Path) -> (bool, bool) {
    (sys::read_at(rf, "soft").as_deref() == Some("1"), sys::read_at(rf, "hard").as_deref() == Some("1"))
}

fn nm_connection(iface: &str) -> Option<String> {
    let out = sys::run("nmcli", &["-g", "GENERAL.CONNECTION", "device", "show", iface], 3).ok()?;
    let c = out.trim();
    (!c.is_empty() && c != "--").then(|| c.to_string())
}

pub fn network(ctx: &mut Ctx, out: &mut Vec<Device>) {
    let mut seen: HashSet<PathBuf> = HashSet::new();
    for e in sys::list(&p("/sys/class/net")) {
        let iface = sys::file_name(&e);
        let link = e.join("device");
        if iface == "lo" || !link.exists() {
            continue; // virtual: bridges, tunnels, WARP, VPNs
        }
        let dev = sys::real(&link);
        let wifi = e.join("wireless").exists() || e.join("phy80211").exists();
        let part = if wifi { Part::Wifi } else { Part::Ethernet };
        let owner_usb = sys::usb_parent(&dev).and_then(|up| ctx.usb_at(&up));
        let owner_pci = sys::pci_parent(&dev).and_then(|pp| ctx.pci_at(&pp));
        let mut d = match (owner_usb, owner_pci) {
            (Some(u), _) => ctx.usb_device(part, u),
            (None, Some(pci)) => ctx.pci_device(part, pci),
            (None, None) => Device::new(part, iface.clone(), dev.clone()),
        };
        if !seen.insert(d.sys.clone()) {
            continue;
        }
        // Intel Wi-Fi chips are named by the chipset ("Raptor Lake PCH CNVi WiFi");
        // the subsystem name is the actual card ("Wi-Fi 6E AX211 160MHz").
        if let Some(sub) = owner_pci.filter(|_| owner_usb.is_none()).and_then(|pci| pci.sub_name.clone()) {
            let v = pci_vendor_short(owner_pci);
            d.name = sys::join_name(&v, &sys::short_device(&sub));
        }
        d.driver = sys::link_name(&link.join("driver")).or(d.driver);
        let conn = nm_connection(&iface);
        let up = sys::read_at(&e, "operstate").as_deref() == Some("up");
        if wifi {
            let rf = rfkill_in(&e.join("phy80211"));
            if let Some(rf) = &rf {
                let (soft, hard) = rfkill_state(rf);
                d.control = Control::Rfkill(rf.clone());
                if hard {
                    d.state = State::Off;
                    d.note = "HARDWARE SWITCH OFF".into();
                } else if soft {
                    d.state = State::Off;
                    d.note = "OFF".into();
                }
            }
            if d.state == State::On {
                d.note = match (&conn, up) {
                    (Some(c), true) => format!("CONNECTED · {c}"),
                    _ => "DISCONNECTED".into(),
                };
            }
        } else if d.state == State::On {
            let carrier = sys::read_at(&e, "carrier");
            d.note = match carrier.as_deref() {
                Some("1") => {
                    let speed = sys::num_at(&e, "speed").filter(|s| *s > 0 && *s < 1_000_000);
                    let s = speed.map(|s| if s >= 1000 { format!("{}G", s / 1000) } else { format!("{s}M") });
                    [Some("LINK".to_string()), s, conn.clone()].into_iter().flatten().collect::<Vec<_>>().join(" · ")
                }
                Some(_) => "NO CABLE".into(),
                None => "DOWN".into(),
            };
        }
        d.add("INTERFACE", iface.clone());
        d.add("MAC", sys::read_at(&e, "address").unwrap_or_default());
        d.add("CONNECTION", conn.unwrap_or_default());
        out.push(d);
    }
    // Network chips with no driver at all (no interface appears).
    for pci in ctx.pci.iter().filter(|d| d.base_class() == 0x02 && d.driver.is_none()) {
        let part = if pci.sub_class() & 0xff == 0x80 { Part::Wifi } else { Part::Ethernet };
        out.push(ctx.pci_device(part, pci));
    }
}

// --------------------------------------------------------------- BLUETOOTH ---

fn bluetooth_power() -> Option<(bool, usize)> {
    let show = sys::run("bluetoothctl", &["show"], 3).ok()?;
    let powered = show.lines().any(|l| l.trim() == "Powered: yes");
    let connected = sys::run("bluetoothctl", &["devices", "Connected"], 3)
        .map(|o| o.lines().filter(|l| l.starts_with("Device ")).count())
        .unwrap_or(0);
    Some((powered, connected))
}

pub fn bluetooth(ctx: &mut Ctx, out: &mut Vec<Device>) {
    for e in sys::list(&p("/sys/class/bluetooth")) {
        let hci = sys::file_name(&e);
        if hci.contains(':') {
            continue; // connections, not adapters
        }
        let dev = sys::real(&e.join("device"));
        let owner_usb = sys::usb_parent(&dev).and_then(|up| ctx.usb_at(&up));
        let mut d = match owner_usb {
            Some(u) => ctx.usb_device(Part::Bluetooth, u),
            None => match sys::pci_parent(&dev).and_then(|pp| ctx.pci_at(&pp)) {
                Some(pci) => ctx.pci_device(Part::Bluetooth, pci),
                None => Device::new(Part::Bluetooth, hci.clone(), dev.clone()),
            },
        };
        d.driver = sys::link_name(&dev.join("driver")).or(d.driver);
        if let Some(rf) = rfkill_in(&e) {
            let (soft, hard) = rfkill_state(&rf);
            d.control = Control::Rfkill(rf);
            if hard {
                d.state = State::Off;
                d.note = "HARDWARE SWITCH OFF".into();
            } else if soft {
                d.state = State::Off;
                d.note = "OFF".into();
            }
        }
        if d.state == State::On {
            d.note = match bluetooth_power() {
                Some((true, 0)) => "ON".into(),
                Some((true, n)) => format!("ON · {n} CONNECTED"),
                Some((false, _)) => {
                    d.state = State::Off;
                    "POWERED OFF".into()
                }
                None => "ON".into(),
            };
        }
        d.add("ADAPTER", hci);
        out.push(d);
    }
    // Wireless controller class (e0) with no driver: a Bluetooth chip btusb did not take.
    for u in ctx.unbound_usb_with_class(0xe0) {
        let d = ctx.unbound_usb(Part::Bluetooth, u, 0xe0);
        out.push(d);
    }
}

// ------------------------------------------------------------------ CAMERA ---

pub fn cameras(ctx: &mut Ctx, out: &mut Vec<Device>) {
    let mut groups: Vec<(PathBuf, Vec<String>)> = Vec::new();
    for e in sys::list(&p("/sys/class/video4linux")) {
        let dev = sys::real(&e.join("device"));
        let owner = sys::usb_parent(&dev).or_else(|| sys::pci_parent(&dev)).unwrap_or(dev);
        let node = sys::file_name(&e);
        match groups.iter_mut().find(|(o, _)| *o == owner) {
            Some((_, nodes)) => nodes.push(node),
            None => groups.push((owner, vec![node])),
        }
    }
    for (owner, nodes) in groups {
        let mut d = if let Some(u) = ctx.usb_at(&owner) {
            if ctx.claimed.contains(&owner) {
                continue;
            }
            let mut d = ctx.usb_device(Part::Camera, u);
            d.driver = u.ifaces.iter().find_map(|i| (i.class == 0x0e).then(|| i.driver.clone()).flatten()).or(d.driver);
            d
        } else if let Some(pci) = ctx.pci_at(&owner) {
            let mut d = ctx.pci_device(Part::Camera, pci);
            d.control = Control::None; // MIPI cameras (IPU): no safe switch
            d
        } else {
            let name = sys::read(&p(&format!("/sys/class/video4linux/{}/name", nodes[0]))).unwrap_or_else(|| "Camera".into());
            Device::new(Part::Camera, name, owner.clone())
        };
        if d.state == State::On {
            let ir = d.name.contains(" IR ") || d.name.ends_with(" IR") || d.name.to_lowercase().contains("infrared");
            d.note = if ir { "IR CAMERA · READY".into() } else { "READY".into() };
        }
        d.add("NODES", nodes.iter().map(|n| format!("/dev/{n}")).collect::<Vec<_>>().join(" "));
        out.push(d);
    }
    for u in ctx.unbound_usb_with_class(0x0e) {
        let d = ctx.unbound_usb(Part::Camera, u, 0x0e);
        out.push(d);
    }
}

// ------------------------------------------------------------- FINGERPRINT ---

/// USB fingerprint readers by vendor (and product range where the vendor
/// also makes other things).
pub fn is_fingerprint(vendor: u32, product: u32) -> bool {
    match vendor {
        0x27c6 // Goodix
        | 0x138a // Validity / Synaptics
        | 0x06cb // Synaptics (USB parts are fingerprint readers)
        | 0x1c7a // LighTuning / EgisTec
        | 0x2808 // FocalTech
        | 0x10a5 // FPC
        | 0x147e // UPEK
        | 0x08ff // AuthenTec
        | 0x298d // Next Biometrics
        | 0x2541 // Chipsailing
        => true,
        0x04f3 => (0x0c00..=0x0cff).contains(&product), // Elan readers (Elan also makes touchpads)
        0x0483 => product == 0x2015 || product == 0x2016, // UPEK on STMicro
        _ => false,
    }
}

pub fn fingerprints(ctx: &mut Ctx, out: &mut Vec<Device>) {
    let fprintd = p("/run/current-system/sw/bin/fprintd-list").exists();
    let readers: Vec<&Usb> = ctx
        .usb
        .iter()
        .filter(|u| !ctx.claimed.contains(&u.path) && is_fingerprint(u.vendor, u.product))
        .collect();
    for u in readers {
        let mut d = ctx.usb_device(Part::Fingerprint, u);
        if d.state == State::On {
            d.name = format!("{} {}", d.name, u.id());
            if fprintd {
                d.note = "FPRINTD".into();
            } else {
                d.state = State::NotBound;
                d.note = "NO SUPPORT".into();
                d.reason = Some("fingerprint support (services.fprintd) is not enabled in THORNS".into());
            }
        }
        d.add("SUPPORT", "libfprint list: fprint.freedesktop.org/supported-devices.html");
        out.push(d);
    }
}

// ------------------------------------------------------- TOUCHPAD / KEYBOARD ---

struct Input {
    path: PathBuf,
    name: String,
    touchpad: bool,
    keyboard: bool,
    mouse: bool,
    joystick: bool,
    owner: PathBuf,
    usb: Option<PathBuf>,
    driver: Option<String>,
    inhibited: bool,
}

fn udev_props(dev_num: &str) -> HashMap<String, String> {
    let text = sys::read(&p(&format!("/run/udev/data/c{dev_num}"))).unwrap_or_default();
    text.lines()
        .filter_map(|l| l.strip_prefix("E:"))
        .filter_map(|l| l.split_once('='))
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

/// Devices that say "keyboard" but are buttons, hotkeys or audio jacks.
fn fake_keyboard(name: &str) -> bool {
    let n = name.to_lowercase();
    [
        "button", "video bus", "hotkey", "wmi", "hid events", "consumer control", "system control",
        "speaker", "headphone", "headset", "microphone", " mic", "hdmi", "avrcp", "lid switch", "switch", "extra buttons",
        "radio control", "acpi", "dell privacy", "tablet mode", "intel hid", "accel",
    ]
    .iter()
    .any(|w| n.contains(w))
}

fn inputs_list() -> Vec<Input> {
    let mut out = Vec::new();
    for e in sys::list(&p("/sys/class/input")) {
        if !sys::file_name(&e).starts_with("input") {
            continue;
        }
        let path = sys::real(&e);
        let ps = path.to_string_lossy();
        // Software devices (ydotool, uinput) are skipped; Bluetooth LE keyboards
        // and mice also live under virtual/ (uhid) but are real.
        if ps.contains("/devices/virtual/") && !ps.contains("/uhid/") {
            continue;
        }
        let name = tidy_input_name(&sys::read_at(&path, "name").unwrap_or_default());
        let event = sys::list(&path).into_iter().find(|c| sys::file_name(c).starts_with("event"));
        let props = event.and_then(|ev| sys::read_at(&ev, "dev")).map(|n| udev_props(&n)).unwrap_or_default();
        let has = |k: &str| props.get(k).map(String::as_str) == Some("1");
        let parent = sys::real(&path.join("device"));
        out.push(Input {
            touchpad: has("ID_INPUT_TOUCHPAD"),
            keyboard: has("ID_INPUT_KEYBOARD") && !fake_keyboard(&name),
            mouse: has("ID_INPUT_MOUSE"),
            joystick: has("ID_INPUT_JOYSTICK"),
            usb: sys::usb_parent(&path),
            driver: sys::link_name(&parent.join("driver")),
            inhibited: sys::read_at(&path, "inhibited").as_deref() == Some("1"),
            owner: parent,
            path,
            name,
        });
    }
    out
}

/// "ELAN1200:00 04F3:3057 Touchpad" -> "ELAN1200 Touchpad" (bus IDs dropped).
fn tidy_input_name(name: &str) -> String {
    let is_id = |t: &str| {
        t.len() == 9 && t.as_bytes()[4] == b':' && t.chars().filter(|c| *c != ':').all(|c| c.is_ascii_hexdigit())
    };
    name.split_whitespace()
        .filter(|t| !is_id(t))
        .map(|t| t.strip_suffix(":00").unwrap_or(t))
        .collect::<Vec<_>>()
        .join(" ")
}

fn input_note(i: &Input) -> &'static str {
    let s = i.path.to_string_lossy();
    if s.contains("bluetooth") || s.contains("/uhid/") {
        "BLUETOOTH"
    } else if i.usb.is_some() {
        "USB"
    } else {
        "BUILT-IN"
    }
}

pub fn inputs(ctx: &mut Ctx, out: &mut Vec<Device>) {
    let list = inputs_list();
    for i in &list {
        if let Some(u) = &i.usb {
            let kinds = ctx.input_kinds.entry(u.clone()).or_default();
            for (flag, word) in [(i.mouse, "MOUSE"), (i.joystick, "GAMEPAD"), (i.keyboard, "KEYBOARD")] {
                if flag && !kinds.contains(&word) {
                    kinds.push(word);
                }
            }
        }
    }
    for part in [Part::Touchpad, Part::Keyboard] {
        // Group by USB device (receivers expose several inputs) or parent device.
        let mut groups: Vec<(PathBuf, Vec<&Input>)> = Vec::new();
        for i in list.iter().filter(|i| if part == Part::Touchpad { i.touchpad } else { i.keyboard && !i.touchpad }) {
            let key = i.usb.clone().unwrap_or_else(|| i.owner.clone());
            match groups.iter_mut().find(|(k, _)| *k == key) {
                Some((_, v)) => v.push(i),
                None => groups.push((key, vec![i])),
            }
        }
        for (key, members) in groups {
            let first = members[0];
            let mut d = match first.usb.as_ref().and_then(|u| ctx.usb_at(u)) {
                Some(u) => {
                    let mut d = ctx.usb_device(part, u);
                    d.state = State::On;
                    d.note.clear();
                    d
                }
                None => Device::new(part, first.name.clone(), key.clone()),
            };
            d.driver = first.driver.clone().or(d.driver.take());
            d.control = Control::Inhibit(members.iter().map(|m| m.path.clone()).collect());
            if members.iter().all(|m| m.inhibited) {
                d.state = State::Off;
                d.note = "DISABLED".into();
            } else {
                d.note = if part == Part::Touchpad { format!("ACTIVE · {}", input_note(first)) } else { input_note(first).into() };
            }
            d.add("INPUTS", members.iter().map(|m| m.name.clone()).collect::<Vec<_>>().join(" + "));
            out.push(d);
        }
    }
}

// -------------------------------------------------------------------- DISK ---

struct Mount {
    point: String,
    source: String,
    majmin: String,
}

fn mounts() -> Vec<Mount> {
    let text = sys::read(&p("/proc/self/mountinfo")).unwrap_or_default();
    text.lines()
        .filter_map(|l| {
            let (left, right) = l.split_once(" - ")?;
            let lf: Vec<&str> = left.split_whitespace().collect();
            let rf: Vec<&str> = right.split_whitespace().collect();
            Some(Mount {
                majmin: lf.get(2)?.to_string(),
                point: lf.get(4)?.replace("\\040", " "),
                source: rf.get(1)?.to_string(),
            })
        })
        .collect()
}

/// Block device name ("sda2", "dm-0", "nvme0n1p3") -> whole disks under it.
fn disks_under(name: &str, out: &mut Vec<String>) {
    let dir = p("/sys/class/block").join(name);
    if !dir.exists() {
        return;
    }
    if dir.join("partition").exists() {
        let parent = sys::file_name(sys::real(&dir).parent().unwrap_or(Path::new("")));
        if !out.contains(&parent) {
            out.push(parent);
        }
        return;
    }
    let slaves = sys::list(&dir.join("slaves"));
    if slaves.is_empty() {
        if !out.contains(&name.to_string()) {
            out.push(name.to_string());
        }
    } else {
        for s in slaves {
            disks_under(&sys::file_name(&s), out);
        }
    }
}

fn mount_disks(m: &Mount, out: &mut Vec<String>) {
    if m.source.starts_with("/dev/") {
        disks_under(&sys::file_name(&sys::real(&p(&m.source))), out);
    }
    if !m.majmin.starts_with("0:") {
        let link = p(&format!("/sys/dev/block/{}", m.majmin));
        if link.exists() {
            disks_under(&sys::file_name(&sys::real(&link)), out);
        }
    }
}

fn skip_block(name: &str) -> bool {
    ["loop", "ram", "zram", "dm-", "md", "nbd", "fd", "sr", "mtdblock", "zd"].iter().any(|s| name.starts_with(s))
}

pub fn disks(ctx: &mut Ctx, out: &mut Vec<Device>) -> Vec<PathBuf> {
    let mounts = mounts();
    let mut root: Vec<String> = Vec::new();
    for point in ["/", "/boot", "/nix", "/nix/store"] {
        if let Some(m) = mounts.iter().rev().find(|m| m.point == point) {
            mount_disks(m, &mut root);
        }
    }
    let mut root_paths = Vec::new();
    for e in sys::list(&p("/sys/block")) {
        let name = sys::file_name(&e);
        if skip_block(&name) {
            continue;
        }
        let bytes = sys::num_at(&e, "size").unwrap_or(0) * 512;
        if bytes == 0 {
            continue; // empty card reader slot
        }
        let path = sys::real(&e);
        let is_root = root.contains(&name);
        if is_root {
            root_paths.push(path.clone());
        }
        let dev = sys::real(&e.join("device"));
        let s = path.to_string_lossy().to_string();
        let usb = sys::usb_parent(&path).and_then(|up| ctx.usb_at(&up));
        let transport = match usb {
            Some(u) => u.speed.clone().unwrap_or_else(|| "USB".into()),
            None if s.contains("/nvme") => "NVME".into(),
            None if s.contains("/ata") => "SATA".into(),
            None if s.contains("/mmc") => "SD/MMC".into(),
            None if s.contains("virtio") => "VIRTIO".into(),
            None => String::new(),
        };
        let model = {
            let vendor = sys::read_at(&dev, "vendor").filter(|v| v != "ATA" && !v.starts_with("0x"));
            let model = sys::read_at(&dev, "model");
            match (usb, vendor, model) {
                (Some(u), _, _) => u.name.clone(),
                (None, Some(v), Some(m)) => sys::join_name(&sys::short_vendor(&v), &m),
                (None, None, Some(m)) => m,
                _ => name.clone(),
            }
        };
        let mut d = Device::new(Part::Disk, format!("{model} {}", sys::human_size(bytes)), path.clone());
        if let Some(u) = usb {
            u.describe(&mut d);
            d.driver = u.drivers();
            ctx.claimed.insert(u.path.clone());
        } else {
            d.driver = if s.contains("/nvme") { Some("nvme".into()) } else { sys::link_name(&dev.join("driver")) };
        }
        // Partitions: file system, label, mount point.
        let mut fs_types: Vec<String> = Vec::new();
        let mut windows = false;
        let mut lines = Vec::new();
        for part in sys::list(&e).into_iter().filter(|c| sys::file_name(c).starts_with(&name) && c.join("partition").exists()) {
            let pname = sys::file_name(&part);
            let props = sys::read_at(&part, "dev").map(|n| udev_props_block(&n)).unwrap_or_default();
            let fs = props.get("ID_FS_TYPE").cloned().unwrap_or_default();
            let label = props.get("ID_FS_LABEL").cloned().unwrap_or_default();
            let ptype = props.get("ID_PART_ENTRY_TYPE").cloned().unwrap_or_default();
            if ptype.eq_ignore_ascii_case("e3c9e316-0b5c-4db8-817d-f92df00215ae") || fs == "BitLocker" {
                windows = true;
            }
            if !fs.is_empty() && !fs_types.contains(&fs.to_uppercase()) {
                fs_types.push(fs.to_uppercase());
            }
            let mnt = mounts
                .iter()
                .find(|m| m.source.starts_with("/dev/") && sys::file_name(&sys::real(&p(&m.source))) == pname)
                .map(|m| m.point.clone())
                .unwrap_or_default();
            let size = sys::human_size(sys::num_at(&part, "size").unwrap_or(0) * 512);
            lines.push(format!("{pname} {size} {fs} {label} {mnt}").split_whitespace().collect::<Vec<_>>().join("  "));
        }
        windows = windows && fs_types.iter().any(|f| f == "NTFS" || f == "BITLOCKER");
        let what = if is_root {
            "THORNS ROOT".to_string()
        } else if windows {
            "WINDOWS".to_string()
        } else if fs_types.is_empty() {
            "NO FILE SYSTEM".to_string()
        } else {
            fs_types.join(" · ")
        };
        d.note = [what, transport.clone()].into_iter().filter(|x| !x.is_empty()).collect::<Vec<_>>().join(" · ");
        d.control = if usb.is_some() && !is_root { Control::Disk(name.clone()) } else { Control::None };
        d.add("BLOCK", format!("/dev/{name}"));
        d.add("SIZE", sys::human_size(bytes));
        d.add("TRANSPORT", transport);
        for (i, l) in lines.into_iter().enumerate() {
            d.add(if i == 0 { "PARTITIONS" } else { "" }, l);
        }
        out.push(d);
    }
    root_paths
}

fn udev_props_block(dev_num: &str) -> HashMap<String, String> {
    let text = sys::read(&p(&format!("/run/udev/data/b{dev_num}"))).unwrap_or_default();
    text.lines()
        .filter_map(|l| l.strip_prefix("E:"))
        .filter_map(|l| l.split_once('='))
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

// --------------------------------------------------------------------- USB ---

fn class_word(class: u32) -> &'static str {
    match class {
        0x01 => "AUDIO",
        0x02 | 0x0a => "MODEM",
        0x03 => "HID",
        0x06 => "IMAGING",
        0x07 => "PRINTER",
        0x08 => "STORAGE",
        0x0b => "SMART CARD",
        0x0d => "SECURITY",
        0x0e => "VIDEO",
        0x0f => "HEALTH",
        0x10 => "AUDIO/VIDEO",
        0xe0 => "WIRELESS",
        0xef => "MISC",
        0xff => "VENDOR",
        _ => "DEVICE",
    }
}

pub fn usb_rest(ctx: &mut Ctx, out: &mut Vec<Device>) {
    let rest: Vec<&Usb> = ctx.usb.iter().filter(|u| !ctx.claimed.contains(&u.path)).collect();
    for u in rest {
        let mut d = ctx.usb_device(Part::Usb, u);
        if d.state != State::On {
            out.push(d);
            continue;
        }
        let kind = ctx
            .input_kinds
            .get(&u.path)
            .map(|k| k.join("+"))
            .filter(|k| !k.is_empty())
            .or_else(|| u.ifaces.first().map(|i| class_word(i.class).to_string()))
            .unwrap_or_else(|| "DEVICE".into());
        let speed = u.speed.clone().unwrap_or_default();
        let has_driver = u.ifaces.iter().any(|i| i.driver.is_some());
        let userspace = !u.ifaces.is_empty() && u.ifaces.iter().all(|i| i.class == 0xff || i.class == 0xfe);
        if has_driver || u.ifaces.is_empty() {
            d.note = [kind, speed].into_iter().filter(|x| !x.is_empty()).collect::<Vec<_>>().join(" · ");
        } else if userspace && ctx.policy.modules_for(&sys::read_at(&u.ifaces[0].path, "modalias").unwrap_or_default()).is_empty() {
            d.note = "USERSPACE".into();
            d.add("NOTE", "vendor-specific device: apps talk to it directly, no kernel driver needed");
        } else {
            let first = u.ifaces[0].path.clone();
            ctx.mark_unbound(&mut d, &first);
        }
        out.push(d);
    }
}

// ------------------------------------------------------------- THUNDERBOLT ---

pub fn thunderbolt(ctx: &mut Ctx, out: &mut Vec<Device>) {
    let base = p("/sys/bus/thunderbolt/devices");
    let all = sys::list(&base);
    for e in all.iter().filter(|e| sys::file_name(e).starts_with("domain")) {
        let dom = sys::file_name(e);
        let num = dom.trim_start_matches("domain").to_string();
        let path = sys::real(e);
        let ctrl = path.parent().and_then(sys::pci_parent).and_then(|pp| ctx.pci_at(&pp));
        let mut d = match ctrl {
            Some(pci) => {
                let mut d = ctx.pci_device(Part::Thunderbolt, pci);
                d.control = Control::None;
                d
            }
            None => Device::new(Part::Thunderbolt, format!("Thunderbolt domain {num}"), path.clone()),
        };
        let devices: Vec<String> = all
            .iter()
            .map(|x| sys::file_name(x))
            .filter(|n| n.starts_with(&format!("{num}-")) && !n.ends_with("-0") && !n.contains(':'))
            .filter_map(|n| {
                let dp = base.join(&n);
                let v = sys::read_at(&dp, "vendor_name").unwrap_or_default();
                let m = sys::read_at(&dp, "device_name")?;
                let auth = if sys::read_at(&dp, "authorized").as_deref() == Some("0") { " (not authorized)" } else { "" };
                Some(format!("{} {m}{auth}", sys::short_vendor(&v)).trim().to_string())
            })
            .collect();
        if d.state == State::On {
            d.note = match devices.len() {
                0 => "NO DEVICE".into(),
                1 => "1 DEVICE".into(),
                n => format!("{n} DEVICES"),
            };
        }
        d.add("SECURITY", sys::read_at(&path, "security").unwrap_or_default());
        for (i, dev) in devices.into_iter().enumerate() {
            d.add(if i == 0 { "CONNECTED" } else { "" }, dev);
        }
        d.add("MANAGE", "boltctl (authorize / forget docks)");
        out.push(d);
    }
}

// ----------------------------------------------------------------- BATTERY ---

pub fn batteries(out: &mut Vec<Device>) {
    let supplies = sys::list(&p("/sys/class/power_supply"));
    let ac = supplies
        .iter()
        .find(|s| sys::read_at(s, "type").as_deref() == Some("Mains"))
        .and_then(|s| sys::read_at(s, "online"));
    for e in &supplies {
        if sys::read_at(e, "type").as_deref() != Some("Battery") || sys::read_at(e, "scope").as_deref() == Some("Device") {
            continue; // wireless mice / headsets report batteries too
        }
        let bat = sys::file_name(e);
        let maker = sys::read_at(e, "manufacturer").map(|m| sys::short_vendor(&m)).unwrap_or_default();
        let model = sys::read_at(e, "model_name").unwrap_or_else(|| bat.clone());
        let mut d = Device::new(Part::Battery, sys::join_name(&maker, &model), sys::real(e));
        let cap = sys::num_at(e, "capacity");
        let status = sys::read_at(e, "status").unwrap_or_default().to_uppercase();
        // energy_* (µWh, µW) or charge_* (µAh, µA): same arithmetic.
        let get = |a: &str, b: &str| sys::num_at(e, a).or_else(|| sys::num_at(e, b));
        let now = get("energy_now", "charge_now");
        let full = get("energy_full", "charge_full");
        let design = get("energy_full_design", "charge_full_design");
        let rate = get("power_now", "current_now").filter(|r| *r > 0);
        let health = match (full, design) {
            (Some(f), Some(dz)) if dz > 0 => Some(f * 100 / dz),
            _ => None,
        };
        let hours = match (status.as_str(), now, full, rate) {
            ("DISCHARGING", Some(n), _, Some(r)) => Some(n as f64 / r as f64),
            ("CHARGING", Some(n), Some(f), Some(r)) if f > n => Some((f - n) as f64 / r as f64),
            _ => None,
        };
        let time = hours.filter(|h| *h < 48.0).map(|h| format!("{}h {:02}m", h as u64, ((h.fract()) * 60.0) as u64));
        let mut parts: Vec<String> = Vec::new();
        if let Some(c) = cap {
            parts.push(format!("{c}%"));
        }
        if !status.is_empty() {
            parts.push(status.replace("NOT CHARGING", "HOLDING"));
        }
        parts.extend(time.clone());
        if let Some(h) = health {
            parts.push(format!("HEALTH {h}%"));
        }
        d.note = parts.join(" · ");
        d.add("NAME", bat);
        d.add("TECHNOLOGY", sys::read_at(e, "technology").unwrap_or_default());
        d.add("CYCLES", sys::read_at(e, "cycle_count").filter(|c| c != "0").unwrap_or_default());
        d.add("AC", ac.as_deref().map(|o| if o == "1" { "PLUGGED IN" } else { "UNPLUGGED" }).unwrap_or(""));
        out.push(d);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn knows_fingerprint_readers() {
        assert!(is_fingerprint(0x27c6, 0x609c));
        assert!(is_fingerprint(0x04f3, 0x0c4b));
        assert!(!is_fingerprint(0x04f3, 0x3057)); // Elan touchpad
        assert!(!is_fingerprint(0x046d, 0xc539));
    }

    #[test]
    fn tidies_input_names() {
        assert_eq!(tidy_input_name("ELAN1200:00 04F3:3057 Touchpad"), "ELAN1200 Touchpad");
        assert_eq!(tidy_input_name("AT Translated Set 2 keyboard"), "AT Translated Set 2 keyboard");
    }

    #[test]
    fn filters_fake_keyboards() {
        assert!(fake_keyboard("Power Button"));
        assert!(fake_keyboard("Video Bus"));
        assert!(fake_keyboard("Asus WMI hotkeys"));
        assert!(fake_keyboard("HDA Intel PCH Headphone"));
        assert!(!fake_keyboard("AT Translated Set 2 keyboard"));
        assert!(!fake_keyboard("Logitech USB Receiver"));
    }
}
