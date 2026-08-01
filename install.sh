#!/bin/sh
# Dale installer — https://github.com/lubluniky/dale
#
#   curl -fsSL https://borkiss.net/dale-install.sh | sh
#
# Downloads the latest `dale` release binary for this platform and installs it
# into ~/.local/bin (override with DALE_INSTALL_DIR). Then run `dale` to pick
# and inject the Dale skills into Codex.

set -eu

REPO="lubluniky/dale"
INSTALL_DIR="${DALE_INSTALL_DIR:-$HOME/.local/bin}"
GREEN="$(printf '\033[38;2;81;200;120m')"
BOLD="$(printf '\033[1m')"
DIM="$(printf '\033[2m')"
RESET="$(printf '\033[0m')"

say() { printf '%s\n' "$1"; }
fail() {
    printf '%serror:%s %s\n' "$BOLD" "$RESET" "$1" >&2
    exit 1
}

say "${GREEN}${BOLD}● Dale${RESET} — Codex skill installer"

os="$(uname -s)"
arch="$(uname -m)"
case "$os" in
Darwin) os_slug="macos" ;;
Linux) os_slug="linux" ;;
*) fail "unsupported OS: $os (macOS and Linux only for now)" ;;
esac
case "$arch" in
arm64 | aarch64) arch_slug="arm64" ;;
x86_64 | amd64) arch_slug="x64" ;;
*) fail "unsupported architecture: $arch" ;;
esac

asset="dale-${os_slug}-${arch_slug}.tar.gz"
url="https://github.com/${REPO}/releases/latest/download/${asset}"

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT INT TERM

say "${DIM}downloading ${asset} ...${RESET}"
if ! curl -fsSL "$url" -o "$tmp/$asset"; then
    fail "download failed: $url"
fi

tar -xzf "$tmp/$asset" -C "$tmp"
[ -f "$tmp/dale" ] || fail "archive did not contain the dale binary"

mkdir -p "$INSTALL_DIR"
install -m 755 "$tmp/dale" "$INSTALL_DIR/dale"

say "installed ${BOLD}${INSTALL_DIR}/dale${RESET}"

case ":$PATH:" in
*":$INSTALL_DIR:"*) ;;
*)
    say ""
    say "${BOLD}note:${RESET} $INSTALL_DIR is not in your PATH. Add this to your shell profile:"
    say "  export PATH=\"$INSTALL_DIR:\$PATH\""
    ;;
esac

say ""
say "run ${GREEN}${BOLD}dale${RESET} to pick and install the skills into Codex."
