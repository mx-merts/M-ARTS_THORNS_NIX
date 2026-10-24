{ config, lib, pkgs, ... }:

let
  cfg = config.programs.spine;
  spine = pkgs.callPackage ./package.nix { };
in
{
  options.programs.spine.enable =
    lib.mkEnableOption "SPINE, a local security event hub and notifier";

  config = lib.mkIf cfg.enable {
    environment.systemPackages = [ spine ];

    # Event log and state: readable by the wheel group, writable by root only.
    systemd.tmpfiles.rules = [
      "d /var/lib/spine 0750 root wheel -"
      "f /var/lib/spine/events.jsonl 0640 root wheel -"
    ];

    # Root-side collector. Read-only observer: it never blocks or kills anything.
    systemd.services.spined = {
      description = "SPINE security event collector";
      wantedBy = [ "multi-user.target" ];
      after = [ "auditd.service" ];
      serviceConfig = {
        ExecStart = "${spine}/bin/spine daemon";
        Restart = "on-failure";
        RestartSec = 10;
        Nice = 10;
        NoNewPrivileges = true;
        ProtectSystem = "strict";
        ReadWritePaths = [ "/var/lib/spine" ];
        ProtectHome = true;
        PrivateTmp = true;
        ProtectKernelTunables = true;
        ProtectKernelModules = true;
        ProtectControlGroups = true;
        RestrictSUIDSGID = true;
        LockPersonality = true;
      };
    };

    # Session-side notifier: turns new events into desktop notifications.
    systemd.user.services.spine-notify = {
      description = "SPINE desktop notifications";
      wantedBy = [ "graphical-session.target" ];
      partOf = [ "graphical-session.target" ];
      after = [ "graphical-session.target" ];
      serviceConfig = {
        ExecStart = "${spine}/bin/spine notify";
        Restart = "on-failure";
        RestartSec = 10;
      };
    };
  };
}
