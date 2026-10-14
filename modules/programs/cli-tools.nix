# AMAC: Terminal araclarinin TEK adresi. Prompt: starship (terminal.nix),
# shell: zsh (shell.nix). Baska modullerde ayni paketleri tekrar yazma.

{ config, lib, pkgs, ... }:

{
  environment.systemPackages = with pkgs; [
    micro
    eza
    bat
    ripgrep
    fd
    btop
    yazi
    delta
    tealdeer
    atuin
    jq

    # Sistem/dosya bilgisi
    dust          # klasor boyutlari (du yerine)
    duf           # disk durumu (df yerine)
    procs         # surec listesi (ps yerine)

    # Gunluk yardimcilar
    ouch          # tek komutla arsiv ac/olustur
    glow          # terminalde markdown oku
    trash-cli     # rm yerine cop kutusu
    pay-respects  # yanlis komutu duzeltir
    hyperfine     # komut hiz olcumu
    tokei         # kod satiri istatistigi
    gping         # grafikli ping

    # NixOS araclari
    nvd           # iki nesil arasi paket farki
    nix-output-monitor  # build ilerleme agaci (nom)
  ];

  programs.nh = {
    enable = true;
    flake = "/etc/nixos";
  };

  programs.direnv = {
    enable = true;
    nix-direnv.enable = true;
  };

  programs.zoxide.enable = true;

  programs.fzf = {
    keybindings = true;
    fuzzyCompletion = true;
  };

  programs.git = {
    enable = true;
    config = {
      core.pager = "delta";
      interactive.diffFilter = "delta --color-only";
      delta = { navigate = true; line-numbers = true; };
      merge.conflictStyle = "zdiff3";
      init.defaultBranch = "main";
    };
  };
}
