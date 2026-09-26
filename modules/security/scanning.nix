{ config, lib, pkgs, ... }:
{
  # AMAC: ClamAV ve rkhunter OTOMATIK calisir ama sadece PERIYODIK ARKA PLAN
  # TARAMASI olarak - gercek-zamanli (on-access) degil, hicbir dosya acilisini
  # yavaslatmaz, hicbir uygulamayi etkilemez. Sonuc sadece journal'a yazilir,
  # otomatik silme/karantina YOK.

  environment.systemPackages = with pkgs; [
    clamav
    rkhunter
    chkrootkit
  ];

  # ClamAV imza veritabani gunluk otomatik guncellenir
  services.clamav = {
    updater.enable = true;
    updater.interval = "daily";
  };

  # ClamAV tam tarama - haftalik, arka planda, sadece log
  systemd.services.clamav-weekly-scan = {
    description = "Haftalik ClamAV taramasi (sadece log, otomatik silme yok)";
    script = ''
      ${pkgs.clamav}/bin/clamscan -r --infected --no-summary \
        --exclude-dir="^/proc|^/sys|^/nix/store" / \
        || true
    '';
    serviceConfig.Type = "oneshot";
    serviceConfig.Nice = 19;
    serviceConfig.IOSchedulingClass = "idle";
  };
  systemd.timers.clamav-weekly-scan = {
    wantedBy = [ "timers.target" ];
    timerConfig = {
      OnCalendar = "weekly";
      Persistent = true;
    };
  };

  # rkhunter - haftalik, sadece log
  systemd.services.rkhunter-scan = {
    description = "Haftalik rkhunter rootkit taramasi (sadece log)";
    script = "${pkgs.rkhunter}/bin/rkhunter --check --skip-keypress --report-warnings-only";
    serviceConfig.Type = "oneshot";
    serviceConfig.Nice = 19;
  };
  systemd.timers.rkhunter-scan = {
    wantedBy = [ "timers.target" ];
    timerConfig = {
      OnCalendar = "weekly";
      Persistent = true;
    };
  };
}
