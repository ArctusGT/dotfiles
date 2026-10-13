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
