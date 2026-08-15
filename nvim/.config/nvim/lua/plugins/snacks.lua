return {
  {
    "folke/snacks.nvim",
    ---@type snacks.Config
    opts = {
      image = {
        force = true,
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
