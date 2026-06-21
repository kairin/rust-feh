# Tasks: Browsing Experience Round

**Input**: [spec.md](./spec.md), [plan.md](./plan.md)

## Phase 1: Setup

- [x] T001 Run `cargo test` baseline
- [x] T002 Run `cargo clippy -- -D warnings` baseline

## Phase 2: Foundational (scanner warnings)

- [x] T003 Add `format_walk_warning` + non-permission `Scan skip:` in `src/scanner.rs`
- [x] T004 Add `summarize_scan_warnings` cap at 50 in `src/scanner.rs`
- [x] T005 Unit test `summarize_scan_warnings` and warning format in `src/scanner.rs`

## Phase 3: US1 feh filelist

- [x] T006 Add `write_feh_filelist` + `feh_filelist_temp_path` in `src/ui_logic.rs`
- [x] T007 Unit test filelist write order in `src/ui_logic.rs`
- [x] T008 Change `open_in_feh` to `--filelist` + filtered indices in `src/main.rs`

## Phase 4: US2 background scan

- [x] T009 Add `ScanComplete` + thread spawn in `scan_directory` in `src/main.rs`
- [x] T010 Poll `scan_rx` in `update()` + `scan_generation` stale drop in `src/main.rs`
- [x] T011 Extract `apply_scan_result` from sync scan path in `src/main.rs`

## Phase 5: US4 copyable log

- [x] T012 Add `join_activity_log` in `src/ui_logic.rs`
- [x] T013 Replace Debug Log UI with selectable TextEdit + Copy log/status in `src/main.rs`

## Phase 6: Polish

- [x] T014 Run full `cargo test` + clippy
- [x] T015 Write `gap-audit.md` FR traceability
- [x] T016 Update `AGENTS.md` feature pointer

## Phase 7: Convergence

- [x] T017 Make Activity log multiline view read-only while keeping text selectable per FR-009 / US4/AC1 (partial)
- [x] T018 Add `t069_scan_skip_non_permission` integration test in `tests/feature_001_validation.rs` per plan.md and 004-FR-006 (missing)
- [x] T019 Ensure `open_in_feh` `--start-at` path is always in the filelist; clear or re-resolve selection when filter excludes it per FR-002 / US1/AC1 (partial)
- [x] T020 Guard Open in feh when filtered list is empty or scan in progress per spec edge case / US1 (partial)
- [x] T021 Add unit test that `write_feh_filelist` path order matches `list_indices` for Path/Name/Folder sorts per US1/AC2 (partial)
- [x] T022 Add `validation-results.md` recording manual quickstart V1–V4 outcomes for SC-001, SC-002, SC-005 (missing)
- [x] T023 Add `format_walk_warning` unit tests for Permission denied vs `Scan skip:` prefixes per FR-007 / FR-006 / 004-FR-003 (partial)
- [x] T024 Update `README.md`: Activity log (not debug log), background scan shipped — remove stale "async scanning" future bullet per 011 traceability (unrequested)
- [x] T025 Update `specs/OUTSTANDING-ISSUES-ROADMAP.md`: mark `004-scanner-resilience` absorbed by 011, note `011-browsing-experience-round` shipped (unrequested)
- [x] T026 Append gap-audit deferral notes for SESSION items not in scope: `feh --conversion-timeout`, wallpaper mode variants → future feature (unrequested)