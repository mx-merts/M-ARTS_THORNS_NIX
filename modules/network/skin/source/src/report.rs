//! What SKIN prints: the state, one layer in detail, help.

use crate::firewall;
use crate::model::{Layer, Mode};
use crate::net::{self, Net, Warp};
use crate::paint::{Paint, Role, VERSION};
use crate::state::{self, State};
use crate::sys;

const RULE: usize = 69;

/// Everything the status screen needs, read once.
pub struct Live {
    pub st: State,
    pub net: Net,
    pub warp: Warp,
    pub dnscrypt: bool,
    pub mac: (Option<String>, Option<String>),
}

pub fn live() -> Live {
    let net = net::scan();
    let mac = net.iface.as_deref().map(net::macs).unwrap_or((None, None));
    Live { st: state::load(), warp: net::warp(), dnscrypt: net::dnscrypt_active(), mac, net }
}

/// (works as asked?, what it is now, detail)
fn describe(l: Layer, lv: &Live) -> (bool, String, String) {
    let m = lv.st.mode(l);
    let real_mac = lv.mac.0.is_some() && lv.mac.0 == lv.mac.1;
    match (l, m) {
        (Layer::Dns, Mode::Seal) if lv.dnscrypt => (true, "encrypted · filtered".into(), "dnscrypt · cloudflare-security + quad9".into()),
        (Layer::Dns, Mode::Seal) => (false, "dnscrypt not running".into(), "sudo SKIN SEAL DNS".into()),
        (Layer::Dns, _) => (true, "network DNS · plain".into(), "only for logging in".into()),
        (Layer::Tun, Mode::Seal) => match lv.warp {
            Warp::Connected => (true, "WARP connected".into(), "everything goes through Cloudflare".into()),
            Warp::Connecting => (false, "WARP connecting".into(), "network may block WARP".into()),
            Warp::Disconnected => (false, "WARP disconnected".into(), "sudo SKIN SEAL TUN".into()),
            Warp::Missing => (false, "warp-svc not answering".into(), "systemctl status warp-svc".into()),
        },
        (Layer::Tun, _) => (true, "withdrawn".into(), format!("WARP {}", if lv.warp == Warp::Connected { "still up" } else { "off" })),
        (Layer::Ks, Mode::Seal) => (true, "armed".into(), "nothing leaves outside the tunnel".into()),
        (Layer::Ks, _) => (true, "disarmed".into(), "traffic may leave outside the tunnel".into()),
        (Layer::Lan, Mode::Seal) if lv.net.known => (true, "open".into(), "known network: printers, phone".into()),
        (Layer::Lan, Mode::Seal) => (true, "isolated".into(), "unknown network: gateway only".into()),
        (Layer::Lan, Mode::Sever) => (true, "gateway only".into(), "other devices unreachable".into()),
        (Layer::Lan, Mode::Rot) => (true, "open".into(), "every local device reachable".into()),
        (Layer::Id, Mode::Rot) => (real_mac, if real_mac { "real MAC".into() } else { "real MAC pending".into() }, "hostname sent".into()),
        (Layer::Id, _) if real_mac => (m != Mode::Seal, "REAL MAC".into(), "sudo SKIN SEAL ID".into()),
        (Layer::Id, _) => (true, "masked".into(), "MAC per network and boot · no hostname".into()),
    }
}

fn pad(s: &str, w: usize) -> String {
    let n = s.chars().count();
    format!("{s}{}", " ".repeat(w.saturating_sub(n)))
}

fn since_text(t: u64) -> String {
    if t == 0 { "since boot".into() } else { format!("since {}", sys::clock(t)) }
}

pub fn status(pt: &Paint) -> String {
    let lv = live();
    let mut out = vec![
        format!(" {} {} {}", pt.title(), pt.c(Role::Strong, VERSION), pt.c(Role::Dim, "── NETWORK MEMBRANE")),
        String::new(),
    ];
    let (word, role, sym) = match lv.st.whole() {
        Some(m) => (m.state_word(), Paint::mode_role(m), m.symbol()),
        None => ("CUSTOM", Role::Custom, "◆"),
    };
    let since = lv.st.since.iter().copied().max().unwrap_or(0);
    let mut line = format!(" {}  {}   {}", pt.c(Role::Dim, "STATE  "), pt.c(role, &format!("{sym} {word}")), pt.c(Role::Dim, &since_text(since)));
    if let Some((m, deadline)) = lv.st.timer {
        let left = deadline.saturating_sub(sys::now());
        line += &format!("   {}", pt.c(Role::Warn, &format!("{} · seals itself in {}", m.name(), sys::duration(left))));
    }
    out.push(line);
    let trust = if lv.net.iface.is_none() {
        String::new()
    } else if lv.net.known {
        pt.c(Role::Good, "KNOWN")
    } else {
        pt.c(Role::Warn, "UNKNOWN")
    };
    out.push(format!(" {}  {}   {}", pt.c(Role::Dim, "NETWORK"), pad(&lv.net.label(), 34), trust));
    out.push(String::new());
    out.push(format!(" {} {}", pt.c(Role::Strong, "▌LAYERS"), pt.c(Role::Dim, &"─".repeat(RULE - 9))));
    for l in Layer::ALL {
        let m = lv.st.mode(l);
        let (ok, what, detail) = describe(l, &lv);
        let (sym, role) = if ok { (m.symbol(), Paint::mode_role(m)) } else { ("✖", Role::Bad) };
        let mark = pt.c(role, &format!("{sym} {:<5}", m.name()));
        let what = if ok { pad(&what, 26) } else { pt.c(Role::Bad, &pad(&what, 26)) };
        let since = if m != Mode::Seal { format!(" · {}", since_text(lv.st.since[l.index()])) } else { String::new() };
        out.push(format!("   {}   {}   {} {}", pt.c(Role::Strong, &pad(l.name(), 10)), mark, what, pt.c(Role::Dim, &format!("{detail}{since}"))));
    }
    out.push(String::new());
    out.push(format!(
        " {} SEAL   {} SEVER   {} ROT   {} CUSTOM   {} FAILED",
        pt.c(Role::Good, "●"),
        pt.c(Role::Warn, "✂"),
        pt.c(Role::Bad, "☠"),
        pt.c(Role::Custom, "◆"),
        pt.c(Role::Bad, "✖")
    ));
    out.push(pt.c(Role::Dim, " SKIN SEAL | SEVER | ROT [LAYER]     SKIN <LAYER>     SKIN HELP"));
    out.join("\n")
}

