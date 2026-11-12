//! The five layers of the membrane and the three states each can be in.

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Layer {
    Dns,
    Tun,
    Ks,
    Lan,
    Id,
}

impl Layer {
    pub const ALL: [Layer; 5] = [Layer::Dns, Layer::Tun, Layer::Ks, Layer::Lan, Layer::Id];

    pub fn name(self) -> &'static str {
        match self {
            Layer::Dns => "DNS",
            Layer::Tun => "TUNNEL",
            Layer::Ks => "KILLSWITCH",
            Layer::Lan => "LAN",
            Layer::Id => "IDENTITY",
        }
    }

    pub fn short(self) -> &'static str {
        match self {
            Layer::Dns => "DNS",
            Layer::Tun => "TUN",
            Layer::Ks => "KS",
            Layer::Lan => "LAN",
            Layer::Id => "ID",
        }
    }

    pub fn parse(s: &str) -> Option<Layer> {
        let s = s.to_ascii_uppercase();
        Layer::ALL.into_iter().find(|l| l.name() == s || l.short() == s)
    }

    pub fn index(self) -> usize {
        Layer::ALL.iter().position(|l| *l == self).unwrap_or(0)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Mode {
    /// Full strength (the default at every boot).
    Seal,
    /// Opened just enough to get through a login page.
    Sever,
    /// Every protection dropped (last resort).
    Rot,
}

impl Mode {
    pub const ALL: [Mode; 3] = [Mode::Seal, Mode::Sever, Mode::Rot];

    pub fn name(self) -> &'static str {
        match self {
            Mode::Seal => "SEAL",
            Mode::Sever => "SEVER",
            Mode::Rot => "ROT",
        }
    }

    /// "SEALED", "SEVERED", "ROTTEN" (whole membrane).
    pub fn state_word(self) -> &'static str {
        match self {
            Mode::Seal => "SEALED",
            Mode::Sever => "SEVERED",
            Mode::Rot => "ROTTEN",
        }
    }

    /// "KILLSWITCH ROTTED", "DNS SEALED" (one layer).
    pub fn done_word(self) -> &'static str {
        match self {
            Mode::Seal => "SEALED",
            Mode::Sever => "SEVERED",
            Mode::Rot => "ROTTED",
        }
    }

    pub fn symbol(self) -> &'static str {
        match self {
            Mode::Seal => "●",
            Mode::Sever => "✂",
            Mode::Rot => "☠",
        }
    }

    pub fn parse(s: &str) -> Option<Mode> {
        let s = s.to_ascii_uppercase();
        Mode::ALL.into_iter().find(|m| m.name() == s)
    }

    /// Minutes until the whole membrane seals itself again.
    pub fn whole_timer_min(self) -> Option<u64> {
        match self {
            Mode::Seal => None,
            Mode::Sever => Some(10),
            Mode::Rot => Some(30),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_names() {
        assert_eq!(Layer::parse("ks"), Some(Layer::Ks));
        assert_eq!(Layer::parse("killswitch"), Some(Layer::Ks));
        assert_eq!(Layer::parse("TUN"), Some(Layer::Tun));
        assert_eq!(Layer::parse("tunnel"), Some(Layer::Tun));
        assert_eq!(Layer::parse("id"), Some(Layer::Id));
        assert_eq!(Layer::parse("x"), None);
        assert_eq!(Mode::parse("sever"), Some(Mode::Sever));
        assert_eq!(Mode::parse("ROT"), Some(Mode::Rot));
        assert_eq!(Mode::parse("snap"), None);
    }
}
