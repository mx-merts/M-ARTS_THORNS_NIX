//! Small helpers: time, text escaping, audit field decoding, user lookup.

use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Local time as "YYYY-MM-DD HH:MM" (uses the system time zone).
pub fn local_time(ts: u64) -> String {
    let t = ts as libc::time_t;
    // SAFETY: tm is a plain C struct we own; localtime_r only writes into it
    // and reads the valid time_t we pass by reference.
    let mut tm: libc::tm = unsafe { std::mem::zeroed() };
    let ok = unsafe { !libc::localtime_r(&t, &mut tm).is_null() };
    if !ok {
        return ts.to_string();
    }
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}",
        tm.tm_year + 1900,
        tm.tm_mon + 1,
        tm.tm_mday,
        tm.tm_hour,
        tm.tm_min
    )
}

pub fn html_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#x27;"),
            _ => out.push(c),
        }
    }
    out
}

/// Decodes an audit field: quoted text, or hex when the value had odd characters.
pub fn dec(v: Option<&str>) -> String {
    let v = match v {
        Some(v) if v != "(null)" => v,
        _ => return String::new(),
    };
    if v.len() >= 2 && v.starts_with('"') && v.ends_with('"') {
        return v[1..v.len() - 1].to_string();
    }
    if !v.is_empty() && v.len() % 2 == 0 && v.bytes().all(|b| b.is_ascii_hexdigit()) {
        let bytes: Vec<u8> = (0..v.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&v[i..i + 2], 16).unwrap_or(0))
            .collect();
        return String::from_utf8_lossy(&bytes).replace('\0', " ");
    }
    v.to_string()
}

/// Numeric uid -> user name (falls back to the number).
pub fn uname(v: Option<&str>) -> String {
    let raw = v.unwrap_or("?");
    match raw.parse::<u64>() {
        Ok(4294967295) => "unset".to_string(),
        Ok(n) => lookup_user(n).unwrap_or_else(|| raw.to_string()),
        Err(_) => raw.to_string(),
    }
}

fn lookup_user(uid: u64) -> Option<String> {
    let text = fs::read_to_string("/etc/passwd").ok()?;
    text.lines().find_map(|l| {
        let mut p = l.split(':');
        let name = p.next()?;
        let _password = p.next()?;
        let id = p.next()?.parse::<u64>().ok()?;
        (id == uid).then(|| name.to_string())
    })
}

pub fn wheel_gid() -> Option<u32> {
    let text = fs::read_to_string("/etc/group").ok()?;
    text.lines().find_map(|l| {
        let mut p = l.split(':');
        let name = p.next()?;
        let _password = p.next()?;
        let gid = p.next()?.parse::<u32>().ok()?;
        (name == "wheel").then_some(gid)
    })
}

/// "/nix/store/<hash>-linux-6.18.56/bzImage" -> "6.18.56"
pub fn kernel_version(path: &str) -> String {
    let b = path.as_bytes();
    let word = |c: u8| c.is_ascii_alphanumeric() || c == b'_';
    let mut from = 0;
    while let Some(pos) = path[from..].find("linux") {
        let mut j = from + pos + 5;
        while j < b.len() {
            if b[j] == b'-' && j + 1 < b.len() && b[j + 1].is_ascii_digit() {
                let mut k = j + 1;
                while k < b.len() && (word(b[k]) || b[k] == b'.') {
                    k += 1;
                }
                if k < b.len() && b[k] == b'/' {
                    return path[j + 1..k].to_string();
                }
            }
            if !(word(b[j]) || b[j] == b'-') {
                break;
            }
            j += 1;
        }
        from += pos + 5;
    }
    "?".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_audit_fields() {
        assert_eq!(dec(Some("\"touch\"")), "touch");
        assert_eq!(dec(Some("2F726F6F742F2E7373682F6D79206B6579")), "/root/.ssh/my key");
        assert_eq!(dec(Some("746F756368")), "touch");
        assert_eq!(dec(Some("(null)")), "");
        assert_eq!(dec(None), "");
        assert_eq!(dec(Some("abc")), "abc"); // odd length: left alone
    }

    #[test]
    fn escapes_markup() {
        assert_eq!(html_escape("a <b> & c"), "a &lt;b&gt; &amp; c");
    }

    #[test]
    fn finds_kernel_version() {
        assert_eq!(kernel_version("/nix/store/nh11-linux-6.18.56/bzImage"), "6.18.56");
        assert_eq!(kernel_version("/nix/store/x-linux-6.19.12/bzImage"), "6.19.12");
        assert_eq!(kernel_version("/nix/store/x-initrd/bzImage"), "?");
    }

    #[test]
    fn unset_auid_and_unknown_uid() {
        assert_eq!(uname(Some("4294967295")), "unset");
        assert_eq!(uname(Some("not-a-number")), "not-a-number");
        assert_eq!(uname(Some("3999999")), "3999999");
        assert_eq!(uname(Some("0")), "root");
    }

    #[test]
    fn local_time_has_expected_shape() {
        let s = local_time(1_700_000_000);
        assert_eq!(s.len(), 16, "{s}");
        assert_eq!(&s[4..5], "-");
        assert_eq!(&s[13..14], ":");
    }
}
