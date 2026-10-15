-- ~/.config/nvim/init.lua
-- Giris kapisi: once kisisel ayarlar, sonra lazy.nvim (eklenti yoneticisi)

vim.g.mapleader = " "       -- leader tusu: bosluk
vim.g.maplocalleader = " "

require("config.options")   -- lua/config/options.lua
require("config.keymaps")   -- lua/config/keymaps.lua

-- lazy.nvim bootstrap (ilk acilista otomatik iner)
local lazypath = vim.fn.stdpath("data") .. "/lazy/lazy.nvim"
if not vim.uv.fs_stat(lazypath) then
  vim.fn.system({
    "git", "clone", "--filter=blob:none",
    "https://github.com/folke/lazy.nvim.git",
    "--branch=stable", lazypath,
  })
end
vim.opt.rtp:prepend(lazypath)

-- lua/plugins/ klasorundeki TUM dosyalar otomatik yuklenir
require("lazy").setup("plugins", {
  install = { colorscheme = { "catppuccin" } },
  checker = { enabled = true, notify = false }, -- guncelleme bildirimi sessiz
})
