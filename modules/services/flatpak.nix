# AMAC: Flathub destegi (DaVinci Resolve gibi NixOS'ta kirilgan olan
# uygulamalar icin). Flathub varsayilan remote olarak otomatik geliyor.

{ config, lib, pkgs, ... }:

{
  services.flatpak.enable = true;
}
