# Tasks: UI Feedback & Network Scan Polish

**Input**: [spec.md](./spec.md), [plan.md](./plan.md)

**Note**: Phases 1–6 fold pre-spec partial implementation from dogfood session into this feature.

## Phase 1: Setup

- [x] T001 Run `cargo test` baseline per SC-006
- [x] T002 Run `cargo clippy -- -D warnings` baseline

## Phase 2: US1 Network scan policy (P1)

- [x] T003 Add `is_network_mount_path` in `src/ui_logic.rs` per FR-002
- [x] T004 Gate `magick_available` with `!on_network` in `scan_directory` in `src/main.rs` per FR-001
- [x] T005 Log network-optimized scan message on network path pick per FR-005 / US1/AC2
- [x] T006 Unit test GVFS SMB path detection in `src/ui_logic.rs`

## Phase 3: US2 Live status bar (P1)

- [x] T007 Add `render_scanning_label` pulse + animated dots in `src/main.rs` per FR-003/FR-004
- [x] T008 Add `activity_pulse_color` fill + busy border on bottom panel per FR-003
- [x] T009 `request_repaint_after(200ms)` during scan per FR-004 / SC-002
- [x] T010 Network scan status copy in `render_scanning_label` per FR-005

## Phase 4: US3 Dependencies OK collapse (P2)

- [x] T011 ✅/⚠ dependencies header + per-dep icons in `render_tool_caps_panel` per FR-006
- [x] T012 `deps_section_open` collapsed when required OK at startup per FR-007
- [x] T013 Recheck collapses section when `!has_missing_required()` per FR-008
- [x] T014 Controlled `CollapsingHeader::open` + header click toggle per US3/AC3

## Phase 5: US4 Bottom-bar speed tips (P2)

- [x] T015 Move `rotating_operation_tip` to bottom status bar per FR-009
- [x] T016 4s rotation interval + spinner glyph per FR-010 / US4/AC2–AC3
- [x] T017 Confirm `operation_timings` absent from right Tools panel per FR-009

## Phase 6: US5 Panel separation + detach log (P2)

- [x] T018 `Frame::group` on inventory, image list, activity log per FR-011
- [x] T019 `activity_log_detached` + `egui::Window` with `.open()` per FR-012/FR-013
- [x] T020 `render_activity_log_body` shared in attached/detached modes per FR-014 / US5/AC5
- [x] T021 Reattach placeholder + buttons in main and detached window per US5/AC4

## Phase 7: Build polish

- [x] T022 `cargo build --release` + copy `rust-feh` binary
- [x] T023 Full `cargo test` + clippy pass after UI changes

## Phase 8: Convergence

- [x] T024 Write `gap-audit.md` FR/SC traceability for all FR-001–FR-014 per plan.md (missing)
- [x] T025 Write `quickstart.md` with V1–V5 manual steps for SC-001–SC-005 per spec.md (missing)
- [x] T026 Write `validation-results.md` recording SMB dogfood outcomes for SC-001, SC-002, SC-003, SC-004, SC-005 (missing)
- [x] T027 Add unit tests for NFS (`/nfs/`) and UNC (`//`) paths in `is_network_mount_path` per FR-002 (partial)
- [x] T028 Extract `scan_magick_enabled(magick_on_path, root: &Path) -> bool` to `src/ui_logic.rs` with unit test per FR-001 / Constitution III (partial)
- [x] T029 Show network-aware inventory hint when scan root is network path (identify skipped) per US1/AC3 (partial)
- [x] T030 Update `README.md` bottom-bar tips, deps collapse, detach log, NAS scan policy per shipped UI (unrequested)
- [x] T031 Update `specs/OUTSTANDING-ISSUES-ROADMAP.md` with 012-ui-feedback-polish status (unrequested)
- [x] T032 Update `AGENTS.md` feature pointer when Phase 8 complete (unrequested)