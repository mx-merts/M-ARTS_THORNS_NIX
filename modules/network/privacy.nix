# AMAC: Ag izini azaltma. WiFi MAC adresi her baglantida degil, "stable ama
# ag operatorune gore farkli" (stable-ssid) olarak ayarlanir -- boylece hem
# farkli aglarda izlenemezsin hem de her baglantida yeniden kimlik dogrulama/
# DHCP gecikmesi yasamazsin. Ayni mantik IPv6 kimliklerine de uygulanir:
# DHCPv6 kimligi (DUID) ve kalici IPv6 adresi aga gore farkli, gecici IPv6
# adresleri tercih edilir; kalici cihaz kimligi disariya sizmaz.

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
    # DHCPv6 kimligi: varsayilan ("lease") machine-id'den uretiliyor ve HER
    # agda ayni. stable-uuid: baglanti basina farkli, o ag icin sabit.
    "ipv6.dhcp-duid" = "stable-uuid";
    # Kalici IPv6 adresi: kayitli baglantilarin varsayilani MAC'ten (EUI-64)
    # turetiyordu. stable-privacy (RFC 7217): ag basina farkli, sabit.
    "ipv6.addr-gen-mode" = "stable-privacy";
    # LLMNR / mDNS her baglantida kapali (dns.nix'teki resolved ayariyla ayni)
    "connection.llmnr" = 0;
    "connection.mdns" = 0;
  };
}
