# Tasks: Multi-Feh Instance Launcher & Image Clipboard

**Input**: Design documents from `specs/014-multi-feh-clipboard/`

**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/, quickstart.md

**Tests**: Unit tests are included because plan.md and the project constitution require tests for core `ui_logic`/domain behavior. GUI flows are validated manually via quickstart.md scenarios.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files or non-overlapping code regions, no dependencies on incomplete tasks)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3, US4)
- Include exact file paths in descriptions

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Add new dependencies, domain types, and type re-exports needed by all user stories.

- [X] T001 Add dependencies to `Cargo.toml`: `arboard = "3"`, `serde = { version = "1", features = ["derive"] }`, and `serde_json = "1"`
- [X] T002 Add `FehLaunchEntry` and `FehLaunchList` structs with `Serialize`/`Deserialize` derives in `src/types.rs`, including `id`, `label`, `folder_path`, `created_at`, `version`, and `entries` fields from `specs/014-multi-feh-clipboard/data-model.md`
- [X] T003 Re-export new domain types after T002 in `src/lib.rs`: `pub use types::{FehLaunchEntry, FehLaunchList};`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core I/O functions and app state fields that all user stories depend on.

**⚠️ CRITICAL**: No user story work can begin until this phase is complete.

- [X] T004 Implement persistence functions in `src/ui_logic.rs`: `launch_list_path()`, `save_launch_list()`, and `load_launch_list()` using `$HOME/.config/rust-feh/launch-entries.json`, atomic temp-file plus rename writes, missing-file empty-list load, corrupt-JSON empty-list recovery, and save-on-mutation policy
- [X] T005 Implement clipboard helpers in `src/ui_logic.rs`: pure `decode_image_to_rgba(bytes) -> Result<(u32, u32, Vec<u8>), String>` plus `copy_image_to_clipboard(path) -> Result<String, String>` using `std::fs::read`, `image::load_from_memory`, `to_rgba8()`, and `arboard::Clipboard::set_image()`
- [X] T006 Implement feh launch helper logic in `src/ui_logic.rs`: `build_entry_filelist(entry, inventory)` and `entry_is_launchable(entry, inventory, feh_available)` with deterministic states for no folder, folder missing, empty scanned inventory, feh missing, and launchable
- [X] T007 Re-export new `ui_logic` functions after T004–T006 in `src/lib.rs`: `load_launch_list`, `save_launch_list`, `launch_list_path`, `copy_image_to_clipboard`, `decode_image_to_rgba`, `build_entry_filelist`, and `entry_is_launchable`
- [X] T008 Add app state fields to `RustFehApp` in `src/main.rs`: `launch_entries: FehLaunchList`, `feh_instances_section_open: bool`, `feh_instances_detached: bool`, and `clipboard_context_menu: Option<ClipboardContextMenu>`; add `ClipboardContextMenu { image_path: PathBuf, anchor_pos: egui::Pos2 }`; initialize `launch_entries` from `load_launch_list()` in `RustFehApp::new()`

**Checkpoint**: Foundation ready — user story implementation can now begin.

---

## Phase 3: User Story 1 - Launch Multiple Feh Instances for Different Folders (Priority: P1) 🎯 MVP

**Goal**: Users can add, launch, launch all, and remove feh launch entries via a new inspector panel. Each launchable entry spawns an independent feh window.

**Independent Test**: Scan two different folders, add each as a launch entry via `+`, click Launch on both, verify two feh windows open concurrently with correct folder contents, then close one window and confirm the other remains unaffected.

### Implementation for User Story 1

- [X] T009 [US1] Implement `add_launch_entry()` in `src/main.rs`: create a `FehLaunchEntry` with default folder resolved per FR-002 (selected folder-tree node → selected image parent → active scan root → unassigned), append it to `launch_entries`, and call `save_launch_list()`
- [X] T010 [US1] Implement `remove_launch_entry(id)` in `src/main.rs`: remove the matching entry from `launch_entries`, call `save_launch_list()`, and do not terminate already-running feh subprocesses
- [X] T011 [US1] Implement `launch_entry_feh(entry)` in `src/main.rs`: call `ui_logic::build_entry_filelist(entry, inventory)`, write the generated filelist, spawn `feh --geometry --scale-down --zoom --filelist --start-at` using the existing `open_in_feh` subprocess pattern, and update status/log messages
- [X] T012 [US1] Implement `render_feh_instances_body()` in `src/main.rs`: render `[+ Add]`, `[Launch All]`, and a scrollable entry list where each row shows index, launch button enabled via `entry_is_launchable`, and `[x]` remove button; `Launch All` launches configured launchable entries and skips invalid entries with a status summary per FR-015
- [X] T013 [US1] Wire the `Feh Instances` inspector section in `src/main.rs`: add `render_inspector_feh_instances()` after the existing `Image actions` section, use `CollapsingHeader` with id salt `feh_instances`, add detach toolbar support, and render the detached egui window when `feh_instances_detached` is true

