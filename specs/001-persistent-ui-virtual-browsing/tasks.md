# Tasks: Persistent UI Layout & Virtual Browsing

**Input**: Design documents from `specs/001-persistent-ui-virtual-browsing/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, quickstart.md
**Tests**: Not explicitly requested. Verification uses quickstart.md scenarios V1–V10.
**Generated**: 2026-06-21 via `/speckit-tasks` (regenerated after `/speckit-clarify` and FR-001–FR-015 expansion)

**Note**: Core layout exists in `src/main.rs` (TopBottomPanel, show_rows, no-auto-feh). Remaining work is gap-fill per `gap-audit.md` plus validation. **Decomposed fix steps**: `remediation.md` (Phases A–H). T004 artifact (`gap-audit.md`) created 2026-06-21 via `/speckit-clarify`; code gaps remain until implement tasks complete. **Implement status (2026-06-21)**: Phases A–H complete in code; 52/64 tasks checked. Manual quickstart V1–V10 + T055/T056/T059 pending.

**Organization**: Tasks grouped by user story. Each story has verify → implement → validate phases.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies on incomplete tasks)
- **[Story]**: US1, US2, or US3 for user-story phase tasks only
- Include exact file paths in descriptions

## Path Conventions

- **Single project**: `src/` at repository root
- Source files: `src/main.rs`, `src/scanner.rs`, `src/types.rs`, `src/image_proc.rs`
- Spec artifacts: `specs/001-persistent-ui-virtual-browsing/`

---

## Phase 1: Setup (Verification Baseline)

**Purpose**: Confirm the project builds and passes basic quality checks.

- [x] T001 Build project in release mode: `cargo build --release` in project root
- [x] T002 Run clippy with warnings-as-errors: `cargo clippy -- -D warnings` and fix any issues in `src/`
- [x] T003 Run `cargo test` to confirm existing tests pass

---

## Phase 2: Foundational (Code Audit & Shared State)

**Purpose**: Audit code against FR-001–FR-015, verify core modules, add shared state used by multiple stories.

**⚠️ CRITICAL**: No user story work begins until T004 gap audit is complete.

- [x] T004 Audit `src/main.rs` and `src/scanner.rs` against FR-001 through FR-015 — gap list in `gap-audit.md` (artifact created 2026-06-21; re-verify after implement and update Status columns to pass)
- [x] T005 [P] Verify `src/scanner.rs` walkdir config: `follow_links(false)`, supported formats, sort order unchanged
- [x] T006 [P] Verify `src/types.rs` has `ImageEntry` defined; note `Selection`/`SortMode` dead-code status for T042
- [x] T007 [P] Verify `src/image_proc.rs` exports `process_image`, `ProcessOptions`, `has_external_magick` unchanged
- [x] T008 [P] Grep `src/` for old maintainer traces: `grep -ri "fa7ad|fahad|8bit.demoncoder|nfeh|gq.fahad|@fa7ad" src/` — zero hits
- [x] T009 [P] Confirm `archive/original-nfeh/` exists but is NOT referenced by `src/` or `Cargo.toml`
- [x] T010 Add `feh_available: bool` to `RustFehApp` in `src/main.rs` — detect via `which::which("feh")` at startup per FR-008a and data-model.md
- [x] T011 Add `scanning: bool` to `RustFehApp` in `src/main.rs` — set true at scan start, false at completion; drive "Scanning…" status per FR-010/FR-013
- [x] T012 Add `list_scroll_offset: f32` (or equivalent) to `RustFehApp` in `src/main.rs` for filter scroll reset per FR-005 and data-model.md
- [x] T013 [P] Extend `src/scanner.rs` to collect permission-denied skip messages per FR-015; return alongside `Vec<ImageEntry>` or via out-parameter
- [x] T014 Wire scanner skip messages into `debug_logs` via `log()` in `scan_directory` in `src/main.rs` per Clarifications session 2026-06-21

**Checkpoint**: Gap audit complete, shared state fields present, scanner logging wired.

---

## Phase 3: User Story 1 - Always-Visible Controls (Priority: P1) 🎯 MVP

**Goal**: Folder picker, filter, actions, menu bar, and status counter always reachable on large collections.

**Independent Test**: quickstart.md V1 — load 5k+ images, scroll to bottom, controls and bottom-bar counter remain visible.

### Verify US1

- [x] T015 [US1] Verify `TopBottomPanel::top("controls")` in `src/main.rs` has Choose folder + menu bar always visible per FR-001
- [x] T016 [US1] Verify loaded-folder controls in top panel: path, filter, recursive, rescan per FR-001
- [x] T017 [US1] Verify action buttons (Open in feh, Set as wallpaper, Quick resize) in top panel when folder loaded per FR-002
- [x] T018 [US1] Verify bottom `TopBottomPanel::bottom("status")` shows status + "Showing X / Y images" per FR-003/FR-006/SC-006 — counter NOT in CentralPanel
- [x] T019 [US1] Verify empty-state: filter/actions hidden or disabled before folder load per FR-001 acceptance scenario 1
- [x] T020 [US1] Verify menu bar actions functional per FR-011: File→Choose folder, File→Rescan, View→Include subfolders, Tools→Open in feh (quickstart V7)

### Implement US1 gaps

- [x] T021 [US1] Move "Showing X / Y images" counter from CentralPanel to bottom status bar in `src/main.rs` per FR-006
- [x] T022 [US1] Implement File→"Choose folder..." to open `rfd::FileDialog` and reuse folder-load logic in `src/main.rs` per FR-011
- [x] T023 [US1] Implement File→"Rescan" to call `scan_directory` on `current_dir` in `src/main.rs` per FR-011
- [x] T024 [US1] Implement View→"Include subfolders" checkbox to toggle `recursive` and trigger rescan in `src/main.rs` per FR-011
- [x] T025 [US1] Implement Tools→"Open in feh" with same guard as toolbar button in `src/main.rs` per FR-011

### Validate US1

- [ ] T026 [US1] Run quickstart.md V1 validation scenario end-to-end
- [x] T027 [US1] Fix any remaining US1 gaps from T004 audit in `src/main.rs` (FR-001, FR-002, FR-003, FR-006, FR-011)

**Checkpoint**: US1 independently verifiable.

---

## Phase 4: User Story 2 - Smooth Browsing (Priority: P1)

**Goal**: Virtualized list smooth on 10k+ images; filter <200ms; scan state visible.

**Independent Test**: quickstart.md V2, V4, V5, V9, V10.

### Verify US2

- [x] T028 [US2] Verify `show_rows` in `src/main.rs` uses pre-computed filtered indices, row height 18.0, no re-filter in closure per FR-004
- [x] T029 [US2] Verify filter is case-insensitive UTF-8 substring, O(n) per frame per FR-005
- [x] T030 [US2] Verify recursive toggle triggers rescan per FR-010
- [x] T031 [US2] Verify status shows "Scanning…" during scan per FR-010/FR-013 (quickstart V9)
- [x] T032 [US2] Verify filter change resets scroll to top per FR-005 (quickstart V10)

### Implement US2 gaps

- [x] T033 [US2] Reset ScrollArea offset to 0.0 when `search` changes in `src/main.rs` per FR-005
- [x] T034 [US2] Show "Scanning…" in bottom status bar while `scanning == true` in `src/main.rs` per FR-010
- [x] T035 [US2] Clear image list immediately on new folder pick before scan starts in `src/main.rs` per FR-013

### Validate US2

- [ ] T036 [US2] Run quickstart.md V2: 10k scroll smooth, RSS under 150MB (`ps -o rss= -p $(pgrep rust-feh)`) per SC-004
- [ ] T037 [US2] Run quickstart.md V4: filter counter accuracy, 0-match case, empty dir per FR-005/FR-006
- [ ] T038 [US2] Run quickstart.md V5: recursive toggle rescans correctly per FR-010
- [ ] T039 [US2] Measure filter response from last keystroke — MUST be under 200ms per SC-003
- [x] T040 [US2] Fix any remaining US2 gaps from T004 audit in `src/main.rs` (FR-004, FR-005, FR-010, FR-013)

**Checkpoint**: US2 independently verifiable.

---

## Phase 5: User Story 3 - Clear Selection vs. Open Model (Priority: P2)

**Goal**: Load selects first image without auto-feh; explicit open; feh degradation.

**Independent Test**: quickstart.md V3, V8.

### Verify US3

- [x] T041 [US3] Verify no `open_in_feh` call in folder-load path in `src/main.rs` per FR-007
- [x] T042 [US3] Verify auto-select-first after load with images; no selection when empty per FR-007
- [x] T043 [US3] Verify Open in feh / Set wallpaper check `selected.is_some()` per FR-008
- [x] T044 [US3] Verify `scan_directory` clears `selected` at start; re-selects first after scan per FR-012
- [x] T045 [US3] Verify feh buttons use `ui.add_enabled(feh_available, …)` per FR-008a/SC-007
- [x] T046 [US3] Verify feh spawn failure shows descriptive status, no crash per FR-008b

### Implement US3 gaps

- [x] T047 [US3] Disable Open in feh and Set as wallpaper buttons when `feh_available == false` in `src/main.rs` per FR-008a
- [x] T048 [US3] Show persistent "feh not found — install with `sudo apt install feh`" in status bar at startup when feh absent in `src/main.rs` per FR-008a
- [x] T049 [US3] Ensure selection lifecycle in `scan_directory`: clear at start, auto-select first on completion in `src/main.rs` per FR-012

### Validate US3

- [ ] T050 [US3] Run quickstart.md V3: no auto-feh, explicit open works per SC-005
- [ ] T051 [US3] Run quickstart.md V8: feh missing — disabled buttons, no spawn per SC-007
- [x] T052 [US3] Fix any remaining US3 gaps from T004 audit in `src/main.rs` (FR-007, FR-008, FR-008a, FR-008b, FR-012)

**Checkpoint**: US3 independently verifiable.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Debug log, error paths, docs, final quality sweep.

- [x] T053 [P] Verify debug log collapsible, collapsed by default, empty state "(no debug messages yet)" per FR-009 in `src/main.rs`
- [x] T054 [P] Verify Quick resize error shows "Process error: …" in status per FR-014 in `src/main.rs`
- [ ] T055 [P] Verify scanner skip warnings appear in debug log per FR-015 (permission-denied subdir test)
- [ ] T056 [P] Remove or implement dead code: `Selection` and `SortMode` in `src/types.rs`
- [x] T057 [P] Remove commented tokio/flume lines from `src/main.rs:50` if not planned this phase
- [ ] T058 Run quickstart.md V6: debug log functional, expandable, clearable
- [x] T059 [P] Update `README.md` with persistent layout, virtualized list, selection model, feh degradation
- [x] T060 Verify `Cargo.toml` has no new dependencies beyond existing set per Constitution §II
- [x] T061 Final clippy: `cargo clippy -- -D warnings` — zero warnings
- [x] T062 Final build: `cargo build --release` → `target/release/rust-feh`
- [x] T063 Re-run maintainer-trace grep repo-wide excluding `archive/` and `target/` — zero hits
- [x] T064 Run `./build-and-place.sh` to place binary at project root

---

## FR-to-Task Traceability

| FR | Primary Tasks |
|----|---------------|
| FR-001 | T015, T016, T019, T021–T025, T027 |
| FR-002 | T017, T027 |
| FR-003 | T018, T021, T027 |
| FR-004 | T028, T040 |
| FR-005 | T029, T032, T033, T037, T039, T040 |
| FR-006 | T018, T021, T027, T037 |
| FR-007 | T041, T042, T052 |
| FR-008 | T043, T052 |
| FR-008a | T010, T045, T047, T048, T051, T052 |
| FR-008b | T046, T052 |
| FR-009 | T053, T058 |
| FR-010 | T011, T030, T031, T034, T038, T040 |
| FR-011 | T020, T022–T025, T027 |
| FR-012 | T044, T049, T052 |
| FR-013 | T031, T035, T040 |
| FR-014 | T054 |
| FR-015 | T013, T014, T055 |

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1** → **Phase 2** → **Phases 3–5** (parallel after Phase 2) → **Phase 6**
- T004 blocks all story phases
- T010–T014 (shared state) block T021–T035 implementation tasks

### User Story Dependencies

- **US1, US2, US3**: Independent after Phase 2; may run in parallel
- Recommended order: US1 → US2 → US3 (US1 layout fixes unblock counter/menu work US2/US3 rely on)

### Parallel Opportunities

```bash
# Phase 2 after T004:
T005, T006, T007, T008, T009, T013  # parallel module checks + scanner extension

