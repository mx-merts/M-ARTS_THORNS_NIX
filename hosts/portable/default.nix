{ config, lib, pkgs, ... }:
let
  # GRUB etiketi: rebuild.sh "T-<sayi>", thorns-upgrade "AUTO-<tarih>" verir
  # (NIXOS_LABEL_VERSION + --impure). Elle rebuild edilirse "dev" gorunur.
  labelVersion =
    let v = builtins.getEnv "NIXOS_LABEL_VERSION";
    in if v == "" then ":-:-:dev:-:-:" else v;
in
{
  imports = [
    ./hardware-configuration.nix

    ../../modules/core/boot.nix
    ../../modules/core/nix-settings.nix
    ../../modules/core/locale.nix
    ../../modules/core/networking.nix
    ../../modules/core/auto-upgrade.nix

    ../../modules/network/dns.nix
    ../../modules/network/vpn.nix
    ../../modules/network/warp.nix
    ../../modules/network/tuning.nix
    ../../modules/network/privacy.nix

    ../../modules/hardware/agnostic.nix
    ../../modules/hardware/audio.nix
    ../../modules/hardware/bluetooth.nix
    ../../modules/hardware/peripherals.nix
    ../../modules/hardware/sensors.nix
    ../../modules/hardware/excalibur-wmi.nix
    ../../modules/hardware/excalibur-rgb.nix
    ../../modules/hardware/nerv

    ../../modules/security/firewall.nix
    ../../modules/security/hardening.nix
    ../../modules/security/sandboxing.nix
    ../../modules/security/scanning.nix
    ../../modules/security/auditing.nix
    ../../modules/security/nocer
    ../../modules/security/spine
    ../../modules/security/aide
    ../../modules/security/lkrg.nix
    ../../modules/security/firefox.nix

    ../../modules/desktop/kde.nix
    ../../modules/desktop/hyprland.nix
    ../../modules/desktop/serpantinum.nix
    ../../modules/desktop/sddm-theme.nix
    ../../modules/desktop/terminal.nix

    ../../modules/services/printing.nix
    ../../modules/services/flatpak.nix

    ../../modules/programs/base.nix
    ../../modules/programs/shell.nix
    ../../modules/programs/cli-tools.nix
    ../../modules/programs/neovim.nix
    ../../modules/programs/jrun.nix
    ../../modules/programs/lrun.nix
    ../../modules/programs/cscan.nix

    ../../modules/apps/development.nix
    ../../modules/apps/gaming.nix
    ../../modules/apps/office.nix
    ../../modules/apps/design.nix
    ../../modules/apps/extras.nix

    ../../modules/users/m-arts.nix
  ];

  networking.hostName = "THORNS";

  system.stateVersion = "26.05";

  system.nixos.label = "NIXOS-DESK-" + labelVersion;

  environment.sessionVariables.NIXOS_OZONE_WL = "1";

  programs.nocer.enable = true;
  programs.spine.enable = true;
  programs.nerv.enable = true;

  hardware.excaliburRgb = {
    enable = true;
    mode = "rainbow";
    color = "FFFFFF";
    brightness = 2;
  };

  # sudo-rs: Rust ile yazilmis, bellek-guvenli sudo (Ubuntu 26.04 LTS varsayilani)
  security.sudo.enable = false;
  security.sudo-rs.enable = true;
  security.sudo-rs.wheelNeedsPassword = true;

  # GRUB girdileri (DESK = ana yapilandirma, specialisation degil):
  #   DESK      Intel / AMD / NVIDIA'siz her makine (NVIDIA varsa nouveau ile acilir)
  #   NVIDIA    RTX 20 / GTX 16 ve sonrasi (RTX 50 dahil), masaustu + laptop
  #   NVIDIA-L  GTX 900 / 1000 (2014-2017), masaustu + laptop
  #   PRIME     sadece bu laptop (busId sabit)
  specialisation."NVIDIA".configuration = { config, lib, ... }: {
    boot.loader.grub.configurationName = lib.mkForce ("NIXOS-NVIDIA-" + labelVersion);
    imports = [ ../../modules/hardware/gpu/nvidia.nix ];
  };

  specialisation."NVIDIA-L".configuration = { config, lib, ... }: {
    boot.loader.grub.configurationName = lib.mkForce ("NIXOS-NVIDIA-L-" + labelVersion);
    imports = [ ../../modules/hardware/gpu/nvidia-legacy.nix ];
  };

  specialisation."PRIME".configuration = { config, lib, ... }: {
    boot.loader.grub.configurationName = lib.mkForce ("NIXOS-PRIME-" + labelVersion);
    imports = [ ../../modules/hardware/gpu/prime.nix ];
  };
}
