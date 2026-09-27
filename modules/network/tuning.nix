# AMAC: TCP/network performans ayarlari. BBR congestion control (Google'in
# gelistirdigi, yuksek gecikme/paket kaybinda klasik CUBIC'ten belirgin daha
# iyi performans veren algoritma) + ilgili kernel ayarlari. VPN/DNS
# modullerinden bagimsiz, saf hiz katmani.

{ config, lib, pkgs, ... }:

{
  boot.kernel.sysctl = {
    # BBR congestion control -- fq (fair queueing) qdisc'i gerektirir
    "net.core.default_qdisc" = "fq";
    "net.ipv4.tcp_congestion_control" = "bbr";

    # TCP Fast Open (istemci + sunucu) -- el sikisma gecikmesini azaltir
    "net.ipv4.tcp_fastopen" = 3;

    # Buyuk pencere olcegi -- yuksek bant genisligi/gecikme baglantilarinda
    # (ozellikle VPN tuneli uzerinden) verimi artirir
    "net.core.rmem_max" = 16777216;
    "net.core.wmem_max" = 16777216;
    "net.ipv4.tcp_rmem" = "4096 87380 16777216";
    "net.ipv4.tcp_wmem" = "4096 65536 16777216";

    # MTU probing -- bazi aglarda/VPN tunellerinde paket parcalanmasini
    # onlemeye yardimci olur
    "net.ipv4.tcp_mtu_probing" = 1;
  };
}
