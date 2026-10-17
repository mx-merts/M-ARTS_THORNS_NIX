-- F5: acik dosyayi kaydeder, alttaki terminalde calistirir, cikti ekranda kalir.
-- Yeni dil eklemek icin command_for icine bir satir ekle.
local M = {}

local term = nil
local OUT = "/tmp/nvim-run-out"
local SCRIPT = "/tmp/nvim-run.sh"

local function command_for(ft, file)
  local f = vim.fn.shellescape(file)
  if ft == "python" then
    return "python3 " .. f
  elseif ft == "c" then
    return "gcc -g -Wall " .. f .. " -o " .. OUT .. " && " .. OUT
  elseif ft == "cpp" then
    return "g++ -std=c++17 -g -Wall " .. f .. " -o " .. OUT .. " && " .. OUT
  elseif ft == "rust" then
    local root = vim.fs.root(file, "Cargo.toml")
    if root then return "cargo run", root end
    return "rustc -g " .. f .. " -o " .. OUT .. " && " .. OUT
  elseif ft == "java" then
    return "java " .. f
  elseif ft == "javascript" or ft == "typescript" then
    return "node " .. f
  elseif ft == "lua" then
    return "nvim -l " .. f
  elseif ft == "sh" or ft == "bash" then
    return "bash " .. f
  end
end

function M.run()
  local file = vim.api.nvim_buf_get_name(0)
  if file == "" or vim.bo.buftype ~= "" then
    vim.notify("Calistirilacak bir dosya acik degil", vim.log.levels.WARN)
    return
  end

  local ft = vim.bo.filetype
  vim.cmd("silent! write")

  if ft == "html" then
    vim.cmd("LivePreview start")
    return
  end

  local cmd, dir = command_for(ft, file)
  if not cmd then
    vim.notify("F5: '" .. ft .. "' icin calistirma tanimli degil", vim.log.levels.WARN)
    return
  end

  vim.fn.writefile({
    "cd " .. vim.fn.shellescape(dir or vim.fn.fnamemodify(file, ":h")),
    cmd,
    "code=$?",
    "echo",
    'echo "[Bitti - cikis kodu: $code]"',
  }, SCRIPT)

  if term then term:shutdown() end
  local Terminal = require("toggleterm.terminal").Terminal
  term = Terminal:new({
    cmd = "bash " .. SCRIPT,
    direction = "horizontal",
    close_on_exit = false,
    hidden = true,
  })
    term:open(math.floor(vim.o.lines * 0.75))
end

function M.close()
  if term then
    term:shutdown()
    term = nil
  end
end

return M
