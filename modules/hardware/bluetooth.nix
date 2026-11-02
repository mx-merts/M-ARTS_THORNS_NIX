{ config, lib, pkgs, ... }:
{
  # AMAC: Bluetooth destegi. Donanim yoksa pasif kalir.
  # Acilista KAPALI gelir (okulda/ortak aglarda saldiri yuzeyi olmasin);
  # gerektiginde KDE sistem tepsisindeki Bluetooth simgesinden acilir.
  hardware.bluetooth.enable = true;
  hardware.bluetooth.powerOnBoot = false;
  services.blueman.enable = true;
}
