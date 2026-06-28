# Gap Audit: Window & Viewer Stability (006)

**Date**: 2026-06-28 | **Gate**: `cargo check && cargo test` (clippy/fmt unavailable in this env)

Maps every FR/SC to concrete evidence (symbol + test, or manual status).

## Functional Requirements

| Req | Status | Evidence |
|-----|--------|----------|
| FR-001 list fills vertical space | ✅ shipped | `auto_shrink([false, false])` on list scroll areas + `available_height`-derived `list_height` in `src/main.rs` (`render_*_image_list`) |
| FR-002 toolbar height stable | ✅ shipped | disabled-vs-enabled controls keep fixed layout in toolbar render path, `src/main.rs` |
| FR-003 min window 640×480 | ✅ shipped | `WINDOW_MIN_RESIZABLE = (640.0, 480.0)` + `clamp_window_size`, `src/ui_logic.rs`; test `clamp_window_size_enforces_floor` |
| FR-004 size presets | ✅ shipped | `WindowSizePreset` + `window_preset_dimensions` (720×540 / 960×720 / 1280×960), `src/ui_logic.rs`; test `window_preset_dimension_values`; View ▸ Window size menu |
| FR-005 resizable toggle | ✅ shipped | `window_resizable` + `apply_window_resize_policy` (Resizable / MinInnerSize / MaxInnerSize lock); "Resizable window" checkbox, `src/main.rs` |
| FR-006 fixed feh geometry/zoom | ✅ shipped | `FEH_VIEWER_GEOMETRY = "1280x960"`, `FEH_VIEWER_ZOOM = "max"`, `src/ui_logic.rs`; `--geometry/--scale-down/--zoom` in `spawn_feh_viewer` |
| FR-007 upscale tiny images | ✅ shipped | `--zoom max` policy keeps small images visible in fixed window |
| FR-008 window pref persistence | ✅ **implemented + live verified (006 round)** | `WindowPreferences` (`src/types.rs`); `window_prefs_path` / `save_window_prefs` / `load_window_prefs` (`src/ui_logic.rs`); startup seed in `create_rust_feh_app`, first-frame viewport apply via `apply_startup_window_prefs`, + `persist_window_prefs` on change in `sync_frame_input_state` (`src/main.rs`); tests `window_prefs_round_trip`, `window_prefs_missing_returns_default`, `window_prefs_corrupt_returns_default`; live restart check with saved `resizable:false` opened non-resizable |

## Success Criteria

| SC | Status | Evidence |
|----|--------|----------|
| SC-001 folder load doesn't resize window | ✅ shipped (auto) | window outer size only changes on explicit preset/lock commands; no resize cmd in scan path |
| SC-002 list ≥50% of panel with 3 images | ✅ manual pass | User verified 2026-06-28 with `/tmp/rust-feh-manual-validation/small`: the 3 image rows sit at the top of one tall bordered list scroll area that fills most of the central panel; blank space is inside the scroll area, not a collapsed-list gap below it |
| SC-003 feh viewer ≥640×480 for tiny image | ✅ shipped (auto by policy); manual pixel confirm unrun | fixed 1280×960 geometry guarantees ≥640×480 by construction; direct GUI/pixel confirmation blocked in this env (screenshot capture unavailable) — see T009 |
| SC-004 switch preset + lock <10s | ✅ manual pass | User verified 2026-06-28: unticking View ▸ Resizable window prevents resize; ticking it allows resize again; Large preset logged as 1280×960 earlier in the same walkthrough |
| SC-005 prefs survive restart | ✅ **implemented + manual pass** | round-trip covered by `window_prefs_round_trip`; load on startup wired in `create_rust_feh_app`; `apply_startup_window_prefs` applies loaded state to viewport; user verified fresh restart with saved `resizable:false` was not resizable |

## Verification Summary

- Automated: `cargo check && cargo test` green — 51 lib + 18 unit ui_logic (incl. 3 new persistence tests) + integration/perf suites, 0 failed.
- Manual 006 GUI checks: SC-002, SC-004, and SC-005 were verified with the user in a live GUI walkthrough; direct agent screenshots remain unavailable because the computer_use desktop driver is broken in this environment.
- Tooling caveats: `cargo clippy` and `cargo fmt` are not installed in this environment; lint/format claims cannot be locally re-verified here.
