-- LSP: sunucular Nix'ten kurulur (neovim.nix), burada baglanir
-- Yeni API: vim.lsp.config + vim.lsp.enable (nvim 0.11+)
return {
  {
    "neovim/nvim-lspconfig",
    event = { "BufReadPre", "BufNewFile" },
    dependencies = { "hrsh7th/cmp-nvim-lsp" },
    config = function()
          local capabilities = require("cmp_nvim_lsp").default_capabilities()

      -- Tum sunuculara ortak
      vim.lsp.config("*", { capabilities = capabilities })

      -- Ozel ayar isteyen sunucu
      vim.lsp.config("lua_ls", {
        settings = {
          Lua = { diagnostics = { globals = { "vim" } } },
        },
      })
      -- Aktif edilecek sunucular (RUST DAHIL DEGIL: rustaceanvim yonetir)
      local servers = {
        "clangd", "pyright", "omnisharp", "ts_ls",
        "html", "cssls", "jsonls", "eslint",
        "bashls", "taplo", "nixd", "cmake",
        "dockerls", "docker_compose_language_service",
        "lua_ls", "ruff", "emmet_language_server",
      }
      for _, s in ipairs(servers) do
        vim.lsp.enable(s)
      end

      -- LSP keymap'leri: LSP baglanan dosyada aktif olur
      vim.api.nvim_create_autocmd("LspAttach", {
        group = vim.api.nvim_create_augroup("UserLspConfig", { clear = false }),
        callback = function(ev)
        if vim.lsp.inlay_hint then
          vim.lsp.inlay_hint.enable(true, { bufnr = ev.buf })
        end
          local map = function(lhs, rhs, desc)
            vim.keymap.set("n", lhs, rhs, { buffer = ev.buf, desc = desc })
          end
          map("gd", vim.lsp.buf.definition, "Tanimaga git")
          map("gD", vim.lsp.buf.declaration, "Deklarasyona git")
          map("gr", vim.lsp.buf.references, "Referanslar")
          map("gi", vim.lsp.buf.implementation, "Implementasyon")
          map("K", vim.lsp.buf.hover, "Hover dokuman")
          map("<leader>rn", vim.lsp.buf.rename, "Yeniden adlandir")
          map("<leader>ca", vim.lsp.buf.code_action, "Code action")
        end,
      })

      -- Error Lens karsiligi: hatalari satir sonunda goster
     vim.diagnostic.config({
       virtual_text = { prefix = "●", spacing = 2 },
       signs = {
         text = {
           [vim.diagnostic.severity.ERROR] = "✘",
           [vim.diagnostic.severity.WARN] = "▲",
           [vim.diagnostic.severity.INFO] = "●",
           [vim.diagnostic.severity.HINT] = "➤",
         },
       },
       underline = true,
       severity_sort = true,
       float = { border = "rounded" },
     })
    end,
  },

  -- Rust: rust-analyzer'i kendi baslatir (lspconfig'den AYRI tutulmali)
  {
    "mrcjkb/rustaceanvim",
    version = "^6",
    ft = { "rust" },
  },
}
