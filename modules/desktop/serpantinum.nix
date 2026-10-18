{ config, lib, pkgs, serpantinum, ... }:
{
  # AMAÇ: Serpantinum shell'i. Yalnızca paket + sistem önkoşulları (PAM servisi,
  # i2c, font). Hiçbir şey kendiliğinden başlamaz: Hyprland içinden
  # `serpantinumd start` ile çalışır. SDDM/GRUB/KDE'ye dokunmaz.
  imports = [ serpantinum.nixosModules.default ];

  programs.serpantinum.enable = true;

  environment.systemPackages = [
    serpantinum.packages.${pkgs.stdenv.hostPlatform.system}.default
  ];
}
