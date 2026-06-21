# Gap Audit: Browsing Experience Round

**Date**: 2026-06-22 | **Status**: PASS (implemented)

| FR-ID | Status | Evidence |
|-------|--------|----------|
| FR-001 | pass | `open_in_feh` uses `--filelist` with filtered sorted paths |
| FR-002 | pass | `--start-at` selected path |
| FR-003 | pass | `--geometry`, `--scale-down`, `--zoom` retained |
| FR-004 | pass | `thread::spawn` + `poll_scan_complete` in `update` |
| FR-005 | pass | `scan_generation` stale drop |
| FR-006 | pass | `format_walk_warning` → `Scan skip:` |
| FR-007 | pass | Permission denied format unchanged; `t068` passes |
| FR-008 | pass | `summarize_scan_warnings` cap 50 |
| FR-009 | pass | Activity log TextEdit + Copy log / Copy status |
| FR-010 | pass | `log()` mirrors stderr |

| SC-ID | Status | Evidence |
|-------|--------|----------|
| SC-003 | pass | Nonexistent dir emits `Scan skip:` |
| SC-004 | pass | `t068_permission_denied_warning` |
| SC-001/002/005 | manual pending | [validation-results.md](./validation-results.md) |

## Deferred (SESSION-2026-06-22 item 10)

| Item | Status | Target |
|------|--------|--------|
| `feh --conversion-timeout` | deferred | future feh-launch feature |
| feh wallpaper mode variants (beyond `--bg-fill`) | deferred | future feh-launch feature |
| ImageMagick convert pipeline | deferred | proposed `010-imagemagick-format-bridge` |