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

    # nixpkgs'teki "retroarch" cekirdeksiz gelir ve icindeki cekirdek
    # indirici derlemede kapatilmistir; cekirdekler burada secilir.
    # PS2 / GameCube-Wii / PS3 asagidaki ayri emulatorlerde.
    (retroarch.withCores (cores: with cores; [
      mesen            # NES / Famicom
      snes9x           # SNES
      mgba             # Game Boy / Color / Advance
      genesis-plus-gx  # Sega Master System / Mega Drive / Game Gear / Sega CD
      mupen64plus      # Nintendo 64
      swanstation      # PlayStation 1
      ppsspp           # PSP
      melonds          # Nintendo DS
    ]))
    pcsx2
    dolphin-emu
    rpcs3
  ];
}
