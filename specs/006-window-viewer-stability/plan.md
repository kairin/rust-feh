# Implementation Plan: Window & Viewer Stability

**Branch**: `006-window-viewer-stability` (work landed on `main` post-001) | **Date**: 2026-06-28 | **Spec**: [spec.md](./spec.md)

**Status**: **Retroactive plan** — FR-001–FR-007 already shipped and unit-tested; FR-008 / SC-005 (window-preference persistence) is the one genuine code gap. This plan formalizes the shipped behavior and scopes the persistence work.

**Input**: Feature specification from `specs/006-window-viewer-stability/spec.md`

**Parent**: [001-persistent-ui-virtual-browsing](../001-persistent-ui-virtual-browsing/spec.md) (layout US1)

## Summary

Window & Viewer Stability fixes three dogfood failures: (1) layout jump / empty list space on folder load, (2) no user control over window size, (3) feh shrinking its window to tiny-image dimensions. Code for all three (FR-001–FR-007) is present and verified by `cargo test`. The remaining requirement is **FR-008 / SC-005**: persist the user's window preset + resizable choice across launches. The approach mirrors the existing launch-list persistence (`save_launch_list` / `load_launch_list` in `src/ui_logic.rs`): a small serde struct written as JSON to `~/.config/rust-feh/`, loaded at startup, saved when the user changes preset or the resizable toggle.

## Verified current state (audit before planning)

Audited live against the tree on 2026-06-28 (`cargo test` green: 109 passed / 0 failed / 2 ignored).

| Req | Status | Evidence (file:symbol) |
|-----|--------|------------------------|
| FR-001 list fills space | shipped | `main.rs` central panel uses `auto_shrink([false, false])` on the list scroll areas (`render_flat_image_list`, `render_tree_image_list`); `list_height` computed from `available_height` |
| FR-002 toolbar height stable | shipped | `render_top_menu_bar` / `render_view_menu` render fixed menu bar; controls disabled-not-removed |
| FR-003 640×480 floor | shipped | `WINDOW_MIN_RESIZABLE = (640.0, 480.0)`, `clamp_window_size()` in `ui_logic.rs`; unit test `clamp_window_size_enforces_floor` |
| FR-004 presets | shipped | `WindowSizePreset {Compact,Default,Large}` in `types.rs`; `window_preset_dimensions` 720×540 / 960×720 / 1280×960 in `ui_logic.rs`; unit test `window_preset_dimension_values`; View ▸ Window size menu in `render_view_menu` |
| FR-005 resizable toggle | shipped | `window_resizable` field + `apply_window_resize_policy` (sets `Resizable`, `MinInnerSize`, `MaxInnerSize` lock) in `main.rs`; "Resizable window" checkbox in `render_view_menu` |
| FR-006 feh fixed geometry | shipped | `FEH_VIEWER_GEOMETRY = "1280x960"`, `spawn_feh_viewer` passes `--geometry 1280x960 --scale-down --zoom max` |
| FR-007 upscale tiny images | shipped | `FEH_VIEWER_ZOOM = "max"` + `--scale-down` in `spawn_feh_viewer` |
| **FR-008 persistence** | **MISSING** | no `window_prefs`/`save_window_prefs`/`load_window_prefs` in `src/`; startup always uses `WindowSizePreset::default()` + `window_resizable: true` (`create_rust_feh_app`) |

So only FR-008 (and its success criterion SC-005) requires new code. Everything else is verify-only.

## Technical Context

**Language/Version**: Rust stable, edition 2021
**Primary Dependencies**: `eframe`/`egui` 0.30 (glow); `serde` + `serde_json` (already deps for feature 014 persistence) — no new dependency required
**Storage**: JSON file at `~/.config/rust-feh/window-prefs.json` (sibling of existing `launch-entries.json`)
**Testing**: `cargo test` — unit tests in `tests/unit/ui_logic.rs` (mirror existing launch-list round-trip / corrupt-file tests); `cargo check`. (`cargo clippy` is NOT installed in this environment — use check+test as the gate.)
**Target Platform**: Linux desktop (glow / X11+Wayland)
**Project Type**: desktop-app feature (core modules + GUI)
**Performance Goals**: persistence load is a single small file read at startup (<1ms); no render-loop impact
**Constraints**: constitution §III — persistence IO is pure and lives in `ui_logic.rs`; `main.rs` only wires load at init and save on change. No GUI types in core.
**Scale/Scope**: one small struct, two functions, one path helper, ~3 wiring points, ~3 unit tests.

## Constitution Check

*GATE: re-checked against constitution v1.0.1 (2026-06-21).*

- **I. Thin-Wrapper**: PASS — no feh feature reimplemented; window prefs are frontend chrome.
- **II. Pure Rust, Minimal Deps**: PASS — reuses existing `serde`/`serde_json`; no new crate.
- **III. Clean Module Separation**: PASS — `window_prefs_path` / `save_window_prefs` / `load_window_prefs` are pure functions in `ui_logic.rs` (no egui); `main.rs` does load/save wiring only. Mirrors the already-compliant launch-list pattern.
- **IV. Linux-First, feh-Centric**: PASS — feh geometry/zoom contract unchanged (FR-006/007 already shipped).
- **V. Performance Awareness**: PASS — one small file read at startup; no scan/render-loop interaction.

