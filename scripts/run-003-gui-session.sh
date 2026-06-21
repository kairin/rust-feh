#!/usr/bin/env bash
# Feature 003 manual GUI session — no sudo required.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

echo "=== 003 GUI performance session (no sudo) ==="
echo ""

if [[ -z "${DISPLAY:-}" && -z "${WAYLAND_DISPLAY:-}" ]]; then
  echo "ERROR: No DISPLAY or WAYLAND_DISPLAY — need a graphical session." >&2
  exit 1
fi

echo "--- Automated preflight ---"
./scripts/validate-gui-performance.sh 2>&1 | tail -3
echo ""

COUNT="${1:-10000}"
FIXTURE="$(./scripts/generate-perf-fixture.sh "$COUNT")"
FILE_COUNT="$(find "$FIXTURE" -maxdepth 1 -name '*.jpg' | wc -l | tr -d ' ')"

echo "--- Fixture ---"
echo "FIXTURE=$FIXTURE"
echo "Files: $FILE_COUNT (expected $COUNT)"
echo ""
echo "Copy this path — Choose folder in rust-feh and paste/select it."
echo ""
echo "--- Launch GUI ---"
echo "In a SECOND terminal while the app runs:"
echo "  ./scripts/sample-rss.sh    # repeat 3× during/after scroll"
echo ""
echo "Protocol: specs/003-gui-performance-validation/quickstart.md Steps 3–5"
echo "Results:  specs/003-gui-performance-validation/validation-results.md"
echo ""
echo "Starting ./rust-feh ..."
exec ./rust-feh