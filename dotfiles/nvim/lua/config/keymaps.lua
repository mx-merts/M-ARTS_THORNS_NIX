-- F5: acik dosyayi kaydet, altta ekranin %75'i kadar terminalde calistir
local run_buf = nil

local function close_run()
  if run_buf and vim.api.nvim_buf_is_valid(run_buf) then
    vim.api.nvim_buf_delete(run_buf, { force = true })
  end
  run_buf = nil
end

local function run_file()
  close_run()
  local file = vim.api.nvim_buf_get_name(0)
  if file == "" or vim.bo.buftype ~= "" then
    vim.notify("Calistirilacak dosya yok", vim.log.levels.WARN)
    return
  end
  vim.cmd("silent! write")

  local f = vim.fn.shellescape(file)
  local out = "/tmp/nvim-run-out"
  local cmds = {
    python = "python3 " .. f,
    c = "gcc -g -Wall " .. f .. " -o " .. out .. " && " .. out,
    cpp = "g++ -std=c++17 -g -Wall " .. f .. " -o " .. out .. " && " .. out,
    java = "java " .. f,
    javascript = "node " .. f,
    lua = "nvim -l " .. f,
    sh = "bash " .. f,
  }
  local cmd = cmds[vim.bo.filetype]
  if not cmd then
    vim.notify("F5: " .. vim.bo.filetype .. " icin tanimli degil", vim.log.levels.WARN)
    return
  end

  local script = "/tmp/nvim-run.sh"
  vim.fn.writefile({
    "cd " .. vim.fn.shellescape(vim.fn.fnamemodify(file, ":h")),
    cmd,
    "code=$?",
    "echo",
    'echo "[Bitti - cikis kodu: $code]  (Esc Esc, sonra q: kapat)"',
  }, script)

  vim.cmd("botright " .. math.floor(vim.o.lines * 0.75) .. "split")
  vim.cmd("terminal bash " .. script)
  run_buf = vim.api.nvim_get_current_buf()
  vim.bo[run_buf].bufhidden = "wipe"

  local o = { buffer = run_buf, nowait = true }
  vim.keymap.set("t", "<Esc><Esc>", [[<C-\><C-n>]], o)
  vim.keymap.set("n", "q", close_run, o)
  vim.cmd("startinsert")
end

vim.keymap.set({ "n", "i" }, "<F5>", function()
  vim.cmd("stopinsert")
  run_file()
end, { desc = "Dosyayi calistir" })


-- VS Code tarzi kisayollar
local map = vim.keymap.set

-- Cik (kaydedilmemis dosya varsa sorar)
map({ "n", "i", "v" }, "<C-q>", "<cmd>confirm qa<cr>", { desc = "Programdan cik" })

-- Kopyala / kes / yapistir (sistem panosu)
map("v", "<C-c>", '"+y', { desc = "Kopyala" })
map("v", "<C-x>", '"+d', { desc = "Kes" })
map("n", "<C-v>", '"+p', { desc = "Yapistir" })
map("v", "<C-v>", '"+p', { desc = "Yapistir" })
map("i", "<C-v>", "<C-r>+", { desc = "Yapistir" })

-- Hepsini sec
map("n", "<C-a>", "ggVG", { desc = "Hepsini sec" })
map("i", "<C-a>", "<Esc>ggVG", { desc = "Hepsini sec" })

-- Sekme degistir
map({ "n", "i" }, "<C-Tab>", "<cmd>bnext<cr>", { desc = "Sonraki sekme" })
map({ "n", "i" }, "<C-S-Tab>", "<cmd>bprevious<cr>", { desc = "Onceki sekme" })

-- Blok secim eskiden Ctrl+V'ydi, artik Space v
map("n", "<leader>v", "<C-v>", { desc = "Blok secim" })

-- Geri al / yinele (Ctrl+Z nvim'i arka plana atmasin)
map({ "n", "v" }, "<C-z>", "u", { desc = "Geri al" })
map("i", "<C-z>", "<C-o>u", { desc = "Geri al" })
map("n", "<C-y>", "<C-r>", { desc = "Yinele" })
map("i", "<C-y>", "<C-o><C-r>", { desc = "Yinele" })

-- Kaydet (insert modundan atmaz)
map({ "n", "i", "v" }, "<C-s>", "<cmd>write<cr>", { desc = "Kaydet" })

-- Secim yokken Ctrl+C / Ctrl+X tum satiri kopyalar / keser
map("n", "<C-c>", '"+yy', { desc = "Satiri kopyala" })
map("n", "<C-x>", '"+dd', { desc = "Satiri kes" })
