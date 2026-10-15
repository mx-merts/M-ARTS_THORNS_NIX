{ config, lib, pkgs, ... }:
{
  # AMAC: Neovim ayarlari repoda (modules/programs/nvim/) durur.
  # ~/.config/nvim buraya sembolik baglanti olur. Store'a kopyalanmaz,
  # yani dosyalar sudo'suz duzenlenir, lazy-lock.json yazilabilir kalir.
  # "L": hedef zaten varsa dokunmaz (veri kaybi riski yok).
  systemd.tmpfiles.rules = [
    "L /home/m-arts/.config/nvim - - - - /etc/nixos/modules/programs/nvim"
  ];
}
