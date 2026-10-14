# modules/programs/neovim.nix
# Neovim + tum LSP sunuculari Nix'ten (Mason kullanilmiyor, NixOS'ta bozuk)
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
    (pkgs.nerd-fonts.jetbrains-mono)  # editor ikonlari icin nerd font
    ripgrep       # telescope live-grep icin sart
    fd            # telescope dosya bulma icin hizli alternatif
    tree-sitter   # syntax highlight parserlari
    gcc           # treesitter parser derlemesi icin
    xclip         # sistem panosu (clipboard) destegi
    nodejs        # js/ts LSP runtime'i

    # --- LSP sunuculari (lsp.lua ile eslesir) ---
    rust-analyzer                       # Rust
    clang-tools                         # C/C++ (clangd + lldb-dap)
    omnisharp-roslyn                    # C#
    pyright                             # Python (tip kontrolu)
    ruff                                # Python (lint + format)
    vscode-langservers-extracted        # html/css/json/eslint
    typescript-language-server          # JS/TS (top-level isim)
    dockerfile-language-server   # Dockerfile
    cmake-language-server               # CMake
    taplo                               # TOML
    nixd                                # NIX (bu repo icin sart)
    bash-language-server                # Bash
    lua-language-server                 # Lua (nvim config'i icin)
    lldb                                # lldb-dap (debug) icin
    docker-compose-language-service     # docker-compose LSP

    # --- Formatterlar (conform.nvim ile eslesir) ---
    prettierd
    stylua
    nixpkgs-fmt
    rustfmt

    # --- Python debug (debugpy iceren python wrapper) ---
    (python3.withPackages (ps: [ ps.debugpy ]))
  ];
}
