# SKIN: THORNS'un ag zari (deri). Bes katman, uc durum:
#   SEAL   en guclu hali (her acilista bu)
#   SEVER  giris sayfasi (kafe, otel) icin yeterince acik; 10 dk sonra SEAL
#   ROT    son care, her koruma kapali; 30 dk sonra SEAL
# Katmanlar: DNS, TUNNEL (TUN), KILLSWITCH (KS), LAN, IDENTITY (ID).
#
#   skin-watch    (root) KS + LAN kurallarini durumla esit tutar, zamanlayiciyi
#                 isletir, tuneli / DNS'i izler, olaylari /run/skin/events'e yazar
#   skin-notify   (kullanici) olaylari bildirime cevirir
#   SKIN          durum ve komut (bkz. README.md)
{ config, lib, pkgs, ... }:

let
  cfg = config.programs.skin;
  skin = pkgs.callPackage ./package.nix {
    warp = config.services.cloudflare-warp.package;
  };
in
{
  options.programs.skin = {
    enable = lib.mkEnableOption "SKIN, the THORNS network membrane";

    knownNetworks = lib.mkOption {
      type = lib.types.listOf lib.types.str;
      default = [ ];
      example = [ "Ev-Modem-5G" ];
      description = ''
        SSID'ler: bu aglarda SEAL iken LAN acik kalir (yazici, telefon).
        DIKKAT: buraya yazilan ev SSID'i git'e gider ve Wi-Fi haritalarinda
        (WiGLE) evin yerini bulmaya yeter. Tercih edilen yol `sudo SKIN TRUST`:
        liste sadece bu makinede (/var/lib/skin/known) tutulur.
      '';
    };
  };

  config = lib.mkIf cfg.enable {
    environment.systemPackages = [ skin ];

    environment.etc."skin/skin.conf".text =
      lib.concatMapStrings (ssid: "known=${ssid}\n") cfg.knownNetworks;

    # Root bekcisi. Ag acilmadan ONCE kurallari koyar (Before=network-pre:
    # NetworkManager onu bekler, Type=notify ile "hazirim" deyince baslar),
    # yani ilk paketten itibaren KILLSWITCH devrede. Kurallar servis durunca
    # SILINMEZ: bekci cokse de kapali kalir (acil cikis: README).
    systemd.services.skin-watch = {
      description = "SKIN network membrane watcher";
      wantedBy = [ "multi-user.target" ];
      before = [ "network-pre.target" ];
      wants = [ "network-pre.target" ];
      after = [ "systemd-udev-trigger.service" "systemd-modules-load.service" ];
      serviceConfig = {
        Type = "notify";
        NotifyAccess = "main";
        # Wi-Fi / Ethernet kartlari sysfs'te gorunsun (en fazla 10 sn);
        # gec gelen kart da 3 sn icinde kurallara eklenir.
        ExecStartPre = "-${pkgs.systemd}/bin/udevadm settle --timeout=10";
        ExecStart = "${skin}/bin/SKIN WATCH";
        Restart = "always";
        RestartSec = 2;
        # /run/skin: tmpfs, her acilis SEAL baslar (makine hafizasi yok).
        # rebuild'de bekci yeniden baslar ama durum korunur (Preserve).
        RuntimeDirectory = "skin";
        RuntimeDirectoryMode = "0755";
        RuntimeDirectoryPreserve = "yes";
        # /var/lib/skin: SKIN TRUST listesi (kalici, git'te degil)
        StateDirectory = "skin";
        StateDirectoryMode = "0755";
      };
    };

    # Kullanici bildirimleri: NERV / SPINE'dan bagimsiz, kendi kanali.
    systemd.user.services.skin-notify = {
      description = "SKIN notifications";
      wantedBy = [ "graphical-session.target" ];
      partOf = [ "graphical-session.target" ];
      after = [ "graphical-session.target" ];
      serviceConfig = {
        ExecStart = "${skin}/bin/SKIN NOTIFY";
        Restart = "on-failure";
        RestartSec = 5;
      };
    };
  };
}
