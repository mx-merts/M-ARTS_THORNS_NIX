{ config, lib, pkgs, ... }:
{
  # AMAC: ClamAV ve Lynis OTOMATIK calisir ama sadece PERIYODIK ARKA PLAN
  # TARAMASI olarak - gercek-zamanli degil, hicbir dosya acilisini
  # yavaslatmaz, hicbir uygulamayi etkilemez. Sonuc sadece journal'a yazilir,
  # otomatik silme/karantina YOK.
  #
  # NOT: rkhunter VE chkrootkit nixpkgs'ten kaldirildi (ikisi de upstream'de
  # terk edilmis/bakimsiz, chkrootkit NixOS'ta calismiyordu bile). Lynis tek
  # basina hem genel guvenlik denetimi hem rootkit/anomali kontrolu yapiyor
  # ve aktif gelistiriliyor.

  environment.systemPackages = with pkgs; [
    clamav
    lynis
  ];

  services.clamav = {
    updater.enable = true;
    updater.interval = "daily";
  };

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

  systemd.services.lynis-audit = {
    description = "Haftalik Lynis guvenlik denetimi (sadece log)";
    script = "${pkgs.lynis}/bin/lynis audit system --quiet || true";
    serviceConfig.Type = "oneshot";
    serviceConfig.Nice = 19;
  };
  systemd.timers.lynis-audit = {
    wantedBy = [ "timers.target" ];
    timerConfig = {
      OnCalendar = "weekly";
      Persistent = true;
    };
  };
}
