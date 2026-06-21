# Implementation Plan: UI Feedback & Network Scan Polish

**Branch**: `012-ui-feedback-polish` | **Date**: 2026-06-22 | **Spec**: [spec.md](./spec.md)

## Summary

Dogfood UI polish and NAS scan policy on top of feature 011:

1. **Network scan policy** — detect GVFS/SMB/NFS/UNC paths; skip ImageMagick identify during scan on network mounts
2. **Live status bar** — pulsing border/background, animated scan label, network responsiveness copy
3. **Dependencies OK state** — ✅/⚠ header; collapsed when required tools OK; auto-collapse on Recheck
4. **Bottom-bar tips** — move `operation_timings()` rotation from right panel to bottom status bar (4s interval + spinner)
5. **Panel separation** — `Frame::group` around inventory, image list, activity log
6. **Detachable activity log** — `egui::Window` with X close; reattach affordance in main panel

**Folded state**: Items 1–6 are partially implemented in `src/main.rs` and `src/ui_logic.rs` (pre-spec session). Convergence closes doc/test/validation gaps.

## Technical Context

**Language**: Rust 2021  
**Dependencies**: No new crates  
**Modules touched**: `ui_logic.rs` (`is_network_mount_path`), `main.rs` (status bar, deps collapse, detach window, scan policy)  
**Testing**: Extend `ui_logic` unit tests; preserve 011 integration tests (SC-006)

## Constitution Check

| Principle | Status |
|-----------|--------|
| I. Thin-Wrapper | ✅ No feh behavior change |
| II. Minimal Dependencies | ✅ egui 0.30 `Frame::none` / `Frame::group` only |
| III. Module Separation | ⚠ `is_network_mount_path` in `ui_logic`; scan policy glue in `main.rs` — acceptable for UI orchestration |
| IV. Linux-First | ✅ GVFS/SMB paths primary dogfood target |
| V. Performance | ✅ Network identify skip + existing background scan |

## File Changes

| File | Change |
|------|--------|
| `src/ui_logic.rs` | `is_network_mount_path`; unit tests for path classes |
| `src/main.rs` | `deps_section_open`, `activity_log_detached`, status animation, bottom tips, panel frames, `magick_available && !on_network` |
| `specs/012-ui-feedback-polish/` | `gap-audit.md`, `quickstart.md`, `validation-results.md` |

## Contracts

See [contracts/status-bar-ui.md](./contracts/status-bar-ui.md), [contracts/network-scan-policy.md](./contracts/network-scan-policy.md).