# AMAC: Qylock "pixel-night-city" SDDM temasini statik PNG arka planla
# paketler. Orijinal tema bg.mp4 (12MB video) kullaniyordu, BackgroundVideo.qml
# statik Image yukleyecek sekilde degistirildi (bkz. assets/sddm/).
#
# Tema dosyalari assets/sddm/pixel-night-city/ altinda repo'ya gomulu,
# boylece tasinabilir flash'ta veya yeni makinede otomatik gelir.
#
# Orijinal tema: qylock (https://github.com/Darkkal44/qylock)
# Lisans: CC BY-NC-SA 4.0 | Author: Darkkal44

{ config, lib, pkgs, ... }:

let
  sddm-theme-pixel-night-city = pkgs.stdenvNoCC.mkDerivation {
    pname = "sddm-theme-pixel-night-city";
    version = "1.0";
    src = ../../assets/sddm/pixel-night-city;
    dontBuild = true;
    installPhase = ''
      runHook preInstall
      mkdir -p $out/share/sddm/themes/pixel-night-city
      cp -r . $out/share/sddm/themes/pixel-night-city/
      runHook postInstall
    '';
    meta = {
      description = "Pixel Night City SDDM theme (static PNG background variant)";
      homepage = "https://github.com/Darkkal44/qylock";
    };
  };
in
{
  services.displayManager.sddm = {
    theme = "pixel-night-city";
    extraPackages = [
      sddm-theme-pixel-night-city
      pkgs.qt6.qt5compat
      pkgs.qt6.qtmultimedia
      pkgs.qt6.qtsvg
    ];
  };

  environment.systemPackages = [
    sddm-theme-pixel-night-city
  ];

  environment.etc."sddm.conf.d/zz-nixos-override.conf".text = ''
    [Theme]
    Current=pixel-night-city
  '';
}
