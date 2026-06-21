#!/bin/sh
# GUI password helper for `sudo -A`. Writes password to stdout; empty on cancel.

prompt="${SUDO_ASKPASS_PROMPT:-Administrator authentication}"

if [ -z "$DISPLAY" ] && [ -n "$WAYLAND_DISPLAY" ]; then
    export DISPLAY=:0
fi

if ! command -v zenity >/dev/null 2>&1; then
    echo "sudo-askpass: zenity not installed (try: sudo apt install zenity)" >&2
    exit 1
fi

exec zenity --password --title="$prompt" 2>/dev/null