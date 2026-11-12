//! SKIN: the network membrane of THORNS.
//!
//!   SKIN                               state of every layer
//!   SKIN <LAYER>                       one layer in detail
//!   SKIN SEAL | SEVER | ROT            the whole membrane
//!   SKIN SEAL | SEVER | ROT <LAYER..>  only those layers
//!   SKIN TRUST / SKIN DISTRUST         this Wi-Fi is home (LAN open)
//!   SKIN WATCH / SKIN NOTIFY           used by the systemd services
//!
//! Layers: DNS, TUNNEL (TUN), KILLSWITCH (KS), LAN, IDENTITY (ID).
//! Names are not case sensitive.

mod firewall;
mod layers;
mod model;
mod net;
mod notify;
mod paint;
mod report;
mod state;
mod sys;
mod watch;

use model::{Layer, Mode};
use paint::{Paint, Role};
use std::env;
use std::io::{self, Write};

fn refused(pt: &Paint, text: &str) -> String {
    format!("   {}  {}", pt.c(Role::Bad, "✖"), pt.c(Role::Bad, text))
}

fn confirm_rot(pt: &Paint) -> bool {
    println!("   {}", pt.c(Role::Bad, "☠  ROT DROPS EVERY PROTECTION"));
    println!("   {}", pt.c(Role::Dim, "DNS plain · no tunnel · no kill switch · LAN open · real MAC"));
    println!("   {}", pt.c(Role::Dim, "Back: sudo SKIN SEAL, or wait 30:00. Incoming stays closed."));
    print!("   {} ", pt.c(Role::Warn, "Type ROT to go on:"));
    let _ = io::stdout().flush();
    let mut line = String::new();
    let _ = io::stdin().read_line(&mut line);
    line.trim().eq_ignore_ascii_case("ROT")
}

fn act(pt: &Paint, mode: Mode, asked: Vec<Layer>) -> i32 {
    // Naming all five layers is the same as naming none.
    let whole = asked.is_empty() || asked.len() == Layer::ALL.len();
    let targets: Vec<Layer> = if whole { Layer::ALL.to_vec() } else { asked };
    let names: Vec<&str> = targets.iter().map(|l| l.short()).collect();
    if !sys::is_root() {
        let cmd = if whole { mode.name().to_string() } else { format!("{} {}", mode.name(), names.join(" ")) };
        println!("{}   {}", refused(pt, "PERMISSION DENIED"), pt.c(Role::Dim, &format!("── run: sudo SKIN {cmd}")));
        return 1;
    }
    if whole && mode == Mode::Rot && !confirm_rot(pt) {
        println!("   {}  {}   {}", pt.c(Role::Good, "●"), pt.c(Role::Good, "CANCELLED"), pt.c(Role::Dim, "nothing was changed"));
        return 1;
    }

    let mut st = state::load();
    // How long the membrane was open (for "was open for 2:13").
    let opened_at = Layer::ALL
        .iter()
        .filter(|l| targets.contains(l) && st.mode(**l) != Mode::Seal)
        .map(|l| st.since[l.index()])
        .filter(|t| *t > 0)
        .min();
    if whole {
        st.timer = mode.whole_timer_min().map(|m| (mode, sys::now() + m * 60));
    }

    let netnow = net::scan();
    let head = if whole { String::new() } else { format!(" {}", pt.c(Role::Strong, &names.join(" "))) };
    println!(" {}{}   {}", pt.c(Paint::mode_role(mode), &format!("{} {}", mode.symbol(), mode.name())), head, netnow.label());

    let outcomes = layers::transition(&targets, mode, &mut st, Some(pt));
    if st.whole() == Some(Mode::Seal) {
        st.timer = None; // nothing left to seal
    }
    let _ = state::save(&st);
    let net_after = net::scan();
    let failed: Vec<&layers::Outcome> = outcomes.iter().filter(|(_, r)| r.is_err()).collect();

    // Verdict.
    let open_for = opened_at.map(|t| format!("was open for {}", sys::duration(sys::now().saturating_sub(t)))).unwrap_or_default();
    if let Some((l, r)) = failed.first().filter(|_| mode == Mode::Seal) {
        let hint = match l {
            Layer::Tun => "without WARP: sudo SKIN SEVER TUN KS",
            Layer::Dns => "journalctl -u dnscrypt-proxy",
            _ => "SKIN for details",
        };
        let ok = outcomes.len() - failed.len();
        println!(
            "   {}   {}",
            pt.c(Role::Bad, &format!("✖  {} SEVERED", l.name())),
            pt.c(Role::Dim, &format!("{ok} of {} sealed · {} · {hint}", outcomes.len(), r.as_ref().err().unwrap()))
        );
    } else {
        match (st.whole(), whole) {
            (Some(Mode::Seal), _) => println!("{}", format!("   {}   {}", pt.c(Role::Good, "●  SEALED"), pt.c(Role::Dim, &open_for)).trim_end()),
            (Some(Mode::Sever), true) => {
                println!("   {}   open {} and log in,", pt.c(Role::Warn, "✂  SEVERED"), pt.link("http://neverssl.com"));
                println!("               then:  {}", pt.c(Role::Strong, "sudo SKIN SEAL"));
                println!("               {}", pt.c(Role::Dim, "seals itself in 10:00 if you forget"));
            }
            (Some(Mode::Rot), true) => println!("   {}   {}", pt.c(Role::Bad, "☠  ROTTEN"), pt.c(Role::Dim, "seals itself in 30:00")),
            (Some(m), false) => println!("   {}", pt.c(Paint::mode_role(m), &format!("{}  {}", m.symbol(), m.state_word()))),
            (None, _) => println!(
                "   {}   {}",
                pt.c(Role::Custom, "◆  CUSTOM"),
                pt.c(Role::Dim, &format!("{} {} · other layers unchanged · stays until you change it", names.join(" "), mode.done_word().to_lowercase()))
            ),
        }
    }
    layers::event_for(&targets, mode, &outcomes, whole, &net_after);
    if failed.is_empty() { 0 } else { 1 }
}

