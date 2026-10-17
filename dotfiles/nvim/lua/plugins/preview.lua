return {
  {
    "brianhuster/live-preview.nvim",
    ft = { "html", "markdown" },
    cmd = "LivePreview",
    keys = {
      { "<leader>ls", "<cmd>LivePreview start<cr>", desc = "Live server baslat" },
      { "<leader>lc", "<cmd>LivePreview close<cr>", desc = "Live server kapat" },
    },
    config = function()
      -- Eklenti uzantiyi buyuk/kucuk harfe duyarli kontrol ediyor (.HTML taninmiyor).
      -- Dosya adini kucuk harfe cevirip eski fonksiyona veriyoruz.
      local utils = require("livepreview.utils")
      local original = utils.supported_filetype
      utils.supported_filetype = function(file_name)
        if type(file_name) == "string" then
          file_name = file_name:lower()
        end
        return original(file_name)
      end

      for _, mod in ipairs({
        "livepreview.server.utils.content_type",
        "livepreview.server.handler",
        "livepreview.server",
        "livepreview",
      }) do
        package.loaded[mod] = nil
      end
    end,
  },
}
