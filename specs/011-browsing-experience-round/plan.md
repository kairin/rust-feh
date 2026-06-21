# Implementation Plan: Browsing Experience Round

**Branch**: `011-browsing-experience-round` | **Date**: 2026-06-22 | **Spec**: [spec.md](./spec.md)

## Summary

Three dogfood fixes in one round:

1. **feh filelist** — `open_in_feh` writes filtered sorted paths to temp file; `feh --filelist … --start-at …`
2. **Background scan** — `std::thread::spawn` + `mpsc`; `update()` polls; `scan_generation` discards stale results
3. **Scanner warnings** — non-permission walkdir → `Scan skip:`; cap at 50 via `summarize_scan_warnings`
4. **Copyable log** — read-only `TextEdit` + Copy log / Copy status buttons

## Technical Context

**Language**: Rust 2021  
**Dependencies**: No new crates (`std::thread`, `std::sync::mpsc`)  
**Modules touched**: `scanner.rs`, `ui_logic.rs`, `main.rs`, `lib.rs` (optional `scan_job` helpers in `ui_logic`)  
**Testing**: `cargo test` unit tests in `scanner`, `ui_logic`; `t068` preserved

## Constitution Check

| Principle | Status |
|-----------|--------|
| I. Thin-Wrapper | ✅ feh filelist delegates navigation to feh |
| II. Minimal Dependencies | ✅ std only for async |
| III. Module Separation | ✅ filelist/warnings in `ui_logic`/`scanner`; GUI polls only |
| IV. Linux-First | ✅ feh subprocess unchanged |
| V. Performance | ✅ UI thread never blocks on walkdir |

## File Changes

| File | Change |
|------|--------|
| `src/scanner.rs` | `format_walk_warning`, `summarize_scan_warnings`, non-permission errs |
| `src/ui_logic.rs` | `write_feh_filelist`, `join_activity_log`, `summarize_scan_warnings` re-export or keep in scanner |
| `src/main.rs` | `ScanJob` state, poll in `update`, filelist feh launch, activity log UI |
| `tests/feature_001_validation.rs` | optional `t069_scan_skip_non_permission` |

## Contracts

See [contracts/feh-filelist.md](./contracts/feh-filelist.md), [contracts/background-scan.md](./contracts/background-scan.md).