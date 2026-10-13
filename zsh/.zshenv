# Read by every zsh, including the non-interactive one `ssh host '<cmd>'`
# starts, which never reads .zshrc. Keep it to environment variables: it runs
# before every script and ssh command, so anything slow or noisy here is felt
# everywhere.

# Keep PATH entries unique: .zshrc and macOS's path_helper prepend some of
# these again in interactive shells.
typeset -U path

# Dirs that come before the system PATH. (N/) drops one that doesn't exist.
# .zshrc re-applies this list after a login shell's path_helper reorders.
user_path=("$HOME/.local/bin" $HOME/.cargo/bin(N/))
path=($user_path $path)
# A shell started without PATH in its environment doesn't export the one it
# builds, so child processes (path_helper included) would not see it.
export PATH

# Only a default: a terminal that sets its own locale (Ghostty) keeps it.
export LANG=${LANG:-en_US.UTF-8}
export EDITOR=nvim