# After Phase 2, stories in parallel:
# Dev A: US1 (T015–T027)
# Dev B: US2 (T028–T040)
# Dev C: US3 (T041–T052)

# Phase 6:
T053, T054, T055, T056, T057, T059  # parallel polish
```

---

## Implementation Strategy

### MVP (US1 + US2)

1. Phase 1: Setup
2. Phase 2: Audit + shared state (T004–T014)
3. Phase 3: US1 layout/menu/counter fixes
4. Phase 4: US2 virtualization/scan/filter fixes
5. **STOP and VALIDATE** V1, V2, V4, V5, V9, V10

### Full Delivery

6. Phase 5: US3 selection/feh degradation
7. Phase 6: Polish + final validation (V3, V6, V7, V8)

---

## Notes

- Total tasks: **69** (T001–T069 after Phase 7 Convergence 2026-06-21)
- US1: 13 tasks | US2: 13 tasks | US3: 12 tasks | Foundational: 11 | Setup: 3 | Polish: 12 | Convergence: 5
- Gap audit format defined in Clarifications session 2026-06-21
- `cargo clippy -- -D warnings` is the automated quality gate
- Issue classification: `outstanding-issues.md`

### Superseded tasks (do not duplicate work)

| Open tasks (Phase 3–6) | Superseded by | Notes |
|------------------------|---------------|-------|
| T026, T036, T037, T038, T039, T050, T051, T058 | **T067** | One quickstart V1–V10 session |
| T055 | **T068** | Permission-denied validation |
| T056 | **T069** | Dead code removal |
| T012, T045 artifact wording | **T066** | Docs synced; tasks remain `[x]` |

Leave superseded checkboxes unchanged for audit trail; close via convergence tasks.

---

## Phase 7: Convergence (2026-06-21)

**Purpose**: Close post-implementation adversarial findings; consolidate 10 open tasks into 5.

**Reference**: `outstanding-issues.md`, `adversarial-review.md` (post-implementation)

- [x] T065 Append feh-not-found to post-scan status when `!feh_available` via `post_scan_status()` in `src/main.rs` per FR-008a (partial)
- [x] T066 Sync artifacts: `data-model.md` (`scroll_generation`), `spec.md` clarify session, `gap-audit.md` two-tier columns per outstanding-issues Bucket A (partial)
- [x] T067 **Automated validation** — `./scripts/validate-feature-001.sh` + `cargo test` (replaces manual V1–V10 where automatable); results in `validation-results.md`. SC-002/SC-004 (60fps/RSS) remain manual-only (partial)
- [x] T068 Verify FR-015: `t068_permission_denied_warning` in `tests/feature_001_validation.rs` (partial)
- [x] T069 Remove dead `Selection` and `SortMode` from `src/types.rs` per T056 supersession (unrequested)