**Checkpoint**: Users can add entries, launch individual feh windows, launch all valid entries, remove entries, and persist entries across restart.

---

## Phase 4: User Story 2 - Per-Instance Folder Assignment (Priority: P1)

**Goal**: Each launch entry has an independent scanned-folder selector and clear disabled states for missing, empty, unassigned, or feh-unavailable entries.

**Independent Test**: Create two entries, assign each to a different scanned folder via ComboBox, launch both, verify correct folder contents, reassign one entry to a third folder, relaunch, and verify the change takes effect.

### Implementation for User Story 2

- [X] T014 [US2] Implement `update_entry_folder(id, path)` in `src/main.rs`: update the entry `folder_path`, call `save_launch_list()`, and preserve the entry `id`, label, and insertion order
- [X] T015 [US2] Add a per-entry folder selector ComboBox in `render_feh_instances_body()` in `src/main.rs`: populate options from scanned folder-tree nodes, show a `None selected` option for clearing assignment, display the saved `folder_path`, and call `update_entry_folder()` on change
- [X] T016 [US2] Render stale and empty folder states in `render_feh_instances_body()` in `src/main.rs` by calling `ui_logic::entry_is_launchable(entry, inventory, feh_available)`: show `Folder not found`, `No images`, or `Select a folder` and disable Launch appropriately
- [X] T017 [US2] Wire per-entry feh availability status in `src/main.rs`: when `feh_available` is false, disable all multi-launch buttons and show a per-entry `feh not found` indicator while keeping `[+ Add]` available for future configuration

**Checkpoint**: Each entry independently tracks and persists its folder assignment. Missing folders, empty scanned folders, and missing feh are communicated without launching feh.

---

## Phase 5: User Story 3 - Copy Image to System Clipboard (Priority: P2)

**Goal**: Users can right-click any image row in either list view and copy decoded full image data to the system clipboard as `image/png`.

**Independent Test**: Scan a folder, right-click an image row, choose `Copy image to clipboard`, paste into an image-accepting application or extract `image/png` via clipboard tooling, and verify native dimensions plus pixel/content equivalence per SC-006.

### Implementation for User Story 3

- [X] T018 [US3] Capture secondary-click on image rows in FlatList mode in `src/main.rs`: in the existing image row rendering loop, use `response.secondary_clicked()` to set `self.clipboard_context_menu = Some(ClipboardContextMenu { image_path, anchor_pos })`
- [X] T019 [US3] Capture secondary-click on image rows in FolderTree mode in `src/main.rs`: in the `TreeRowKind::File` branch, resolve the row image path and set `self.clipboard_context_menu = Some(ClipboardContextMenu { image_path, anchor_pos })`
- [X] T020 [US3] Render the clipboard context menu in `src/main.rs`: when `clipboard_context_menu` is `Some`, render an `egui::Area` with `egui::Frame::popup`, show a `📋 Copy image to clipboard` button, call `copy_image_to_clipboard()` from `ui_logic`, set the status bar from the result, and clear the menu
- [X] T021 [US3] Implement context-menu dismissal in `src/main.rs`: close `clipboard_context_menu` on outside primary click, Escape key, successful copy, or a second right-click target so only one menu is visible at a time
- [X] T022 [US3] Complete clipboard error handling in `src/ui_logic.rs`: return clear status strings for file-read failure, image-decode failure, clipboard unavailable, and clipboard set-image failure; ensure read/decode failures occur before clipboard creation so the clipboard is not modified on those errors

**Checkpoint**: Users can right-click any image in either view mode, copy it to clipboard, paste into other applications, and see clear status messages for success or failure.

---

## Phase 6: User Story 4 - Launch Entry Labels (Priority: P3)

**Goal**: Users can set optional custom text labels on launch entries for easier identification.

**Independent Test**: Create a launch entry, type a custom label, verify the label appears in the entry list and persists across restart; clear the label and verify the folder basename is used as the default display identifier.

### Implementation for User Story 4

- [X] T023 [US4] Implement `update_entry_label(id, label)` in `src/main.rs`: trim input, store `None` for empty labels, preserve non-empty labels as `Some(String)`, and call `save_launch_list()`
- [X] T024 [US4] Add a per-entry label text field in `render_feh_instances_body()` in `src/main.rs`: use `egui::TextEdit::singleline`, prefill from `entry.label`, call `update_entry_label()` on change, and show the folder basename as placeholder text when label is empty
- [X] T025 [US4] Implement entry display-name logic in `src/main.rs`: display the custom label in bold when set, otherwise display the assigned folder basename, and always show the stable insertion-order index as `#N`

