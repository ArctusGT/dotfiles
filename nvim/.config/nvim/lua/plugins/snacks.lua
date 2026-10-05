return {
  {
    "folke/snacks.nvim",
    ---@type snacks.Config
    opts = {
      -- snacks.image probes the terminal with XTVERSION (ESC [ > q) on the first
      -- markdown buffer; a late reply leaks into the buffer as "ostty 1.3.1".
      image = {
        enabled = false,
      },
      picker = {
        hidden = true,
        sources = {
          files = {
            hidden = true, -- Show hidden/dotfiles
          },
        },
      },
    },
  },
}
