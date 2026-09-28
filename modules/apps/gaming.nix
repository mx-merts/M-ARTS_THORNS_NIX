# AMAC: Oyun altyapisi. Steam icin ozel NixOS modulu (32-bit OpenGL/audio
# otomatik ayarlanir), Heroic (Epic/GOG/Amazon, FHS'li surum daha az sorun
# cikarir), ProtonUp-Qt (Lutris Flatpak olarak flatpak.nix'te), Wine/Bottles
# (Steam/Lutris disi bagimsiz kullanim), RetroArch + konsol-ozel emulatorler.

{ config, lib, pkgs, ... }:

{
  programs.steam.enable = true;

  environment.systemPackages = with pkgs; [
    heroic
    protonup-qt

    wineWow64Packages.stable
    winetricks
    bottles

    retroarch
    pcsx2
    dolphin-emu
    rpcs3
  ];
}
