# Tasks: Image List Presentation

**Input**: Design documents from `specs/005-image-list-presentation/`  
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/scan-inventory-ui.md, quickstart.md  
**Tests**: SC-004, SC-006 require `cargo test` coverage — test tasks included.  
**Generated**: 2026-06-22 via `/speckit-tasks`  
**Session**: [SESSION-2026-06-22-TRACEABILITY.md](../SESSION-2026-06-22-TRACEABILITY.md)

**Organization**: US1–US3 retroactive verify + gap-fill; US4–US5 new implementation. Depends on **009** `magick_available` for magick classify (can stub false in tests).

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Parallelizable (different files, no incomplete-task dependency)
- **[Story]**: US1–US5 on user-story phases only
- Exact file paths required

## Path Conventions

- Source: `src/types.rs`, `src/scanner.rs`, `src/ui_logic.rs`, `src/main.rs`, `src/tool_caps.rs`
- Tests: `tests/feature_005_list.rs`
- Spec: `specs/005-image-list-presentation/`

---

## Phase 1: Setup (Verification Baseline)

**Purpose**: Confirm build and existing 001 tests before 005 changes.

- [x] T001 Run `cargo build --release` in project root
- [x] T002 Run `cargo clippy -- -D warnings` and fix any pre-existing issues in `src/`
- [x] T003 Run `cargo test` and `./scripts/validate-feature-001.sh` — baseline green before 005 edits

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Shared types and scanner enrichment for US4–US5. **No user story work until T004 gap audit and T005–T007 types land.**

- [x] T004 Audit `src/ui_logic.rs` and `src/main.rs` against FR-001–FR-007 — write `specs/005-image-list-presentation/gap-audit.md` with pass/partial/missing per FR
- [x] T005 Add `FileStatus`, `ListViewMode`, `ScanInventory` to `src/types.rs` per `data-model.md`
- [x] T006 Extend `ImageEntry` in `src/types.rs` with `status: FileStatus` (default `NativeListed`)
- [x] T007 Add `ScanResult` struct in `src/scanner.rs` (entries, warnings, inventory) replacing tuple return; update `src/lib.rs` exports if needed
- [x] T008 Implement non-image file counting in `src/scanner.rs` during walk per FR-014 and research R1
- [x] T009 [P] Implement optional `magick identify` classify in `src/scanner.rs` gated on `tool_caps::ToolCapabilities::detect().magick_available`, cap 500 calls per research R2
- [x] T010 [P] Add `detect_converted_status(path)` helper in `src/ui_logic.rs` for `{stem}_processed.*` sibling per FR-012 and research R3
- [x] T011 Update `scan_directory` in `src/main.rs` to consume `ScanResult` and store `scan_inventory: Option<ScanInventory>`
- [x] T012 [P] Create `tests/feature_005_list.rs` with stub test `fn feature_005_test_harness_exists()`

**Checkpoint**: Types and scanner return new shape; main compiles with inventory state.

---

## Phase 3: User Story 1 - See Where Each Image Lives (Priority: P1) 🎯 MVP

**Goal**: Folder + Filename columns distinguish paths in recursive scans.

**Independent Test**: quickstart.md V1 — `sub` vs `sub/deep` rows distinguishable.

### Verify & gap-fill US1

- [x] T013 [US1] Verify Folder + Filename column headers in `src/main.rs` per FR-007
- [x] T014 [US1] Verify `relative_folder()` in `src/ui_logic.rs` returns `.` for root files per spec acceptance scenario 2
- [x] T015 [US1] Verify nested path `sub/deep` displays as folder column per FR-001
- [x] T016 [US1] Fix any US1 gaps from T004 in `src/main.rs` or `src/ui_logic.rs`

### Validate US1

- [x] T017 [US1] Run quickstart.md V1 manually or document result in `gap-audit.md`

**Checkpoint**: US1 independently verifiable (retroactive).

---

## Phase 4: User Story 2 - Sort the List (Priority: P1)

**Goal**: Path / Name / Folder sort with scroll reset on change.

**Independent Test**: quickstart.md V1 steps 3–4 — sort modes deterministic.

### Verify & gap-fill US2

