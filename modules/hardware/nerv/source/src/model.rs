//! What NERV knows about a part: its category, state and how to reach it.

use std::path::PathBuf;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Part {
    Gpu,
    Audio,
    Wifi,
    Ethernet,
    Bluetooth,
    Camera,
    Fingerprint,
    Touchpad,
    Keyboard,
    Disk,
    Usb,
    Thunderbolt,
    Battery,
}

impl Part {
    /// Report order.
    pub const ALL: [Part; 13] = [
        Part::Gpu,
        Part::Audio,
        Part::Wifi,
        Part::Ethernet,
        Part::Bluetooth,
        Part::Camera,
        Part::Fingerprint,
        Part::Touchpad,
        Part::Keyboard,
        Part::Disk,
        Part::Usb,
        Part::Thunderbolt,
        Part::Battery,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Part::Gpu => "GPU",
            Part::Audio => "AUDIO",
            Part::Wifi => "WIFI",
            Part::Ethernet => "ETHERNET",
            Part::Bluetooth => "BLUETOOTH",
            Part::Camera => "CAMERA",
            Part::Fingerprint => "FINGERPRINT",
            Part::Touchpad => "TOUCHPAD",
            Part::Keyboard => "KEYBOARD",
            Part::Disk => "DISK",
            Part::Usb => "USB",
            Part::Thunderbolt => "THUNDERBOLT",
            Part::Battery => "BATTERY",
        }
    }

    /// Short name for typing; None when the name is already short.
    pub fn short(self) -> Option<&'static str> {
        match self {
            Part::Audio => Some("AUD"),
            Part::Wifi => Some("WF"),
            Part::Ethernet => Some("ETH"),
            Part::Bluetooth => Some("BT"),
            Part::Camera => Some("CAM"),
            Part::Fingerprint => Some("FP"),
            Part::Touchpad => Some("TP"),
            Part::Keyboard => Some("KB"),
            Part::Thunderbolt => Some("TB"),
            Part::Battery => Some("BAT"),
            Part::Gpu | Part::Disk | Part::Usb => None,
        }
    }

    pub fn parse(s: &str) -> Option<Part> {
        let s = s.to_ascii_uppercase();
        Part::ALL
            .into_iter()
            .find(|p| p.name() == s || p.short() == Some(s.as_str()))
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum State {
    /// Working.
    On,
    /// Switched off on purpose (rfkill, NERV DISABLE, inhibited).
    Off,
    /// Its driver was missing at boot and NERV bound it.
    BoundByNerv,
    /// Seen, but no driver is attached.
    NotBound,
    /// Its driver is blacklisted on purpose (hardening.nix).
    Blocked,
}

impl State {
    pub fn symbol(self) -> &'static str {
        match self {
            State::On => "●",
            State::Off => "○",
            State::BoundByNerv => "◐",
            State::NotBound => "✖",
            State::Blocked => "⊘",
        }
    }
}

/// How NERV switches a part on and off (decided by the scanner).
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Control {
    /// No ENABLE/DISABLE/RESET (GPU, battery, Thunderbolt, internal disks).
    None,
    /// Radio switch: /sys/class/rfkill/rfkillN/soft.
    Rfkill(PathBuf),
    /// USB device: authorized 0/1.
    UsbAuth(PathBuf),
    /// PCI function: driver unbind / drivers_probe.
    PciBind(PathBuf),
    /// Input devices: inhibited 1/0 (all inputN directories of the part).
    Inhibit(Vec<PathBuf>),
    /// Block device name ("sdb"), switched off with udisksctl.
    Disk(String),
    /// Nothing to do: only RESET is available (sound cards on PCI).
    ResetOnly(PathBuf),
}

#[derive(Clone, Debug)]
pub struct Device {
    pub part: Part,
    pub name: String,
    pub driver: Option<String>,
    pub state: State,
    /// Right-hand column of the report.
    pub note: String,
    /// Why it is not bound / blocked (shown in the footer and INFO).
    pub reason: Option<String>,
    /// Canonical sysfs path (what safety checks compare against).
    pub sys: PathBuf,
    pub control: Control,
    /// Extra INFO lines.
    pub info: Vec<(String, String)>,
    /// "vvvv:pppp" when known.
    pub id: Option<String>,
    /// Keyboard that is also a mouse (receivers): never counted as a spare.
    pub mixed: bool,
}

impl Device {
    pub fn new(part: Part, name: String, sys: PathBuf) -> Device {
        Device {
            part,
            name,
            driver: None,
            state: State::On,
            note: String::new(),
            reason: None,
            sys,
            control: Control::None,
            info: Vec::new(),
            id: None,
            mixed: false,
        }
    }

    pub fn add(&mut self, key: &str, value: impl Into<String>) {
        let value = value.into();
        if !value.is_empty() {
            self.info.push((key.to_string(), value));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_long_and_short_names_in_any_case() {
        assert_eq!(Part::parse("bluetooth"), Some(Part::Bluetooth));
        assert_eq!(Part::parse("BT"), Some(Part::Bluetooth));
        assert_eq!(Part::parse("wf"), Some(Part::Wifi));
        assert_eq!(Part::parse("Gpu"), Some(Part::Gpu));
        assert_eq!(Part::parse("fp"), Some(Part::Fingerprint));
        assert_eq!(Part::parse("blue"), None);
    }
}
