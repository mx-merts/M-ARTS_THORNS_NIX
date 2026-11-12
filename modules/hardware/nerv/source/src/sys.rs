//! System access: paths, file helpers, external commands, PCI/USB names.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::OnceLock;
use std::thread;
use std::time::{Duration, Instant};

/// Every system path NERV touches goes through `p()`. NERV_ROOT (tests and
/// development only) points it at a fake tree instead of "/".
fn root() -> &'static Path {
    static ROOT: OnceLock<PathBuf> = OnceLock::new();
    ROOT.get_or_init(|| match env::var_os("NERV_ROOT") {
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

pub fn read_at(dir: &Path, name: &str) -> Option<String> {
    read(&dir.join(name))
}

/// Last part of a symlink's target, e.g. ".../driver" -> "iwlwifi".
pub fn link_name(path: &Path) -> Option<String> {
    let target = fs::read_link(path).ok()?;
    Some(target.file_name()?.to_string_lossy().into_owned())
}

pub fn real(path: &Path) -> PathBuf {
    fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

/// Directory entries, sorted (stable numbering between runs).
pub fn list(dir: &Path) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = match fs::read_dir(dir) {
        Ok(rd) => rd.flatten().map(|e| e.path()).collect(),
        Err(_) => Vec::new(),
    };
    out.sort_by_key(|a| natural_key(a));
    out
}

/// "card10" sorts after "card2".
fn natural_key(path: &Path) -> (String, u64) {
    let name = file_name(path);
    let digits: String = name.chars().rev().take_while(|c| c.is_ascii_digit()).collect();
    let stem = name[..name.len() - digits.len()].to_string();
    let num = digits.chars().rev().collect::<String>().parse().unwrap_or(0);
    (stem, num)
}

pub fn file_name(path: &Path) -> String {
    path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default()
}

pub fn hex(s: &str) -> Option<u32> {
    u32::from_str_radix(s.trim().trim_start_matches("0x"), 16).ok()
}

pub fn hex_at(dir: &Path, name: &str) -> Option<u32> {
    read_at(dir, name).and_then(|s| hex(&s))
}

pub fn num_at(dir: &Path, name: &str) -> Option<u64> {
    read_at(dir, name)?.parse().ok()
}

pub fn write(path: &Path, value: &str) -> Result<(), String> {
    fs::write(path, value).map_err(|e| e.to_string())
}

/// True when running as root (effective uid 0).
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

/// Runs a command and returns its stdout. Gives up after `secs` seconds, so a
/// hung helper (bluetoothctl with no daemon, for example) cannot freeze NERV.
pub fn run(cmd: &str, args: &[&str], secs: u64) -> Result<String, String> {
    let mut child = Command::new(cmd)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("{cmd}: {e}"))?;
    let start = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) if start.elapsed() > Duration::from_secs(secs) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!("{cmd}: timed out"));
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
        let err = String::from_utf8_lossy(&out.stderr).trim().to_string();
        Err(if err.is_empty() { format!("{cmd} failed") } else { err })
    }
}

pub fn sleep_ms(ms: u64) {
    thread::sleep(Duration::from_millis(ms));
}

/// Walks up from `path` to the first directory that has `marker` in it.
pub fn ancestor_with(path: &Path, marker: &str) -> Option<PathBuf> {
    let stop = p("/sys/devices");
    let mut cur = Some(path);
    while let Some(dir) = cur {
        if dir.join(marker).exists() {
            return Some(dir.to_path_buf());
        }
        if dir == stop {
            return None;
        }
        cur = dir.parent();
    }
    None
}

/// The USB device (not interface) a sysfs path belongs to.
pub fn usb_parent(path: &Path) -> Option<PathBuf> {
    ancestor_with(path, "idVendor")
}

/// The PCI function a sysfs path belongs to ("class" + "vendor" + "config").
pub fn pci_parent(path: &Path) -> Option<PathBuf> {
    let mut cur = Some(path);
    while let Some(dir) = cur {
        if dir.join("config").exists() && dir.join("class").exists() && dir.join("vendor").exists() {
            return Some(dir.to_path_buf());
        }
        cur = dir.parent();
    }
    None
}

// ---------------------------------------------------------------- names ----

fn ids_file(name: &str) -> Option<&'static str> {
    static PCI: OnceLock<Option<String>> = OnceLock::new();
    static USB: OnceLock<Option<String>> = OnceLock::new();
    let cell = if name == "pci.ids" { &PCI } else { &USB };
    cell.get_or_init(|| {
        let mut dirs: Vec<PathBuf> = Vec::new();
        if let Some(d) = env::var_os("NERV_HWDATA") {
            dirs.push(PathBuf::from(d));
        }
        dirs.push(p("/run/current-system/sw/share/hwdata"));
        dirs.push(p("/usr/share/hwdata"));
        dirs.push(p("/usr/share/misc"));
        dirs.iter().find_map(|d| fs::read_to_string(d.join(name)).ok())
    })
    .as_deref()
}

