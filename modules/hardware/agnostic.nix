{ config, lib, pkgs, ... }:
{
  hardware.enableAllFirmware = true;
  hardware.enableRedistributableFirmware = true;

  zramSwap.enable = true;
  zramSwap.memoryPercent = 50;

  hardware.graphics.enable = true;
  hardware.graphics.enable32Bit = true;

  # --- Donanim varsa devreye giren, yoksa bosta duran destekler ---
  # Thunderbolt cihaz/dock yetkilendirme (Plasma'da ayar sayfasi da gelir).
  services.hardware.bolt.enable = true;
  # 2'si 1 arada laptoplar: ekran dondurme ve ortam isigi sensoru.
  hardware.sensor.iio.enable = true;
}
