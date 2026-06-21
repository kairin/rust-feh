#!/usr/bin/env bash
# Sample rust-feh RSS/CPU while idle or after RUST_FEH_START_FOLDER auto-load.
# Usage: ./scripts/measure-resources.sh [image_count] [sample_seconds]
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

COUNT="${1:-10000}"
DURATION="${2:-90}"
BINARY="${ROOT}/rust-feh"

if [[ ! -x "$BINARY" ]]; then
  echo "Missing release binary. Run: cargo build --release && cp target/release/rust-feh ./rust-feh"
  exit 1
fi

echo "=== rust-feh resource measurement ==="
echo "Images in fixture: $COUNT"
echo "Sample duration:   ${DURATION}s"
echo ""

FIXTURE="$(./scripts/generate-perf-fixture.sh "$COUNT")"
echo "Fixture: $FIXTURE"
trap 'rm -rf "$FIXTURE"; pkill -x rust-feh 2>/dev/null || true' EXIT

pkill -x rust-feh 2>/dev/null || true
sleep 0.5

export RUST_FEH_START_FOLDER="$FIXTURE"
"$BINARY" >/tmp/rust-feh-measure.log 2>&1 &
APP_PID=""
for _ in $(seq 1 30); do
  APP_PID="$(pgrep -n -x rust-feh 2>/dev/null || true)"
  [[ -n "$APP_PID" ]] && break
  sleep 0.2
done

if [[ -z "$APP_PID" ]]; then
  echo "Failed to start rust-feh"
  cat /tmp/rust-feh-measure.log
  exit 1
fi

echo "PID: $APP_PID"
echo "Sampling RSS (KB), VSZ (KB), %CPU each second..."
echo ""

RSS_MIN=999999999
RSS_MAX=0
RSS_SUM=0
SAMPLES=0

printf "%-6s %-10s %-10s %-8s %-12s %s\n" "sec" "RSS_MB" "VSZ_MB" "%CPU" "state" "notes"
for sec in $(seq 1 "$DURATION"); do
  if ! kill -0 "$APP_PID" 2>/dev/null; then
    echo "Process exited early at ${sec}s"
    break
  fi

  read -r rss vsz cpu < <(ps -o rss=,vsz=,pcpu= -p "$APP_PID" | tr -s ' ')
  rss="${rss:-0}"
  vsz="${vsz:-0}"
  cpu="${cpu:-0}"

  RSS_MIN=$((rss < RSS_MIN ? rss : RSS_MIN))
  RSS_MAX=$((rss > RSS_MAX ? rss : RSS_MAX))
  RSS_SUM=$((RSS_SUM + rss))
  SAMPLES=$((SAMPLES + 1))

  rss_mb=$(awk "BEGIN {printf \"%.1f\", $rss/1024}")
  vsz_mb=$(awk "BEGIN {printf \"%.1f\", $vsz/1024}")

  state="running"
  notes=""
  if grep -q "Scanning" /tmp/rust-feh-measure.log 2>/dev/null; then
    state="scanning"
  fi
  if grep -q "Loaded.*images" /tmp/rust-feh-measure.log 2>/dev/null; then
    state="loaded"
    notes="scan complete"
  fi

  printf "%-6s %-10s %-10s %-8s %-12s %s\n" "$sec" "$rss_mb" "$vsz_mb" "$cpu" "$state" "$notes"
  sleep 1
done

if [[ "$SAMPLES" -gt 0 ]]; then
  RSS_AVG=$((RSS_SUM / SAMPLES))
  hwm_kb=$(awk '/VmHWM:/ {print $2}' "/proc/$APP_PID/status" 2>/dev/null || echo 0)
  rss_peak_mb=$(awk "BEGIN {printf \"%.1f\", $RSS_MAX/1024}")
  rss_avg_mb=$(awk "BEGIN {printf \"%.1f\", $RSS_AVG/1024}")
  rss_min_mb=$(awk "BEGIN {printf \"%.1f\", $RSS_MIN/1024}")
  hwm_mb=$(awk "BEGIN {printf \"%.1f\", ${hwm_kb:-0}/1024}")

  echo ""
  echo "=== Summary (PID $APP_PID) ==="
  echo "RSS min:     ${rss_min_mb} MB"
  echo "RSS avg:     ${rss_avg_mb} MB"
  echo "RSS peak:    ${rss_peak_mb} MB"
  echo "VmHWM peak:  ${hwm_mb} MB  (kernel high-water mark)"
  echo "SC-004 goal: < 150 MB @ ${COUNT} images (metadata-only list)"
  if awk "BEGIN {exit !($RSS_MAX/1024 < 150)}"; then
    echo "Verdict:     PASS (peak RSS under 150 MB)"
  else
    echo "Verdict:     FAIL (peak RSS >= 150 MB)"
  fi
fi

echo ""
echo "Log tail:"
tail -5 /tmp/rust-feh-measure.log 2>/dev/null || true