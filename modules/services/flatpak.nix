# AMAC: Flathub destegi + deklaratif flatpak paketleri. Sober (Roblox Linux
# istemcisi) nixpkgs'te paket olarak yok, resmi kurulum yontemi Flathub --
# bu yuzden burada tanimli, git'e kayitli, her rebuild'de garantili kurulu.
# Oyun baslaticilari (Lutris, Heroic) Flatpak: internetten gelen .exe'leri
# calistirdiklari icin ev dizininden ayrilmis bir sandbox'ta durmalari gerekir.

{ config, lib, pkgs, ... }:

let
  # Oyun baslaticilari icin ortak kural: ev dizininin TAMAMI kapali.
  # Bir oyun/mod/hile kiliginda gelen token calici (or. RedTiger) Discord
  # token'larini, Firefox profilini, ~/.ssh'i GOREMEZ. Baslaticilarin kendi
  # verisi ~/.var/app/<id> altinda durur, bu kuraldan etkilenmez.
  gameSandbox = gamesDir: extra: [
    "!home"                   # ev dizininin tamami: KAPALI
    "${gamesDir}:create"      # sadece kendi oyun klasoru (yoksa olusturulur)
    "/run/media"              # USB / harici diskler
    "/media"
  ] ++ extra;
in
{
  services.flatpak = {
    enable = true;
    remotes = [
      {
        name = "flathub";
        location = "https://dl.flathub.org/repo/flathub.flatpakrepo";
      }
    ];
    packages = [
      "org.vinegarhq.Sober"
      "net.lutris.Lutris"
      "com.heroicgameslauncher.hgl"
    ];

    overrides.settings = {
      # Lutris: oyunlar ~/OYUNLAR'da; indirilen kurulum dosyalari salt-okunur.
      "net.lutris.Lutris".Context.filesystems =
        gameSandbox "~/OYUNLAR" [ "xdg-download:ro" ];

      # Heroic (Epic/GOG/Amazon): oyunlar ~/GAMES'te.
      # Ilk acilista Ayarlar > Varsayilan kurulum yolu = /home/m-arts/GAMES yapilmali.
      "com.heroicgameslauncher.hgl".Context.filesystems =
        gameSandbox "~/GAMES" [ ];
    };
  };
}
