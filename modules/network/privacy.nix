# AMAC: Ag izini azaltma. WiFi MAC adresi her baglantida degil, "stable ama
# ag operatorune gore farkli" (stable-ssid) olarak ayarlanir -- boylece hem
# farkli aglarda izlenemezsin hem de her baglantida yeniden kimlik dogrulama/
# DHCP gecikmesi yasamazsin. IPv6 privacy extensions ile de gecici IPv6
# adresleri kullanilir, kalici cihaz kimligi disariya sizmaz.

{ config, lib, pkgs, ... }:

{
  networking.networkmanager.wifi.macAddress = "stable-ssid";
  networking.networkmanager.ethernet.macAddress = "stable";

  # IPv6 privacy extensions -- gecici (rastgele) adresler tercih edilir,
  # kalici arayuz kimligine dayali adresler disari sizmaz
  networking.tempAddresses = "default";

  networking.networkmanager.connectionConfig = {
    # DHCP isteklerinde bilgisayar adini ("THORNS") gonderme: okul/kafe
    # modemi ve ag yoneticisi seni adinla goremesin. MAC rastgelelestirme
    # ile birlikte anlamli olur.
    "ipv4.dhcp-send-hostname" = false;
    "ipv6.dhcp-send-hostname" = false;
    # LLMNR / mDNS her baglantida kapali (dns.nix'teki resolved ayariyla ayni)
    "connection.llmnr" = 0;
    "connection.mdns" = 0;
  };
}
