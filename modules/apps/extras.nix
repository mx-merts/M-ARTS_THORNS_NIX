# AMAC: Gunluk kullanim uygulamalari. discord, spotify, obs-studio, obsidian,
# kitty (terminal emulator, alternatif), qbittorrent, yacreader.

{ config, lib, pkgs, ... }:

{
  environment.systemPackages = with pkgs; [
    discord
    spotify
    obs-studio
    obsidian
    kitty
    qbittorrent
    yacreader
  ];
}
