---

description: "Task list for Window & Viewer Stability (006)"
---

# Tasks: Window & Viewer Stability

**Input**: Design documents from `/specs/006-window-viewer-stability/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/window-prefs.md

**Tests**: Included for the persistence story (US4), mirroring the existing launch-list unit/integration tests. The constitution favors automated coverage for `ui_logic` IO helpers.

**Organization**: Grouped by user story. US1–US3 are already shipped (verify-only); US4 (FR-008/SC-005 persistence) is the real implementation work.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: US1–US4 mapping to spec.md user stories
- Exact file paths included

## Path Conventions

- Single Rust crate: `src/` and `tests/` at repo root (per plan.md Structure Decision).

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Confirm tooling baseline before changes.

- [X] T001 Confirm build/test baseline is green: run `cargo check && cargo test` from repo root and record pass/fail (clippy unavailable in this env; check+test is the gate).

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Verify the already-shipped window/feh behavior the persistence story builds on; no new code here.

- [X] T002 [P] Verify FR-001/FR-002 list-fills-space: confirm `auto_shrink([false, false])` on list scroll areas and `list_height` derived from `available_height` in `src/main.rs` (`render_flat_image_list`, `render_tree_image_list`, `render_central_image_panel`).
- [X] T003 [P] Verify FR-003 floor: confirm `WINDOW_MIN_RESIZABLE = (640.0, 480.0)` and `clamp_window_size` in `src/ui_logic.rs`, covered by unit test `clamp_window_size_enforces_floor`.
- [X] T004 [P] Verify FR-004 presets: confirm `WindowSizePreset` variants and `window_preset_dimensions` (720×540 / 960×720 / 1280×960) in `src/ui_logic.rs`, covered by `window_preset_dimension_values`; View ▸ Window size menu in `render_view_menu` (`src/main.rs`).
- [X] T005 [P] Verify FR-005 resizable: confirm `window_resizable` field + `apply_window_resize_policy` (Resizable / MinInnerSize / MaxInnerSize lock) and the "Resizable window" checkbox in `src/main.rs`.
- [X] T006 [P] Verify FR-006/FR-007 feh policy: confirm `FEH_VIEWER_GEOMETRY = "1280x960"`, `FEH_VIEWER_ZOOM = "max"` in `src/ui_logic.rs` and `--geometry/--scale-down/--zoom` flags in `spawn_feh_viewer` (`src/main.rs`).

**Checkpoint**: Shipped behavior confirmed; persistence story can proceed.

---

## Phase 3: User Story 1 - Stable Application Window (Priority: P1) ✅ SHIPPED

**Goal**: Window does not jump / list does not leave a big blank gap on folder load.

**Independent Test**: Load a 3-image folder; list scroll area height ≥50% of central panel (SC-002); outer window unchanged (SC-001).

- [X] T007 [US1] Manual GUI validation of SC-001 + SC-002 per `quickstart.md` — verified 2026-06-28 with `/tmp/rust-feh-manual-validation/small`: user confirmed the empty space was inside one tall bordered list scroll area that filled most of the central panel, not outside a collapsed short list. SC-002 pass; SC-001 covered by no scan-path viewport resize commands.

---

## Phase 4: User Story 2 - User-Controlled Window Size (Priority: P1) ✅ SHIPPED (in-session)

**Goal**: User picks size preset and lock state from View menu.

**Independent Test**: Switch presets and lock size within 10s via View menu (SC-004).

- [X] T008 [US2] Manual GUI validation of SC-004 per `quickstart.md` (verified 2026-06-28 with user: toggling View ▸ Resizable window OFF prevents resizing; toggling it ON allows resizing again; earlier Large preset selection logged as 1280×960).

---

## Phase 5: User Story 3 - Predictable feh Viewer Window (Priority: P1) ✅ SHIPPED

**Goal**: feh viewer keeps a stable, visible window even for tiny images.

**Independent Test**: Open a 5×5 image; feh effective visible area ≥640×480 (SC-003).

- [ ] T009 [US3] Manual GUI validation of SC-003 per `quickstart.md` (BLOCKED in this env). Code policy launches feh with fixed 1280×960 geometry (`--geometry 1280x960 --scale-down --zoom max`); direct pixel/manual confirmation remains unrun because screenshot capture is unavailable.

---

## Phase 6: User Story 4 - Remember Window Preferences (Priority: P2) 🎯 REAL WORK (FR-008 / SC-005)

**Goal**: Window size preset + resizable toggle persist across restarts.

**Independent Test**: Set Large/default + resizable off, restart, settings restored (SC-005). Covered by automated round-trip tests plus live restart verification with user.

### Tests for User Story 4 (write first; expected to fail until impl lands) ⚠️

- [X] T010 [P] [US4] Add unit test `window_prefs_round_trip` in `tests/unit/ui_logic.rs`: set `HOME` to temp dir, `save_window_prefs(&WindowPreferences{version:1, preset:Large, resizable:false})`, assert `load_window_prefs()` equals it.
- [X] T011 [P] [US4] Add unit test `window_prefs_missing_returns_default` in `tests/unit/ui_logic.rs`: temp `HOME`, no file, assert `load_window_prefs() == WindowPreferences::default()`.
- [X] T012 [P] [US4] Add unit test `window_prefs_corrupt_returns_default` in `tests/unit/ui_logic.rs`: write `not-json` to `~/.config/rust-feh/window-prefs.json`, assert `load_window_prefs() == WindowPreferences::default()` (no panic).

### Implementation for User Story 4

- [X] T013 [US4] In `src/types.rs`: add `Serialize, Deserialize` derives to `WindowSizePreset` (currently `Debug, Clone, Copy, PartialEq, Eq, Default`). Verify `cargo check`.
- [X] T014 [US4] In `src/types.rs`: add `WindowPreferences { version: u32, preset: WindowSizePreset, resizable: bool }` with serde derives and a `Default` impl (version 1, `WindowSizePreset::Default`, `resizable: true`). (depends on T013)
- [X] T015 [P] [US4] In `src/ui_logic.rs`: add `window_prefs_path() -> PathBuf` → `~/.config/rust-feh/window-prefs.json` (mirror `launch_list_path`).
- [X] T016 [US4] In `src/ui_logic.rs`: add `save_window_prefs(&WindowPreferences) -> Result<(), String>` using temp-file + rename (mirror `save_launch_list`). (depends on T014, T015)
- [X] T017 [US4] In `src/ui_logic.rs`: add `load_window_prefs() -> WindowPreferences`, missing/corrupt → default + `eprintln!` warning (mirror `load_launch_list`). (depends on T014, T015)
- [X] T018 [US4] In `src/lib.rs`: re-export `window_prefs_path`, `save_window_prefs`, `load_window_prefs` from `ui_logic` alongside the launch-list exports. (depends on T015–T017)
- [X] T019 [US4] In `src/main.rs` `create_rust_feh_app`: call `load_window_prefs()` and seed `window_size`, `prior_window_size`, `window_resizable`, `prior_window_resizable` from it (replace hard-coded defaults); apply loaded prefs to the viewport once on first frame via `apply_startup_window_prefs` so restored lock state reaches the OS window. (depends on T018)
- [X] T020 [US4] In `src/main.rs` `sync_frame_input_state`: in the existing `window_size != prior_window_size` and `window_resizable != prior_window_resizable` branches, call `save_window_prefs(&WindowPreferences{version:1, preset: self.window_size, resizable: self.window_resizable})` and log on error (mirror launch-entry persistence). (depends on T018)

**Checkpoint**: `cargo test` green incl. new persistence tests → SC-005 satisfied automatically.

---

## Phase 7: Polish & Cross-Cutting Concerns

- [X] T021 [P] Create `specs/006-window-viewer-stability/gap-audit.md` mapping FR-001..FR-008 + SC-001..SC-005 to evidence (symbol/test or manual status).
- [X] T022 Update `specs/006-window-viewer-stability/checklists/requirements.md` to reflect FR-008 implemented.
- [X] T023 Run full `cargo check && cargo test`; record final pass count (verified 2026-06-28 after startup-apply fix: 122 passed, 0 failed, 2 ignored).

---

## Dependencies & Execution Order

- Setup (T001) → Foundational verify (T002–T006) → stories.
- US1–US3 (T007–T009) are manual GUI checks; independent of US4 code.
- US4 spine: T013 → T014 → (T015 ∥) → T016/T017 → T018 → T019/T020.
- US4 tests T010–T012 [P] written first; they fail until T013–T020 land, then pass.
- Polish (T021–T023) after US4 implementation.

### Parallel Opportunities

- T002–T006 all [P] (read-only checks, distinct concerns).
- T010–T012 [P] (same file but independent test fns; can be authored together).
- T015 [P] vs T013/T014 (different file region).

---

## Implementation Strategy

### MVP scope

US4 is the only code increment. US1–US3 are already delivered; their tasks are verification/validation only.

1. Baseline green (T001).
2. Confirm shipped behavior (T002–T006).
3. Implement persistence with tests (T010–T020).
4. Polish + audit (T021–T023).

---

## Notes

- [P] = different files/regions, no incomplete dependencies.
- Manual GUI task T009 is blocked by the broken computer_use desktop driver in this environment; T007/T008 were manually verified with the user on 2026-06-28.
- `cargo clippy` is not installed here; `cargo check && cargo test` is the canonical gate.
- Reuse the proven launch-list persistence pattern; do not introduce new dependencies (constitution §II).