pub fn layer(pt: &Paint, l: Layer) -> String {
    let lv = live();
    let m = lv.st.mode(l);
    let (ok, what, detail) = describe(l, &lv);
    let (sym, role) = if ok { (m.symbol(), Paint::mode_role(m)) } else { ("✖", Role::Bad) };
    let mut rows: Vec<(&str, String)> = vec![("NOW", format!("{what} · {detail}"))];
    match l {
        Layer::Dns => {
            rows.push(("SEAL", "dnscrypt-proxy: encrypted, malware domains filtered".into()));
            rows.push(("SEVER / ROT", "the network's own DNS, plain (login pages need it)".into()));
        }
        Layer::Tun => {
            rows.push(("PROVIDER", "Cloudflare WARP · tunnel_only · MASQUE".into()));
            rows.push(("STATUS", format!("{:?}", lv.warp).to_lowercase()));
            rows.push(("SEAL", "WARP up, everything through it".into()));
            rows.push(("SEVER / ROT", "WARP withdrawn (login pages, networks that block it)".into()));
        }
        Layer::Ks => {
            let (held, _) = firewall::counters();
            rows.push(("SEAL", "only the tunnel, WARP itself and encrypted DNS may go out".into()));
            rows.push(("SEVER / ROT", "disarmed".into()));
            rows.push(("HELD", format!("{held} packets kept inside since sealed")));
            rows.push(("RULES", format!("sudo nft list table {}", firewall::TABLE)));
        }
        Layer::Lan => {
            let (_, lan) = firewall::counters();
            rows.push(("SEAL", "unknown network: gateway only · known network: open".into()));
            rows.push(("SEVER", "gateway only".into()));
            rows.push(("ROT", "open".into()));
            rows.push(("BLOCKED", format!("{lan} packets to local devices refused")));
            rows.push(("KNOWN", state::known_networks().join(", ")));
            rows.push(("TRUST", "sudo SKIN TRUST (this Wi-Fi) · sudo SKIN DISTRUST".into()));
        }
        Layer::Id => {
            rows.push(("MAC NOW", lv.mac.0.clone().unwrap_or_else(|| "?".into())));
            rows.push(("SEAL", "MAC new on every network and every boot, no hostname".into()));
            rows.push(("SEVER", "unchanged (the login page remembers this MAC)".into()));
            rows.push(("ROT", "real MAC + hostname (networks that register MACs)".into()));
            rows.push(("NOTE", "changing it reconnects the network".into()));
        }
    }
    rows.push(("CHANGE", format!("sudo SKIN SEAL {0} · sudo SKIN SEVER {0} · sudo SKIN ROT {0}", l.short())));
    let mut out = vec![format!(" {}   {}", pt.c(Role::Strong, &format!("▌{}", l.name())), pt.c(role, &format!("{sym} {}", m.name())))];
    for (k, v) in rows {
        if !v.is_empty() {
            out.push(format!("   {}  {v}", pt.c(Role::Dim, &pad(k, 12))));
        }
    }
    out.join("\n")
}

pub fn help(pt: &Paint) -> String {
    let d = |s: &str| pt.c(Role::Dim, s);
    let rows = [
        ("SKIN", "state of every layer", ""),
        ("SKIN <LAYER>", "one layer in detail", "SKIN TUN"),
        ("SKIN SEAL", "everything at full strength", "sudo"),
        ("SKIN SEVER", "open for a login page", "sudo · 10 min"),
        ("SKIN ROT", "drop every protection", "sudo · 30 min · asks"),
        ("SKIN SEAL|SEVER|ROT <L..>", "only those layers, no timer", "sudo SKIN ROT KS"),
        ("SKIN TRUST | DISTRUST", "Wi-Fi as home: LAN open", "sudo"),
    ];
    let mut out = vec![format!(" {} {} {}", pt.title(), pt.c(Role::Strong, VERSION), d("── NETWORK MEMBRANE")), String::new()];
    for (cmd, what, ex) in rows {
        out.push(format!(" {}  {} {}", pt.c(Role::Strong, &pad(cmd, 27)), pad(what, 28), d(ex)).trim_end().to_string());
    }
    out.push(String::new());
    out.push(format!(" {}  DNS · TUNNEL TUN · KILLSWITCH KS · LAN · IDENTITY ID", d("LAYERS")));
    out.push(d(" Modes end at reboot, TRUST stays. Incoming connections are never opened."));
    out.push(d(" Login page (cafe, hotel): sudo SKIN SEVER → log in → sudo SKIN SEAL"));
    out.join("\n")
}
