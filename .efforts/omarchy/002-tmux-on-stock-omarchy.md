## Tmux on stock Omarchy

**Category:** enhancement
**Summary:** On an Omarchy host, tmux runs Omarchy's shipped config as it is, following Omarchy's updates to it, plus the sidebar's tmux file and its waiting marker; Omarchy's own bindings all stay
**Status:** ready-for-agent
**blocked-by:** [omarchy/001-sidebar-tmux-bindings-as-one-file]

**Current behavior:**
The `tmux` package installs Oh my tmux! as `~/.tmux.conf`, which tmux
loads in place of the XDG config Omarchy keeps, so stowing it on an Omarchy
host silently drops Omarchy's tmux config. The Omarchy workstation's XDG
config is today byte for byte Omarchy's shipped copy. Omarchy binds
`C-M-arrow` to focus panes, `C-M-S-arrow` to resize, `M-1`…`M-9` to
windows, and its `prefix q` reloads its XDG config; its `prefix` is
`C-Space`, as the Mac's is.

**Desired behavior:**
- An Omarchy host stows a package that loads Omarchy's shipped tmux config
  as it is, then the sidebar's tmux file; a change Omarchy ships to its
  config reaches the host with no edit here
- The waiting marker is added to Omarchy's `status-right` without removing
  what it shows
- Every Omarchy binding stays, including `C-M-S-arrow` resizing; the
  sidebar's edge-aware `C-M-arrow` behaves as Omarchy's inside a window
- `prefix q` reloads the whole of it
- Readme says which tmux package each kind of host stows, and that an
  Omarchy host never stows the Oh my tmux! one

**Key interfaces:**
- Omarchy's shipped tmux config and the XDG path Omarchy keeps the user's
  copy at
- The sidebar's tmux file and its marker format (001)

**Acceptance criteria:**
- [ ] On a throwaway tmux server loaded the way an Omarchy host loads it,
      `list-keys` differs from Omarchy's shipped config alone only in keys
      the sidebar's file binds, and the `C-M-S-arrow` keys still resize
- [ ] The status line shows Omarchy's own content and the marker for a
      window carrying `@claude_wait`
- [ ] `prefix q` on that server reloads without error and leaves
      `list-keys` unchanged
- [ ] On the Omarchy workstation, with agent-sidebar installed, the
      Maintainer sees Omarchy's tmux bindings and status line as before,
      and the marker after a turn ends in an unwatched window

**Out of scope:**
- The frame's keys, and whether `M-1`…`M-9` inside the frame belong to the
  sidebar or to Omarchy's windows: agent-sidebar's keybindings file, per
  installation
- Theming Omarchy's status line
- The Mac's tmux
