return {
  "neovim/nvim-lspconfig",
  opts = function(_, opts)
    opts.servers = opts.servers or {}
    opts.servers.clangd = require("esp32").lsp_config()
    return opts
  end,
}
