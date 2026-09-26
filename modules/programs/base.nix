{ config, lib, pkgs, ... }:
{
  # AMAÇ: Her zaman istenen temel programlar. Yeni paket eklemek için buraya yaz.
  # Kategoriye özel paketler için: development.nix, gaming.nix, media.nix (ileride)
  programs.firefox.enable = true;
  environment.systemPackages = with pkgs; [
    git
  ];
}
