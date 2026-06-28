# Implementation Plan: Multi-Feh Instance Launcher & Image Clipboard

**Branch**: `014-multi-feh-clipboard` | **Date**: 2026-06-26 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `specs/014-multi-feh-clipboard/spec.md`

## Summary

Add two capabilities to rust-feh: (1) a multi-instance feh launcher panel where users can configure and launch independent feh windows for different scanned folders, and (2) a right-click context menu on image rows to copy full image data to the system clipboard. Both are additive — the existing single "Open in feh" button is preserved unchanged. The feature introduces 3 new dependencies (arboard, serde, serde_json) justified by concrete user needs, and adds new types to `types.rs`, new I/O functions to `ui_logic.rs`, and new UI widgets to `main.rs`.

## Technical Context

**Language/Version**: Rust stable, edition 2021

**Primary Dependencies**: eframe 0.30 / egui 0.30 (glow), rfd 0.15, walkdir 2, image 0.25, which 6. **New**: arboard 3.x (clipboard), serde 1.x + serde_json 1.x (persistence).

**Storage**: JSON file at `~/.config/rust-feh/launch-entries.json` (new; first persistent state in the application).

**Testing**: `cargo test` (unit: `tests/unit/ui_logic.rs`, integration: `tests/integration/image_tools.rs`). New tests for `FehLaunchList` serialization and clipboard copy logic.

**Target Platform**: Linux (X11/Wayland). Clipboard requires a running display server (provided by eframe). feh subprocess spawning unchanged.

**Project Type**: Desktop GUI application (single binary).

**Performance Goals**: UI stays responsive at 60fps during feh launches (non-blocking `Command::spawn`). Clipboard copy completes in <3s for typical images; large images may block briefly (single-threaded egui; v1 limitation documented).

**Constraints**: No new threading infrastructure (v1 uses main-thread blocking for clipboard). Launch entries must survive restart. Must not regress existing single-launch "Open in feh" flow.

**Scale/Scope**: Supports 1–20+ launch entries (persisted). Clipboard handles images up to ~50MB without noticeable delay. No cross-instance coordination between feh processes.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### I. Thin-Wrapper Architecture
- **Multi-instance**: Spawns more `feh` subprocesses via `Command::new("feh")` — same pattern as existing `open_in_feh`. No feh feature reimplementation.
- **Clipboard**: Copies image data to system clipboard; feh itself has no clipboard feature. This is an application-level utility, not a feh feature duplicate.
- **Status**: ✅ PASS

### II. Pure Rust, Minimal Dependencies
- **New deps**: `arboard` (clipboard), `serde` + `serde_json` (persistence).
- **Justification**: No Rust stdlib clipboard API. No built-in JSON serialization. Both are crates.io, actively maintained, minimal transitive footprint.
- **Status**: ✅ PASS (justified in Complexity Tracking)

