# modules/programs/neovim.nix
# Neovim + LazyVim'in sistemden bekledigi araclar.
#
# Kisisel ayarlar ~/.config/nvim'de (LazyVim, elle yonetilir; push.sh
# dotfiles/nvim altina yedekler). Dil sunucularinin cogunu LazyVim, Mason ile
# ~/.local/share/nvim/mason altina kendisi kurar (nix-ld sayesinde calisir,
# bkz. development.nix). Burada sadece:
#   - Mason'in ve treesitter'in calismak icin ihtiyac duydugu araclar,
#   - LazyVim'in Mason'dan DEGIL, PATH'ten aldigi sunucular/araclar.
# Mason'in zaten kurdugu sunucularin (pyright, vtsls, OmniSharp, bash/lua/
# docker/cmake/toml sunuculari vb.) Nix kopyalari kaldirildi.
{ config, lib, pkgs, ... }:

{
  # NixOS'un built-in programs.neovim modulu
  programs.neovim = {
    enable = true;
    defaultEditor = true;
    viAlias = true;
    vimAlias = true;
  };

  environment.systemPackages = with pkgs; [
    # --- Mason / treesitter calisma araclari ---
    tree-sitter   # parser derleme CLI'si
    gcc           # treesitter parser derlemesi
    nodejs        # Mason'in npm tabanli sunuculari (vtsls, pyright...)
    unzip         # Mason paketlerini acmak icin
    wl-clipboard  # Wayland panosu (wl-copy / wl-paste)
    xclip         # X11 panosu (XWayland / X11 oturumu)
    # Mason'in pip tabanli araclari + nvim-dap-python icin debugpy
    (python3.withPackages (ps: [ ps.debugpy ]))

    # --- LazyVim'in PATH'ten aldiklari ---
    rust-analyzer                  # rustaceanvim (lang.rust) Mason'u kullanmaz
    rustfmt                        # rust-analyzer bicimlendirmesi
    vscode-langservers-extracted   # html/css/json (coding.lua: mason = false)
    emmet-language-server          # emmet (coding.lua: mason = false)
    nil                            # lang.nix: nil_ls (Mason'da kurulu degil)
    nixfmt                         # lang.nix: conform bicimlendiricisi
    statix                         # lang.nix: nvim-lint denetleyicisi

    # --- Editor disinda da kullanilan CLI araclari ---
    clang-tools   # clang-format, clang-tidy (clangd'yi Neovim Mason'dan alir)
    lldb          # komut satiri hata ayiklayici
    ruff          # Python lint/format
    stylua        # Lua bicimlendirici
    nixd          # Nix sunucusu (VS Code vb. editorler icin)
  ];
}
