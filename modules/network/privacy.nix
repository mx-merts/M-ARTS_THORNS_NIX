# AMAC: Ag izini azaltma. WiFi MAC adresi her baglantida degil, "stable ama
# ag operatorune gore farkli" (stable-ssid) olarak ayarlanir -- boylece hem
# farkli aglarda izlenemezsin hem de her baglantida yeniden kimlik dogrulama/
# DHCP gecikmesi yasamazsin. IPv6 privacy extensions ile de gecici IPv6
# adresleri kullanilir, kalici cihaz kimligi disariya sizmaz.

{ config, lib, pkgs, ... }:

{
  networking.networkmanager.wifi.macAddress = "stable-ssid";
  networking.networkmanager.ethernet.macAddress = "stable-ssid";

  # IPv6 privacy extensions -- gecici (rastgele) adresler tercih edilir,
  # kalici arayuz kimligine dayali adresler disari sizmaz
  networking.tempAddresses = "default";
}
