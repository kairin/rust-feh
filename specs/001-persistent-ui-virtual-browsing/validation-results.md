# Validation Results (automated)

**Run**: 2026-06-22T01:47:16+08:00
**Script**: scripts/validate-feature-001.sh

| Metric | Result |
|--------|--------|
| Checks passed | 13 |
| Checks failed | 0 |
| cargo test | pass |
| SC-003 filter 10k <200ms | see test sc003_filter_10k_under_200ms |
| SC-004 RSS <150MB | **pass** — peak ~126 MB @10k, ~124 MB @1k (2026-06-22; see 003 validation-results) |
| SC-002 60fps scroll | manual GUI only — **pending** |

## Quickstart mapping

| Scenario | Automated |
|----------|-----------|
| V1 layout scroll | static panel checks only |
| V2 10k scroll/RSS | scan 10k test; RSS/scroll manual |
| V3 no auto-feh | status logic test + static grep |
| V4 filter counter | filter + label tests |
| V5 recursive | recursive scan test |
| V6 debug log | static only (empty log policy in code) |
| V7 menu | static pick_folder/menu grep |
| V8 feh missing | post_scan_status test |
| V9 scan state | scanning static + empty scan test |
| V10 scroll reset | scroll_generation static |

## GUI tier (feature 003)

RSS audit (SC-004 **pass**): [specs/003-gui-performance-validation/validation-results.md](../003-gui-performance-validation/validation-results.md). Scroll (SC-002) still pending.

Runbook: [specs/003-gui-performance-validation/quickstart.md](../003-gui-performance-validation/quickstart.md) · Entry: `./scripts/validate-gui-performance.sh`
