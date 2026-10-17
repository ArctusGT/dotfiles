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
- `jq` (`claude-sessions` and the Claude sessions sidebar), plus `fzf` 0.74 or
  later, `curl` and `perl` for the sidebar
- `tectonic` (`cargo install tectonic`)
- `mmdc` (`sudo npm install -g @mermaid-js/mermaid-cli`)
- `tree-sitter-cli` (`cargo install --locked tree-sitter-cli`)
`)

### RHEL/RockyLinux/Fedora
```sh
sudo dnf config-manager --set-enabled crb
sudo dnf install git curl zsh tmux fzf stow ripgrep fd-find ghostscript cargo npm libpng-devel openssl-devel freetype-devel luarocks ImageMagick
```

On EL10 (Enterprise Linux 10: RHEL and Rocky Linux 10), dnf's tmux is a git
snapshot (`tmux -V` says `next-3.4`) that aborts the whole server on any
`capture-pane`, which the theme's `F` binding uses. Build a
tmux release into `~/.local/opt/tmux-<version>` and link it as
`~/.local/bin/tmux`, which `zsh/.zshenv` puts first on PATH. Leave the dnf
package installed; it is only a fallback.

```sh
sudo dnf install libevent-devel ncurses-devel bison
v=3.8; mkdir -p ~/.local/src && cd ~/.local/src
curl -fLO https://github.com/tmux/tmux/releases/download/$v/tmux-$v.tar.gz
tar -xzf tmux-$v.tar.gz && cd tmux-$v
./configure --prefix=$HOME/.local/opt/tmux-$v && make -j"$(nproc)" && make install
ln -sfn ~/.local/opt/tmux-$v/bin/tmux ~/.local/bin/tmux
```

A running server keeps the old binary until it is ended (`/usr/bin/tmux
kill-server`), which closes every session on the host.

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
stow zsh tmux nvim bin sesh claude
```

On the Mac, also `stow ghostty` (Ghostty's keys; hosts like mgmt-1 do not
need it).

This creates symlinks like:
- `~/.zshrc -> ~/dotfiles/zsh/.zshrc`
- `~/.tmux.conf -> ~/dotfiles/tmux/.tmux.conf`
- `~/.config/nvim -> ~/dotfiles/nvim/.config/nvim`
- `~/.local/bin/espidf-index -> ~/dotfiles/bin/.local/bin/espidf-index`

`zsh` also links `~/.zshenv`, which every zsh reads, including the one
`ssh host '<cmd>'` runs. Stow refuses to replace a real file, so on a host
that already has a `~/.zshenv`, fold anything it needs into
`zsh/.zshenv` and remove it before stowing.

Keep one sesh entry per remote host (see `sesh/.config/sesh/sesh.toml`). A
second entry for the same host adds a second client to its tmux, and the
sidebar's jumps can then move the wrong one. On the host, a new repo session
is `tmux new -d -s <repo> -c ~/<repo>`, then `prefix prefix s` to switch to
it.

While a host's sesh entry (`mgmt-1`) is open, the Mac hides its own bar in
that Mac session, so the host's bar, with the host's windows, is the only one
on screen.
`ctrl+left` and `ctrl+right` there move between the host's windows; the
host's other keys take the prefix twice (`C-Space C-Space`).

### Panes

| key | does |
|---|---|
| `Cmd+arrow` | focus the pane that way; at the window's right edge nothing, at its other edges see the sidebar's keys below |
| `Cmd+Shift+arrow` | swap with the pane that way, focus staying on the moved pane |
| `Option+drag` from one pane to another | swap the two, in place of tmux's own move of a floating pane |

In an `ssh-tmux` window such as `mgmt-1`, the `Cmd` keys and `Option+drag`
act on the host's panes. They come from the `ghostty` package, which gives up Ghostty's own
`Cmd+arrow` defaults (line start and end, jump to prompt): tmux has no `Cmd`
modifier, so Ghostty sends them as `Ctrl+Option(+Shift)+arrow`, and pressing
those directly works too.

