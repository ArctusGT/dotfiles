# Environment for every zsh, even `ssh host '<cmd>'`, which skips .zshrc.
# Keep it to variables: it runs before every script and ssh command.

# Keep PATH entries unique: .zshrc (interactive) and macOS's path_helper
# (login) prepend some of these again.
typeset -U path

# Dirs that come before the system PATH. ~/.local/bin always goes in; (N/)
# keeps Cargo's bin only where it exists. .zshrc re-applies this list after a
# login shell's path_helper reorders.
user_path=(~/.local/bin ~/.cargo/bin(N/))
path=($user_path $path)
# A shell started without PATH in its environment doesn't export the one it
# builds, so child processes (path_helper included) would not see it.
export PATH

# Only a default: a terminal that sets its own locale (Ghostty) keeps it.
export LANG=${LANG:-en_US.UTF-8}
export EDITOR=nvim

# fzf in TokyoNight Night, for every fzf: the shell's, sesh-pick and the
# Claude sidebar, which run under tmux and so get the environment of the zsh
# that started the server. One already running takes it with
#   tmux set-environment -g FZF_DEFAULT_OPTS "$FZF_DEFAULT_OPTS"
# Palette numbers, and hex for #283457, TokyoNight's selection blue, which
# the 16 lack. fzf keeps the terminal's own foreground and background, and
# fg+:-1 keeps it on the current line too. The gutter is palette 0, a shade
# under the background, as fzf draws it as a bar
export FZF_DEFAULT_OPTS="--color=gutter:0,fg+:-1,bg+:#283457\
,hl:6,hl+:6,pointer:5,marker:5,spinner:5,prompt:6,info:8,header:3\
,border:4,separator:8,scrollbar:8"
