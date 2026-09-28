# AMAC: Flathub destegi + deklaratif flatpak paketleri. Sober (Roblox Linux
# istemcisi) nixpkgs'te paket olarak yok, resmi kurulum yontemi Flathub --
# bu yuzden burada tanimli, git'e kayitli, her rebuild'de garantili kurulu.

{ config, lib, pkgs, ... }:

{
  services.flatpak = {
    enable = true;
    remotes = [
      {
        name = "flathub";
        location = "https://dl.flathub.org/repo/flathub.flatpakrepo";
      }
    ];
    packages = [
      "org.vinegarhq.Sober"
    ];
  };
}
