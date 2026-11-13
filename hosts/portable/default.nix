{ config, lib, pkgs, ... }:
{
  imports = [
    ./hardware-configuration.nix

    ../../modules/core/boot.nix
    ../../modules/core/boot-menu.nix
    ../../modules/core/nix-settings.nix
    ../../modules/core/locale.nix
    ../../modules/core/networking.nix
    ../../modules/core/auto-upgrade.nix

    ../../modules/network/dns.nix
    ../../modules/network/vpn.nix
    ../../modules/network/warp.nix
    ../../modules/network/tuning.nix
    ../../modules/network/privacy.nix
    ../../modules/network/skin

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
    ../../modules/desktop/grub-theme.nix
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

  # Surum etiketi: modules/core/boot-menu.nix (GRUB'daki adlar da orada)
  system.nixos.label = "NIXOS-DESK-" + config.thorns.bootMenu.label;

  environment.sessionVariables.NIXOS_OZONE_WL = "1";

  programs.nocer.enable = true;
  programs.spine.enable = true;
  programs.nerv.enable = true;
  # Ev agi git'e yazilmaz: evde bir kez "sudo SKIN TRUST" (bkz. SKIN README)
  programs.skin.enable = true;

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
  # GRUB'da "<ad> · T-[113]" olarak gorunur (modules/core/boot-menu.nix).
  specialisation."NVIDIA".configuration = {
    thorns.bootMenu.name = "NVIDIA";
    imports = [ ../../modules/hardware/gpu/nvidia.nix ];
  };

  specialisation."NVIDIA-L".configuration = {
    thorns.bootMenu.name = "NVIDIA-L";
    imports = [ ../../modules/hardware/gpu/nvidia-legacy.nix ];
  };

  specialisation."PRIME".configuration = {
    thorns.bootMenu.name = "PRIME";
    imports = [ ../../modules/hardware/gpu/prime.nix ];
  };
}
