//! Moving layers between SEAL, SEVER and ROT. Every step has a time limit
//! and a failing step never stops the ones after it: the membrane always
//! ends in a known state.

use crate::firewall::{self, Plan};
use crate::model::{Layer, Mode};
use crate::net::{self, Net, Warp};
use crate::paint::Paint;
use crate::state::{self, State};
use crate::sys;

pub type Outcome = (Layer, Result<String, String>);

/// The order layers change in. Sealing: DNS first (WARP may need it), the
/// firewall after the tunnel is asked to come up, identity last (it
/// reconnects). Opening: the firewall first, so nothing is held meanwhile.
pub fn order(mode: Mode) -> [Layer; 5] {
    match mode {
        Mode::Seal => [Layer::Dns, Layer::Tun, Layer::Ks, Layer::Lan, Layer::Id],
        Mode::Sever => [Layer::Ks, Layer::Lan, Layer::Tun, Layer::Dns, Layer::Id],
        Mode::Rot => [Layer::Ks, Layer::Lan, Layer::Id, Layer::Tun, Layer::Dns],
    }
}

/// What each layer becomes, for the step line ("DNS → encrypted").
pub fn target_word(l: Layer, m: Mode, net: &Net) -> &'static str {
    match (l, m) {
        (Layer::Dns, Mode::Seal) => "encrypted",
        (Layer::Dns, _) => "network",
        (Layer::Tun, Mode::Seal) => "WARP",
        (Layer::Tun, _) => "withdrawn",
        (Layer::Ks, Mode::Seal) => "armed",
        (Layer::Ks, _) => "disarmed",
        (Layer::Lan, Mode::Seal) if net.known => "open (known)",
        (Layer::Lan, Mode::Seal) => "isolated",
        (Layer::Lan, Mode::Sever) => "gateway only",
        (Layer::Lan, Mode::Rot) => "open",
        (Layer::Id, Mode::Seal) => "masked MAC",
        (Layer::Id, Mode::Sever) => "unchanged",
        (Layer::Id, Mode::Rot) => "real MAC",
    }
}

fn need_iface(net: &Net) -> Result<String, String> {
    net.iface.clone().ok_or_else(|| "no connected network".to_string())
}

fn dns(mode: Mode, net: &Net) -> Result<String, String> {
    match mode {
        Mode::Seal => {
            sys::run("systemctl", &["start", "dnscrypt-proxy"], 20)?;
            if let Some(i) = &net.iface {
                // Gives the link back its normal DNS settings.
                let _ = sys::run("nmcli", &["device", "reapply", i], 15);
            }
            Ok(String::new())
        }
        Mode::Sever | Mode::Rot => {
            let i = need_iface(net)?;
            sys::run("systemctl", &["stop", "dnscrypt-proxy"], 15)?;
            sys::run("resolvectl", &["domain", &i, "~."], 5)?;
            Ok(String::new())
        }
    }
}

fn tun(mode: Mode) -> Result<String, String> {
    if net::warp() == Warp::Missing {
        return Err("warp-cli does not answer".into());
    }
    match mode {
        Mode::Seal => {
            let start = sys::now();
            sys::run("warp-cli", &["--accept-tos", "connect"], 10)?;
            for _ in 0..30 {
                if net::warp() == Warp::Connected {
                    return Ok(format!("connected in {}s", sys::now() - start));
                }
                sys::sleep_ms(500);
            }
            let now = match net::warp() {
                Warp::Connecting => "still connecting",
                _ => "not connected",
            };
            Err(format!("WARP {now} after 15s"))
        }
        Mode::Sever | Mode::Rot => sys::run("warp-cli", &["--accept-tos", "disconnect"], 10).map(|_| String::new()),
    }
}

/// KS and LAN: rebuild the whole nftables table from the state.
pub fn apply_firewall(st: &State, net: &Net) -> Result<String, String> {
    let wg = net::wg_endpoints();
    let script = firewall::ruleset(&Plan { ks: st.mode(Layer::Ks), lan: st.mode(Layer::Lan), net, wg: &wg });
    firewall::apply(&script)?;
    let _ = sys::write_atomic(&sys::p(&format!("{}/rules", state::DIR)), &script);
    Ok(String::new())
}

