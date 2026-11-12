//! SKIN WATCH: the root daemon (skin-watch.service).
//!   - keeps the nftables table in line with the state (also re-creates it
//!     if a firewall reload removed it, and follows Proton's endpoints)
//!   - seals the membrane when a SEVER / ROT timer runs out
//!   - watches the tunnel and DNS, reports a new network
//!
//! Every finding becomes an event for the user's notifier.

use crate::firewall;
use crate::layers;
use crate::model::{Layer, Mode};
use crate::net::{self, Warp};
use crate::state::{self, State};
use crate::sys::{self, p};

const TICK_MS: u64 = 3000;

#[derive(Default)]
struct Watch {
    last_rules: String,
    fw_error: bool,
    last_ssid: Option<String>,
    first: bool,
    tun_down_since: Option<u64>,
    tun_severed: bool,
    tun_rotting: bool,
    dns_fails: u32,
    dns_severed: bool,
    last_dns_check: u64,
}

pub fn run() -> i32 {
    if !sys::is_root() {
        eprintln!("SKIN WATCH must run as root (skin-watch.service)");
        return 1;
    }
    // A fresh boot (/run is empty): everything sealed.
    if !p(&format!("{}/state", state::DIR)).exists() {
        let _ = state::save(&State::default());
    }
    // First rules from sysfs alone, before NetworkManager (it waits for
    // READY: Before=network-pre.target). No nmcli here: asking
    // NetworkManager now would start it early and wait on ourselves.
    let st = state::load();
    let first = firewall::ruleset(&firewall::Plan { ks: st.mode(Layer::Ks), lan: st.mode(Layer::Lan), net: &net::uplinks(), wg: &[] });
    if let Err(e) = firewall::apply(&first) {
        state::event("KILLSWITCH SEVERED", &format!("firewall rules failed: {e}"), true);
    }
    sys::notify_ready();
    let mut w = Watch { first: true, ..Watch::default() };
    loop {
        tick(&mut w);
        sys::sleep_ms(TICK_MS);
    }
}

fn tick(w: &mut Watch) {
    let now = sys::now();
    let mut st = state::load();

    // 1. Timer of a whole-membrane SEVER / ROT.
    if let Some((mode, deadline)) = st.timer {
        if now >= deadline {
            st.timer = None;
            let outcomes = layers::transition(&Layer::ALL, Mode::Seal, &mut st, None);
            let _ = state::save(&st);
            state::event(&format!("{} EXPIRED ── SEALED", mode.name()), &format!("sealed by itself · {}", layers::summary(&outcomes)), false);
            w.last_rules.clear();
            return;
        }
    }

    let n = net::scan();

    // 2. Firewall follows the state, the network and the WireGuard endpoints.
    let wg = net::wg_endpoints();
    let script = firewall::ruleset(&firewall::Plan { ks: st.mode(Layer::Ks), lan: st.mode(Layer::Lan), net: &n, wg: &wg });
    if script != w.last_rules || !firewall::present() {
        match firewall::apply(&script) {
            Ok(()) => {
                if w.fw_error {
                    state::event("KILLSWITCH SEALED", "firewall rules loaded again", false);
                }
                w.fw_error = false;
                w.last_rules = script.clone();
                let _ = sys::write_atomic(&p(&format!("{}/rules", state::DIR)), &script);
            }
            Err(e) => {
                if !w.fw_error {
                    state::event("KILLSWITCH SEVERED", &format!("firewall rules failed: {e}"), true);
                }
                w.fw_error = true;
            }
        }
    }

    // 3. A new network.
    if n.iface.is_some() && n.ssid != w.last_ssid {
        if !w.first || n.ssid.is_some() {
            if let Some(ssid) = &n.ssid {
                if !n.known {
                    let lan = if st.mode(Layer::Lan) == Mode::Seal { "LAN isolated" } else { "LAN not isolated" };
                    state::event(&format!("UNKNOWN NETWORK ── {ssid}"), &format!("{lan} · login page? sudo SKIN SEVER"), false);
                }
            }
        }
        w.last_ssid = n.ssid.clone();
        w.tun_down_since = None;
    }
    w.first = false;

    // 4. Tunnel. What warp-cli said is kept in /run/skin/watch (SKIN TUN).
    if st.mode(Layer::Tun) == Mode::Seal && n.iface.is_some() {
        let (warp, said) = net::warp_said();
        let _ = sys::write_atomic(&p(&format!("{}/watch", state::DIR)), &format!("{now}\t{warp:?}\t{said}\n"));
        if warp == Warp::Connected {
            if w.tun_severed {
                state::event("TUNNEL SEALED", "WARP back · traffic released", false);
            }
            w.tun_down_since = None;
            w.tun_severed = false;
            w.tun_rotting = false;
        } else {
            let since = *w.tun_down_since.get_or_insert(now);
            let held = st.mode(Layer::Ks) == Mode::Seal;
            if now - since >= 15 && !w.tun_severed {
                let body = if held {
                    format!("WARP lost · traffic held by KILLSWITCH · warp-cli: {said} · without WARP: sudo SKIN SEVER TUN KS")
                } else {
                    format!("WARP lost · traffic goes out directly (KILLSWITCH is off) · warp-cli: {said}")
                };
                state::event("TUNNEL SEVERED", &body, true);
                w.tun_severed = true;
            }
            if now - since >= 300 && !w.tun_rotting {
                state::event("TUNNEL ROTTING", "WARP down for 5 min · this network may block WARP · sudo SKIN SEVER TUN KS", true);
                w.tun_rotting = true;
            }
        }
    } else {
        w.tun_down_since = None;
        w.tun_severed = false;
        w.tun_rotting = false;
    }

    // 5. DNS, every 30 s while connected.
    if st.mode(Layer::Dns) == Mode::Seal && n.iface.is_some() && now - w.last_dns_check >= 30 {
        w.last_dns_check = now;
        if net::dnscrypt_active() && net::dns_ok() {
            if w.dns_severed {
                state::event("DNS SEALED", "encrypted DNS answers again", false);
            }
            w.dns_fails = 0;
            w.dns_severed = false;
        } else {
            w.dns_fails += 1;
            if w.dns_fails >= 2 && !w.dns_severed {
                state::event("DNS SEVERED", "dnscrypt does not answer · retrying", true);
                w.dns_severed = true;
            }
        }
    }
}
