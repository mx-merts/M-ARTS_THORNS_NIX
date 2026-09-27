# AMAC: Terminal deneyimini guclendiren araclar. micro (basit editor),
# eza/bat/fzf/zoxide/ripgrep/btop klasik unix araclarinin modern
# alternatifleri. starship EKLENMEDI -- zaten oh-my-zsh + robbyrussell
# temasi var, iki prompt sistemi cakisir.

{ config, lib, pkgs, ... }:

{
  environment.systemPackages = with pkgs; [
    micro
    eza
    bat
    fzf
    zoxide
    ripgrep
    btop
  ];
}
