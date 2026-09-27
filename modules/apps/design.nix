# AMAC: Tasarim/foto/video araclari + dosya sikistirma. Basit+profesyonel
# ikili mantigi: GIMP (basit) / Krita (profesyonel illustrasyon), Inkscape
# (vektor, tek basina yeterli), Kdenlive (basit video) / DaVinci Resolve
# NixOS'ta kirilgan (kutuphane/GPU surucu sorunlari yaygin), o yuzden buraya
# eklemedik -- Flathub uzerinden kurulmasi onerilir (services.flatpak.nix).
# Figma web tabanli, kurulum gerekmiyor, tarayicidan kullanilir.

{ config, lib, pkgs, ... }:

{
  environment.systemPackages = with pkgs; [
    gimp
    krita
    inkscape
    kdePackages.kdenlive
    peazip
  ];
}
