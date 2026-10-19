## Every modifier of ours on Alt: the arrows leave Cmd, and step through every entry

**Category:** enhancement
**Summary:** The pane and sidebar keys bound to `Cmd+arrow` and `Cmd+Shift+arrow`, the last of our bindings on Cmd, move to `Option+arrow` and `Option+Shift+arrow`, so nothing of ours sits on Super and Omarchy's defaults stay clear of it; and stepping from a window's edge visits every sidebar entry in list order, indented and background ones included, where today it skips to the next numbered one
**Status:** ready-for-agent
**blocked-by:**

**Current behavior:**
The `ghostty` package rebinds `Cmd+arrow` and `Cmd+Shift+arrow` to send
`Ctrl+Option(+Shift)+arrow`, since tmux has no `Cmd` modifier. The usual
tmux binds those: focus the pane that way, swap with it, and at a
window's left, top or bottom edge run `claude-sidebar edge`, which moves
into the sidebar or shows the next or previous entry; in an `ssh-tmux`
window the keys are sent on to the host, whose tmux binds them the same.
The frame's tmux binds the same keys in the sidebar to fzf's `Up`, `Down`
and `Enter`. Ghostty's own `Cmd+arrow` defaults, line start and end and
jump to prompt, are lost to this. Every other binding of ours is on
Alt already: the frame's `Alt+S`, `Alt+1..9` and `Alt+Shift+1..9`, the
sidebar's `Alt+P`, and `Option+drag` and `Option+right-click`.

The Maintainer chose Alt for all of them. The configs are headed for
Omarchy, where Super is the window manager's modifier, so a binding of
ours on Super would clash there; Alt is ours on every host.

`edge up` and `edge down` step by the numbered entries only: `step` reads
the rows, keeps the entries with a number, finds the one whose pane is
in the current window, and picks the one before or after it. An indented
entry, a second chat of a repo already numbered, and a background entry,
one with no pane, are skipped over, and from a window whose only entry
is indented the keys do nothing.

**Desired behavior:**
- Every binding that `Cmd+arrow` and `Cmd+Shift+arrow` reach today is
  reached by `Option+arrow` and `Option+Shift+arrow` instead, in the
  usual tmux, the keys it sends on to a host, the frame's tmux and the
  Readme's key tables; the `ghostty` package no longer rebinds the `Cmd`
  arrows, so Ghostty's defaults for them return
- After this, no binding of ours uses Super or Cmd, on the Mac, on a
  host, or in the frame
- `Option+Up` and `Option+Down` at a window's top or bottom edge show the
  entry before or after the window's own in the list's order, whatever
  kind it is: numbered, indented or background. A background entry is
  shown as a jump to it shows it today, by attaching it. Nothing past
  either end
- From a window showing an indented entry, the keys step from that entry
- The same on mgmt-1 through `ssh-tmux`, where the usual tmux config and
  the sidebar script are the same stowed files

**Key interfaces:**
- The `ghostty` package's `keybind` lines for the `super` arrows
- The usual tmux's root bindings for the `C-M` and `C-M-S` arrows,
  including the `send-keys` to a host, the frame config's, and the
  Readme's pane and sidebar key tables
- `claude-sidebar edge` and `step`, which read the rows' fields: the host,
  pane, relay, id and number of each entry, and the indent the renderer
  gives an entry under its repo's first
- `pick`, which `step` hands the chosen entry to, and which today takes
  an entry number; an entry with no number needs another handle, such
  as its position in the list, which fzf's `pos` already uses

**Acceptance criteria:**
- [ ] `Option+arrow` focuses and `Option+Shift+arrow` swaps panes as
      `Cmd+arrow` and `Cmd+Shift+arrow` did, on the Mac and in the
      mgmt-1 window; `Cmd+Left` and `Cmd+Right` in a shell outside tmux
      go to line start and end again
- [ ] In the sidebar, `Option+Up` and `Option+Down` move the cursor and
      `Option+Right` is Enter; `Option+Left` in a window's leftmost pane
      moves into the sidebar
- [ ] A search of the stowed configs and the Readme finds no binding of
      ours on `super`, `Cmd` or a `C-M` arrow
- [ ] On a throwaway sidebar with a numbered entry, an indented entry
      under it and a background entry, `edge down` from each entry's
      window in turn shows the next in list order, and `edge up` the
      previous, with nothing past the ends; by the end-to-end check
- [ ] The Maintainer steps through the list with `Option+Up` and
      `Option+Down` from a window of each kind, locally and on mgmt-1

**Out of scope:**
- New keys for anything the `Cmd` arrows did not do
- Changing the list's order or which entries are indented
- The bindings already on Alt
- Omarchy itself; this clears the way, it does not set anything up there

**For the Maintainer:**
- `Option+Left` and `Option+Right` reach tmux before any pane does, so a
  program's own use of them, a word back and forward in a shell, in
  Claude Code's prompt or in nvim, goes. The Maintainer accepted that:
  word jumping is never used, and `Cmd+Left` and `Cmd+Right` come back
  as line start and end.
