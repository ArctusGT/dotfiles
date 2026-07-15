# Dotfiles: Prerequisites + Setup

This repo uses **GNU Stow** to symlink configs into place.

### Required packages
- `git`
- `zsh`
- `stow` (GNU Stow)
- `tmux`
- `neovim` (`nvim`)
- `fzf` (required by `zsh-fzf-history-search`)
- `zoxide` (`cargo install zoxide --locked`)
- `lazygit`
`sudo dnf copr enable dejan/lazygit
sudo dnf install lazygit`
- `tectonic` (`cargo install tectonic`)
- `mmdc` (`sudo npm install -g @mermaid-js/mermaid-cli`)
- `tree-sitter-cli` (`cargo install --locked tree-sitter-cli`)
`)

### RHEL/RockyLinux/Fedora
```sh
sudo dnf config-manager --set-enabled crb
sudo dnf install git curl zsh tmux fzf stow ripgrep fd-find ghostscript cargo npm libpng-devel openssl-devel freetype-devel luarocks ImageMagick
```

### neovim
```sh
sudo dnf install ninja-build libtool autoconf automake cmake gcc gcc-c++ make pkgconfig unzip patch gettext
git clone https://github.com/neovim/neovim
cd neovim
git checkout stable
make CMAKE_BUILD_TYPE=RelWithDebInfo
sudo make install
```

### oh-my-zsh
- **oh-my-zsh** framework
Third-party oh-my-zsh plugins (installed under `${ZSH_CUSTOM:-~/.oh-my-zsh/custom}/plugins/`):
- `git clone https://github.com/zsh-users/zsh-autosuggestions ${ZSH_CUSTOM:-~/.oh-my-zsh/custom}/plugins/zsh-autosuggestions`
- `git clone https://github.com/zsh-users/zsh-syntax-highlighting.git ${ZSH_CUSTOM:-~/.oh-my-zsh/custom}/plugins/zsh-syntax-highlighting`  
- `git clone https://github.com/joshskidmore/zsh-fzf-history-search ${ZSH_CUSTOM:=~/.oh-my-zsh/custom}/plugins/zsh-fzf-history-search`  

Theme (installed under `${ZSH_CUSTOM:-~/.oh-my-zsh/custom}/themes/`):
- `git clone --depth=1 https://github.com/romkatv/powerlevel10k.git "${ZSH_CUSTOM:-$HOME/.oh-my-zsh/custom}/themes/powerlevel10k"`  

### tmux plugin manager
- **TPM** (Tmux Plugin Manager)  
  `git clone https://github.com/tmux-plugins/tpm ~/.tmux/plugins/tpm`

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
