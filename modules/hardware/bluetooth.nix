{ config, lib, pkgs, ... }:
{
  # AMAC: Bluetooth destegi. Donanim yoksa pasif kalir.
  # Acilista KAPALI gelir (okulda/ortak aglarda saldiri yuzeyi olmasin);
  # gerektiginde KDE sistem tepsisindeki Bluetooth simgesinden acilir.
  hardware.bluetooth.enable = true;
  hardware.bluetooth.powerOnBoot = false;

  # Blueman: Plasma disindaki oturumlar icin (Hyprland/Serpantinum).
  services.blueman.enable = true;

  # Blueman'in otomatik baslatma dosyasi KDE'yi haric tutmuyor; Plasma'da
  # kendi araci (bluedevil) ile yan yana ikinci bir tepsi simgesi ve ikinci
  # bir eslestirme ajani aciliyordu. /etc/xdg, paketin kendi dosyasindan
  # once okundugu icin ayni adli bu dosya onu gecersiz kilar.
  environment.etc."xdg/autostart/blueman.desktop".text = ''
    [Desktop Entry]
    Name=Blueman Applet
    Comment=Blueman Bluetooth Manager
    Icon=blueman
    Exec=blueman-applet
    Terminal=false
    Type=Application
    Categories=
    NotShowIn=KDE;
  '';
}
