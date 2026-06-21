#!/usr/bin/env bash
# Generate temp directory with N minimal .jpg files for GUI perf validation (feature 003).
set -euo pipefail

COUNT="${1:-10000}"
DIR="$(mktemp -d /tmp/rust-feh-perf-XXXXXX)"

for ((i = 0; i < COUNT; i++)); do
  printf 'x' >"$DIR/photo_$(printf '%05d' "$i").jpg"
done

echo "$DIR"