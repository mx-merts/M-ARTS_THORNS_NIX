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
    yazi          # terminal dosya yoneticisi
    delta         # guzel git diff
    tealdeer      # tldr: kisa, ornekli man
    atuin         # aranabilir shell gecmisi
    jq
  ];

  # nix-direnv: klasore girince flake/shell ortami otomatik yuklenir
  programs.direnv = {
    enable = true;
    nix-direnv.enable = true;
  };

  # z <klasor> ile hizli gecis (zsh entegrasyonu otomatik)
  programs.zoxide.enable = true;

  # Ctrl+T (dosya sec), Alt+C (klasore atla), ** + Tab tamamlama
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
