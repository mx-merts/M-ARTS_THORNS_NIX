-- Tum ayarlar burada (VSCode settings.json karsiligi)
local opt = vim.opt

opt.number = true              -- satir numaralari
opt.relativenumber = true      -- goreceli numaralar (5j = 5 satir asagi)
opt.mouse = "a"                -- mouse destegi
opt.undofile = true            -- kapatip acinca geri almaya devam
opt.ignorecase = true          -- aramada buyuk/kucuk onemsiz...
opt.smartcase = true           -- ...ama buyuk harf yazarsan hassas olur
opt.signcolumn = "yes"         -- git isaretleri icin sabit sutun
opt.updatetime = 250
opt.splitright = true          -- dikey bolme sag tarafada acilir
opt.splitbelow = true          -- yatay bolme altta acilir
opt.scrolloff = 8              -- imlece 8 satir tampon

-- girinti (VSCode'daki 2 bosluk tarzi)
opt.tabstop = 2
opt.shiftwidth = 2
opt.expandtab = true
opt.smartindent = true

-- gorsel
opt.termguicolors = true
opt.clipboard = "unnamedplus"  -- sistem panosu entegrasyonu


-- Yumusak kenarlar (nvim 0.11+): hover, diagnostic gibi tum yuzen pencereler
opt.winborder = "rounded"

-- Imlec: normalde blok, yazarken ince cizgi (kitty'deki gibi), yanip soner
opt.guicursor = "a:ver25-Cursor,a:blinkon700-blinkoff400"
opt.cursorline = true          -- bulundugun satir vurgulu


-- Shift+ok tuslariyla secim (VSCode gibi)
opt.keymodel = "startsel,stopsel"
opt.selectmode = "key"   -- Shift+ok ile secince Select moduna gir (yazinca degistir)
