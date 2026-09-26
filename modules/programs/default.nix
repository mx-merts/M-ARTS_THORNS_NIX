{ config, lib, pkgs, ... }:
{
  programs.firefox.enable = true;
  environment.systemPackages = with pkgs; [
    git
  ];
}
