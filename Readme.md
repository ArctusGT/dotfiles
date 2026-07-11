# Dotfiles Prerequisites

Install these on each destination machine before (or right after) applying the
dotfiles, so `zsh` starts cleanly and your prompt/plugins work.

## Core
- `zsh`
- `git`

## oh-my-zsh
- **oh-my-zsh** framework

## oh-my-zsh plugins (third-party)
Install into:

- `${ZSH_CUSTOM:-~/.oh-my-zsh/custom}/plugins/<plugin-name>/`

Required plugins:
- `zsh-autosuggestions` (https://github.com/zsh-users/zsh-autosuggestions)
- `zsh-syntax-highlighting` (https://github.com/zsh-users/zsh-syntax-highlighting)
- `zsh-fzf-history-search` (https://github.com/joshskidmore/zsh-fzf-history-search)

## Theme
Install into:

- `${ZSH_CUSTOM:-~/.oh-my-zsh/custom}/themes/powerlevel10k/`

Required theme:
- `powerlevel10k` (https://github.com/romkatv/powerlevel10k)

## CLI tools
- `fzf` (required by `zsh-fzf-history-search`)
- `zoxide`

## Optional (recommended)
- A Nerd Font installed and selected in your terminal for Powerlevel10k icons
  (e.g. MesloLGS NF).