/// (vendor, device) names from a pci.ids / usb.ids style database.
pub fn lookup(text: &str, vendor: u32, device: u32) -> (Option<String>, Option<String>) {
    let vk = format!("{vendor:04x}");
    let dk = format!("{device:04x}");
    let mut vname = None;
    for line in text.lines() {
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        if let Some(rest) = line.strip_prefix('\t') {
            if vname.is_some() && !rest.starts_with('\t') {
                if let Some(name) = id_line(rest, &dk) {
                    return (vname, Some(name));
                }
            }
        } else if vname.is_some() {
            break; // left the vendor's block
        } else if let Some(name) = id_line(line, &vk) {
            vname = Some(name);
        }
    }
    (vname, None)
}

fn id_line(line: &str, key: &str) -> Option<String> {
    let id = line.get(..4)?;
    let rest = line.get(4..)?;
    (id.eq_ignore_ascii_case(key) && rest.starts_with(' ')).then(|| rest.trim().to_string())
}

/// Subsystem name (the exact card, e.g. "Wi-Fi 6E AX211 160MHz") from pci.ids.
pub fn pci_subsystem_name(vendor: u32, device: u32, sv: u32, sd: u32) -> Option<String> {
    let text = ids_file("pci.ids")?;
    let (vk, dk, sk) = (format!("{vendor:04x}"), format!("{device:04x}"), format!("{sv:04x} {sd:04x}"));
    let mut in_vendor = false;
    let mut in_device = false;
    for line in text.lines() {
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        if let Some(rest) = line.strip_prefix("\t\t") {
            if in_device && rest.get(..9).is_some_and(|k| k.eq_ignore_ascii_case(&sk)) {
                return Some(rest[9..].trim().to_string());
            }
        } else if let Some(rest) = line.strip_prefix('\t') {
            if in_device {
                return None;
            }
            in_device = in_vendor && id_line(rest, &dk).is_some();
        } else if in_vendor {
            return None;
        } else {
            in_vendor = id_line(line, &vk).is_some();
        }
    }
    None
}

pub fn pci_names(vendor: u32, device: u32) -> (Option<String>, Option<String>) {
    ids_file("pci.ids").map_or((None, None), |t| lookup(t, vendor, device))
}

pub fn usb_names(vendor: u32, device: u32) -> (Option<String>, Option<String>) {
    ids_file("usb.ids").map_or((None, None), |t| lookup(t, vendor, device))
}

/// "[AMD/ATI]" -> Some("AMD/ATI"), only for a trailing bracket.
fn bracket(s: &str) -> Option<&str> {
    let s = s.trim_end();
    if !s.ends_with(']') {
        return None;
    }
    let open = s.rfind('[')?;
    let inner = s[open + 1..s.len() - 1].trim();
    (!inner.is_empty()).then_some(inner)
}

/// "Realtek Semiconductor Co., Ltd." -> "Realtek", "Intel Corporation" -> "Intel".
pub fn short_vendor(v: &str) -> String {
    if let Some(b) = bracket(v) {
        return b.split('/').next().unwrap_or(b).trim().to_string();
    }
    const SUFFIXES: [&str; 24] = [
        ", Inc.", " Inc.", " Inc", ", Inc", " Corporation", " Corp.", " Corp", " Co., Ltd.",
        " Co., Ltd", " Co.,Ltd.", " Co.,Ltd", " Co Ltd", " Co. Ltd.", ", Ltd.", " Ltd.", " Ltd",
        " Limited", " Technology", " Technologies", " Semiconductor", " Electronics",
        " International", " LLC", " GmbH",
    ];
    let mut s = v.trim().trim_end_matches(',').to_string();
    loop {
        let before = s.len();
        for suf in SUFFIXES {
            let lower = s.to_ascii_lowercase();
            if s.len() > suf.len() && lower.ends_with(&suf.to_ascii_lowercase()) {
                s.truncate(s.len() - suf.len());
                s = s.trim_end_matches(',').trim_end().to_string();
            }
        }
        if s.len() == before {
            break;
        }
    }
    match s.to_ascii_lowercase().as_str() {
        "mediatek" => "MediaTek".to_string(),
        "hewlett-packard company" | "hewlett-packard" => "HP".to_string(),
        "asustek computer" | "asustek" => "ASUS".to_string(),
        "micro-star international co." | "micro-star" => "MSI".to_string(),
        _ => s,
    }
}

