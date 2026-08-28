-- Options are automatically loaded before lazy.nvim startup
-- Default options that are always set: https://github.com/LazyVim/LazyVim/blob/main/lua/lazyvim/config/options.lua
-- Add any additional options here

vim.opt.guicursor = ""
vim.opt.nu = true
vim.g.snacks_animate = false

-- Only force OSC 52 on a remote host. Locally the OS clipboard tools work and
-- nvim picks them up on its own. Check SSH_CONNECTION as well as SSH_TTY:
-- tmux's default update-environment propagates the former into new panes but
-- not the latter, so an SSH_TTY-only test fails inside tmux over SSH.
if vim.env.SSH_TTY or vim.env.SSH_CONNECTION then
  vim.g.clipboard = {
    name = "OSC 52",
    copy = {
      ["+"] = require("vim.ui.clipboard.osc52").copy("+"),
      ["*"] = require("vim.ui.clipboard.osc52").copy("*"),
    },
    -- OSC 52 paste asks the terminal to send the clipboard back. Terminals
    -- generally refuse, so the query blocks until it times out. Read the
    -- local register instead; use the terminal's own paste for outside text.
    paste = {
      ["+"] = function()
        return { vim.fn.split(vim.fn.getreg('"'), "\n"), vim.fn.getregtype('"') }
      end,
      ["*"] = function()
        return { vim.fn.split(vim.fn.getreg('"'), "\n"), vim.fn.getregtype('"') }
      end,
    },
  }
end

vim.opt.clipboard = "unnamedplus"
