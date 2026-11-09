# AMAC: Cloudflare WARP (ucretsiz, sinirsiz, hizli). Tum trafigi Cloudflare'e
# sifreli tunelle (MASQUE = HTTP/3'e benzeyen, engellenmesi zor) gonderir:
# okul agi / ISP hangi sitelere girdigini goremez, ISP seviyesindeki site
# yasaklari (DNS/IP/SNI) islemez. Bir gizlilik VPN'i DEGILDIR: konum yine
# Turkiye gorunur ve trafigi Cloudflare gorur. Gercek gizlilik/ulke secimi
# icin Proton (vpn.nix). Ikisi AYNI ANDA acilmaz.
#
# DNS: WARP "tunnel_only" modunda kullanilir (bir kerelik: warp-cli mode
# tunnel_only). Bu modda WARP DNS'e dokunmaz; tek DNS otoritesi yine
# dnscrypt-proxy'dir (dns.nix, zararli alan adi filtresi korunur) ve onun
# sorgulari da zaten tunelin icinden gider. WARP'in kendi DNS modu
# resolv.conf'u ele gecirip resolved/dnscrypt ile Cloudflare+Proton'daki
# gibi cakisirdi.
#
# Ac/kapa: menuden "Cloudflare WARP" (sistem tepsisi) ya da
# warp-cli connect / warp-cli disconnect. WARP son durumu hatirlar: acik
# birakirsan acilista yeniden baglanir, kapali birakirsan kapali kalir.

{ config, lib, pkgs, ... }:

{
  services.cloudflare-warp = {
    enable = true;
    # WARP sadece DISARI baglanir; cevaplar stateful firewall'dan zaten gecer.
    # Modulun varsayilani UDP 2408'i DISARIDAN gelen baglantilara aciyor:
    # istemci icin gereksiz, okul aginda bos yere acik bir port olurdu.
    openFirewall = false;
  };
}
