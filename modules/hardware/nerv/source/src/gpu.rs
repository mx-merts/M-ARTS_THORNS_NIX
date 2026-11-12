//! NVIDIA generations and which THORNS boot entry fits the machine.
//!
//!   DESK      no NVIDIA, or an NVIDIA too old for current drivers (nouveau)
//!   NVIDIA-L  Maxwell, Pascal, Volta (GTX 900 / 1000): proprietary 580 driver
//!   NVIDIA    Turing and newer (GTX 16, RTX 20+): open 595 driver
//!   PRIME     Turing+ in THIS laptop (bus IDs fixed in prime-bus.nix)

use crate::sys;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Arch {
    Older,
    Kepler,
    Maxwell,
    Pascal,
    Volta,
    Turing,
    Ampere,
    Hopper,
    Ada,
    Blackwell,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Family {
    /// nouveau only (DESK)
    Desk,
    /// legacy_580 (NVIDIA-L)
    Legacy,
    /// stable, open module (NVIDIA / PRIME)
    Modern,
}

impl Arch {
    pub fn name(self) -> &'static str {
        match self {
            Arch::Older => "FERMI OR OLDER",
            Arch::Kepler => "KEPLER",
            Arch::Maxwell => "MAXWELL",
            Arch::Pascal => "PASCAL",
            Arch::Volta => "VOLTA",
            Arch::Turing => "TURING",
            Arch::Ampere => "AMPERE",
            Arch::Hopper => "HOPPER",
            Arch::Ada => "ADA",
            Arch::Blackwell => "BLACKWELL",
        }
    }

    pub fn family(self) -> Family {
        match self {
            Arch::Older | Arch::Kepler => Family::Desk,
            Arch::Maxwell | Arch::Pascal | Arch::Volta => Family::Legacy,
            _ => Family::Modern,
        }
    }
}

/// Generation from the chip code name in pci.ids ("GP106 [...]"), or, for
/// cards newer than the database, from the PCI device ID range.
pub fn nvidia_arch(device_id: u32, pci_name: Option<&str>) -> Arch {
    if let Some(code) = pci_name.and_then(|n| n.split_whitespace().next()) {
        let c = code.to_ascii_uppercase();
        let by_code = [
            ("GK", Arch::Kepler),
            ("GM", Arch::Maxwell),
            ("GP", Arch::Pascal),
            ("GV", Arch::Volta),
            ("TU", Arch::Turing),
            ("GA", Arch::Ampere),
            ("GH", Arch::Hopper),
            ("AD", Arch::Ada),
            ("GB", Arch::Blackwell),
            ("GF", Arch::Older),
            ("GT", Arch::Older),
            ("G8", Arch::Older),
            ("G9", Arch::Older),
            ("NV", Arch::Older),
        ];
        for (prefix, arch) in by_code {
            if c.starts_with(prefix) && c[prefix.len()..].starts_with(|ch: char| ch.is_ascii_digit()) {
                return arch;
            }
        }
    }
    match device_id {
        0..=0x0fbf => Arch::Older,
        0x0fc0..=0x133f => Arch::Kepler,
        0x15f0..=0x15ff | 0x1b00..=0x1d7f => Arch::Pascal,
        0x1340..=0x17ff => Arch::Maxwell,
        0x1d80..=0x1dff => Arch::Volta,
        0x1e00..=0x1fff | 0x2180..=0x21ff => Arch::Turing,
        0x2300..=0x233f => Arch::Hopper,
        0x2080..=0x20ff | 0x2200..=0x25ff => Arch::Ampere,
        0x2600..=0x28ff => Arch::Ada,
        _ => Arch::Blackwell,
    }
}

/// /etc/nerv/nerv.conf, written by the NixOS module.
#[derive(Default, Debug)]
pub struct Conf {
    pub session: String,
    pub prime_igpu: Option<String>,
    pub prime_dgpu: Option<String>,
}

pub fn load_conf() -> Conf {
    let text = sys::read(&sys::p("/etc/nerv/nerv.conf")).unwrap_or_default();
    parse_conf(&text)
}

pub fn parse_conf(text: &str) -> Conf {
    let mut c = Conf { session: "UNKNOWN".into(), ..Conf::default() };
    for line in text.lines() {
        let Some((k, v)) = line.split_once('=') else { continue };
        let v = v.trim();
        match k.trim() {
            "session" if !v.is_empty() => c.session = v.to_ascii_uppercase(),
            "prime_igpu" => c.prime_igpu = nix_bus(v),
            "prime_dgpu" => c.prime_dgpu = nix_bus(v),
            _ => {}
        }
    }
    c
}

/// NixOS bus ID ("PCI:1@0:0:0" or "PCI:1:0:0", decimal) -> sysfs "0000:01:00.0".
pub fn nix_bus(s: &str) -> Option<String> {
    let s = s.trim().strip_prefix("PCI:")?;
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() != 3 {
        return None;
    }
    let (bus, domain) = match parts[0].split_once('@') {
        Some((b, d)) => (b.parse::<u32>().ok()?, d.parse::<u32>().ok()?),
        None => (parts[0].parse().ok()?, 0),
    };
    let dev: u32 = parts[1].parse().ok()?;
    let func: u32 = parts[2].parse().ok()?;
    Some(format!("{domain:04x}:{bus:02x}:{dev:02x}.{func:x}"))
}

/// One GPU as the recommender sees it.
pub struct Gpu {
    pub addr: String,
    pub vendor: u32,
    pub arch: Option<Arch>,
}

pub struct Rec {
    /// "NVIDIA ADA", "INTEL", "AMD"...
    pub detected: String,
    pub recommended: &'static str,
    pub current: String,
    /// Why, in one short line (shown under the » line when not matching).
    pub why: &'static str,
}

