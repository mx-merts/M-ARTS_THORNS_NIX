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
# HER ZAMAN ACIK: warp-autoconnect her acilista (gerekirse kayit olur)
# tunnel_only modunu ayarlar ve baglanir; bu ayarlar elle yapilmaz, yeni bir
# kurulumda da kendiliginden gelir. Gecici kapatmak (or. Proton icin):
# warp-cli disconnect -> bir sonraki acilista yine baglanir.

{ config, lib, pkgs, ... }:

{
  services.cloudflare-warp = {
    enable = true;
    # WARP sadece DISARI baglanir; cevaplar stateful firewall'dan zaten gecer.
    # Modulun varsayilani UDP 2408'i DISARIDAN gelen baglantilara aciyor:
    # istemci icin gereksiz, okul aginda bos yere acik bir port olurdu.
    openFirewall = false;
  };

  # Her acilista: kayitli degilse kaydol (Cloudflare kosullari --accept-tos ile
  # kabul), DNS'e dokunmayan modu zorla, baglan. Kayit ve mod warp-svc'nin
  # kendi durum dosyasinda tutuldugu icin deklaratif hale getirmenin yolu bu.
  systemd.services.warp-autoconnect = {
    description = "Cloudflare WARP: tunnel_only mode + connect at boot";
    wantedBy = [ "multi-user.target" ];
    requires = [ "cloudflare-warp.service" ];
    after = [ "cloudflare-warp.service" ];
    path = [ config.services.cloudflare-warp.package ];
    serviceConfig = {
      Type = "oneshot";
      RemainAfterExit = true;
    };
    script = ''
      # warp-svc soketi hazir olana kadar bekle (en fazla 30 sn)
      for _ in $(seq 30); do
        warp-cli --accept-tos status >/dev/null 2>&1 && break
        sleep 1
      done
      warp-cli --accept-tos registration show >/dev/null 2>&1 \
        || warp-cli --accept-tos registration new
      warp-cli --accept-tos mode tunnel_only
      warp-cli --accept-tos connect
    '';
  };
}
