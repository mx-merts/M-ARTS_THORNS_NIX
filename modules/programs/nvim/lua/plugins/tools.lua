-- Araclar: git, hata paneli, todo, terminal, acilis ekrani
return {
  -- Git komutlari (GitLens'in cekirdegi)
  { "tpope/vim-fugitive", cmd = { "Git", "G", "Gvdiffsplit" } },

  -- Gorsel diff ve commit gecmisi
  {
    "sindrets/diffview.nvim",
    cmd = { "DiffviewOpen", "DiffviewFileHistory" },
    dependencies = { "nvim-lua/plenary.nvim" },
    keys = {
      { "<leader>gh", "<cmd>DiffviewFileHistory %<cr>", desc = "Dosya gecmisi" },
      { "<leader>gH", "<cmd>DiffviewFileHistory<cr>", desc = "Repo gecmisi" },
    },
    config = function()
      require("diffview").setup({})
    end,
  },

  -- Hata/diagnostic paneli (Error Lens listesi)
  {
    "folke/trouble.nvim",
    cmd = "Trouble",
    keys = {
      { "<leader>xx", "<cmd>Trouble diagnostics toggle<cr>", desc = "Tum hatalar" },
      { "<leader>xd", "<cmd>Trouble diagnostics toggle filter.buf=0<cr>", desc = "Bu dosyanin hatalari" },
    },
    config = function()
      require("trouble").setup({})
    end,
  },

  -- TODO/NOTE/FIXME vurgulama (better-comments'in pratik karsiligi)
  {
    "folke/todo-comments.nvim",
    event = { "BufReadPost", "BufNewFile" },
    dependencies = { "nvim-lua/plenary.nvim" },
    config = function()
      require("todo-comments").setup({})
    end,
  },

  -- Ic terminal (Ctrl-\ ile ac/kapat)
  {
    "akinsho/toggleterm.nvim",
    keys = { "<C-\\>" },
    config = function()
      require("toggleterm").setup({
        open_mapping = [[<c-\>]],
        direction = "horizontal",
        size = function(term)
          if term.direction == "horizontal" then
            return math.floor(vim.o.lines * 0.75)
          end
        end,
        
      })
    end,
  },

  -- Acilis ekrani (son dosyalar + kisayollar)
  {
    "nvimdev/dashboard-nvim",
    event = "VimEnter",
    dependencies = { "nvim-tree/nvim-web-devicons" },
    config = function()
      require("dashboard").setup({
        theme = "hyper",
        config = {
          week_header = { enable = true },
          shortcut = {
            { desc = " Dosyalar", group = "@property", action = "Telescope find_files", key = "ff" },
            { desc = " Metin ara", group = "@property", action = "Telescope live_grep", key = "fg" },
          },
        },
      })
    end,
  },
}
