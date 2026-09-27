# AMAC: ProtonVPN resmi GUI istemcisi (sistray, kolay ac/kapa). NixOS'ta bilinen
# "baglaniyor ama trafik gecmiyor" sorunu icin reverse-path filtering kapatildi
# (WireGuard'in asimetrik routing'i ile checkReversePath cakisiyor, kaynak:
# NixOS Discourse #65837). Otomatik baglanma yok, kill-switch yok -- sen ac/kapa
# kontrolundesin.

{ config, lib, pkgs, ... }:

{
  environment.systemPackages = with pkgs; [
    proton-vpn
    wireguard-tools
  ];

  # WireGuard'in asimetrik routing'i reverse-path filtering ile cakisiyor,
  # bu olmadan ProtonVPN GUI baglanir ama hicbir paket disari cikmaz.
  networking.firewall.checkReversePath = false;
}
