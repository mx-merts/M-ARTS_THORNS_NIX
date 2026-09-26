{ config, lib, pkgs, ... }:
{
  users.users."m-arts" = {
    isNormalUser = true;
    description = "M-ARTS";
    extraGroups = [ "networkmanager" "wheel" ];
    packages = with pkgs; [
      kdePackages.kate
    ];
  };
}