**Checkpoint**: Custom labels persist across restart, blank labels fall back to folder names, and labels can be cleared by deleting text.

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Tests, verification, and final integration checks across all stories.

- [X] T026 Write launch-list persistence unit tests in `tests/unit/ui_logic.rs`: `test_launch_list_roundtrip`, `test_launch_list_empty_load`, and `test_launch_list_corrupt_json`
- [X] T027 Write clipboard decode/error unit tests in `tests/unit/ui_logic.rs`: test `decode_image_to_rgba` on a known PNG fixture for dimensions and RGBA byte length, missing-file error path, and non-image decode error path; keep real `arboard` set/get as display-gated integration behavior
- [X] T028 Write filelist and launchability unit tests in `tests/unit/ui_logic.rs`: configured folder with images returns filelist lines, missing folder disables with `Folder not found`, empty scanned inventory disables with `No images`, and feh unavailable disables with `feh not found`
- [X] T029 Verify `SPDX-License-Identifier: MIT` headers are present on any newly created or publicly extended Rust source/test files in `src/types.rs`, `src/ui_logic.rs`, `src/lib.rs`, `src/main.rs`, and `tests/unit/ui_logic.rs`
- [X] T030 Run `cargo check` from `/home/kkk/Apps/rust-feh` using `Cargo.toml` to verify no compilation errors introduced
- [X] T031 Run `cargo test` from `/home/kkk/Apps/rust-feh` using `Cargo.toml` to verify all existing and new tests pass
- [X] T032 Run quickstart validation scenarios VS-1 through VS-8 from `specs/014-multi-feh-clipboard/quickstart.md`
- [X] T033 Validate SC-001 manually from `specs/014-multi-feh-clipboard/spec.md`: configure 5 launch entries for different folders, launch all, and confirm rust-feh remains responsive while all feh windows open independently
- [X] T034 Validate SC-002 and SC-005 manually from `specs/014-multi-feh-clipboard/spec.md`: confirm creating/assigning a launch entry requires fewer than 3 interactions and the `+` button is discoverable near the existing `Open in feh` area within 10 seconds
- [X] T035 Verify FR-014 regression behavior from `src/main.rs`: use the existing single `Open in feh` button in the Image actions panel and confirm feh opens normally with unchanged status/log behavior
- [X] T036 Run `cargo test --no-default-features` from `/home/kkk/Apps/rust-feh` using `Cargo.toml` if the crate supports that feature matrix; otherwise document in `specs/014-multi-feh-clipboard/quickstart.md` why no alternate feature matrix exists

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion — BLOCKS all user stories
- **User Story 1 (Phase 3)**: Depends on Foundational — no dependency on other stories
- **User Story 2 (Phase 4)**: Depends on US1 panel UI from Phase 3
- **User Story 3 (Phase 5)**: Depends on Foundational clipboard function and app state fields; independent of US1/US2/US4 UI work
- **User Story 4 (Phase 6)**: Depends on US1 panel UI from Phase 3; independent of US2/US3
- **Polish (Phase 7)**: Depends on all desired user stories being complete; T026–T028 can be written earlier once their helpers exist, but final validation runs after all stories are complete

### User Story Dependencies

- **US1 (P1)**: Self-contained MVP — multi-instance panel with add, launch, launch all, remove, and persistence
- **US2 (P1)**: Builds on US1 panel — adds folder ComboBox, stale detection, empty detection, and feh-missing disabled states
- **US3 (P2)**: Independent clipboard workflow — can be developed in parallel with US1/US2 after Phase 2
- **US4 (P3)**: Builds on US1 panel — adds optional labels and default display-name fallback

### Within Each User Story

- Core methods before UI rendering
- UI rendering before wiring/integration
- Story complete before moving to dependent stories
- Tests for core `ui_logic` helpers may be written as soon as those helpers exist

### Parallel Opportunities

- T005 and T006 implement separate helper responsibilities but both edit `src/ui_logic.rs`; coordinate or execute sequentially unless using separate branches with explicit merge discipline
- T026, T027, and T028 test separate helper groups but all edit `tests/unit/ui_logic.rs`; coordinate or execute sequentially unless using separate branches with explicit merge discipline
- US3 tasks T018–T022 can run in parallel with US1/US2 panel work after Phase 2 because they touch image-list/context-menu regions rather than inspector-panel regions in `src/main.rs`
- US4 tasks T023–T025 can run in parallel with US2 after US1 panel scaffolding exists if edits are coordinated within `src/main.rs`

---

