//! Every filesystem location SPINE touches, in one place.

use std::path::PathBuf;

#[derive(Clone)]
pub struct Paths {
    pub events: PathBuf,
    pub state: PathBuf,
    pub audit_log: PathBuf,
    pub booted_kernel: PathBuf,
    pub latest_kernel: PathBuf,
}

impl Paths {
    /// The real locations on a NixOS system.
    pub fn system() -> Self {
        let dir = PathBuf::from("/var/lib/spine");
        Paths {
            events: dir.join("events.jsonl"),
            state: dir.join("state.json"),
            audit_log: PathBuf::from("/var/log/audit/audit.log"),
            booted_kernel: PathBuf::from("/run/booted-system/kernel"),
            // Newest generation: what the next boot will run. Comparing with
            // /run/current-system would miss `nixos-rebuild boot` upgrades.
            latest_kernel: PathBuf::from("/nix/var/nix/profiles/system/kernel"),
        }
    }

    /// Isolated locations under a fresh temp directory (tests only).
    #[cfg(test)]
    pub fn temp(tag: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("spine-test-{}-{}", std::process::id(), tag));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        Paths {
            events: dir.join("events.jsonl"),
            state: dir.join("state.json"),
            audit_log: dir.join("audit.log"),
            booted_kernel: dir.join("booted"),
            latest_kernel: dir.join("latest"),
        }
    }
}
