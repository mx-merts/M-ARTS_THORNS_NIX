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
    warp_said().0
}

/// WARP's state and what warp-cli actually said: its status line, or the
/// error ("no answer in 5s"). The watcher keeps the second part so a wrong
/// "TUNNEL SEVERED" can be traced (SKIN TUN → WATCHER).
pub fn warp_said() -> (Warp, String) {
    match sys::run("warp-cli", &["--accept-tos", "status"], 5) {
        Ok(out) => {
            let line = out.lines().find(|l| l.contains("Status update")).unwrap_or("").trim().to_string();
            let low = line.to_lowercase();
            let w = if low.contains("connected") && !low.contains("disconnected") {
                Warp::Connected
            } else if low.contains("connecting") {
                Warp::Connecting
            } else {
                Warp::Disconnected
            };
            let said = if line.is_empty() { out.lines().next().unwrap_or("no output").trim().to_string() } else { line };
            (w, said)
        }
        Err(e) => (Warp::Missing, e),
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

/// Outer endpoints of the VPN tunnels the kill switch must let through:
///   - kernel WireGuard (Proton's classic WireGuard): `wg show`
///   - Proton protocols (Smart / Wireguard UDP-TCP / Stealth, NetworkManager
///     plugin "nm-protun"): a userspace tun device, invisible to `wg`. Its
///     server is in the active VPN connection's settings JSON ("endpoint"),
///     which NetworkManager publishes while the connection is still
///     activating, so the rule is in place before the first packet.
///
/// Sorted and deduplicated, so the ruleset only changes when an endpoint does.
pub fn vpn_endpoints() -> Vec<String> {
    let mut eps = wg_endpoints();
    eps.extend(protun_endpoints());
    eps.sort();
    eps.dedup();
    eps
}

fn wg_endpoints() -> Vec<String> {
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

fn protun_endpoints() -> Vec<String> {
    let active = sys::run("nmcli", &["-t", "-f", "UUID,TYPE", "connection", "show", "--active"], 3)
        .unwrap_or_default();
    active
        .lines()
        .filter_map(|l| l.split_once(':'))
        .filter(|(_, kind)| *kind == "vpn")
        .flat_map(|(uuid, _)| {
            let data = sys::run("nmcli", &["-g", "vpn.data", "connection", "show", "uuid", uuid], 3)
                .unwrap_or_default();
            endpoints_in_vpn_data(&data)
        })
        .collect()
}

/// `vpn.data` holds a JSON blob, escaped by nmcli (`\:`, `\\`), e.g.
/// `"endpoint"\: "185.107.56.69"`. Splitting on quotes is enough: the value
/// is the first of the next few tokens that parses as an IP address.
fn endpoints_in_vpn_data(data: &str) -> Vec<String> {
    let toks: Vec<&str> = data.split('"').collect();
    let mut out = Vec::new();
    for (i, t) in toks.iter().enumerate() {
        if *t != "endpoint" {
            continue;
        }
        if let Some(ip) = toks.iter().skip(i + 1).take(3).find_map(|v| v.trim().parse::<std::net::IpAddr>().ok()) {
            out.push(ip.to_string());
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_protun_endpoint_in_vpn_data() {
        // Real nmcli -g vpn.data output of a Proton protocols connection.
        let data = r#"settings = {"version"\: 1\\\, "peers"\: [{"id"\: "NL-FREE#126"\\\, "endpoint"\: "185.107.56.69"\\\, "udp-ports"\: [1224\\\, 443]\\\, "tcp-ports"\: [443]\\\, "tls-ports"\: [443]\\\, "priority"\: 0}]\\\, "pcap-file"\: null}"#;
        assert_eq!(endpoints_in_vpn_data(data), vec!["185.107.56.69"]);
        let v6 = r#""endpoint"\: "2a07:b944::2:2""#;
        assert_eq!(endpoints_in_vpn_data(v6), vec!["2a07:b944::2:2"]);
        assert!(endpoints_in_vpn_data("remote = 1.2.3.4").is_empty());
    }

    #[test]
    fn splits_terse_output() {
        assert_eq!(split_terse("wlo1:wifi:connected:abc"), vec!["wlo1", "wifi", "connected", "abc"]);
        assert_eq!(split_terse("yes:My\\:Cafe"), vec!["yes", "My:Cafe"]);
    }
}
