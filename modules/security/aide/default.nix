{ config, lib, pkgs, ... }:

let
  home = "/home/m-arts";

  # Builds one AIDE config. Selection lines are regular expressions matched
  # against full paths ("!" excludes). Lines are kept un-indented on purpose.
  mkConf = name: selections: pkgs.writeText "aide-${name}.conf"
    (lib.concatStringsSep "\n" ([
      "database_in=file:/var/lib/aide/${name}.db"
      "database_out=file:/var/lib/aide/${name}.db.new"
      "database_new=file:/var/lib/aide/${name}.db.new"
      "gzip_dbout=no"
      "report_url=stdout"
      ""
      # Permissions, type, owner, group, size and SHA-256. mtime/inode are left
      # out on purpose: they change for harmless reasons and only add noise.
      "Norm = p+ftype+u+g+s+sha256"
      ""
    ] ++ selections ++ [ "" ]));

  # /boot changes on every rebuild, so it is checked BEFORE a rebuild and
  # re-baselined after it (see rebuild.sh and thorns-upgrade).
  bootConf = mkConf "boot" [
    "/boot Norm"
    "!/boot/grub/grubenv$"
  ];

  # Persistence points: places malware (or a mistake) leaves something that
  # runs at login. Accepted manually: sudo thorns-aide accept persist
  persistConf = mkConf "persist" [
    "/root/\\.ssh Norm"
    "${home}/\\.ssh Norm"
    "${home}/\\.config/autostart Norm"
    "${home}/\\.config/systemd/user Norm"
    "${home}/\\.config/environment\\.d Norm"
    "${home}/\\.config/plasma-workshop/(env|shutdown) Norm"
    "${home}/\\.(zshrc|zshenv|zprofile|bashrc|bash_profile|profile)$ Norm"
    "${home}/\\.config/fish/config\\.fish$ Norm"
  ];

  confDir = pkgs.linkFarm "thorns-aide-conf" [
    { name = "boot.conf"; path = bootConf; }
    { name = "persist.conf"; path = persistConf; }
  ];

  thorns-aide = pkgs.writeShellApplication {
    name = "thorns-aide";
    runtimeInputs = [ pkgs.python3 pkgs.aide pkgs.coreutils ];
    text = ''
      export THORNS_AIDE_CONF_DIR=${confDir}
      exec python3 ${./thorns_aide.py} "$@"
    '';
  };
in
{
  # PURPOSE: File integrity monitoring (AIDE), narrowly scoped. /nix/store is
  # immutable and verified monthly (scanning.nix), and /etc/nixos is under git,
  # so AIDE watches only what those do not cover: /boot and persistence points.
  # Detects and reports to SPINE; blocks nothing.

  environment.systemPackages = [ pkgs.aide thorns-aide ];

  # Baselines, reports and status live here (root only).
  systemd.tmpfiles.rules = [ "d /var/lib/aide 0700 root root -" ];

  systemd.services.thorns-aide-check = {
    description = "Daily AIDE file integrity check (reports to SPINE)";
    serviceConfig = {
      Type = "oneshot";
      ExecStart = "${thorns-aide}/bin/thorns-aide check all --alert";
      # Exit code 1 means "differences found", which is already reported to
      # SPINE; only real errors (exit 2) should mark the unit as failed.
      SuccessExitStatus = [ 1 ];
      Nice = 19;
      IOSchedulingClass = "idle";
    };
  };

  systemd.timers.thorns-aide-check = {
    wantedBy = [ "timers.target" ];
    timerConfig = {
      OnCalendar = "daily";
      Persistent = true;
      RandomizedDelaySec = "30min";
    };
  };
}
