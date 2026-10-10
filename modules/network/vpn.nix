# AMAC: ProtonVPN resmi GUI istemcisi (sistray, kolay ac/kapa). NixOS'ta bilinen
# "baglaniyor ama trafik gecmiyor" sorunu icin reverse-path filtering kapatildi
# (WireGuard'in asimetrik routing'i ile checkReversePath cakisiyor, kaynak:
# NixOS Discourse #65837). Otomatik baglanma yok, kill-switch yok -- sen ac/kapa
# kontrolundesin (not: proton-vpn kendi IPv6 kill-switch'ini otomatik ekliyor,
# bu bizim eklemedigimiz ama zararsiz/faydali bir davranis).
#
# NOT: VPN acikken DNS sorgulari kasitli olarak Proton'un kendi sunucusuna
# (10.2.0.1) gidiyor, bizim dnscrypt-proxy zincirimizi degil. Bu bir sizinti
# DEGIL -- tum trafik zaten WireGuard tuneli icinde sifreli, ISP hicbir sey
# goremiyor. VPN kapaliyken DNS otoritesi yine dnscrypt-proxy'de (dns.nix).
# Bunu zorla degistirmeye calismak (dispatcher script vb.) NetworkManager'in
# kendi senkronizasyonuyla surekli cakisiyor ve kararsiz davraniyor, bu yuzden
# bilincli olarak Proton'un varsayilan davranisina birakildi.

{ config, lib, pkgs, ... }:

{
  environment.systemPackages = with pkgs; [
    proton-vpn
    wireguard-tools
    networkmanagerapplet
  ];

  # WireGuard'in asimetrik routing'i reverse-path filtering ile cakisiyor,
  # bu olmadan ProtonVPN GUI baglanir ama hicbir paket disari cikmaz.
  networking.firewall.checkReversePath = false;
}
