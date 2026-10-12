# modules/programs/neovim.nix
# Neovim — vim'in modern varisi, sistem geneli kurulum
{ config, lib, pkgs, ... }:

{
  # NixOS'un built-in programs.neovim modülü
  programs.neovim = {
    enable = true;

    # EDITOR ve VISUAL ortam degiskenlerini nvim yapar
    # (git commit, crontab -e vb. nvim'i acar)
    defaultEditor = true;

    # terminalde "vi" veya "vim" yazinca da nvim acilsin
    viAlias = true;
    vimAlias = true;
  };

  # Neovim'in tam calismasi icin yardimci aracclar
  environment.systemPackages = with pkgs; [
  	(pkgs.nerd-fonts.jetbrains-mono)  # editor ikonlari icin nerd font
    ripgrep       # telescope live-grep icin sart
    fd            # telescope dosya bulma icin hizli alternatif
    tree-sitter   # syntax highlight parserlari
    gcc           # treesitter parser derlemesi icin
    xclip         # sistem panosu (clipboard) destegi
  ];
}
