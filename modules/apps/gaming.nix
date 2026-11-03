# AMAC: Oyun altyapisi. Steam icin ozel NixOS modulu (32-bit OpenGL/audio
# otomatik ayarlanir), ProtonUp-Qt, Wine (bagimsiz kullanim), RetroArch +
# konsol-ozel emulatorler.
# Lutris ve Heroic Flatpak olarak flatpak.nix'te (ev dizinine erisimi kapali).
# Native Heroic ve Bottles KALDIRILDI: native olduklari icin calistirdiklari
# .exe'ler tum ev dizinini (Discord token, Firefox profili) gorebiliyordu.

{ config, lib, pkgs, ... }:

{
  programs.steam.enable = true;

  environment.systemPackages = with pkgs; [
    protonup-qt

    wineWow64Packages.stable
    winetricks

    retroarch
    pcsx2
    dolphin-emu
    rpcs3
  ];
}
