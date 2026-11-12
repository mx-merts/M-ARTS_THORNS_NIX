{ config, lib, pkgs, ... }:
{
  users.users."m-arts" = {
    isNormalUser = true;
    description = "M-ARTS";
    # scanner + lp: tarayici erisimi (hardware/peripherals.nix)
    extraGroups = [ "networkmanager" "wheel" "scanner" "lp" ];
  };
}
