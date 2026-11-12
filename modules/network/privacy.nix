# AMAC: Ag izini azaltma (SKIN'in IDENTITY katmaninin SEAL hali). MAC adresi
# AG BASINA ve ACILIS BASINA farkli: ayni agi yarin tekrar ziyaret eden
# "ayni cihaz" diye taninmaz, farkli aglar da birbirine baglanamaz. Ayni
# acilista kopup yeniden baglaninca MAC degismez: kafede giris sayfasi
# girisi Wi-Fi kopmalariyla kaybolmaz. Ayni mantik IPv6 kimliklerine de
# uygulanir (DHCPv6 DUID ve kalici IPv6 adresi stable-id'den turetilir);
# gecici IPv6 adresleri tercih edilir, kalici cihaz kimligi disariya sizmaz.
# Gercek MAC sadece SKIN ROT ID ile (bellekte, gecici) kullanilir.

{ config, lib, pkgs, ... }:

{
  # "stable" = connection.stable-id'den uretilir; stable-id asagida
  # baglanti + acilis kimligi: ag basina farkli, her acilista yeni.
  networking.networkmanager.wifi.macAddress = "stable";
  networking.networkmanager.ethernet.macAddress = "stable";

  # IPv6 privacy extensions -- gecici (rastgele) adresler tercih edilir,
  # kalici arayuz kimligine dayali adresler disari sizmaz
  networking.tempAddresses = "default";

  networking.networkmanager.connectionConfig = {
    # MAC, DHCPv6 DUID ve kalici IPv6 adresinin tohumu: ${CONNECTION} =
    # baglanti (ag) kimligi, ${BOOT} = bu acilisin kimligi.
    "connection.stable-id" = "\${CONNECTION}/\${BOOT}";
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
