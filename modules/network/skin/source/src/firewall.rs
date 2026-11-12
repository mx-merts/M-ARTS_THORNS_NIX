//! KILLSWITCH and LAN live in one nftables table, "inet skin", rebuilt as a
//! whole every time (atomic: one nft transaction). It only filters traffic
//! leaving through a physical uplink (Wi-Fi / Ethernet); loopback, tunnels
//! and everything else pass untouched. Incoming traffic is not touched here:
//! the NixOS firewall already closes it, in every mode.

use crate::model::Mode;
use crate::net::Net;
use crate::sys;

pub const TABLE: &str = "inet skin";

/// Cloudflare WARP client endpoints (developers.cloudflare.com, "Cloudflare
/// One Client with firewall"): orchestration API, consumer WARP (192), Zero
/// Trust WireGuard (193) and MASQUE (197, connectivity checks included),
/// consumer MASQUE (198) and their IPv6 ranges. All Cloudflare's own.
const WARP4: &str = "162.159.137.105, 162.159.138.105, 162.159.192.0/24, 162.159.193.0/24, 162.159.197.0/24, 162.159.198.0/24";
const WARP6: &str = "2606:4700:7::a29f:8969, 2606:4700:7::a29f:8a69, 2606:4700:d0::/48, 2606:4700:d1::/48, 2606:4700:100::/48, 2606:4700:102::/48, 2606:4700:103::/48";
/// Encrypted DNS resolvers of dns.nix (cloudflare-security DoH, Quad9
/// DNSCrypt) and dnscrypt-proxy's bootstrap resolver. Already encrypted: they
/// may go out while the tunnel is down, so WARP itself can resolve names.
const DNS4: &str = "1.1.1.2, 1.0.0.2, 9.9.9.9, 149.112.112.9, 149.112.112.112, 9.9.9.11";
const DNS6: &str = "2606:4700:4700::1112, 2606:4700:4700::1002";
/// Private, link-local, CGNAT, multicast and broadcast: "the local network".
const LAN4: &str = "10.0.0.0/8, 100.64.0.0/10, 169.254.0.0/16, 172.16.0.0/12, 192.168.0.0/16, 224.0.0.0/4, 255.255.255.255";
const LAN6: &str = "fc00::/7, fe80::/10, ff00::/8";

pub struct Plan<'a> {
    pub ks: Mode,
    pub lan: Mode,
    pub net: &'a Net,
    pub wg: &'a [String],
}

impl Plan<'_> {
    /// LAN open: ROT always, SEAL only on a known network.
    pub fn lan_open(&self) -> bool {
        self.lan == Mode::Rot || (self.lan == Mode::Seal && self.net.known)
    }
    pub fn ks_armed(&self) -> bool {
        self.ks == Mode::Seal
    }
}

fn quote_list(items: &[String]) -> String {
    items.iter().map(|i| format!("\"{i}\"")).collect::<Vec<_>>().join(", ")
}

/// The complete nft script for one state of KS + LAN.
pub fn ruleset(plan: &Plan) -> String {
    let mut r = String::new();
    r += "table inet skin\ndelete table inet skin\ntable inet skin {\n";
    r += &format!("\tset warp4 {{ type ipv4_addr; flags interval; elements = {{ {WARP4} }} }}\n");
    r += &format!("\tset warp6 {{ type ipv6_addr; flags interval; elements = {{ {WARP6} }} }}\n");
    r += &format!("\tset dns4 {{ type ipv4_addr; elements = {{ {DNS4} }} }}\n");
    r += &format!("\tset dns6 {{ type ipv6_addr; elements = {{ {DNS6} }} }}\n");
    r += &format!("\tset lan4 {{ type ipv4_addr; flags interval; elements = {{ {LAN4} }} }}\n");
    r += &format!("\tset lan6 {{ type ipv6_addr; flags interval; elements = {{ {LAN6} }} }}\n");
    r += "\tchain out {\n\t\ttype filter hook output priority filter; policy accept;\n";

    let uplinks = &plan.net.uplinks;
    if uplinks.is_empty() || (!plan.ks_armed() && plan.lan_open()) {
        r += "\t}\n}\n";
        return r;
    }
    // Only traffic leaving through a physical uplink is our business.
    r += &format!("\t\toifname != {{ {} }} accept\n", quote_list(uplinks));
    // Getting an address must always work.
    r += "\t\tudp sport 68 udp dport 67 accept\n";
    r += "\t\tudp dport 547 accept\n";
    r += "\t\ticmpv6 type { nd-router-solicit, nd-neighbor-solicit, nd-neighbor-advert, mld2-listener-report } accept\n";

    // LAN layer.
    if plan.lan_open() {
        r += "\t\tip daddr @lan4 accept\n\t\tip6 daddr @lan6 accept\n";
    } else {
        if let Some(gw) = &plan.net.gw4 {
            r += &format!("\t\tip daddr {gw} accept\n");
        }
        if let Some(gw) = &plan.net.gw6 {
            r += &format!("\t\tip6 daddr {gw} accept\n");
        }
        r += "\t\tip daddr @lan4 counter reject with icmpx admin-prohibited comment \"skin: LAN isolated\"\n";
        r += "\t\tip6 daddr @lan6 counter reject with icmpx admin-prohibited comment \"skin: LAN isolated\"\n";
    }

    // KILLSWITCH layer.
    if plan.ks_armed() {
        r += "\t\tip daddr @warp4 accept\n\t\tip6 daddr @warp6 accept\n";
        r += "\t\tip daddr @dns4 accept\n\t\tip6 daddr @dns6 accept\n";
        let (v6, v4): (Vec<&String>, Vec<&String>) = plan.wg.iter().partition(|a| a.contains(':'));
        if !v4.is_empty() {
            r += &format!("\t\tip daddr {{ {} }} accept comment \"skin: wireguard endpoints\"\n", join(&v4));
        }
        if !v6.is_empty() {
            r += &format!("\t\tip6 daddr {{ {} }} accept comment \"skin: wireguard endpoints\"\n", join(&v6));
        }
        r += "\t\tcounter reject with icmpx admin-prohibited comment \"skin: outside the tunnel\"\n";
    }
    r += "\t}\n}\n";
    r
}

