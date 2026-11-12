# NERV: THORNS'un donanim sinir sistemi.
#   nerv-boot     acilista bir kez (root): surucusu eksik parcalari baglar,
#                 sonucu /run/nerv/boot'a yazar
#   nerv-notify   giriste bir kez (kullanici): oturum onerisi, baglanamayan
#                 parcalar, parmak izi okuyucu; sorun yoksa sessiz
#   NERV          rapor ve yonetim komutu (bkz. README.md)
{ config, lib, pkgs, ... }:

let
  cfg = config.programs.nerv;
  nerv = pkgs.callPackage ./package.nix { };

  # PRIME girdisinin PCI adresleri (prime.nix ile ayni dosya). NERV bunlari
  # makinedeki kartlarla karsilastirip PRIME'i sadece bu laptopta onerir.
  primeBus = import ../gpu/prime-bus.nix;

  # Bu yapilandirma hangi GRUB girdisi? Her specialisation kendi degerini
  # alir, cunku bu modul her birinde yeniden degerlendirilir.
  nv = config.hardware.nvidia;
  session =
    if nv.prime.offload.enable then "PRIME"
    else if lib.elem "nvidia" config.services.xserver.videoDrivers then
      (if nv.open then "NVIDIA" else "NVIDIA-L")
    else "DESK";
in
{
  options.programs.nerv.enable =
    lib.mkEnableOption "NERV, the THORNS hardware nervous system";

  config = lib.mkIf cfg.enable {
    environment.systemPackages = [ nerv ];

    environment.etc."nerv/nerv.conf".text = ''
      session=${session}
      prime_igpu=${primeBus.intel}
      prime_dgpu=${primeBus.nvidia}
    '';

    # Acilis taramasi. Sadece EKLER: surucu yukler ve baglar, hicbir seyi
    # ayirmaz; hardening.nix kara listesindeki suruculeri yuklemez (modprobe -b).
    # Girisi geciktirmez (multi-user.target onu beklemez).
    systemd.services.nerv-boot = {
      description = "NERV boot hardware scan";
      wantedBy = [ "multi-user.target" ];
      after = [ "systemd-udev-trigger.service" "systemd-modules-load.service" ];
      # rebuild sirasinda yeniden calismasin: rapor "acilis" taramasini gostersin
      restartIfChanged = false;
      serviceConfig = {
        Type = "oneshot";
        RemainAfterExit = true;
        # udev'in ilk turu bitsin (en fazla 30 sn), sonra eksikler aransin
        ExecStartPre = "-${pkgs.systemd}/bin/udevadm settle --timeout=30";
        ExecStart = "${nerv}/bin/NERV boot";
        # /run/nerv: tmpfs, yeniden baslatinca silinir (makine hafizasi yok)
        RuntimeDirectory = "nerv";
        RuntimeDirectoryMode = "0755";
        RuntimeDirectoryPreserve = "yes";
      };
    };

    # Giris bildirimleri: SPINE'dan bagimsiz, kendi kanali.
    systemd.user.services.nerv-notify = {
      description = "NERV login notifications";
      wantedBy = [ "graphical-session.target" ];
      partOf = [ "graphical-session.target" ];
      after = [ "graphical-session.target" ];
      serviceConfig = {
        Type = "oneshot";
        ExecStart = "${nerv}/bin/NERV notify";
      };
    };
  };
}
