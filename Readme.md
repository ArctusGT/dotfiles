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
- `sesh` + `fd` (tmux session picker: `brew install sesh fd`)
- `workmux` (worktree + tmux window per task, macOS only for now: `brew install raine/workmux/workmux`)
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
stow zsh tmux nvim bin sesh claude workmux
```

This creates symlinks like:
- `~/.zshrc -> ~/dotfiles/zsh/.zshrc`
- `~/.tmux.conf -> ~/dotfiles/tmux/.tmux.conf`
- `~/.config/nvim -> ~/dotfiles/nvim/.config/nvim`
- `~/.local/bin/espidf-index -> ~/dotfiles/bin/.local/bin/espidf-index`

### 3) Preview changes (dry-run)
```bash
stow -n zsh tmux nvim bin
```

### 4) Undo (remove symlinks)
```bash
stow -D zsh tmux nvim bin
```

### 5) Restow (remove + re-link)
Useful after reorganising files:

```bash
stow -R zsh tmux nvim bin
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

---

## `espidf-index`: reading ESP-IDF firmware in Neovim

For anyone editing an ESP-IDF project that is **built on another machine**, where
clangd has no compile database and reports every ESP-IDF header as missing.

Run this on the workstation, once per project, and again whenever the ESP-IDF
version or `sdkconfig` changes:

```bash
espidf-index --host sysadmin@BUILD-HOST --remote-dir '$HOME/firmware/device' ~/myrepo/firmware/device
```

Expect it to print the entry count, the ESP-IDF tag, what it mirrored, and
`registered <name>`. It takes well under a minute; a first run for a new
ESP-IDF version copies about 150 MB of headers. Then restart the language
server:

```
:LspRestart
```

`--host` and `--key` default to `$ESPIDF_INDEX_HOST` / `$ESPIDF_INDEX_KEY`,
falling back to `$FW_HOST` / `$FW_KEY`. `--remote-dir` defaults to
`~/firmware/<project name>` on the build Host, and needs single quotes if it
contains `$HOME`. To check what Neovim ended up with, `:ESPInfo`.

### What it does, and where things land

It copies the build Host's `compile_commands.json`, repoints every path in it at
local copies of the headers, drops the handful of GCC flags upstream clang
rejects, and registers the result for that project alone.

| what | where |
|---|---|
| ESP-IDF headers, shared per version | `~/.cache/espidf-index/idf/<tag>/` |
| the rewritten database | `~/.cache/espidf-index/projects/<name>/` |
| the per-project registration | `~/.config/clangd/config.yaml` |

Nothing is written into the ESP-IDF project, and nothing changes on the build
Host.

`~/.config/clangd/config.yaml` is generated and holds machine-local absolute
paths. Note that `~/.config` is itself a symlink to `nvim/.config`, so that file
physically lands **inside this repository**; `.gitignore` is what keeps it out
of git and off other machines.

### Traps

- **It is a copy, and nothing tells it when it is stale.** Re-run after an
  ESP-IDF bump, a `sdkconfig` change, or a new source file. `--refresh-idf`
  forces the header mirror to be re-copied.
- **A project must have been built on the build Host at least once**, or there
  is no database to read.
- **Other C projects are unaffected.** Each registration is scoped by path, so
  a normal x86 project keeps finding its own database. Do not be tempted to add
  a global `CompileFlags: Remove: [-m*, -f*]` to fix a flag complaint — it
  silently strips `-fstack-protector-strong`, `-fno-omit-frame-pointer` and
  `-march` from unrelated projects, with no error shown.
- **Diagnostics are close to the build, not identical to it.** Upstream clangd
  has no Xtensa backend, so it parses as the host architecture using the
  cross-compiler's own headers and target macros. Type layout is right and
  navigation is right; anything depending on real Xtensa codegen is not.
  Espressif's `esp-clang` is the fix if that ever matters.
