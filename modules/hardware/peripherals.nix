{ config, lib, pkgs, ... }:
{
  # AMAC: USB depolama otomatik baglama, kamera/mikrofon, kart okuyucu, genel cevre birimleri.
  # Kamera/mikrofon suruculeri kernel + pipewire ile otomatik gelir, burada sadece
  # otomatik baglama/algilama katmanini acik tutuyoruz.

  services.udisks2.enable = true;
  services.gvfs.enable = true;

  # Tarayici (scanner): USB ve ag tarayicilari. Kullanici "scanner" ve "lp"
  # gruplarinda olmali (users/m-arts.nix). Tarama uygulamasi: Skanpage.
  hardware.sane.enable = true;

  environment.systemPackages = with pkgs; [
    usbutils
    pciutils
    kdePackages.skanpage
  ];
}
