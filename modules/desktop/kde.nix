{ config, lib, pkgs, ... }:
{
  # AMAÇ: KDE Plasma masaüstü ortamı - sadece görsel/masaüstü katmanı.
  # Ses için: modules/hardware/audio.nix
  # Yazıcı için: modules/services/printing.nix
  services.xserver.enable = true;
  services.displayManager.sddm.enable = true;
  services.desktopManager.plasma6.enable = true;

  services.xserver.xkb = {
    layout = "tr";
    variant = "";
  };
}