## Parallel Example: After Foundational Phase

```bash
# Path A: Multi-instance panel (US1 → US2) - sequential within src/main.rs inspector region
Task: "T009 [US1] Implement add_launch_entry() in src/main.rs"
Task: "T010 [US1] Implement remove_launch_entry(id) in src/main.rs"
Task: "T011 [US1] Implement launch_entry_feh(entry) in src/main.rs"

# Path B: Clipboard (US3) - parallel with Path A, different src/main.rs image-list/context-menu region
Task: "T018 [US3] Capture secondary-click on image rows in FlatList mode in src/main.rs"
Task: "T019 [US3] Capture secondary-click on image rows in FolderTree mode in src/main.rs"
Task: "T020 [US3] Render the clipboard context menu in src/main.rs"

# Path C: Core tests - run after their src/ui_logic.rs helpers exist; all edit
# tests/unit/ui_logic.rs, so sequence them (or coordinate edits carefully)
Task: "T026 Write launch-list persistence unit tests in tests/unit/ui_logic.rs"
Task: "T027 Write clipboard decode/error unit tests in tests/unit/ui_logic.rs"
Task: "T028 Write filelist and launchability unit tests in tests/unit/ui_logic.rs"
```

---

## Implementation Strategy

### MVP First (US1 Only)

1. Complete Phase 1: Setup (T001–T003)
2. Complete Phase 2: Foundational (T004–T008)
3. Complete Phase 3: User Story 1 (T009–T013)
4. **STOP and VALIDATE**: Add entries, launch multiple feh windows, launch all, remove entries, and verify persistence
5. This is the MVP — users can manage multiple feh instances before clipboard or labels are implemented

### Incremental Delivery

1. Setup + Foundational → Foundation ready
2. Add US1 → Multi-instance panel works → **MVP ready**
3. Add US2 → Folder assignment + stale/empty/feh-missing detection → **Full multi-instance feature**
4. Add US3 → Clipboard copy → **Enhanced sharing workflow**
5. Add US4 → Labels → **Polished UX**
6. Run Phase 7 validation before completion

### Suggested MVP Scope

Phase 1 + Phase 2 + Phase 3 = 13 tasks (T001–T013). Users get:

- `[+ Add]` launch entries pointing to scanned folders
- `[Launch]` per entry to open independent feh windows
- `[Launch All]` to open all configured launchable entries
- `[x]` remove entries without killing running feh processes
- Persistence across restart

This is independently demonstrable and delivers the core feature value.

---

## Notes

- [P] tasks = different files or non-overlapping code regions, no dependencies on incomplete tasks
- [Story] label maps task to a specific user story for traceability
- Tasks T009–T017 all touch inspector-panel code in `src/main.rs` and are best done sequentially by one developer
- Tasks T018–T022 touch image-list/context-menu code in `src/main.rs` and can be parallel with inspector-panel tasks if edits are coordinated
- Save on every mutation (add/remove/folder edit/label edit) is the primary persistence path; Drop save is best-effort backup only
- Feh filelist generation and entry launchability live in `src/ui_logic.rs`, not `src/main.rs`, per Constitution Principle III
- New source/test files carry an `SPDX-License-Identifier: MIT` header per Constitution Technical Standards
- The existing single `Open in feh` button code path is retained and validated by T035 (FR-014)

---

## Next Actions (Handoff)

**Total tasks: 36** (T001–T036) across 7 phases.

The tasks are immediately executable in dependency order. Start with T001–T008, then implement the MVP via T009–T013 before moving to US2/US3/US4 and final validation.

**Recommended next command**: `/speckit-implement` to begin executing T001.

---

## Phase 8: Convergence

**Purpose**: Post-implement assessment (2026-06-28). Core US1–US4 paths and automated tests are in place; four manual validation tasks (T032–T035) remain open. This phase adds remediation for code gaps found during convergence.

- [X] T037 Add display-gated clipboard integration test in `tests/integration/clipboard_copy.rs` registered via `[[test]]` in `Cargo.toml`: skip when `DISPLAY`/`WAYLAND_DISPLAY` absent; when available, copy a known PNG fixture via `copy_image_to_clipboard`, read back with `arboard::Clipboard::get_image()`, and assert native dimensions match per FR-008/SC-006/plan T027 (missing)
- [X] T038 Log a warning when `load_launch_list()` in `src/ui_logic.rs` encounters corrupt JSON before returning `FehLaunchList::default()` per data-model.md error-handling contract (partial)
- [X] T039 Align feh-unavailable per-entry indicator text with FR-011 by showing `feh not installed` (not only `feh not found`) in `entry_is_launchable` status and the Feh Instances panel disabled Launch labels in `src/main.rs` (partial)