/// "AD107M [GeForce RTX 4050 Max-Q / Mobile]" -> "GeForce RTX 4050 Laptop",
/// "... High Definition Audio Controller" -> "... HD Audio".
pub fn short_device(d: &str) -> String {
    let mut s = bracket(d).unwrap_or(d).trim().to_string();
    for (from, to) in [
        (" Max-Q / Mobile", " Laptop"),
        (" Laptop GPU", " Laptop"),
        (" Mobile", " Laptop"),
        ("High Definition Audio", "HD Audio"),
        (" Series Chipset Family", ""),
        (" Chipset Family", ""),
        (" Family", ""),
    ] {
        s = s.replace(from, to);
    }
    for suffix in [" Controller", " Adapter"] {
        if s.len() > suffix.len() + 8 && s.ends_with(suffix) {
            s.truncate(s.len() - suffix.len());
        }
    }
    s
}

/// "Intel" + "Wi-Fi 6E AX211" -> "Intel Wi-Fi 6E AX211" (no doubled vendor).
pub fn join_name(vendor: &str, device: &str) -> String {
    if vendor.is_empty() {
        return device.to_string();
    }
    let first = device.split_whitespace().next().unwrap_or("");
    if first.eq_ignore_ascii_case(vendor.split_whitespace().next().unwrap_or("")) {
        device.to_string()
    } else {
        format!("{vendor} {device}")
    }
}

/// Fits text into `width` columns, ending with "…" when cut.
pub fn fit(s: &str, width: usize) -> String {
    if s.chars().count() <= width {
        return s.to_string();
    }
    let mut out: String = s.chars().take(width.saturating_sub(1)).collect();
    out = out.trim_end().to_string();
    out.push('…');
    out
}

/// Bytes -> "512GB", "1TB", "64GB" (decimal, like drive labels).
pub fn human_size(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut v = bytes as f64;
    let mut i = 0;
    while v >= 1000.0 && i < UNITS.len() - 1 {
        v /= 1000.0;
        i += 1;
    }
    if v >= 100.0 || v.fract() < 0.05 {
        format!("{:.0}{}", v, UNITS[i])
    } else {
        format!("{:.1}{}", v, UNITS[i])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DB: &str = "# comment\n\
10de  NVIDIA Corporation\n\
\t28a1  AD107M [GeForce RTX 4050 Max-Q / Mobile]\n\
\t\t1043 1e1f  Some subsystem\n\
\t1c03  GP106 [GeForce GTX 1060 6GB]\n\
10df  Emulex Corporation\n\
\t1c03  Not this one\n";

    #[test]
    fn looks_up_vendor_and_device() {
        let (v, d) = lookup(DB, 0x10de, 0x1c03);
        assert_eq!(v.as_deref(), Some("NVIDIA Corporation"));
        assert_eq!(d.as_deref(), Some("GP106 [GeForce GTX 1060 6GB]"));
        let (v, d) = lookup(DB, 0x10de, 0x9999);
        assert_eq!(v.as_deref(), Some("NVIDIA Corporation"));
        assert_eq!(d, None);
        assert_eq!(lookup(DB, 0x1234, 0x1c03), (None, None));
    }

    #[test]
    fn shortens_names() {
        assert_eq!(short_vendor("Realtek Semiconductor Co., Ltd."), "Realtek");
        assert_eq!(short_vendor("Intel Corporation"), "Intel");
        assert_eq!(short_vendor("Advanced Micro Devices, Inc. [AMD/ATI]"), "AMD");
        assert_eq!(short_vendor("Samsung Electronics Co Ltd"), "Samsung");
        assert_eq!(short_vendor("Micron Technology Inc"), "Micron");
        assert_eq!(short_vendor("Logitech, Inc."), "Logitech");
        assert_eq!(short_device("Wi-Fi 6E AX211"), "Wi-Fi 6E AX211");
        assert_eq!(short_device("AD107M [GeForce RTX 4050 Max-Q / Mobile]"), "GeForce RTX 4050 Laptop");
        assert_eq!(short_device("AD107 High Definition Audio Controller"), "AD107 HD Audio");
        assert_eq!(short_vendor("ASUSTeK COMPUTER INC."), "ASUS");
        assert_eq!(join_name("Intel", "Wi-Fi 6E AX211"), "Intel Wi-Fi 6E AX211");
        assert_eq!(join_name("Intel", "Intel Iris Xe"), "Intel Iris Xe");
    }

    #[test]
    fn fits_and_sizes() {
        assert_eq!(fit("short", 10), "short");
        assert_eq!(fit("a very long device name", 10), "a very lo…");
        assert_eq!(human_size(512_110_190_592), "512GB");
        assert_eq!(human_size(1_000_204_886_016), "1TB");
        assert_eq!(human_size(62_537_072_640), "62.5GB");
    }
}
