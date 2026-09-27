# AMAC: ProtonVPN resmi GUI istemcisi (sistray, kolay ac/kapa). NixOS'ta bilinen
# "baglaniyor ama trafik gecmiyor" sorunu icin reverse-path filtering kapatildi
# (WireGuard'in asimetrik routing'i ile checkReversePath cakisiyor, kaynak:
# NixOS Discourse #65837). Otomatik baglanma yok, kill-switch yok -- sen ac/kapa
# kontrolundesin (not: proton-vpn kendi IPv6 kill-switch'ini otomatik ekliyor,
# bu bizim eklemedigimiz ama zararsiz/faydali bir davranis).

{ config, lib, pkgs, ... }:

{
  environment.systemPackages = with pkgs; [
    proton-vpn
    wireguard-tools
  ];

  # WireGuard'in asimetrik routing'i reverse-path filtering ile cakisiyor,
  # bu olmadan ProtonVPN GUI baglanir ama hicbir paket disari cikmaz.
  networking.firewall.checkReversePath = false;

  # proton0 arayuzu kendi DNS'ini (10.2.0.1) push ediyor, bu da bizim global
  # dnscrypt-proxy otoritesini (dns.nix) bypass ediyor. Arayuz adi sunucu/profil
  # degisse de sabit kaliyor (proton0), o yuzden bu script hangi sunucuya
  # baglanirsan baglan calisir -- profile-bagimli bir cozum degil.
  networking.networkmanager.dispatcherScripts = [{
    source = pkgs.writeShellScript "proton-dns-fix" ''
      if [[ "$1" == proton* && ( "$2" == "up" || "$2" == "vpn-up" ) ]]; then
        ${pkgs.systemd}/bin/resolvectl dns "$1" ""
        ${pkgs.systemd}/bin/resolvectl domain "$1" ""
        logger "proton-dns-fix: $1 icin per-link DNS temizlendi, global otorite (127.0.0.1) korunuyor"
      fi
    '';
    type = "basic";
  }];
}
