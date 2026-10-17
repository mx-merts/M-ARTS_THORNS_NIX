-- Editor islevi eklentileri
return {
  -- Dosya agaci (VSCode Explorer)
  -- Dosya agaci (VSCode Explorer): Ctrl+B ile ac/kapat
  {
    "nvim-tree/nvim-tree.lua",
    dependencies = { "nvim-tree/nvim-web-devicons" },
    keys = {
      { "<leader>e", "<cmd>NvimTreeToggle<cr>", desc = "Explorer" },
      { "<C-b>", "<cmd>NvimTreeToggle<cr>", desc = "Explorer" },
    },
    config = function()
      require("nvim-tree").setup({
        view = { width = 30 },
        renderer = { indent_markers = { enable = true } },
        filters = { dotfiles = false },
        git = { ignore = false },
      })
    end,
  },

  -- Fuzzy finder (VSCode Ctrl+P + Ctrl+Shift+F)
  {
    "nvim-telescope/telescope.nvim",
    cmd = "Telescope",
    dependencies = { "nvim-lua/plenary.nvim" },
    keys = {
      { "<leader>ff", "<cmd>Telescope find_files<cr>", desc = "Dosya bul" },
      { "<leader>fg", "<cmd>Telescope live_grep<cr>", desc = "Metin ara" },
      { "<leader>fb", "<cmd>Telescope buffers<cr>", desc = "Acik dosyalar" },
    },
    config = function()
      require("telescope").setup({
        defaults = {
          borderchars = { "─", "│", "─", "│", "╭", "╮", "╯", "╰" },
        },
      })
    end,
  },

  -- Hangi tuslara bastigini gosteren popup
  {
    "folke/which-key.nvim",
    event = "VeryLazy",
  },

  -- Modern syntax highlighting (yeni nesil API)
  {
    "nvim-treesitter/nvim-treesitter",
    build = ":TSUpdate",
    config = function()
      require("nvim-treesitter").install({
        "lua", "vim", "vimdoc", "bash", "rust", "nix",
        "c", "cpp", "c_sharp", "python", "java",
        "javascript", "typescript", "tsx",
        "html", "css", "json", "toml", "yaml",
        "dockerfile", "cmake", "markdown", "markdown_inline",
      })
      
            vim.api.nvim_create_autocmd("FileType", {
              callback = function()
                pcall(vim.treesitter.start)
              end,
            })
    end,
  },

  -- Otomatik parantez kapatma
  {
    "windwp/nvim-autopairs",
    event = "InsertEnter",
    config = function()
      require("nvim-autopairs").setup({})
    end,
  },

  -- HTML/XML etiket otomatik kapatma + rename (auto-rename-tag karsiligi)
  {
    "windwp/nvim-ts-autotag",
    ft = { "html", "xml", "javascript", "typescript", "javascriptreact", "typescriptreact", "svelte", "vue" },
    config = function()
      require("nvim-ts-autotag").setup({})
    end,
  },

  -- Parantez/etiket cevresini sarma, degistirme, silme
  {
    "kylechui/nvim-surround",
    keys = { "cs", "ds", "ys" },
    config = function()
      require("nvim-surround").setup({})
    end,
  },

  -- Hizli imlec atlamasi (NOT: normal modda "s" artik Flash)
  {
    "folke/flash.nvim",
    keys = {
      { "s", function() require("flash").jump() end, mode = { "n", "x", "o" }, desc = "Flash jump" },
      { "S", function() require("flash").treesitter() end, mode = { "n", "x", "o" }, desc = "Flash treesitter" },
    },
    config = function()
      require("flash").setup({})
    end,
  },

  -- Kaydederken otomatik format (prettier karsiligi)
  -- Formatlayıcılar: otomatik değil, elle çalışır (Space fm)
  {
    "stevearc/conform.nvim",
    cmd = { "ConformInfo" },
    keys = {
      { "<leader>fm", function() require("conform").format({ async = true, lsp_format = "fallback" }) end, desc = "Formatla" },
    },
    config = function()
      require("conform").setup({
        formatters_by_ft = {
          javascript = { "prettierd" },
          typescript = { "prettierd" },
          javascriptreact = { "prettierd" },
          typescriptreact = { "prettierd" },
          html = { "prettierd" },
          css = { "prettierd" },
          scss = { "prettierd" },
          json = { "prettierd" },
          yaml = { "prettierd" },
          markdown = { "prettierd" },
          python = { "ruff" },
          rust = { "rustfmt" },
          lua = { "stylua" },
          c = { "clang-format" },
          cpp = { "clang-format" },
          nix = { "nixpkgs_fmt" },
          toml = { "taplo" },
        },
      })
    end,
  },

  -- HTTP istemci (Thunder Client karsiligi): test.http dosyasi olustur
  {
    "mistweaverco/kulala.nvim",
    ft = { "http", "rest" },
    keys = {
      { "<leader>Rs", function() require("kulala").run() end, ft = { "http", "rest" }, desc = "HTTP istegi gonder" },
    },
    config = function()
      require("kulala").setup({})
    end,
  },
}
