#!/usr/bin/env bash
# Automated validation for feature 001 (substitutes manual quickstart where possible).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
FEATURE="specs/001-persistent-ui-virtual-browsing"
RESULTS="$FEATURE/validation-results.md"
PASS=0
FAIL=0

pass() { echo "  ✓ $1"; PASS=$((PASS + 1)); }
fail() { echo "  ✗ $1"; FAIL=$((FAIL + 1)); }

echo "=== rust-feh feature 001 automated validation ==="

echo "--- Build & lint ---"
cargo build --release
cargo clippy -- -D warnings
pass "cargo build --release"
pass "cargo clippy -D warnings"

echo "--- Unit + integration tests (T067/T068 logic) ---"
cargo test
pass "cargo test (includes feature_001_validation.rs)"

echo "--- Static layout / FR checks (src/main.rs) ---"
MAIN=src/main.rs

if rg -n 'Showing.*images' "$MAIN" | rg -v 'showing_count_label' >/dev/null 2>&1; then
  fail "FR-006: raw Showing string outside showing_count_label"
else
  pass "FR-006: counter via showing_count_label (bottom panel)"
fi

if rg -n 'for now note|trigger the logic' src/ >/dev/null 2>&1; then
  fail "FR-011: menu stubs remain"
else
  pass "FR-011: no menu stubs"
fi

if rg -A12 'fn pick_folder' "$MAIN" | rg -q 'open_in_feh'; then
  fail "FR-007: open_in_feh in pick_folder path"
else
  pass "FR-007: pick_folder does not call open_in_feh"
fi

if rg -n 'TopBottomPanel::top\("controls"\)' "$MAIN" >/dev/null && \
   rg -n 'TopBottomPanel::bottom\("status"\)' "$MAIN" >/dev/null; then
  pass "FR-001/003: persistent top and bottom panels"
else
  fail "FR-001/003: missing TopBottomPanel markers"
fi

if rg -n 'show_rows' "$MAIN" >/dev/null; then
  pass "FR-004: show_rows virtualization present"
else
  fail "FR-004: show_rows missing"
fi

if rg -n 'scroll_generation' "$MAIN" >/dev/null; then
  pass "FR-005: scroll reset via scroll_generation"
else
  fail "FR-005: scroll_generation missing"
fi

if rg -n 'feh_available' "$MAIN" >/dev/null; then
  pass "FR-008a: feh_available present"
else
  fail "FR-008a: feh_available missing"
fi

if rg -n 'scanning' "$MAIN" >/dev/null && rg -n 'Scanning' "$MAIN" >/dev/null; then
  pass "FR-010: scanning indicator"
else
  fail "FR-010: scanning indicator missing"
fi

if rg -n 'pick_folder' "$MAIN" >/dev/null && rg -n 'Choose folder' "$MAIN" >/dev/null; then
  pass "FR-011: pick_folder wired"
else
  fail "FR-011: pick_folder missing"
fi

echo "--- Maintainer trace ---"
# Exclude bare "nfeh" — allowed in archival footer strings (see archive/original-nfeh/).
if rg -ri 'fa7ad|fahad|8bit\.demoncoder|gq\.fahad|@fa7ad' src/ >/dev/null 2>&1; then
  fail "maintainer traces in src/"
else
  pass "no maintainer traces in src/"
fi

{
  echo "# Validation Results (automated)"
  echo ""
  echo "**Run**: $(date -Iseconds)"
  echo "**Script**: scripts/validate-feature-001.sh"
  echo ""
  echo "| Metric | Result |"
  echo "|--------|--------|"
  echo "| Checks passed | $PASS |"
  echo "| Checks failed | $FAIL |"
  echo "| cargo test | pass |"
  echo "| SC-003 filter 10k <200ms | see test sc003_filter_10k_under_200ms |"
  echo "| SC-004 RSS <150MB | manual GUI only — not automated |"
  echo "| SC-002 60fps scroll | manual GUI only — not automated |"
  echo ""
  echo "## Quickstart mapping"
  echo ""
  echo "| Scenario | Automated |"
  echo "|----------|-----------|"
  echo "| V1 layout scroll | static panel checks only |"
  echo "| V2 10k scroll/RSS | scan 10k test; RSS/scroll manual |"
  echo "| V3 no auto-feh | status logic test + static grep |"
  echo "| V4 filter counter | filter + label tests |"
  echo "| V5 recursive | recursive scan test |"
  echo "| V6 debug log | static only (empty log policy in code) |"
  echo "| V7 menu | static pick_folder/menu grep |"
  echo "| V8 feh missing | post_scan_status test |"
  echo "| V9 scan state | scanning static + empty scan test |"
  echo "| V10 scroll reset | scroll_generation static |"
} > "$RESULTS"

echo ""
echo "Results written to $RESULTS"
echo "Passed: $PASS  Failed: $FAIL"

if [[ "$FAIL" -gt 0 ]]; then
  exit 1
fi

echo "=== All automated checks passed ==="