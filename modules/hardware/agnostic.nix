{ config, lib, pkgs, ... }:
{
  hardware.enableAllFirmware = true;
  hardware.enableRedistributableFirmware = true;

  zramSwap.enable = true;
  zramSwap.memoryPercent = 50;

  hardware.graphics.enable = true;
  hardware.graphics.enable32Bit = true;
}
