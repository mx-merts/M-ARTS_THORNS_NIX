{ config, lib, pkgs, ... }:
{
  # AMAC: ClamAV, chkrootkit, Lynis OTOMATIK calisir ama sadece PERIYODIK
  # ARKA PLAN TARAMASI olarak - gercek-zamanli degil, hicbir dosya acilisini
  # yavaslatmaz, hicbir uygulamayi etkilemez. Sonuc sadece journal'a yazilir,
  # otomatik silme/karantina YOK.
  #
  # NOT: rkhunter nixpkgs 26.05'te kaldirildi (upstream'de 2018'den beri
  # guncellenmiyordu). Yerine chkrootkit + Lynis kullaniyoruz, ikisi de
  # aktif gelistiriliyor.

  environment.systemPackages = with pkgs; [
    clamav
    chkrootkit
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

  systemd.services.chkrootkit-scan = {
    description = "Haftalik chkrootkit taramasi (sadece log)";
    script = "${pkgs.chkrootkit}/bin/chkrootkit || true";
    serviceConfig.Type = "oneshot";
    serviceConfig.Nice = 19;
  };
  systemd.timers.chkrootkit-scan = {
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
