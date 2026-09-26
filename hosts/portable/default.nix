{ config, lib, pkgs, ... }:
{
  imports = [
    ./hardware-configuration.nix
    ../../modules/core/boot.nix
    ../../modules/core/nix-settings.nix
    ../../modules/core/locale.nix
    ../../modules/hardware/agnostic.nix
    ../../modules/desktop/kde.nix
    ../../modules/programs/default.nix
    ../../modules/users/m-arts.nix
  ];

  networking.hostName = "nixos-portable";
  networking.networkmanager.enable = true;

  system.stateVersion = "26.05";
}
