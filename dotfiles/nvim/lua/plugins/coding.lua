return {
  -- HTML/CSS sunuculari Nix'ten geliyor, Mason indirmesin
  {
    "neovim/nvim-lspconfig",
    opts = {
      servers = {
        emmet_language_server = { mason = false },
        html = { mason = false },
        cssls = { mason = false },
        jsonls = { mason = false },
      },
    },
  },

  -- Tab = oneriyi kabul et / snippet icinde ilerle
  {
    "saghen/blink.cmp",
    opts = { keymap = { preset = "super-tab" } },
  },
}
