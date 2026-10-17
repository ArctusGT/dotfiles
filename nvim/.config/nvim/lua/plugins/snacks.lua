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
      -- lazygit takes its theme from lazygit's own config, as it does outside
      -- nvim, rather than from one snacks writes from nvim's highlights
      lazygit = {
        configure = false,
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
