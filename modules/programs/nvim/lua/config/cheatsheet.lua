-- Kısayol listesi: F1 veya Boşluk + ? ile aç/kapat
-- Yeni kısayol eklediğinde aşağıdaki "sections" listesine bir satır ekle.
local M = {}

local sections = {
  { "TEMEL (TUTORIAL)", {
    { "i", "Yazmaya başla (insert modu)" },
    { "Esc veya jk", "Yazmayı bitir, normal moda dön" },
    { "v", "Seçim modu (ok tuşlarıyla seç)" },
    { "dd / yy / p", "Satırı sil / kopyala / yapıştır" },
    { "u", "Geri al" },
    { "/kelime", "Dosyada ara (n: sonraki sonuç)" },
    { ":w  :q  :wq", "Kaydet / çık / kaydet ve çık" },
  } },
  { "DOSYA VE ARAMA", {
    { "Ctrl+S  /  Space w", "Kaydet" },
    { "Ctrl+B  /  Space e", "Dosya ağacını aç/kapat" },
    { "Ctrl+P  /  Space ff", "Dosya bul" },
    { "Space fg", "Tüm projede metin ara" },
    { "Space fb", "Açık dosyalar listesi" },
    { "Ctrl+F", "Bu dosyada ara" },
    { "Space fc", "Komut listesi (VSCode Ctrl+Shift+P)" },
    { "Shift+H / Shift+L", "Önceki / sonraki sekme" },
    { "Space bd", "Sekmeyi kapat" },
    { "Space q", "Pencereyi kapat" },
  } },
  { "DOSYA AĞACI (içindeyken)", {
    { "Enter", "Dosyayı aç" },
    { "a", "Yeni dosya/klasör (klasör için sonuna / koy)" },
    { "d  /  r", "Sil / yeniden adlandır" },
    { "?", "Ağacın tüm kısayolları" },
  } },
  { "DÜZENLEME (VSCode gibi)", {
    { "Ctrl+A", "Hepsini seç (yazarsan yerine geçer, Backspace siler)" },
    { "Ctrl+C / X / V", "Kopyala / kes / yapıştır" },
    { "Ctrl+Z / Ctrl+Y", "Geri al / yinele" },
    { "Ctrl+/", "Yorum satırı yap/kaldır" },
    { "Alt+Yukarı/Aşağı", "Satırı taşı" },
    { "Shift+Alt+Yukarı/Aşağı", "Satırı çoğalt" },
    { "Shift+Ok", "Ok tuşlarıyla seçim" },
    { "Ctrl+Q", "Blok seçim (eski Ctrl+V)" },
    { "Ctrl+Backspace", "Önceki kelimeyi sil (yazarken)" },
    { "Space fm", "Dosyayı formatla" },
    { "ys / cs / ds", "Etrafına parantez/tırnak ekle / değiştir / sil" },
  } },
  { "GEZİNME", {
    { "Ctrl+H/J/K/L", "Pencereler arası geç (sol/aşağı/yukarı/sağ)" },
    { "s + 2 harf", "Ekranda istediğin yere atla (flash)" },
    { "gg / G", "Dosyanın başı / sonu" },
    { "Ctrl+D / Ctrl+U", "Yarım sayfa aşağı / yukarı" },
    { "Ctrl+\\", "Terminal penceresini aç/kapat" },
  } },
  { "KOD ZEKASI (LSP)", {
    { "gd", "Tanıma git" },
    { "gr", "Referansları göster" },
    { "gi / gD", "Implementasyon / deklarasyon" },
    { "K", "Üzerindeki şeyin dokümanı" },
    { "F2  /  Space rn", "Yeniden adlandır (her yerde)" },
    { "Space ca", "Düzeltme önerileri (VSCode'daki ampul)" },
  } },
  { "TAMAMLAMA, SNIPPET, COPILOT", {
    { "Tab", "Copilot önerisini / menü seçimini kabul et, snippet'te ilerle" },
    { "Shift+Tab", "Snippet'te geri git" },
    { "Ctrl+n / Ctrl+p", "Menüde aşağı / yukarı" },
    { "Ctrl+Space", "Tamamlama menüsünü zorla aç" },
    { "Ctrl+e", "Menüyü kapat" },
    { "Alt+L", "Copilot önerisini kabul et" },
    { "Alt+K", "Copilot önerisinden bir kelime al" },
    { "Alt+] / Alt+[", "Sonraki / önceki Copilot önerisi" },
    { "Ctrl+]", "Copilot önerisini kapat" },
    { "h1 + Tab (html)", "Emmet: <h1></h1> yazar, imleci ortaya koyar" },
    { "HT / HT5 + Tab", "Kendi HTML snippet'lerin" },
  } },
  { "GIT VE HATALAR", {
    { "Space xx", "Tüm projedeki hatalar" },
    { "Space xd", "Bu dosyanın hataları" },
    { "Space gh / gH", "Dosya / repo geçmişi (diffview)" },
    { ":Git", "Git komutları (fugitive)" },
  } },
  { "HATA AYIKLAMA (DEBUG)", {
    { "F5", "Debug başlat / devam et" },
    { "F10 / F11 / F12", "Step over / into / out" },
    { "Space db", "Breakpoint koy/kaldır" },
    { "Space du", "Debug arayüzünü aç/kapat" },
  } },
  { "DİĞER", {
    { "F1  /  Space ?", "Bu liste" },
    { "Space (bekle)", "which-key: o tuştan sonra gelen kısayolları gösterir" },
    { "Space ls / lc", "Live server başlat / kapat (HTML önizleme)" },
    { "Space tt", "Açık / koyu tema değiştir" },
    { "Space aa / ac", "AI Actions / Chat (codecompanion, GLM anahtarı ister)" },
    { ":Lazy", "Plugin yöneticisi" },
  } },
}

-- Türkçe karakterli metinlerde de sütunlar hizalı kalsın diye ekrandaki genişliğe göre doldur
local function pad(text, width)
  return text .. string.rep(" ", math.max(1, width - vim.fn.strdisplaywidth(text)))
end

local function build()
  local lines = {}
  for _, sec in ipairs(sections) do
    table.insert(lines, "## " .. sec[1])
    table.insert(lines, "")
    for _, row in ipairs(sec[2]) do
      table.insert(lines, "  " .. pad(row[1], 28) .. row[2])
    end
    table.insert(lines, "")
  end
  return lines
end

local win

function M.toggle()
  if win and vim.api.nvim_win_is_valid(win) then
    vim.api.nvim_win_close(win, true)
    win = nil
    return
  end

  local buf = vim.api.nvim_create_buf(false, true)
  vim.api.nvim_buf_set_lines(buf, 0, -1, false, build())
  vim.bo[buf].modifiable = false
  vim.bo[buf].bufhidden = "wipe"
  vim.bo[buf].filetype = "markdown"

  local width = math.min(104, vim.o.columns - 4)
  local height = math.floor(vim.o.lines * 0.8)
  win = vim.api.nvim_open_win(buf, true, {
    relative = "editor",
    width = width,
    height = height,
    row = math.floor((vim.o.lines - height) / 2),
    col = math.floor((vim.o.columns - width) / 2),
    style = "minimal",
    border = "rounded",
    title = " Kısayollar  (q / Esc / F1: kapat) ",
    title_pos = "center",
  })
  vim.wo[win].cursorline = true

  for _, key in ipairs({ "q", "<Esc>", "<F1>" }) do
    vim.keymap.set("n", key, function() M.toggle() end, { buffer = buf, nowait = true })
  end
end

return M