- [x] T018 [US2] Verify `SortMode` in `src/types.rs` has Path, Name, Folder variants per FR-002
- [x] T019 [US2] Verify `list_indices()` in `src/ui_logic.rs` applies sort after filter per FR-004
- [x] T020 [US2] Verify sort change bumps `scroll_generation` in `src/main.rs` per FR-006
- [x] T021 [P] [US2] Add or extend unit tests for sort modes in `src/ui_logic.rs` `#[cfg(test)]` per SC-002

### Validate US2

- [x] T022 [US2] Run `cargo test ui_logic::tests::sort` (or full ui_logic tests) — all pass

**Checkpoint**: US2 independently verifiable.

---

## Phase 5: User Story 3 - Filter Matches Paths (Priority: P2)

**Goal**: Filter matches filename, folder, and full path; &lt;200ms @10k.

**Independent Test**: quickstart.md V1 step 5 + `sc003_filter_10k_under_200ms`.

### Verify & gap-fill US3

- [x] T023 [US3] Verify `filter_indices` / `entry_matches_search` in `src/ui_logic.rs` matches folder path per FR-003
- [x] T024 [US3] Verify case-insensitive UTF-8 substring behavior per FR-003
- [x] T025 [US3] Run `cargo test sc003_filter_10k_under_200ms` in `tests/feature_001_validation.rs` — pass per SC-003

### Validate US3

- [x] T026 [US3] Document filter edge case (0 matches) in `gap-audit.md` per spec edge cases

**Checkpoint**: US3 independently verifiable.

---

## Phase 6: User Story 5 - Scan Inventory Summary (Priority: P1)

**Goal**: Inventory bar with native / magick / converted / awaiting / non-image counts after scan.

**Independent Test**: quickstart.md V2 — mixed fixture counts.

**Note**: Implemented before US4 tree so folder lines can use inventory counts (plan P3 before P5).

### Implement US5

- [x] T027 [US5] Compute `ScanInventory` fields in `src/scanner.rs` including `awaiting_convert` invariant per FR-010–FR-011
- [x] T028 [US5] Apply `FileStatus` via `detect_converted_status` in `src/scanner.rs` or post-scan pass in `src/ui_logic.rs` per FR-012–FR-013
- [x] T029 [US5] Render inventory summary bar in `src/main.rs` CentralPanel per `contracts/scan-inventory-ui.md`
- [x] T030 [US5] Show magick-absent hint in inventory bar when `!tool_caps.magick_available` per spec US5 scenario 2
- [x] T031 [P] [US5] Add unit test mixed fixture inventory counts in `tests/feature_005_list.rs` per SC-006
- [x] T032 [P] [US5] Add unit test `awaiting_convert == magick_detected - converted` in `tests/feature_005_list.rs`

### Validate US5

- [x] T033 [US5] Run quickstart.md V2 with `/tmp/rust-feh-005-fixture` per quickstart.md

**Checkpoint**: US5 independently verifiable.

---

## Phase 7: User Story 4 - Browse by Folder Tree (Priority: P1)

**Goal**: Toggle Folder tree view with expandable hierarchy and per-folder counts.

**Independent Test**: quickstart.md V3 — expand/collapse; toggle back to flat.

### Implement US4

- [x] T034 [US4] Add `list_view_mode: ListViewMode` and `tree_expanded_paths` to `RustFehApp` in `src/main.rs` per `data-model.md`
- [x] T035 [US4] Add Flat list / Folder tree toggle UI in `src/main.rs` per FR-008 and contract view toggle
- [x] T036 [US4] Implement `build_folder_tree()` in `src/ui_logic.rs` from entries + inventory counts per FR-009
- [x] T037 [US4] Render lazy-expand tree rows in `src/main.rs` using `ScrollArea::show_rows` per research R4 and FR-005
- [x] T038 [US4] Apply filter to tree visible rows in `src/ui_logic.rs` per US4 acceptance scenario 4
- [x] T039 [P] [US4] Add unit test `build_folder_tree` fixture shape in `tests/feature_005_list.rs`

### Validate US4

- [x] T040 [US4] Run quickstart.md V3 — tree expand/collapse and flat toggle

