{ config, lib, pkgs, ... }:
{
  # AMAÇ: KDE Plasma masaüstü ortamı - sadece görsel/masaüstü katmanı.
  # Ses için: modules/hardware/audio.nix
  # Yazıcı için: modules/services/printing.nix
  services.xserver.enable = true;
  services.displayManager.sddm.enable = true;
  # Giris ekrani (SDDM) Wayland'de, KWin ile: ekran karti duzenine (iGPU,
  # NVIDIA, hibrit laptop) kendisi uyum saglar, X11 ayarina bagimli degildir.
  # Klavye duzeni asagidaki services.xserver.xkb'den (systemd-localed) gelir.
  # X11 Plasma oturumu yine secenek olarak durur.
  services.displayManager.sddm.wayland = {
    enable = true;
    compositor = "kwin";
  };
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
