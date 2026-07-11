# Dotfiles (zsh, tmux, nvim) — Prerequisites + Stow

This repo is meant to be installed via symlinks using **GNU Stow**.

## Prerequisites

### Core
- `git`
- `stow` (GNU Stow)
- `zsh`

### zsh framework / theme / plugins
- **oh-my-zsh**

Install locations used by oh-my-zsh:
- Plugins: `${ZSH_CUSTOM:-~/.oh-my-zsh/custom}/plugins/<plugin-name>/`
- Themes: `${ZSH_CUSTOM:-~/.oh-my-zsh/custom}/themes/<theme-name>/`

Required third-party plugins:
- `zsh-autosuggestions`  
  https://github.com/zsh-users/zsh-autosuggestions
- `zsh-syntax-highlighting`  
  https://github.com/zsh-users/zsh-syntax-highlighting
- `zsh-fzf-history-search`  
  https://github.com/joshskidmore/zsh-fzf-history-search

Theme:
- `powerlevel10k`  
  https://github.com/romkatv/powerlevel10k

CLI tools used by the shell config:
- `fzf` (required by `zsh-fzf-history-search`)
- `zoxide`

Optional (recommended):
- A Nerd Font for the terminal (for Powerlevel10k icons), e.g. **MesloLGS NF**.

---

## Using GNU Stow

### 1) Clone the repo
```bash
git clone <YOUR_DOTFILES_REPO_URL> ~/dotfiles
cd ~/dotfiles
```

### 2) Inspect the layout (optional)
```bash
tree -a -L 3
```

### 3) Symlink configs into `$HOME`
Run stow from inside `~/dotfiles`:

```bash
stow zsh tmux nvim
```

This creates symlinks such as:
- `~/.zshrc -> ~/dotfiles/zsh/.zshrc`
- `~/.tmux.conf -> ~/dotfiles/tmux/.tmux.conf`
- `~/.config/nvim -> ~/dotfiles/nvim/.config/nvim`

### 4) Dry run (recommended if you already have configs)
```bash
stow -n zsh tmux nvim
```

### 5) Undo (remove symlinks)
```bash
stow -D zsh tmux nvim
```

### 6) Restow (after reorganising files)
```bash
stow -R zsh tmux nvim
```

---

## Notes / Gotchas

- If stow reports conflicts, you likely already have files at the target paths.
  Back them up and remove them (or move them into this repo) before re-running
  `stow`.
- This repo does not automatically install zsh plugins/themes; those must be
  installed separately (see prerequisites above).
