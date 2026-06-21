#!/usr/bin/env bash
# Install ~/.local/bin/sudo-askpass + fish conf for GUI sudo prompts.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
ASKPASS="$HOME/.local/bin/sudo-askpass"
FISH_CONF="$HOME/.config/fish/conf.d/50-sudo-askpass.fish"

mkdir -p "$HOME/.local/bin" "$HOME/.config/fish/conf.d"

install -m 0755 "$ROOT/scripts/sudo-askpass-bin.sh" "$ASKPASS"
install -m 0644 "$ROOT/scripts/sudo-askpass.fish" "$FISH_CONF"

echo "Installed: $ASKPASS"
echo "Installed: $FISH_CONF"
echo ""
echo "New fish shells use GUI sudo automatically when DISPLAY or WAYLAND_DISPLAY is set."
echo "Test: sudo -A true"
echo "Agent/cmdline: SUDO_ASKPASS=$ASKPASS sudo -A apt install feh"