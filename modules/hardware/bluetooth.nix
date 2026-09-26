{ config, lib, pkgs, ... }:
{
  # AMAC: Bluetooth destegi. Her makinede guvenli, donanim yoksa pasif kalir.
  hardware.bluetooth.enable = true;
  hardware.bluetooth.powerOnBoot = true;
  services.blueman.enable = true;
}
