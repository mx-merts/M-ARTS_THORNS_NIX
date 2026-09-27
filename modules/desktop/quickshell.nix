# AMAC: Quickshell (Qt/QML tabanli shell framework). Su an aktif
# kullanilmiyor, ileride Hyprland ricing icin hazir bekletiliyor.

{ config, lib, pkgs, ... }:

{
  environment.systemPackages = with pkgs; [
    quickshell
  ];
}