**Checkpoint**: US4 independently verifiable.

---

## Phase 8: Polish & Cross-Cutting

**Purpose**: Status column, perf, docs, cross-feature alignment.

- [x] T041 [P] Add Status column to flat list in `src/main.rs` per FR-013 and contract flat columns
- [x] T042 Run `cargo test` and `./scripts/validate-feature-001.sh` — no regression per SC-007
- [x] T043 Update `specs/005-image-list-presentation/gap-audit.md` — all FR-001–FR-016 pass or documented deferral
- [x] T044 [P] Sync `specs/001-persistent-ui-virtual-browsing/data-model.md` with `ListViewMode`, `ScanInventory`, `FileStatus`
- [x] T045 [P] Update `README.md` Current Features for inventory bar, tree toggle, status column
- [x] T046 [P] Align `src/tool_caps.rs` format route notes with 005 inventory labels if drift found (coordination with **008**)
- [x] T047 Run full `specs/005-image-list-presentation/quickstart.md` V1–V4 checklist

---

## Dependencies & Execution Order

### Phase dependencies

```text
Phase 1 (Setup) → Phase 2 (Foundational) → Phase 3–5 (US1–US3 verify, parallel OK)
                                         → Phase 6 (US5) → Phase 7 (US4)
                                         → Phase 8 (Polish)
```

### User story dependencies

| Story | Depends on | Independent test |
|-------|------------|------------------|
| US1 | Phase 2 T004 only (verify) | quickstart V1 columns |
| US2 | US1 code paths | sort unit tests |
| US3 | US1–US2 | filter perf test |
| US5 | Phase 2 T005–T011 | quickstart V2 |
| US4 | US5 inventory + tree builder | quickstart V3 |

### Parallel opportunities

```bash
# After T007 lands:
T009 magick classify (scanner.rs) || T010 converted helper (ui_logic.rs)

# After US5 core:
T031 inventory tests || T032 invariant test || T039 tree tests

# Polish:
T044 data-model || T045 README || T046 tool_caps
```

---

## FR Traceability

| FR | Tasks |
|----|-------|
| FR-001–FR-007 | T004, T013–T026 (verify/gap-fill) |
| FR-008 | T034–T035, T040 |
| FR-009 | T036–T037 |
| FR-010–FR-011 | T027, T029, T031–T032 |
| FR-012–FR-013 | T010, T028, T041 |
| FR-014–FR-016 | T008–T009, T036, T021 |

---

## Implementation Strategy

### MVP (minimum shippable increment)

1. Complete Phase 1–2  
2. Phase 3–5 gap-audit (US1–US3) — confirms retroactive spec  
3. Phase 6 US5 inventory bar — delivers dinner-session “counts + magick vs converted” without tree  

### Full feature

4. Phase 7 US4 folder tree  
5. Phase 8 polish + cross-doc sync  

### Notes

- **Total tasks**: 54 (T001–T054)
- **Open**: 0 — T001–T054 complete

---

## Phase 9: Adversarial Remediation (2026-06-22)

**Input**: [adversarial-review.md](./adversarial-review.md)  
**Prerequisites**: Phase 8 complete

- [x] T048a Clarify FR-011, SC-005, resize, per-folder skipped in `spec.md`
- [x] T048b Sync `data-model.md` ScanInventory + FolderTreeNode rules
- [x] T048c Update `gap-audit.md` + `quickstart.md` + `research.md`
- [x] T049 Cache magick binary once per `scan_images` in `src/scanner.rs`
- [x] T050 `refresh_entry_and_inventory` after Quick resize in `src/main.rs`
- [x] T051 Tree `listed_count` = `NativeListed` only in `src/ui_logic.rs`
- [x] T052 `native_converted_fr011` unit test in `src/ui_logic.rs`
- [x] T053 SC-005 tree/inventory alignment tests in `ui_logic` + `feature_005_list.rs`
- [x] T054 `#[ignore]` heic magick integration test in `tests/feature_005_list.rs`
- Default view remains **Flat list** per research R5  
- Magick convert execution stays **010** — 005 only detects/classifies  
- If T009 perf fails @10k, record decision in `spec.md` Clarifications before disabling (007 advisory)