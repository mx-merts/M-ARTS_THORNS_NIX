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

  # sddm-kcm: System Settings > Startup and Shutdown > Login Screen (SDDM)
  # altinda tema indirme/onizleme/uygulama arayuzu acar.
  # NOT: GUI'den secilen tema rebuild'de ezilir; kalici tema
  # sddm-theme.nix'te tanimli.
  environment.systemPackages = with pkgs; [
    kdePackages.sddm-kcm
  ];
}
