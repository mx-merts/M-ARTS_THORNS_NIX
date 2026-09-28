# AMAC: SKlauncher'i NixOS'ta calistiran "mc" komutu. Launcher arayuzu JavaFX
# kullaniyor, JavaFX'in native kutuphaneleri GTK/X11 kutuphanelerine ihtiyac
# duyuyor, NixOS'ta bunlar /usr/lib'de olmadigi icin LD_LIBRARY_PATH ile
# gosteriliyor. Launcher Java 11-19 istiyor (jdk17 sabit), oyunun kendisi
# icin sistem Java'si (development.nix) kullanilir. Launcher'in actigi oyun
# sureci bu LD_LIBRARY_PATH'i miras alir, LWJGL icin de isine yarar.

{ config, lib, pkgs, ... }:

let
  mcLibs = with pkgs; [
    gtk3 glib pango cairo gdk-pixbuf atk libGL
    libx11 libxtst libxxf86vm libxext libxrender libxi libxrandr
    libxcursor libxinerama fontconfig freetype
    alsa-lib libpulseaudio openal
  ];

  mc = pkgs.writeShellApplication {
    name = "mc";
    text = ''
      export LD_LIBRARY_PATH="${lib.makeLibraryPath mcLibs}:/run/opengl-driver/lib''${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
      exec ${pkgs.jdk17}/bin/java -jar "$HOME/Desktop/SKlauncher-3.2.18.jar" "$@"
    '';
  };
in
{
  environment.systemPackages = [ mc ];
}