### III. Clean Module Separation
- **types.rs**: `FehLaunchEntry`, `FehLaunchList` — domain types, no egui
- **ui_logic.rs**: `save_launch_list()`, `load_launch_list()`, `copy_image_to_clipboard()`, `build_entry_filelist()` (feh filelist emission for an entry's folder), `entry_is_launchable()` — pure I/O / logic functions, no egui
- **main.rs**: Multi-instance panel UI, right-click context menu — egui rendering only; delegates filelist generation and launchability checks to `ui_logic`
- **Source of truth**: `src/lib.rs` re-exports new types and functions
- **Status**: ✅ PASS — feh filelist emission and launchability logic live in `ui_logic` per Principle III; `main.rs` only spawns the subprocess and renders.

### IV. Linux-First, feh-Centric
- Clipboard targets Linux X11/Wayland (arboard's Linux support)
- Persistence uses XDG `~/.config/` standard
- feh integration unchanged (same `--filelist`, `--start-at`, `--geometry` flags)
- **Status**: ✅ PASS

### V. Performance Awareness
- feh launch: `Command::spawn` is non-blocking (instant return)
- Clipboard read: blocks the render loop for the duration of `image::load_from_memory` + arboard `set_image`. Acceptable for v1 (typical <3s). Documented limitation.
- Persistence: JSON read/write happens at app start/exit and on user actions (add/remove/edit) — negligible I/O (<1ms for 20 entries)
- **Status**: ✅ PASS

## Project Structure

### Documentation (this feature)

```text
specs/014-multi-feh-clipboard/
├── plan.md              # This file
├── research.md          # Phase 0: clipboard, persistence, UI architecture decisions
├── data-model.md        # Phase 1: FehLaunchEntry, FehLaunchList, JSON schema
├── quickstart.md        # Phase 1: 8 validation scenarios
├── contracts/
│   ├── feh-instance-panel.md    # Multi-instance panel UI contract
│   └── clipboard-context-menu.md # Right-click clipboard contract
└── tasks.md             # Phase 2: /speckit-tasks (NOT created by /speckit-plan)
```

### Source Code (changes only — existing files not shown)

```text
src/
├── types.rs             # ADD: FehLaunchEntry, FehLaunchList
├── ui_logic.rs          # ADD: copy_image_to_clipboard(), save/load_launch_list(), build_entry_filelist(), entry_is_launchable()
├── lib.rs               # MOD: re-export new types and functions
└── main.rs              # MOD: multi-instance panel UI, context menu, new fields

tests/
└── unit/
    └── ui_logic.rs      # ADD: serialization round-trip, clipboard mock tests

Cargo.toml               # ADD: arboard, serde, serde_json
```

**Structure Decision**: Single-project layout (existing pattern). All changes are within the existing `src/` modules respecting Principle III separation. No new modules needed — the feature is small enough to fit into existing module boundaries. All newly created source/test files MUST carry an `SPDX-License-Identifier: MIT` header per Constitution Technical Standards.

## Implementation Decomposition

The feature is decomposed into 36 independently implementable and testable tasks in `tasks.md`. The high-level buckets below mirror the task phases while preserving the fine-grained task IDs used for implementation:

### Task 1: Add Dependencies
- **File**: `Cargo.toml`
- **Changes**: Add `arboard = "3"`, `serde = { version = "1", features = ["derive"] }`, `serde_json = "1"`
- **Verification**: `cargo check` compiles with new deps

### Task 2: Add Domain Types
- **File**: `src/types.rs`
- **Changes**: Add `FehLaunchEntry` struct, `FehLaunchList` struct, derive macros (Debug, Clone, Serialize, Deserialize)
- **Verification**: `cargo check --lib`, new types compile

### Task 3 / Task 7: Re-export New Public API
- **File**: `src/lib.rs`
- **Changes**:
  - `T003`: after `T002`, add `pub use types::{FehLaunchEntry, FehLaunchList};`
  - `T007`: after `T004`/`T005`/`T006`, add `pub use ui_logic::{load_launch_list, save_launch_list, launch_list_path, copy_image_to_clipboard, decode_image_to_rgba, build_entry_filelist, entry_is_launchable};`
- **Verification**: `cargo check`
- **Ordering Constraint**: Do not re-export `ui_logic` functions before they exist, or intermediate commits will fail to compile.

### Task 4: Implement Persistence (Save/Load)
- **File**: `src/ui_logic.rs`
- **Functions**: `launch_list_path() -> PathBuf`, `load_launch_list() -> FehLaunchList`, `save_launch_list(&FehLaunchList) -> io::Result<()>`
- **Logic**: XDG config dir creation, atomic write (temp + rename), graceful corruption recovery
- **Persistence policy**: save immediately on every mutation (add/remove/folder edit/label edit); optional `Drop` save is best-effort backup only, not the primary persistence path
- **Verification**: Unit tests for round-trip: create list → save → load → assert equality; corrupt JSON → empty list

### Task 5: Implement Clipboard Copy
- **File**: `src/ui_logic.rs`
- **Function**: `copy_image_to_clipboard(path: &Path) -> Result<String, String>` — returns success/error message for status bar
- **Logic**: `std::fs::read` → `image::load_from_memory` → `to_rgba8()` → `arboard::Clipboard::set_image()`
- **Testability**: factor the decode step into a pure `decode_image_to_rgba(bytes) -> Result<(width, height, Vec<u8>), String>` helper so format/dimension correctness can be unit-tested without a display server; the `arboard` set-image call is the only display-dependent part
- **Edge cases**: File not found, decode failure, clipboard unavailable
- **Verification**: Unit-test `decode_image_to_rgba` against a known PNG fixture (assert dimensions + non-empty RGBA) and a non-image file (assert decode error). Real-clipboard set/get is a gated integration test that skips when no display is available.

### Task 6: Implement Filelist & Launchability Logic
- **File**: `src/ui_logic.rs`
- **Functions**: `build_entry_filelist(entry, inventory)`, `entry_is_launchable(entry, inventory, feh_available)` — pure, no egui
- **Logic**: generate the feh filelist lines for an entry's assigned folder from the scanned inventory; compute a launchability result covering folder-set, folder-exists, has-viewable-images, and feh-available conditions
- **Suggested result shape**: a `Launchability` enum (e.g. `Launchable`, `NoFolder`, `FolderMissing`, `Empty`, `FehMissing`) so UI status text maps deterministically and is unit-testable
- **Verification**: Unit tests (T028) for filelist output and each launchability variant

### Task 8: Add App State Fields
- **File**: `src/main.rs`
- **Changes**: Add to `RustFehApp` struct:
  - `launch_entries: FehLaunchList`
  - `feh_instances_section_open: bool`
  - `feh_instances_detached: bool`
  - `clipboard_context_menu: Option<ClipboardContextMenu>`
- **Struct**: `ClipboardContextMenu { image_path: PathBuf, anchor_pos: egui::Pos2 }`
- **Init**: `RustFehApp::new()` calls `load_launch_list()`
- **Persistence Policy**: save on every mutation (add/remove/folder edit/label edit) as the primary path; optional `Drop` save is best-effort backup only.
- **Verification**: `cargo check`, app compiles with new fields

### Task 7: Implement Multi-Instance Panel UI
- **File**: `src/main.rs`
- **Methods**: 
  - `render_inspector_feh_instances()` — top-level section with detach toolbar
  - `render_feh_instances_body()` — [+ Add] button, [Launch All], entry list
  - `render_feh_instance_entry()` — single entry: label, folder ComboBox, [Launch], [×]
  - `add_launch_entry()` — creates new entry, saves
  - `remove_launch_entry(id)` — removes by id, saves
  - `update_entry_label(id, label)` — sets label, saves
  - `update_entry_folder(id, path)` — sets folder, saves
  - `launch_entry_feh(entry)` — calls `ui_logic::build_entry_filelist(entry, inventory)` for filelist generation, then spawns feh (same `Command::new("feh")` pattern as existing `open_in_feh`); UI only spawns + reports status
- **Folder candidates**: `build_folder_tree()` from existing scan results; extract folder paths as ComboBox options
- **Verification**: `cargo check`, manual test with `quickstart.md` VS-1, VS-3, VS-7

### Task 8: Implement Right-Click Context Menu
- **File**: `src/main.rs`
- **Changes**: 
  - Capture `response.secondary_clicked()` in both FlatList and FolderTree image row rendering loops
  - Render `egui::Area` context menu when `clipboard_context_menu` is `Some`
  - Dismiss on primary click or Escape
  - Call `copy_image_to_clipboard()` from ui_logic on menu click
- **Verification**: `cargo check`, manual test with `quickstart.md` VS-4, VS-5, VS-6

### Task 9: Wire Inspector Layout
- **File**: `src/main.rs`
- **Changes**: 
  - Add "Feh Instances" section to the `render_inspector_sidebar()` method (after "Image actions")
  - Add detached window rendering for feh instances panel
- **Verification**: UI renders correctly in inspector sidebar

### Task 10: Tests, Validation & Quality Gates (Phase 7)
- **Files**: `tests/unit/ui_logic.rs`, plus repo-wide checks
- **Unit tests**:
  - `T026`: persistence — `test_launch_list_roundtrip`, `test_launch_list_empty_load`, `test_launch_list_corrupt_json`
  - `T027`: clipboard decode/error — `decode_image_to_rgba` on a known PNG fixture (dimensions + RGBA length), missing file, non-image file; real `arboard` set/get is a gated integration test skipped without a display
  - `T028`: `build_entry_filelist()` + `entry_is_launchable()` variants (filelist lines, folder missing, empty inventory, feh missing)
- **Quality gates / validation**:
  - `T029`: verify `SPDX-License-Identifier: MIT` headers on new source/test files
  - `T030`: `cargo check`
  - `T031`: `cargo test`
  - `T032`: run quickstart VS-1…VS-8 manually
  - `T033`: SC-001 — 5 concurrent feh instances stay responsive
  - `T034`: SC-002 / SC-005 — entry creation under 3 interactions, "+" discoverable within 10s
  - `T035`: FR-014 regression — single "Open in feh" button still works
  - `T036`: `cargo test --no-default-features` (or document why no alternate feature matrix exists)

## Dependency Graph (implementation order)

```
T001 (deps)
  └─ T002 (types) ── T003 (re-export types)
                  ├─ T004 (persist) ─┐
                  ├─ T005 (clipboard)├─ T007 (re-export fns) ── T008 (state fields)
                  └─ T006 (filelist/launchable) ─┘
T008 ── US1 (T009–T013) ── US2 (T014–T017)
     └─ US3 (T018–T022, parallel with inspector-panel work after Phase 2)
US1 ── US4 (T023–T025)
US1..US4 ── Phase 7 (T026–T036: tests, SPDX, cargo check/test, manual SC validation)
```

T002 requires T001 (serde). T003 re-exports types right after T002; T007 re-exports `ui_logic` functions only after T004/T005/T006 define them, otherwise intermediate commits will not compile. T004, T005, and T006 implement separate `ui_logic` responsibilities but edit the same file, so run them sequentially or coordinate edits carefully. US1/US2/US4 are sequential within the `src/main.rs` inspector-panel region; US3 (T018–T022) is a different image-list/context-menu region and can run in parallel after Phase 2 if edits are coordinated. Phase 7 tests (T026/T027/T028) can be written as soon as their helpers exist; final validation (T030–T036) runs after all stories complete.

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|---|---|---|
| New dep: `arboard` | System clipboard for image paste (FR-008). No Rust stdlib clipboard API. | Manual X11/Wayland FFI: 500+ lines of unsafe code, fragile across compositors, violates "minimal dependencies" principle in a worse way. |
| New dep: `serde` + `serde_json` | JSON serialization for launch entry persistence (FR-006). No built-in JSON in Rust stdlib. | Hand-written JSON formatting: error-prone, non-standard, adds maintenance burden. serde is the Rust ecosystem standard. |