impl Rec {
    pub fn ok(&self) -> bool {
        self.current == self.recommended
    }
}

fn vendor_word(v: u32) -> &'static str {
    match v {
        0x10de => "NVIDIA",
        0x8086 => "INTEL",
        0x1002 | 0x1022 => "AMD",
        0x1af4 | 0x1234 | 0x15ad | 0x80ee => "VIRTUAL",
        _ => "GPU",
    }
}

pub fn recommend(gpus: &[Gpu], conf: &Conf) -> Rec {
    let current = conf.session.clone();
    // Newest NVIDIA card decides.
    let nvidia = gpus
        .iter()
        .filter(|g| g.vendor == 0x10de)
        .max_by_key(|g| g.arch.map(Arch::family).unwrap_or(Family::Modern));
    let Some(nv) = nvidia else {
        let mut words: Vec<&str> = gpus.iter().map(|g| vendor_word(g.vendor)).collect();
        words.dedup();
        let detected = if words.is_empty() { "NO GPU".to_string() } else { words.join(" + ") };
        return Rec { detected, recommended: "DESK", current, why: "No NVIDIA card: DESK fits every Intel/AMD machine" };
    };
    let arch = nv.arch.unwrap_or(Arch::Blackwell);
    let detected = format!("NVIDIA {}", arch.name());
    let (recommended, why) = match arch.family() {
        Family::Desk => ("DESK", "Too old for current NVIDIA drivers, nouveau (DESK) runs it"),
        Family::Legacy => ("NVIDIA-L", "GTX 900/1000 era, needs the 580 legacy driver"),
        Family::Modern => {
            let igpu = gpus.iter().find(|g| g.vendor != 0x10de);
            let is_prime_machine = igpu.is_some()
                && conf.prime_dgpu.as_deref() == Some(nv.addr.as_str())
                && igpu.map(|g| Some(g.addr.as_str()) == conf.prime_igpu.as_deref()).unwrap_or(false);
            if is_prime_machine {
                ("PRIME", "This laptop's own entry (bus IDs match)")
            } else {
                ("NVIDIA", "Turing or newer, open 595 driver")
            }
        }
    };
    Rec { detected, recommended, current, why }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn conf(session: &str) -> Conf {
        parse_conf(&format!("session={session}\nprime_igpu=PCI:0@0:2:0\nprime_dgpu=PCI:1@0:0:0\n"))
    }

    #[test]
    fn classifies_by_code_name_then_id() {
        assert_eq!(nvidia_arch(0x1c03, Some("GP106 [GeForce GTX 1060 6GB]")), Arch::Pascal);
        assert_eq!(nvidia_arch(0x28a1, Some("AD107M [GeForce RTX 4050 Max-Q / Mobile]")), Arch::Ada);
        assert_eq!(nvidia_arch(0x1f82, Some("TU117 [GeForce GTX 1650]")), Arch::Turing);
        assert_eq!(nvidia_arch(0x2b85, Some("GB202 [GeForce RTX 5090]")), Arch::Blackwell);
        assert_eq!(nvidia_arch(0x1180, Some("GK104 [GeForce GTX 680]")), Arch::Kepler);
        // Not in the database: by ID range.
        assert_eq!(nvidia_arch(0x1b80, None), Arch::Pascal);
        assert_eq!(nvidia_arch(0x1401, None), Arch::Maxwell);
        assert_eq!(nvidia_arch(0x2d04, None), Arch::Blackwell);
        assert_eq!(nvidia_arch(0x0de1, None), Arch::Older);
    }

    #[test]
    fn converts_nix_bus_ids() {
        assert_eq!(nix_bus("PCI:0@0:2:0").as_deref(), Some("0000:00:02.0"));
        assert_eq!(nix_bus("PCI:1@0:0:0").as_deref(), Some("0000:01:00.0"));
        assert_eq!(nix_bus("PCI:10:0:1").as_deref(), Some("0000:0a:00.1"));
        assert_eq!(nix_bus("garbage"), None);
    }

    #[test]
    fn recommends_the_right_entry() {
        let intel = Gpu { addr: "0000:00:02.0".into(), vendor: 0x8086, arch: None };
        let ada = Gpu { addr: "0000:01:00.0".into(), vendor: 0x10de, arch: Some(Arch::Ada) };
        let r = recommend(&[intel, ada], &conf("PRIME"));
        assert_eq!((r.detected.as_str(), r.recommended, r.ok()), ("NVIDIA ADA", "PRIME", true));

        // Same card generation, other laptop (other bus): NVIDIA, not PRIME.
        let intel = Gpu { addr: "0000:00:02.0".into(), vendor: 0x8086, arch: None };
        let ampere = Gpu { addr: "0000:02:00.0".into(), vendor: 0x10de, arch: Some(Arch::Ampere) };
        assert_eq!(recommend(&[intel, ampere], &conf("DESK")).recommended, "NVIDIA");

        let pascal = Gpu { addr: "0000:01:00.0".into(), vendor: 0x10de, arch: Some(Arch::Pascal) };
        let r = recommend(&[pascal], &conf("DESK"));
        assert_eq!((r.recommended, r.ok()), ("NVIDIA-L", false));

        let amd = Gpu { addr: "0000:03:00.0".into(), vendor: 0x1002, arch: None };
        let r = recommend(&[amd], &conf("DESK"));
        assert_eq!((r.detected.as_str(), r.recommended, r.ok()), ("AMD", "DESK", true));

        let kepler = Gpu { addr: "0000:01:00.0".into(), vendor: 0x10de, arch: Some(Arch::Kepler) };
        assert_eq!(recommend(&[kepler], &conf("NVIDIA")).recommended, "DESK");
    }
}
