# Read by every zsh, including the non-interactive one `ssh host '<cmd>'`
# starts, which never reads .zshrc. Keep it to environment variables: it runs
# before every script and ssh command, so anything slow or noisy here is felt
# everywhere.

# Keep PATH entries unique: .zshrc and macOS's path_helper prepend some of
# these again in interactive shells.
typeset -U path

[[ -f $HOME/.cargo/env ]] && . "$HOME/.cargo/env"
path=("$HOME/.local/bin" $path)

# Only a default: a terminal that sets its own locale (Ghostty) keeps it.
export LANG=${LANG:-en_US.UTF-8}
export EDITOR=nvim
