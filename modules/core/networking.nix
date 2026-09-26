{ config, lib, pkgs, ... }:
{
  # AMAÇ: Ağ temel ayarları. Hostname host dosyasında (hosts/*/default.nix) tanımlanır.
  networking.networkmanager.enable = true;
}
