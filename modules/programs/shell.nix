# AMAC: Zsh + Oh My Zsh (populer, plugin/tema ekosistemi genis, baskalarindan
# kopyalamak kolay) + fastfetch (her yeni terminalde otomatik sistem bilgisi,
# ozel ASCII logo + sade/duzenli bilgi formati -- config /etc/fastfetch/).

{ config, lib, pkgs, ... }:

{
  programs.zsh = {
    enable = true;
    ohMyZsh = {
      enable = true;
      theme = "robbyrussell";
      plugins = [ "git" "sudo" "history-substring-search" ];
    };
    interactiveShellInit = ''
      fastfetch --config /etc/fastfetch/config.jsonc
    '';
  };

  environment.systemPackages = with pkgs; [
    fastfetch
  ];

  users.users.m-arts.shell = pkgs.zsh;
}
