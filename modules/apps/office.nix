# AMAC: Ofis paketleri. LibreOffice + OnlyOffice birlikte (MS Office
# formatlarina uyumluluk icin ikisi de). Hizli/basit not almak icin KWrite
# zaten KDE Plasma ile geliyor, burada ayrica eklemiyoruz.

{ config, lib, pkgs, ... }:

{
  environment.systemPackages = with pkgs; [
    libreoffice-fresh
    onlyoffice-desktopeditors
  ];
}
