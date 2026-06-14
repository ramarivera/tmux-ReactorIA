#!/usr/bin/env bash
set -euo pipefail

PLUGIN_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CRATE_NAME="tmux-reactoria"
BIN_NAME="tmux-reactoria"
STATE_DIR="${XDG_STATE_HOME:-"$HOME/.local/state"}/tmux-reactoria"
STATE_FILE="$STATE_DIR/install-state.env"
LOCK_DIR="$STATE_DIR/install.lock"
LOG_FILE="$STATE_DIR/install.log"

mkdir -p "$STATE_DIR"

tmux_msg() {
  if command -v tmux >/dev/null 2>&1 && [ -n "${TMUX:-}" ]; then
    tmux display-message "tmux-ReactorIA: $*" 2>/dev/null || true
  fi
  printf 'tmux-ReactorIA: %s\n' "$*" >>"$LOG_FILE"
}

version_from_manifest() {
  awk -F '"' '/^version[[:space:]]*=/ { print $2; exit }' "$PLUGIN_DIR/Cargo.toml"
}

installed_version() {
  if command -v "$BIN_NAME" >/dev/null 2>&1; then
    "$BIN_NAME" --version 2>/dev/null | awk '{ print $2; exit }'
  fi
}

write_state() {
  {
    printf 'crate=%q\n' "$CRATE_NAME"
    printf 'version=%q\n' "$1"
    printf 'installed_at=%q\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
    printf 'source=%q\n' "$2"
    printf 'binary=%q\n' "$(command -v "$BIN_NAME" 2>/dev/null || true)"
  } >"$STATE_FILE"
}

main() {
  : >"$LOG_FILE"

  expected_version="$(version_from_manifest)"
  if [ -z "$expected_version" ]; then
    tmux_msg "could not read crate version from Cargo.toml"
    exit 1
  fi

  current_version="$(installed_version || true)"
  if [ "$current_version" = "$expected_version" ]; then
    tmux_msg "binary already installed ($BIN_NAME $current_version)"
    write_state "$current_version" "existing"
    exit 0
  fi

  if ! mkdir "$LOCK_DIR" 2>/dev/null; then
    tmux_msg "install already running"
    exit 0
  fi
  trap 'rmdir "$LOCK_DIR" 2>/dev/null || true' EXIT

  if ! command -v cargo >/dev/null 2>&1; then
    tmux_msg "cargo not found; install Rust/Cargo or install $BIN_NAME manually"
    exit 1
  fi

  tmux_msg "installing $CRATE_NAME $expected_version via cargo"
  if cargo install --locked "$CRATE_NAME" --version "$expected_version" >>"$LOG_FILE" 2>&1; then
    installed="$(installed_version || true)"
    tmux_msg "installed $BIN_NAME ${installed:-unknown}"
    write_state "${installed:-$expected_version}" "cargo"
  else
    tmux_msg "install failed; see $LOG_FILE"
    exit 1
  fi
}

main "$@"
