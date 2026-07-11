#!/usr/bin/env bash
set -euo pipefail

DOTFILES_DIR="${DOTFILES_DIR:-$HOME/dotfiles}"
ZSH_CUSTOM_DIR="${ZSH_CUSTOM:-$HOME/.oh-my-zsh/custom}"

have() {
  command -v "$1" >/dev/null 2>&1
}

os_id() {
  if [[ -r /etc/os-release ]]; then
    # shellcheck disable=SC1091
    . /etc/os-release
    echo "${ID:-unknown}"
  else
    echo "unknown"
  fi
}

sudo_cmd() {
  if have sudo; then
    echo "sudo"
  else
    echo ""
  fi
}

install_pkgs_debian() {
  local sudo
  sudo="$(sudo_cmd)"
  $sudo apt-get update -y
  $sudo apt-get install -y \
    ca-certificates \
    curl \
    git \
    stow \
    tmux \
    zsh \
    neovim \
    fzf \
    zoxide
}

install_pkgs_rocky() {
  local sudo
  sudo="$(sudo_cmd)"
  $sudo dnf -y install epel-release || true
  $sudo dnf -y install \
    ca-certificates \
    curl \
    git \
    stow \
    tmux \
    zsh \
    neovim \
    fzf \
    zoxide || true
}

clone_or_update() {
  local repo="$1"
  local dir="$2"

  if [[ -d "$dir/.git" ]]; then
    git -C "$dir" pull --ff-only
  elif [[ -d "$dir" ]]; then
    echo "ERROR: $dir exists but is not a git repo"
    exit 1
  else
    git clone --depth=1 "$repo" "$dir"
  fi
}

install_oh_my_zsh() {
  if [[ -d "$HOME/.oh-my-zsh" ]]; then
    return 0
  fi

  if ! have curl; then
    echo "ERROR: curl is required to install oh-my-zsh"
    exit 1
  fi

  RUNZSH=no CHSH=no KEEP_ZSHRC=yes \
    sh -c "$(curl -fsSL \
      https://raw.githubusercontent.com/ohmyzsh/ohmyzsh/master/tools/install.sh)"
}

install_zoxide_fallback() {
  if have zoxide; then
    return 0
  fi

  echo "zoxide not found via package manager; installing to ~/.local/bin"
  curl -fsSL \
    https://raw.githubusercontent.com/ajeetdsouza/zoxide/main/install.sh |
    bash

  if [[ ":$PATH:" != *":$HOME/.local/bin:"* ]]; then
    echo "NOTE: ensure ~/.local/bin is on PATH for zoxide"
  fi
}

install_fzf_fallback() {
  if have fzf; then
    return 0
  fi

  echo "fzf not found via package manager; installing to ~/.fzf"
  if [[ -d "$HOME/.fzf" ]]; then
    git -C "$HOME/.fzf" pull --ff-only
  else
    git clone --depth 1 https://github.com/junegunn/fzf.git "$HOME/.fzf"
  fi
  "$HOME/.fzf/install" --bin

  if [[ ":$PATH:" != *":$HOME/.fzf/bin:"* ]]; then
    echo "NOTE: ensure ~/.fzf/bin is on PATH for fzf"
  fi
}

main() {
  local id
  id="$(os_id)"

  case "$id" in
  debian | ubuntu)
    install_pkgs_debian
    ;;
  rocky | rhel | centos | fedora)
    install_pkgs_rocky
    ;;
  *)
    echo "Unsupported OS ID: $id"
    echo "Edit the script to add your distro."
    exit 1
    ;;
  esac

  install_oh_my_zsh

  mkdir -p "$ZSH_CUSTOM_DIR/plugins" "$ZSH_CUSTOM_DIR/themes"

  clone_or_update \
    https://github.com/zsh-users/zsh-autosuggestions \
    "$ZSH_CUSTOM_DIR/plugins/zsh-autosuggestions"

  clone_or_update \
    https://github.com/zsh-users/zsh-syntax-highlighting \
    "$ZSH_CUSTOM_DIR/plugins/zsh-syntax-highlighting"

  clone_or_update \
    https://github.com/joshskidmore/zsh-fzf-history-search \
    "$ZSH_CUSTOM_DIR/plugins/zsh-fzf-history-search"

  clone_or_update \
    https://github.com/romkatv/powerlevel10k \
    "$ZSH_CUSTOM_DIR/themes/powerlevel10k"

  install_zoxide_fallback
  install_fzf_fallback

  if [[ -d "$DOTFILES_DIR" ]] && have stow; then
    echo "Stowing dotfiles from $DOTFILES_DIR"
    (cd "$DOTFILES_DIR" && stow zsh tmux nvim) || true
  fi

  echo
  echo "Done."
  echo "Restart your shell: exec zsh"
  echo "If you want zsh as default: chsh -s \"$(command -v zsh)\""
}

main "$@"
