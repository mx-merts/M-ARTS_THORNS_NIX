{ config, lib, pkgs, ... }:
{
  # AMAC: ClamAV gercek-zamanli koruma (sadece risk giris noktalari:
  # Downloads, Desktop, USB). Oyun/Steam klasorleri izlenmez.
  # Haftalik tarama home'u tarar (clamd uzerinden, hizli). Lynis haftalik log.
  # Aylik: Nix store butunluk dogrulamasi.

  environment.systemPackages = with pkgs; [ clamav lynis ];

  services.clamav = {
    updater.enable = true;
    updater.interval = "daily";
    daemon.enable = true;
    clamonacc.enable = true;
    daemon.settings = {
      OnAccessIncludePath = [
        "/home/m-arts/Downloads"
        "/home/m-arts/Desktop"
        "/run/media"
      ];
      OnAccessPrevention = true;
      OnAccessExcludeUname = "clamav";
      ExcludePath = [
        "^/home/m-arts/.local/share/Steam/"
        "^/home/m-arts/.var/"
        "^/home/m-arts/Games/"
      ];
    };
  };

  systemd.services.clamav-weekly-scan = {
    description = "Haftalik ClamAV home taramasi (sadece log)";
    after = [ "clamav-daemon.service" ];
    requires = [ "clamav-daemon.service" ];
    script = ''
      ${pkgs.clamav}/bin/clamdscan --config-file=/etc/clamav/clamd.conf \
        --fdpass --multiscan --infected /home/m-arts || true
    '';
    serviceConfig = {
      Type = "oneshot";
      Nice = 19;
      IOSchedulingClass = "idle";
    };
  };
  systemd.timers.clamav-weekly-scan = {
    wantedBy = [ "timers.target" ];
    timerConfig = { OnCalendar = "weekly"; Persistent = true; };
  };

  systemd.services.lynis-audit = {
    description = "Haftalik Lynis guvenlik denetimi (sadece log)";
    script = "${pkgs.lynis}/bin/lynis audit system --quiet || true";
    serviceConfig = { Type = "oneshot"; Nice = 19; };
  };
  systemd.timers.lynis-audit = {
    wantedBy = [ "timers.target" ];
    timerConfig = { OnCalendar = "weekly"; Persistent = true; };
  };

  systemd.services.nix-store-verify = {
    description = "Aylik Nix store butunluk dogrulamasi (sadece log)";
    script = "${config.nix.package}/bin/nix-store --verify --check-contents || true";
    serviceConfig = {
      Type = "oneshot";
      Nice = 19;
      IOSchedulingClass = "idle";
    };
  };
  systemd.timers.nix-store-verify = {
    wantedBy = [ "timers.target" ];
    timerConfig = { OnCalendar = "monthly"; Persistent = true; };
  };
}
