## Sidebar tmux bindings as one file

**Category:** enhancement
**Summary:** The sidebar's bindings, hooks and settings in the main tmux move out of the Oh my tmux! local config into one file that any tmux config in this repo loads with one line, so the Mac and Omarchy hosts share them
**Status:** ready-for-human
**blocked-by:**

**Current behavior:**
About sixty lines of the Oh my tmux! local config belong to the sidebar,
mixed with the theme's variables, so they load only under Oh my tmux!:
- hooks that kick the sidebar when a client changes session or window, that
  unset `@claude_wait`, and that mark a bell from an `ssh-tmux` window
- `prefix a`, choosing among waiting windows; `detach-on-destroy off`; vi
  copy keys and extended keys under the frame
- `C-Left`/`C-Right` and the pane keys passing through to an `ssh-tmux`
  window
- edge-aware `C-M-arrow` focus, crossing into the sidebar at an edge
- `C-M-S-arrow` swapping panes
- `M-S-1`…`M-S-9` pinning the shown repo
- Option+drag swapping panes, and a status click jumping to a waiting window
- the waiting marker, `● session:window`, inside the theme's `status_right`

**Desired behavior:**
- One file holds all of the above but the `C-M-S-arrow` swap and the
  marker's place in a status line, in a stow package that both the Mac and
  Omarchy hosts stow and that holds nothing overriding Omarchy's own tmux
  config
- The Oh my tmux! local config loads it with one `source-file` line and
  keeps the `C-M-S-arrow` swap itself, so the Mac's tmux behaves exactly as
  before
- The file defines the waiting marker as one format a config adds to its
  own status line; the marker stays clickable wherever it is placed
- Loading it again, as a config reload does, leaves the bindings and hooks
  as one load left them

**Key interfaces:**
- The `source-file` line, and the stow package that carries the file
- The marker's format, under a name a status line references
- `@claude_wait`, `@ssh_tmux` and the `claude-sidebar` subcommands the
  bindings run (`kick`, `edge`, `pin-shown`), which agent-sidebar installs

**Acceptance criteria:**
- [x] On a throwaway tmux server (its own `-L` socket and `TMPDIR`), the
      Oh my tmux! config with the file loaded gives the same `list-keys`
      and `show-hooks -g` as the config before the change
- [x] Sourcing the file twice leaves `list-keys` and `show-hooks -g` as
      sourcing it once
- [x] A status line holding the marker's format shows `● session:window`
      for a window carrying `@claude_wait`, and clicking it switches there
- [ ] On the Mac, the Maintainer sees the sidebar's keys, the waiting
      marker and the status click behave as before

**Out of scope:**
- Omarchy's tmux config (002)
- The frame's own keys, which agent-sidebar reads from a keybindings file
- Moving the Mac's tmux to Omarchy's conventions

## Comments

- "holds nothing overriding Omarchy's own tmux config" traces to no
  decision the file could meet literally: it rebinds Omarchy's
  `C-M-arrow` focus keys (which 002 accepts; they don't wrap at a
  window's edge as Omarchy's do) and, inside the frame only, raises
  Omarchy's `extended-keys on` to `always`. What holds is that the
  package adds nothing tmux reads on its own and nothing under
  `~/.config/tmux/`. Proposed wording: "adds no file tmux loads on its
  own".
- `tmux/.tmux-frame.conf`, which `tmux-frame` loads, is in the `tmux`
  package that an Omarchy host never stows (002), so the frame has no
  config there. It belongs with the sidebar, in `tmux-sidebar` or
  agent-sidebar: 002's call, or agent-sidebar's?
- Omarchy's `status-right-length 50` will cut the marker short with
  more than one waiting window beside its own content; 002's status
  check is worth making with two.
- The status click runs `run-shell 'tmux switch-client ...'`, a `tmux`
  found by the server's PATH (EL10's broken one on mgmt-1 if that PATH
  lacks `~/.local/bin`). `run-shell -C`, as the Option+drag release
  uses, would avoid the lookup but changes `list-keys`, so it was left.
- Readme's sidebar section calls the sidebar's magenta `●` "the dot the
  status bar draws"; the status bar's marker is yellow (colour 3).
  Predates this slice.
- Every host that stows `tmux` (the Mac, mgmt-1) needs
  `stow tmux-sidebar` before its next tmux start or reload.
