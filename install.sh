#!/bin/bash

set -e

INSTALL_DIR="/usr/local/bin"
CMVM_BIN_FILE="cmvm"
HOME_DIR="$HOME/.cmvm"
CACHE_DIR="$HOME_DIR/cache"
VERSIONS_DIR="$HOME_DIR/cache"
FILENAME="cmvm-macos"
RELEASE="latest"

parse_args() {
  while [[ $# -gt 0 ]]; do
    key="$1"
    case $key in
    -r | --release)
      RELEASE="$2"
      shift # past release argument
      shift # past release value
      ;;
    *)
      echo "Unrecognized argument $key"
      exit 1
      ;;
    esac
  done
}

download() {
  if [ "$RELEASE" = "latest" ]; then
    URL="https://github.com/iepsen/cmvm/releases/latest/download/$FILENAME.zip"
  else
    URL="https://github.com/iepsen/cmvm/releases/download/$RELEASE/$FILENAME.zip"
  fi

  DOWNLOAD_DIR=$(mktemp -d)

  echo "[cmvm] Downloading $URL..."

  mkdir -p "$HOME_DIR" &>/dev/null
  mkdir -p "$CACHE_DIR" &>/dev/null
  mkdir -p "$VERSIONS_DIR" &>/dev/null

  if ! curl --progress-bar --fail -L "$URL" -o "$DOWNLOAD_DIR/$FILENAME.zip"; then
    echo "[cmvm] Failed to download from $URL"
    exit 1
  fi

  unzip -q "$DOWNLOAD_DIR/$FILENAME.zip" -d "$DOWNLOAD_DIR"

  if [ -f "$DOWNLOAD_DIR/$CMVM_BIN_FILE" ]; then
    mv "$DOWNLOAD_DIR/$CMVM_BIN_FILE" "$INSTALL_DIR/$CMVM_BIN_FILE"
  else
    mv "$DOWNLOAD_DIR/$FILENAME/$CMVM_BIN_FILE" "$INSTALL_DIR/$CMVM_BIN_FILE"
  fi

  chmod u+x "$INSTALL_DIR/$CMVM_BIN_FILE"
}

setup_shell() {
  CURRENT_SHELL="$(basename "$SHELL")"

  if [ "$CURRENT_SHELL" = "zsh" ]; then
    CONF_FILE=${ZDOTDIR:-$HOME}/.zshrc
  elif [ "$CURRENT_SHELL" = "bash" ]; then
    if [ "$OS" = "Darwin" ]; then
      CONF_FILE=$HOME/.profile
    else
      CONF_FILE=$HOME/.bashrc
    fi
  else
    echo "[cmvm] Could not infer shell type. Please set up manually."
    exit 1
  fi

  echo "[cmvm] Settin up cmvm on $CONF_FILE"
  echo '# cmvm' >>$CONF_FILE
  echo 'export PATH='$HOME_DIR':$PATH' >>$CONF_FILE

  source $CONF_FILE

  echo "[cmvm] Setup complete."
}

parse_args "$@"
download
setup_shell