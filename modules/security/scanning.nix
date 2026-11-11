{ config, lib, pkgs, ... }:
{
  # AMAC: ClamAV gercek-zamanli koruma (sadece risk giris noktalari:
  # Downloads, Desktop, USB). Oyun/Steam klasorleri izlenmez.
  # Haftalik tarama home'u tarar (clamd uzerinden, hizli). Lynis haftalik log.
  # Aylik: Nix store butunluk dogrulamasi.
  # fangfrisch: resmi ClamAV imzalarina ek, topluluk imza kaynaklari.

  environment.systemPackages = with pkgs; [ clamav lynis ];

  services.clamav = {
    updater.enable = true;
    updater.interval = "daily";
    daemon.enable = true;
    clamonacc.enable = true;
    daemon.settings = {
      # 16 thread'in yarisi: tarama sirasinda sistem akici kalir
      MaxThreads = 8;
      MaxQueue = 200;

      MaxFileSize = "4000M";
      MaxScanSize = "4000M";
      MaxScanTime = 0;
      MaxFiles = 1000000;

      OnAccessIncludePath = [
        "/home/m-arts/Downloads"
        "/home/m-arts/Desktop"
        "/run/media"
      ];
      OnAccessPrevention = true;
      OnAccessExcludeUname = "clamav";
      OnAccessExcludeRootUID = true;
      OnAccessMaxFileSize = "100M";  # anlik tarama siniri (ClamAV varsayilani 5M)

    };

    # fangfrisch: resmi imzalara EK imza kaynaklari indirir (saatlik).
    # nixpkgs'in kendi ClamAV modulunun parcasi; root yetkisi gerektirmez,
    # clamav kullanicisi olarak calisir. clamd yeni imzalari SelfCheck ile
    # (varsayilan 10 dk) kendiliginden yukler, yeniden baslatma gerekmez.
    # Sanesecurity: varsayilan olarak acik (zararli/phishing/dolandiricilik).
    # URLhaus (abuse.ch, aktif dagitilan zararlilar): ucretsiz bir Auth-Key
    # gerektirir; anahtar alininca ayri bir adimda acilacak.
    fangfrisch.enable = true;
  };

  
    # Yanlis alarm listesi (.ign2): buradaki imza ADLARI tum taramalarda yok sayilir.
    # Sadece dogrulanmis yanlis alarmlar eklenir; her satira neden + tarih yazilir.
    #   Porcupine.Malware.58887 (Sanesecurity, UNOFFICIAL): Valve'in Proton
    #     Experimental'indaki wine-mono .NET araclarini (sgen.exe, wsdl.exe) ve
    #     GOG'un Quake II Enhanced exe'sini isaretliyor. 2026-11-09.
    # clamd yeni listeyi SelfCheck ile (10 dk) kendiliginden yukler.
    systemd.tmpfiles.settings."10-clamav-ignore"."/var/lib/clamav/thorns.ign2"."L+".argument =
      toString (pkgs.writeText "thorns.ign2" ''
        Porcupine.Malware.58887
      '');

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
