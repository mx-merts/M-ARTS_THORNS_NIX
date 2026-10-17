-- Tüm kısayollar burada (VSCode tarzı)
-- <leader> = boşluk tuşu
local map = vim.keymap.set

-- Insert modundan çıkmak için jk
map("i", "jk", "<Esc>")

map("n", "<leader>w", "<cmd>write<cr>", { desc = "Kaydet" })
map("n", "<leader>q", "<cmd>quit<cr>", { desc = "Çık" })

-- Pencereler arası gezinme (Ctrl+hjkl)
map("n", "<C-h>", "<C-w>h")
map("n", "<C-j>", "<C-w>j")
map("n", "<C-k>", "<C-w>k")
map("n", "<C-l>", "<C-w>l")

-- Açık dosyalar arası geçiş (Shift+h/l, sekmeler gibi) şimdilik kaldırıldı
-- map("n", "<S-h>", "<cmd>bprevious<cr>")
-- map("n", "<S-l>", "<cmd>bnext<cr>")

-- Seçili metni taşı (Visual modda J/K)
map("v", "J", ":m '>+1<cr>gv=gv")
map("v", "K", ":m '<-2<cr>gv=gv")

-- ===== VSCode tarzı kısayollar =====

-- Kaydet
map({ "n", "i", "v" }, "<C-s>", "<cmd>write<cr>", { desc = "Kaydet" })

-- Geri al / yinele
map("n", "<C-z>", "u")
map("i", "<C-z>", "<C-o>u")
map("n", "<C-y>", "<C-r>")
map("i", "<C-y>", "<C-o><C-r>")

-- Hepsini seç (Select modu: yazarsan yerine geçer, Backspace/Delete siler)
map("n", "<C-a>", "gggH<C-o>G")
map("i", "<C-a>", "<C-o>gg<C-o>gH<C-o>G")
map("v", "<C-a>", "<esc>gggH<C-o>G")

-- Fareyle yapılan seçimde de Backspace/Delete silsin
map("x", "<BS>", '"_d')
map("x", "<Del>", '"_d')

-- Kopyala / kes / yapıştır (sistem panosu)
map("v", "<C-c>", '"+y')
map("v", "<C-x>", '"+d')
map("n", "<C-v>", '"+p')
map("v", "<C-v>", '"+p')
map("i", "<C-v>", "<C-r><C-p>+")
map("n", "<C-q>", "<C-v>", { desc = "Blok seçim (eski Ctrl+V)" })

-- Ara
map("n", "<C-f>", "<cmd>Telescope current_buffer_fuzzy_find<cr>", { desc = "Dosyada ara" })
map("n", "<C-p>", "<cmd>Telescope find_files<cr>", { desc = "Dosya bul" })
map("n", "<C-S-f>", "<cmd>Telescope live_grep<cr>", { desc = "Tüm projede ara" })

-- Yorum satırı (Ctrl+/): terminale göre <C-_> olarak gelir, ikisi de tanımlı
map("n", "<C-_>", "gcc", { remap = true })
map("v", "<C-_>", "gc", { remap = true })
map("n", "<C-/>", "gcc", { remap = true })
map("v", "<C-/>", "gc", { remap = true })

-- Satırı taşı (Alt+Yukarı/Aşağı)
map("n", "<A-Down>", "<cmd>m .+1<cr>==")
map("n", "<A-Up>", "<cmd>m .-2<cr>==")
map("i", "<A-Down>", "<esc><cmd>m .+1<cr>==gi")
map("i", "<A-Up>", "<esc><cmd>m .-2<cr>==gi")
map("v", "<A-Down>", ":m '>+1<cr>gv=gv")
map("v", "<A-Up>", ":m '<-2<cr>gv=gv")

-- Satırı çoğalt (Shift+Alt+Yukarı/Aşağı)
map("n", "<S-A-Down>", "<cmd>t.<cr>")
map("n", "<S-A-Up>", "<cmd>t.-1<cr>")

-- Insert modunda Ctrl+Backspace önceki kelimeyi siler
map("i", "<C-BS>", "<C-w>")

-- Kısayol listesi (F1 veya Boşluk + ?)
map("n", "<F1>", function() require("config.cheatsheet").toggle() end, { desc = "Kısayol listesi" })
map("i", "<F1>", function()
  vim.cmd("stopinsert")
  require("config.cheatsheet").toggle()
end, { desc = "Kısayol listesi" })
map("n", "<leader>?", function() require("config.cheatsheet").toggle() end, { desc = "Kısayol listesi" })

-- VSCode benzeri ekler
map("n", "<F2>", vim.lsp.buf.rename, { desc = "Yeniden adlandır" })
map("n", "<leader>bd", "<cmd>bdelete<cr>", { desc = "Sekmeyi kapat" })
map("n", "<leader>fc", "<cmd>Telescope commands<cr>", { desc = "Komutlar" })



-- ===== Calistirma (F5) =====
map({ "n", "i" }, "<F5>", function()
  vim.cmd("stopinsert")
  require("config.run").run()
end, { desc = "Dosyayi calistir" })
map("n", "<leader>rc", function() require("config.run").close() end, { desc = "Cikti terminalini kapat" })

-- Terminalden cikis ve geri donus
map("t", "<Esc><Esc>", [[<C-\><C-n>]], { desc = "Terminal normal mod" })
map("t", "<C-h>", [[<C-\><C-n><C-w>h]])
map("t", "<C-j>", [[<C-\><C-n><C-w>j]])
map("t", "<C-k>", [[<C-\><C-n><C-w>k]])
map("t", "<C-l>", [[<C-\><C-n><C-w>l]])

-- ===== Sekmeler (buffer) =====
map({ "n", "i" }, "<C-Tab>", "<Cmd>BufferLineCycleNext<CR>", { desc = "Sonraki sekme" })
map({ "n", "i" }, "<C-S-Tab>", "<Cmd>BufferLineCyclePrev<CR>", { desc = "Onceki sekme" })

local function close_buffer()
  local cur = vim.api.nvim_get_current_buf()
  if #vim.fn.getbufinfo({ buflisted = 1 }) > 1 then
    vim.cmd("bprevious")
  end
  vim.cmd("bdelete " .. cur)
end
map({ "n", "i" }, "<A-w>", close_buffer, { desc = "Sekmeyi kapat" })
map("n", "<leader>bd", close_buffer, { desc = "Sekmeyi kapat" })
map("n", "<leader>bp", "<Cmd>BufferLinePick<CR>", { desc = "Sekme sec (harfle)" })

-- ===== Bolmeler (window) =====
map("n", "<leader>sv", "<cmd>vsplit<cr>", { desc = "Dikey bol" })
map("n", "<leader>sh", "<cmd>split<cr>", { desc = "Yatay bol" })