/// SKIN TRUST / DISTRUST: the current Wi-Fi becomes (or stops being) a known
/// network. Stored in /var/lib/skin/known, never in git.
fn trust(pt: &Paint, on: bool) -> i32 {
    let word = if on { "TRUST" } else { "DISTRUST" };
    if !sys::is_root() {
        println!("{}   {}", refused(pt, "PERMISSION DENIED"), pt.c(Role::Dim, &format!("── run: sudo SKIN {word}")));
        return 1;
    }
    let n = net::scan();
    let Some(ssid) = n.ssid.clone() else {
        println!("{}   {}", refused(pt, "NO WI-FI NETWORK"), pt.c(Role::Dim, "connect to the network first"));
        return 1;
    };
    if !on && state::known_nix().contains(&ssid) {
        println!("{}   {}", refused(pt, "SET IN NIX"), pt.c(Role::Dim, "remove it from programs.skin.knownNetworks"));
        return 1;
    }
    match state::set_trusted(&ssid, on) {
        Err(e) => {
            println!("{}   {}", refused(pt, "NOT SAVED"), pt.c(Role::Dim, &e));
            return 1;
        }
        Ok(false) => {
            let was = if on { "already trusted" } else { "was not trusted" };
            println!("   {}  {}   {}", pt.c(Role::Good, "●"), pt.c(Role::Strong, &ssid), pt.c(Role::Dim, was));
            return 0;
        }
        Ok(true) => {}
    }
    let fw = layers::apply_firewall(&state::load(), &net::scan());
    if on {
        println!("   {}   {}", pt.c(Role::Good, &format!("●  TRUSTED ── {ssid}")), pt.c(Role::Dim, "LAN open here while sealed"));
    } else {
        println!("   {}   {}", pt.c(Role::Warn, &format!("✂  DISTRUSTED ── {ssid}")), pt.c(Role::Dim, "LAN isolated here"));
    }
    println!("   {}", pt.c(Role::Dim, &format!("kept in {} (this machine only, not in git)", state::TRUSTED)));
    if let Err(e) = fw {
        println!("{}   {}", refused(pt, "FIREWALL NOT UPDATED"), pt.c(Role::Dim, &e));
        return 1;
    }
    0
}

fn bad(pt: &Paint, text: &str) -> i32 {
    println!("{}   {}", refused(pt, text), pt.c(Role::Dim, "SKIN HELP"));
    2
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let pt = Paint::detect();
    let up: Vec<String> = args.iter().map(|a| a.to_ascii_uppercase()).collect();
    let w: Vec<&str> = up.iter().map(String::as_str).collect();
    let code = match w.as_slice() {
        [] => {
            println!("{}", report::status(&pt));
            0
        }
        ["HELP"] | ["--HELP"] | ["-H"] => {
            println!("{}", report::help(&pt));
            0
        }
        ["--VERSION"] | ["-V"] | ["VERSION"] => {
            println!("SKIN-{}", paint::VERSION);
            0
        }
        ["TRUST"] => trust(&pt, true),
        ["DISTRUST"] => trust(&pt, false),
        ["WATCH"] => watch::run(),
        ["NOTIFY"] => notify::run(false),
        ["NOTIFY", "--DRY"] => notify::run(true),
        [one] if Layer::parse(one).is_some() => {
            println!("{}", report::layer(&pt, Layer::parse(one).unwrap()));
            0
        }
        [mode, rest @ ..] if Mode::parse(mode).is_some() => {
            let mut layers = Vec::new();
            for r in rest {
                match Layer::parse(r) {
                    Some(l) if !layers.contains(&l) => layers.push(l),
                    Some(_) => {}
                    None => {
                        std::process::exit(bad(&pt, &format!("UNKNOWN LAYER: {r} (DNS, TUN, KS, LAN, ID)")));
                    }
                }
            }
            act(&pt, Mode::parse(mode).unwrap(), layers)
        }
        _ => bad(&pt, &format!("UNKNOWN COMMAND: {}", args.join(" "))),
    };
    std::process::exit(code);
}
