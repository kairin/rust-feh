#!/usr/bin/env bash
# Capture a window screenshot with ImageMagick import (no xdotool/wmctrl required).
# Usage: ./scripts/capture-app-screenshot.sh [output.png] [window_name]
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

OUT="${1:-docs/assets/readme-screenshot.png}"
WINDOW_NAME="${2:-rust-feh}"

if ! command -v import >/dev/null 2>&1; then
  echo "ImageMagick import not found. Install: sudo apt install imagemagick"
  exit 1
fi

if [[ -z "${DISPLAY:-}" ]]; then
  echo "DISPLAY is not set; cannot capture X11 window."
  exit 1
fi

mkdir -p "$(dirname "$OUT")"

# import -window name matches WM_NAME / title substring on X11.
import -window "$WINDOW_NAME" "$OUT"
echo "Saved: $OUT ($(file -b "$OUT"))"