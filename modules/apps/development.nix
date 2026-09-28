# AMAC: Yazilim gelistirme araclari. VS Code Microsoft'un resmi build'i
# (Copilot + marketplace icin sart, VSCodium degil -- telemetri/marka
# temizlenmis fork farkli bir paket). nix-ld, nixpkgs'te paketlenmemis
# / uçuncu parti indirilen binary'lerin (Unity Hub kurulumlari, npm/pip'in
# cektigi on-derlenmis binary'ler, bazi oyun launcher'lari) calismasi icin.

{ config, lib, pkgs, ... }:

{
  environment.systemPackages = with pkgs; [
    vscode
    jdk25
    unityhub
    lazygit
    gh
  ];

  programs.nix-ld.enable = true;
}