No violations → Complexity Tracking table not required.

## Project Structure

### Documentation (this feature)

```text
specs/006-window-viewer-stability/
├── spec.md              # existing
├── plan.md              # this file
├── research.md          # decisions (persistence format/location/save-timing)
├── data-model.md        # WindowPreferences entity
├── quickstart.md        # manual + automated validation guide
├── contracts/
│   └── window-prefs.md  # JSON schema + feh launch flag contract
├── checklists/
│   └── requirements.md  # existing
└── tasks.md             # produced by /speckit-tasks (NOT this command)
```

### Source Code (repository root)

```text
src/
├── types.rs       # WindowSizePreset (exists). ADD: WindowPreferences { version, preset, resizable } (serde)
├── ui_logic.rs    # window helpers (exist). ADD: window_prefs_path / save_window_prefs / load_window_prefs
├── lib.rs         # re-exports. ADD the three new fns alongside launch_list_* exports
└── main.rs        # ADD: load_window_prefs() in create_rust_feh_app init; save in sync_frame_input_state
                   #      on window_size / window_resizable change (next to existing apply_* calls)

tests/unit/ui_logic.rs   # ADD: round-trip, empty/missing -> defaults, corrupt -> defaults
```

**Structure Decision**: Single-crate desktop app (constitution layout). Persistence is added to the existing `ui_logic` core module; no new module needed.

## Implementation approach (decomposed for /speckit-tasks)

Smallest independently-verifiable units, in dependency order:

1. **U1 — `WindowPreferences` type** (`types.rs`): `#[derive(Serialize, Deserialize, Clone, PartialEq)] struct WindowPreferences { version: u32, preset: WindowSizePreset, resizable: bool }` + `Default` (version 1, Default preset, resizable true). Requires `WindowSizePreset` to derive `Serialize`/`Deserialize` (currently only `Debug, Clone, Copy, PartialEq, Eq, Default`). Verify: `cargo check`. *(no dep)*
2. **U2 — path helper** (`ui_logic.rs`): `window_prefs_path() -> PathBuf` → `~/.config/rust-feh/window-prefs.json` (copy `launch_list_path` shape). Verify: unit test asserts filename suffix. *(dep: none)*
3. **U3 — save** (`ui_logic.rs`): `save_window_prefs(&WindowPreferences) -> Result<(), String>` using temp-file + rename (copy `save_launch_list`). *(dep: U1, U2)*
4. **U4 — load** (`ui_logic.rs`): `load_window_prefs() -> WindowPreferences`, missing/corrupt → default with eprintln warning (copy `load_launch_list`). *(dep: U1, U2)*
5. **U5 — re-export** (`lib.rs`): add the three fns to the `pub use ui_logic::{…}` block. *(dep: U2–U4)*
6. **U6 — load at startup** (`main.rs` `create_rust_feh_app`): set `window_size` / `prior_window_size` / `window_resizable` / `prior_window_resizable` from `load_window_prefs()` instead of hard-coded defaults. *(dep: U5)*
7. **U7 — save on change** (`main.rs` `sync_frame_input_state`): in the existing `window_size != prior_window_size` and `window_resizable != prior_window_resizable` branches, call `save_window_prefs(&WindowPreferences{…})` and log/surface error like `persist_launch_entries`. *(dep: U5)*
8. **U8 — unit tests** (`tests/unit/ui_logic.rs`): round-trip; missing→default; corrupt→default. Mirror existing launch-list tests (set `HOME` to temp dir). Can be written in parallel with U6/U7. *(dep: U3, U4)*
9. **U9 — docs/closeout**: add `006/gap-audit.md` mapping FR→evidence; flip SC-005 to verified-by-test.

Parallelizable: U2 ∥ U1; U8 ∥ U6 ∥ U7 (different files/regions). Serial spine: U1→U3/U4→U5→U6/U7.

Verification after U7: `cargo check && cargo test`. SC-005 is then covered by automated round-trip tests (no GUI required), removing the persistence gap. SC-002 list-height has a 2026-06-28 manual pass recorded in `gap-audit.md`; SC-003 feh visible-window behavior is guaranteed by the fixed 1280×960 launch policy but still lacks direct pixel-level/manual confirmation in this environment — see quickstart.

## Phase notes

- **Phase 0 (research.md)**: format = JSON (consistent with launch-entries.json); location = XDG `~/.config/rust-feh/`; save timing = on-change (not on-exit; eframe `on_exit` is less reliable than the existing on-change pattern already used for the in-session apply). No NEEDS CLARIFICATION remain.
- **Phase 1 (data-model.md, contracts/, quickstart.md)**: one entity (`WindowPreferences`); contract documents the JSON shape + the feh launch flags (existing FR-006/007 subprocess contract); quickstart covers automated tests + manual SC-002/003/004.

## Done When (plan scope)

- [x] Current code audited; shipped vs missing established
- [x] research / data-model / quickstart / contracts generated
- [x] Agent context (AGENTS.md) repointed to this plan
- [ ] `/speckit-tasks` run to emit `tasks.md` (next command)
