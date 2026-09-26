{ config, lib, pkgs, ... }:
{
  imports = [
    ./hardware-configuration.nix

    ../../modules/core/boot.nix
    ../../modules/core/nix-settings.nix
    ../../modules/core/locale.nix
    ../../modules/core/networking.nix

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
    ../../modules/services/printing.nix

    ../../modules/programs/base.nix
    ../../modules/users/m-arts.nix
  ];

  networking.hostName = "nixos-portable";

  system.stateVersion = "26.05";
  system.nixos.tags = [ "DESK" ];

  specialisation."PRIME".configuration = {
    system.nixos.tags = [ "PRIME" ];
    imports = [ ../../modules/hardware/gpu/nvidia-laptop.nix ];
  };
}
