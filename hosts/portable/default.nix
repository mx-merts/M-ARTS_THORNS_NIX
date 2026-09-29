{ config, lib, pkgs, ... }:
{
  imports = [
    ./hardware-configuration.nix

    ../../modules/core/boot.nix
    ../../modules/core/nix-settings.nix
    ../../modules/core/locale.nix
    ../../modules/core/networking.nix


    ../../modules/network/dns.nix
    ../../modules/network/vpn.nix
    ../../modules/network/tuning.nix
    ../../modules/network/privacy.nix

    ../../modules/hardware/agnostic.nix
    ../../modules/hardware/audio.nix
    ../../modules/hardware/bluetooth.nix
    ../../modules/hardware/peripherals.nix

    ../../modules/security/firewall.nix
    ../../modules/security/hardening.nix
    ../../modules/security/apparmor.nix
    ../../modules/security/sandboxing.nix
    ../../modules/security/scanning.nix

    ../../modules/desktop/kde.nix
    ../../modules/desktop/sddm-theme.nix
    ../../modules/services/printing.nix

    ../../modules/programs/base.nix
    ../../modules/programs/shell.nix
    ../../modules/programs/cli-tools.nix
    ../../modules/apps/development.nix
    ../../modules/apps/gaming.nix
    ../../modules/apps/office.nix
    ../../modules/apps/design.nix
    ../../modules/apps/extras.nix
    ../../modules/apps/minecraft.nix
    ../../modules/services/flatpak.nix
    ../../modules/desktop/quickshell.nix
    ../../modules/users/m-arts.nix
  ];

  networking.hostName = "THORNS";

  system.stateVersion = "26.05";
  system.nixos.tags = [ "DESK" ];

  specialisation."PRIME".configuration = { config, lib, ... }: {
    system.nixos.tags = lib.mkForce [ "PRIME" ];
    # configurationName set edilince GRUB tarih/versiyon formatini atlar,
    # sadece bu string'i kullanir -> "NixOS - PRIME-r4" gibi temiz cikti
    boot.loader.grub.configurationName = config.system.nixos.label;
    imports = [ ../../modules/hardware/gpu/nvidia-laptop.nix ];
  };

  specialisation."NVIDIA-DESKTOP".configuration = { config, lib, ... }: {
    system.nixos.tags = lib.mkForce [ "NVIDIA-DESKTOP" ];
    boot.loader.grub.configurationName = config.system.nixos.label;
    imports = [ ../../modules/hardware/gpu/nvidia-desktop.nix ];
  };
}
