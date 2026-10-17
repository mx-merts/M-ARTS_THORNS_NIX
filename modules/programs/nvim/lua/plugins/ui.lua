-- Gorunum eklentileri
-- Tema: Space tt ile acik/koyu arasinda gecis
return {
  {
    "polirritmico/monokai-nightasty.nvim",
    lazy = false,
    priority = 1000,
    config = function()
      require("monokai-nightasty").setup({
        dark_style_background = "transparent",
        light_style_background = "default",  -- acik temada opak zemin
        hl_styles = { floats = "dark", sidebars = "dark" },
      })

      -- Mevcut renk grubunu koruyup sadece verilen alanları değiştirir
      local function tweak(name, opts)
        local cur = vim.api.nvim_get_hl(0, { name = name, link = false })
        vim.api.nvim_set_hl(0, name, vim.tbl_extend("force", cur, opts))
      end

      -- Imleç ve açık temada okunabilirlik
      local function cursor_hl()
        vim.api.nvim_set_hl(0, "Cursor", { bg = "#f92672", fg = "#ffffff" })
        vim.api.nvim_set_hl(0, "lCursor", { bg = "#f92672", fg = "#ffffff" })

        if vim.o.background == "light" then
          tweak("Normal", { fg = "#111111" })
          tweak("Comment", { fg = "#5a5a5a" })
          tweak("LineNr", { fg = "#666666" })
          tweak("NonText", { fg = "#777777" })
          tweak("ErrorMsg", { fg = "#b00020", bold = true })
          tweak("WarningMsg", { fg = "#8a5a00", bold = true })
          tweak("MoreMsg", { fg = "#00600f", bold = true })
        end
      end
      vim.api.nvim_create_autocmd("ColorScheme", { callback = cursor_hl })

      vim.o.background = "light"   -- koyu istersen "dark" yap
      vim.cmd.colorscheme("monokai-nightasty")
      cursor_hl()

      vim.keymap.set("n", "<leader>tt", "<Cmd>MonokaiToggleLight<CR>", { desc = "Acik/koyu tema" })
    end,
  },

  -- Ustte sekmeler
  {
    "akinsho/bufferline.nvim",
    version = "*",
    dependencies = { "nvim-tree/nvim-web-devicons" },
    config = function()
      require("bufferline").setup({
        options = {
          diagnostics = "nvim_lsp",
          separator_style = "slope",
          offsets = {
            { filetype = "NvimTree", text = "Dosyalar", text_align = "left", separator = true },
          },
        },
      })
    end,
  },

  -- Alt cubuk: hap (pill) seklinde
  {
    "nvim-lualine/lualine.nvim",
    dependencies = { "nvim-tree/nvim-web-devicons" },
    config = function()
      local L = vim.fn.nr2char(0xe0b6)  -- sol yuvarlak kapak
      local R = vim.fn.nr2char(0xe0b4)  -- sag yuvarlak kapak
      require("lualine").setup({
        options = {
          theme = "auto",
          globalstatus = true,
          component_separators = "",
          section_separators = { left = R, right = L },
        },
        sections = {
          lualine_a = { { "mode", separator = { left = L }, right_padding = 2 } },
          lualine_b = { "filename", "branch" },
          lualine_c = { "diagnostics" },
          lualine_x = {},
          lualine_y = { "filetype", "progress" },
          lualine_z = { { "location", separator = { right = R }, left_padding = 2 } },
        },
      })
    end,
  },

  -- Satir yaninda git isaretleri
  {
    "lewis6991/gitsigns.nvim",
    config = function()
      require("gitsigns").setup()
    end,
  },
}
