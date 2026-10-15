-- AI: codecompanion + GLM (OpenAI uyumlu endpoint)
-- Anahtar ortam degiskeninden okunur: GLM_API_KEY
return {
  {
    "olimorris/codecompanion.nvim",
    cmd = { "CodeCompanion", "CodeCompanionChat", "CodeCompanionActions" },
    dependencies = { "nvim-lua/plenary.nvim", "nvim-treesitter/nvim-treesitter" },
    keys = {
      { "<leader>aa", "<cmd>CodeCompanionActions<cr>", mode = { "n", "v" }, desc = "AI Actions" },
      { "<leader>ac", "<cmd>CodeCompanionChat Toggle<cr>", mode = { "n", "v" }, desc = "AI Chat" },
    },
    config = function()
      require("codecompanion").setup({
        adapters = {
          glm = function()
            return require("codecompanion.adapters").extend("openai_compatible", {
              name = "glm",
              env = {
                url = "https://open.bigmodel.cn/api/paas/v4",
                api_key = "GLM_API_KEY",
                chat_url = "/chat/completions",
              },
              schema = {
                model = { default = "glm-4.6" },  -- modeli buradan degistir
              },
            })
          end,
        },
        strategies = {
          chat = { adapter = "glm" },
          inline = { adapter = "glm" },
        },
      })
    end,
  },
}