/// IDENTITY: the MAC is set per connection in memory only (nmcli
/// --temporary: nothing is written to disk) and needs a reconnect.
fn id(mode: Mode, net: &Net) -> Result<String, String> {
    let (Some(i), Some(uuid)) = (&net.iface, &net.uuid) else {
        return Ok("no network".into());
    };
    let prop = if net.wifi { "802-11-wireless.cloned-mac-address" } else { "802-3-ethernet.cloned-mac-address" };
    let (cur, perm) = net::macs(i);
    let real = cur.is_some() && cur == perm;
    let (mac, hostname) = match mode {
        Mode::Sever => return Ok(String::new()),
        // "" = the global default (random, privacy.nix)
        Mode::Seal if !real => return Ok("already masked".into()),
        Mode::Seal => ("", "no"),
        Mode::Rot if real => return Ok("already real".into()),
        Mode::Rot => ("permanent", "yes"),
    };
    sys::run(
        "nmcli",
        &[
            "connection", "modify", "--temporary", uuid,
            prop, mac,
            "ipv4.dhcp-send-hostname", hostname,
            "ipv6.dhcp-send-hostname", hostname,
        ],
        10,
    )?;
    sys::run("nmcli", &["connection", "up", uuid], 45)?;
    Ok(format!("{i} reconnected"))
}

/// Moves the given layers to `mode`. Prints step lines when `paint` is
/// given (CLI), stays quiet otherwise (watcher). Returns every outcome.
pub fn transition(layers: &[Layer], mode: Mode, st: &mut State, paint: Option<&Paint>) -> Vec<Outcome> {
    let mut net = net::scan();
    let mut out = Vec::new();
    for l in order(mode).into_iter().filter(|l| layers.contains(l)) {
        st.set(l, mode);
        let _ = state::save(st);
        let r = match l {
            Layer::Dns => dns(mode, &net),
            Layer::Tun => tun(mode),
            Layer::Ks | Layer::Lan => apply_firewall(st, &net),
            Layer::Id => {
                let r = id(mode, &net);
                net = net::scan(); // new MAC, maybe new address
                r
            }
        };
        if let Some(pt) = paint {
            println!("{}", pt.step(l.name(), target_word(l, mode, &net), &r));
        }
        out.push((l, r));
    }
    // Final pass: the firewall always matches the state at the end.
    let _ = apply_firewall(st, &net::scan());
    out
}

/// "DNS ✓  TUNNEL ✓  KILLSWITCH ✓  LAN ✓  IDENTITY ✗"
pub fn summary(outcomes: &[Outcome]) -> String {
    Layer::ALL
        .iter()
        .filter_map(|l| outcomes.iter().find(|(x, _)| x == l))
        .map(|(l, r)| format!("{} {}", l.name(), if r.is_ok() { "✓" } else { "✗" }))
        .collect::<Vec<_>>()
        .join("  ")
}

/// The notification for a finished change.
pub fn event_for(layers: &[Layer], mode: Mode, outcomes: &[Outcome], whole: bool, net: &Net) {
    let failed: Vec<&Outcome> = outcomes.iter().filter(|(_, r)| r.is_err()).collect();
    let place = net.ssid.clone().or(net.iface.clone()).unwrap_or_else(|| "no network".into());
    if whole {
        let ok = outcomes.len() - failed.len();
        let title = if mode == Mode::Seal {
            format!("SEALED ── {ok} of {} layers", outcomes.len())
        } else {
            format!("{} ── {place}", mode.state_word())
        };
        let mut body = summary(outcomes);
        if let Some(min) = mode.whole_timer_min() {
            body += &format!(" · seals itself in {min} min");
        }
        state::event(&title, &body, mode != Mode::Seal || !failed.is_empty());
    } else {
        let names: Vec<&str> = layers.iter().map(|l| l.name()).collect();
        state::event(&format!("{} {}", names.join(" "), mode.done_word()), &summary(outcomes), mode == Mode::Rot);
    }
    // A layer that would not seal is cut: "TUNNEL SEVERED".
    for (l, r) in failed {
        if mode == Mode::Seal {
            state::event(&format!("{} SEVERED", l.name()), r.as_ref().err().map(String::as_str).unwrap_or(""), true);
        }
    }
}
