# Dotfiles: Prerequisites + Setup

This repo uses **GNU Stow** to symlink configs into place.

## Prerequisites

### Required packages
- `git`
- `zsh`
- `stow` (GNU Stow)
- `tmux`
- `neovim` (`nvim`)
- `fzf` (required by `zsh-fzf-history-search`)
- `zoxide`

### oh-my-zsh
- **oh-my-zsh** framework

Third-party oh-my-zsh plugins (installed under
`${ZSH_CUSTOM:-~/.oh-my-zsh/custom}/plugins/`):
- `zsh-autosuggestions`  
  https://github.com/zsh-users/zsh-autosuggestions
- `zsh-syntax-highlighting`  
  https://github.com/zsh-users/zsh-syntax-highlighting
- `zsh-fzf-history-search`  
  https://github.com/joshskidmore/zsh-fzf-history-search

Theme (installed under `${ZSH_CUSTOM:-~/.oh-my-zsh/custom}/themes/`):
- `powerlevel10k`  
  https://github.com/romkatv/powerlevel10k

### tmux plugin manager
- **TPM** (Tmux Plugin Manager)  
  https://github.com/tmux-plugins/tpm  
  Install location: `~/.tmux/plugins/tpm`

### Optional (recommended)
- A Nerd Font enabled in your terminal for Powerlevel10k icons (e.g. MesloLGS NF)

---

## Using GNU Stow

### 1) Clone the dotfiles repo
```bash
git clone <YOUR_REPO_URL> ~/dotfiles
cd ~/dotfiles
```

### 2) Symlink configs into `$HOME`
From inside `~/dotfiles`, run stow for the packages you want:

```bash
stow zsh tmux nvim
```

This creates symlinks like:
- `~/.zshrc -> ~/dotfiles/zsh/.zshrc`
- `~/.tmux.conf -> ~/dotfiles/tmux/.tmux.conf`
- `~/.config/nvim -> ~/dotfiles/nvim/.config/nvim`

### 3) Preview changes (dry-run)
```bash
stow -n zsh tmux nvim
```

### 4) Undo (remove symlinks)
```bash
stow -D zsh tmux nvim
```

### 5) Restow (remove + re-link)
Useful after reorganising files:

```bash
stow -R zsh tmux nvim
```

---

## Post-install notes

### TPM: install tmux plugins
After TPM is installed and your `~/.tmux.conf` references it:

1. Start tmux:
   ```bash
   tmux
   ```
2. Install plugins:
   - Press `prefix` + `I` (capital i)  
     (`prefix` is usually `Ctrl-b` unless you changed it)

### Neovim plugins
Your dotfiles will sync, but plugin installation depends on your Neovim plugin
manager. Open `nvim` and run the appropriate sync/install command for your setup.
```bash
nvim
```