### Claude sessions sidebar

Run `tmux-frame` in place of `tmux`. It opens a frame: a tmux of its own
with no bar, the sidebar on the left and the usual tmux on the right, bar and
windows as always. The frame binds no prefix, so every tmux key reaches the
right pane, and `prefix f` there switches only that pane. Detaching the right
pane closes the frame.

The sidebar lists every Claude session on the Mac and on each host a sesh
entry reaches through `ssh-tmux`, under one heading per host. Each entry shows
Clawd, Claude Code's mascot, beside its number, repo, name, then state, time in
that state and running subagents (`+N`). Clawd is orange while the session
works, yellow while it waits on you, and grey while it is idle. The entry ends
in a line across the sidebar showing its context use: filled
in the tmux battery bar's colours up to the share used, with the percentage at
its end, and all grey before the session reports any. At or past
`CLAUDE_CTX_WARN` percent (default 50) the filled part and percentage turn
red, the cue to hand off and clear. A host that does not answer shows
`unreachable` under its heading. The list comes from `claude-sessions` on each
host, rebuilt every second, and at once when a tmux client changes session or
window, over one shared ssh connection per host.

| key | does |
|---|---|
| click, or `Enter` in the sidebar | show that session in the right pane |
| `M-1` … `M-9` | the same for the entry numbered 1 to 9 |
| `M-s` | move into the sidebar, and back out |
| `Cmd+Left` in a window's leftmost pane | move into the sidebar |
| `Cmd+Up`, `Cmd+Down` in the sidebar | move its cursor |
| `Cmd+Right` in the sidebar | as `Enter` |
| `Cmd+Down` in a window's lowest pane, `Cmd+Up` in its top one | show the next or previous numbered entry, across hosts; nothing past either end, or from a pane that is no numbered entry |

In the Mac's `mgmt-1` window the edges are those of the host's pane: the Mac
asks the host over the sidebar's ssh connection, so in the frame
`Cmd+Left`, `Cmd+Up` and `Cmd+Down` there each take a round trip to the host.
Outside the frame the edges do nothing.

Only sessions in a tmux pane are numbered. A background session, one a `claude
daemon` runs (`--bg`, or sent to the background from Claude Code's session
list), is dimmed below them as `background`. Selecting it opens it with
`claude attach` in a new window on its own host, in the tmux session on show
there, and the entry is numbered like the rest while that window is open.
Selecting it again goes back to the same window. Closing the window, or
`ctrl+z` in it, leaves the session running and the entry dimmed again. Claude
desktop app sessions are left out; the app's own sidebar lists them. A remote session shows through the
Mac's session for that host, which `prefix f` opens; the sidebar then moves
that host's one client to the session's pane.

`M-` is Option, which Ghostty sends as Alt only with `macos-option-as-alt =
true` in its config.

#### Mirror float

A floating pane of the frame stays on screen over the right pane whatever
window, session or host that shows, so it can hold a live, read-only copy of
another pane: a Claude session's instructions, say, while acting in the window
they are about.

1. `Ctrl+drag` over the right pane opens a float (tmux's own binding); drag
   its border to move it, and the corner to resize it.
2. `Option+drag` a pane, on the Mac or in the `mgmt-1` window, and release it
   over the float. The float now shows that pane, cut to the float's size,
   bottom lines first. Whatever the float was running ends. Drop another pane
   on it to switch.
3. Keys typed in the float go nowhere. Close it with its border's close
   control or `kill-pane`; the mirrored pane carries on. The float also
   closes when the mirrored pane does.

`tmux-mirror` draws it, from a `capture-pane` of the source every 0.2s, over
the sidebar's ssh connection for a host.

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
paths. Where `~/.config/clangd` is stowed from `nvim/.config/clangd`, as on
mgmt-1, that file physically lands **inside this repository**; `.gitignore` is
what keeps it out of git and off other machines.

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
