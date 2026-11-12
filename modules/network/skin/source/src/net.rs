//! What the network looks like right now: uplink, SSID, gateway, WARP,
//! MAC address. Everything is read with a time limit.

use crate::state;
use crate::sys::{self, p};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Net {
    /// Connected uplink (Wi-Fi first, else Ethernet): "wlo1".
    pub iface: Option<String>,
    pub wifi: bool,
    /// NetworkManager connection of the uplink.
    pub uuid: Option<String>,
    pub ssid: Option<String>,
    /// Every physical uplink, connected or not (firewall scope).
    pub uplinks: Vec<String>,
    pub gw4: Option<String>,
    pub gw6: Option<String>,
    pub known: bool,
}

/// nmcli -t escapes ':' and '\' inside values.
pub fn split_terse(line: &str) -> Vec<String> {
    let mut out = vec![String::new()];
    let mut esc = false;
    for c in line.chars() {
        if esc {
            out.last_mut().unwrap().push(c);
            esc = false;
        } else if c == '\\' {
            esc = true;
        } else if c == ':' {
            out.push(String::new());
        } else {
            out.last_mut().unwrap().push(c);
        }
    }
    out
}

fn route_via(family: &str, iface: &str) -> Option<String> {
    let out = sys::run("ip", &[family, "route", "show", "default", "dev", iface], 3).ok()?;
    let mut w = out.split_whitespace();
    while let Some(t) = w.next() {
        if t == "via" {
            return w.next().map(str::to_string);
        }
    }
    None
}

/// Physical uplinks straight from sysfs (a "device" link and type 1 =
/// Ethernet-like: Wi-Fi and Ethernet cards, not lo / tunnels / bridges).
/// Known before NetworkManager starts: the kill switch covers them from the
/// first packet. Nothing else is filled in.
pub fn uplinks() -> Net {
    let mut n = Net::default();
    for e in std::fs::read_dir(p("/sys/class/net")).into_iter().flatten().flatten() {
        let path = e.path();
        let name = e.file_name().to_string_lossy().into_owned();
        let is_ether = sys::read(&path.join("type")).as_deref() == Some("1");
        if path.join("device").exists() && is_ether && !n.uplinks.contains(&name) {
            n.uplinks.push(name);
        }
    }
    n.uplinks.sort();
    n
}

pub fn scan() -> Net {
    let mut n = uplinks();
    let devices = sys::run("nmcli", &["-t", "-f", "DEVICE,TYPE,STATE,CON-UUID", "device"], 5).unwrap_or_default();
    let mut eth: Option<(String, String)> = None;
    for line in devices.lines() {
        let f = split_terse(line);
        if f.len() < 4 {
            continue;
        }
        let (dev, kind, st, uuid) = (&f[0], &f[1], &f[2], &f[3]);
        if kind != "wifi" && kind != "ethernet" {
            continue;
        }
        if !n.uplinks.contains(dev) {
            n.uplinks.push(dev.clone());
        }
        if !st.starts_with("connected") || n.iface.is_some() && n.wifi {
            continue;
        }
        if kind == "wifi" {
            n.iface = Some(dev.clone());
            n.uuid = Some(uuid.clone());
            n.wifi = true;
        } else if eth.is_none() {
            eth = Some((dev.clone(), uuid.clone()));
        }
    }
    if n.iface.is_none() {
        if let Some((dev, uuid)) = eth {
            n.iface = Some(dev);
            n.uuid = Some(uuid);
        }
    }
    if let Some(i) = n.iface.clone() {
        if n.wifi {
            let aps = sys::run("nmcli", &["-t", "-f", "ACTIVE,SSID", "device", "wifi", "list", "ifname", &i, "--rescan", "no"], 5)
                .unwrap_or_default();
            n.ssid = aps.lines().map(split_terse).find(|f| f.len() >= 2 && f[0] == "yes").map(|f| f[1..].join(":"));
        }
        n.gw4 = route_via("-4", &i);
        n.gw6 = route_via("-6", &i);
    }
    let known = state::known_networks();
    n.known = n.ssid.as_ref().is_some_and(|s| known.contains(s));
    n
}

impl Net {
    /// "wlo1 · Cafe_Nero_FREE"
    pub fn label(&self) -> String {
        match (&self.iface, &self.ssid) {
            (Some(i), Some(s)) => format!("{i} · {s}"),
            (Some(i), None) => i.clone(),
            _ => "no network".into(),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Warp {
    Connected,
    Connecting,
    Disconnected,
    Missing,
}

pub fn warp() -> Warp {
    match sys::run("warp-cli", &["--accept-tos", "status"], 5) {
        Ok(out) => {
            let first = out.lines().find(|l| l.contains("Status update")).unwrap_or("").to_lowercase();
            if first.contains("connected") && !first.contains("disconnected") {
                Warp::Connected
            } else if first.contains("connecting") {
                Warp::Connecting
            } else {
                Warp::Disconnected
            }
        }
        Err(_) => Warp::Missing,
    }
}

pub fn dnscrypt_active() -> bool {
    sys::run("systemctl", &["is-active", "--quiet", "dnscrypt-proxy"], 3).is_ok()
}

/// Can a name be resolved right now? (bounded: never hangs)
pub fn dns_ok() -> bool {
    sys::run("resolvectl", &["query", "--cache=no", "--legend=no", "nixos.org"], 8).is_ok()
}

/// (current MAC, permanent MAC) of an interface.
pub fn macs(iface: &str) -> (Option<String>, Option<String>) {
    let cur = sys::read(&p(&format!("/sys/class/net/{iface}/address")));
    // "ip -d link" prints "permaddr xx:..." only when the MAC was changed.
    let perm = sys::run("ip", &["-d", "link", "show", "dev", iface], 3).ok().and_then(|out| {
        let mut w = out.split_whitespace();
        while let Some(t) = w.next() {
            if t == "permaddr" {
                return w.next().map(str::to_string);
            }
        }
        None
    });
    let perm = perm.or_else(|| cur.clone());
    (cur, perm)
}

/// Endpoints of WireGuard tunnels (Proton): the kill switch lets them through.
pub fn wg_endpoints() -> Vec<String> {
    let out = sys::run("wg", &["show", "all", "endpoints"], 3).unwrap_or_default();
    out.lines()
        .filter_map(|l| l.split_whitespace().nth(2))
        .filter_map(|ep| {
            let host = match ep.strip_prefix('[') {
                Some(rest) => rest.split(']').next()?,
                None => ep.rsplit_once(':')?.0,
            };
            (!host.is_empty() && host != "(none)").then(|| host.to_string())
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_terse_output() {
        assert_eq!(split_terse("wlo1:wifi:connected:abc"), vec!["wlo1", "wifi", "connected", "abc"]);
        assert_eq!(split_terse("yes:My\\:Cafe"), vec!["yes", "My:Cafe"]);
    }
}