fn join(items: &[&String]) -> String {
    items.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", ")
}

/// Loads the ruleset (one atomic transaction).
pub fn apply(script: &str) -> Result<(), String> {
    sys::run_input("nft", &["-f", "-"], Some(script), 10).map(|_| ())
}

/// Is our table still there? (a firewall reload could remove it)
pub fn present() -> bool {
    sys::run("nft", &["list", "table", "inet", "skin"], 5).is_ok()
}

/// Packets held / rejected so far: (outside the tunnel, LAN isolated).
pub fn counters() -> (u64, u64) {
    let out = sys::run("nft", &["list", "table", "inet", "skin"], 5).unwrap_or_default();
    let count = |tag: &str| {
        out.lines()
            .filter(|l| l.contains(tag))
            .filter_map(|l| {
                let mut w = l.split_whitespace();
                while let Some(t) = w.next() {
                    if t == "packets" {
                        return w.next()?.parse::<u64>().ok();
                    }
                }
                None
            })
            .sum::<u64>()
    };
    (count("skin: outside the tunnel"), count("skin: LAN isolated"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn net(known: bool) -> Net {
        Net {
            iface: Some("wlo1".into()),
            wifi: true,
            uplinks: vec!["wlo1".into(), "enp46s0".into()],
            gw4: Some("192.168.1.1".into()),
            gw6: Some("fe80::1".into()),
            known,
            ..Net::default()
        }
    }

    #[test]
    fn sealed_on_unknown_network_isolates_and_arms() {
        let n = net(false);
        let r = ruleset(&Plan { ks: Mode::Seal, lan: Mode::Seal, net: &n, wg: &[] });
        assert!(r.contains("oifname != { \"wlo1\", \"enp46s0\" } accept"));
        assert!(r.contains("ip daddr 192.168.1.1 accept"));
        assert!(r.contains("skin: LAN isolated"));
        assert!(r.contains("skin: outside the tunnel"));
    }

    #[test]
    fn sealed_on_known_network_keeps_lan_open() {
        let n = net(true);
        let r = ruleset(&Plan { ks: Mode::Seal, lan: Mode::Seal, net: &n, wg: &[] });
        assert!(r.contains("ip daddr @lan4 accept"));
        assert!(!r.contains("skin: LAN isolated"));
        assert!(r.contains("skin: outside the tunnel"));
    }

    #[test]
    fn rotten_is_an_empty_chain() {
        let n = net(false);
        let r = ruleset(&Plan { ks: Mode::Rot, lan: Mode::Rot, net: &n, wg: &[] });
        assert!(!r.contains("reject"));
        assert!(!r.contains("oifname"));
    }

    #[test]
    fn wireguard_endpoints_pass_the_kill_switch() {
        let n = net(false);
        let wg = vec!["185.159.157.10".to_string(), "2a07:b944::2:2".to_string()];
        let r = ruleset(&Plan { ks: Mode::Seal, lan: Mode::Sever, net: &n, wg: &wg });
        assert!(r.contains("ip daddr { 185.159.157.10 } accept"));
        assert!(r.contains("ip6 daddr { 2a07:b944::2:2 } accept"));
    }
}
