#!/usr/bin/env bash
# Print RSS in KB for the newest rust-feh process (feature 003 SC-004 helper).
set -euo pipefail

PID="$(pgrep -n -x rust-feh 2>/dev/null || pgrep -n -f '/rust-feh$' 2>/dev/null || true)"

if [[ -z "${PID:-}" ]]; then
  echo "0"
  exit 1
fi

ps -o rss= -p "$PID" | tr -d ' '