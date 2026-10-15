-- Copilot: yazarken gri (ghost text) oneri. Giris: :Copilot auth
return {
  {
    "zbirenbaum/copilot.lua",
    cmd = "Copilot",
    event = "InsertEnter",
    config = function()
      require("copilot").setup({
        panel = { enabled = false },
        suggestion = {
          enabled = true,
          auto_trigger = true,
          keymap = {
            accept = "<M-l>",       -- Alt+l: oneriyi kabul et
            accept_word = "<M-k>",  -- Alt+k: sadece bir kelime
            next = "<M-]>",
            prev = "<M-[>",
            dismiss = "<C-]>",
          },
        },
        filetypes = { markdown = true, help = false },
      })
    end,
  },
}
