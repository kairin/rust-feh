# Tasks: External Tool Runtime

**Input**: Design documents from `specs/009-external-tool-runtime/`  
**Prerequisites**: plan.md (required), spec.md, research.md, data-model.md, contracts/tool-runtime-ui.md  
**Generated**: 2026-06-22 | **Supersedes**: 002 | **Status**: Complete (T001–T022)

**Tests**: Unit tests for `is_feh_not_found` classifier (SC-005); manual quickstart for GUI flows.

## Phase 1: Setup (Shared Infrastructure)

- [x] T001 Run `cargo test tool_caps` baseline in `src/tool_caps.rs`
- [x] T002 Run `cargo clippy -- -D warnings` on workspace
- [x] T003 Audit gap table in `specs/009-external-tool-runtime/plan.md` vs current `src/main.rs` and `src/tool_caps.rs`

---

## Phase 2: Foundational (Blocking Prerequisites)

- [x] T004 Verify `ToolCapabilities::detect()` covers feh + magick in `src/tool_caps.rs` per FR-001
- [x] T005 Verify `feh_available` derives from `tool_caps` at startup in `src/main.rs` per FR-002
- [x] T006 Add `is_feh_not_found` and `feh_confirmed_missing` helpers with unit tests in `src/tool_caps.rs` per research R3/R6 and SC-005

---

## Phase 3: User Story 1 - Install feh Mid-Session (Priority: P1) 🎯 MVP verify

- [x] T007 [US1] Verify panel **Recheck tools on PATH** calls `refresh_tool_caps` in `src/main.rs` per FR-003
- [x] T008 [US1] Verify recheck sets `feh_available` and enables `feh_button` controls in `src/main.rs` per FR-005
- [x] T009 [US1] Run quickstart V1 in `specs/009-external-tool-runtime/quickstart.md`

---

## Phase 4: User Story 2 - Install ImageMagick Mid-Session (Priority: P1)

- [x] T010 [US2] Verify `refresh_tool_caps` updates `magick_available` in `src/tool_caps.rs` per SC-004
- [x] T011 [US2] Run quickstart V4 in `specs/009-external-tool-runtime/quickstart.md`

---

## Phase 5: User Story 4 - Discover Recheck From Menu (Priority: P2) 🎯 MVP gap-fill #1

- [x] T012 [US4] Add Tools → **Recheck tools on PATH** in `src/main.rs` menu calling `refresh_tool_caps` per FR-004
- [x] T013 [US4] Verify menu and panel both call `refresh_tool_caps` only in `src/main.rs` per FR-005
- [x] T014 [US4] Run quickstart V2 in `specs/009-external-tool-runtime/quickstart.md`

---

## Phase 6: User Story 3 - Recover After feh Spawn Failure (Priority: P2) 🎯 MVP gap-fill #2

- [x] T015 [US3] Apply spawn-failure unavailable sync in `open_in_feh` in `src/main.rs` per FR-008 using `is_feh_not_found`
- [x] T016 [US3] Set `tool_caps.feh_available = false` and `status = feh_missing_status()` in `open_in_feh` in `src/main.rs` per contract and converge F4/F5
- [x] T017 [US3] Apply same spawn-failure sync in `set_wallpaper` in `src/main.rs` per FR-008 and converge F3
- [x] T018 [US3] Verify capabilities panel shows feh not installed after spawn failure in `src/main.rs` per US3/AC3
- [x] T019 [US3] Run quickstart V3 in `specs/009-external-tool-runtime/quickstart.md`

---

## Phase 7: Polish & Cross-Cutting Concerns

- [x] T020 Write `specs/009-external-tool-runtime/gap-audit.md` marking 002 superseded per FR-011
- [x] T021 [P] Add unit test that `ToolCapabilities::detect()` snapshot fields update in `src/tool_caps.rs` tests per SC-005
- [x] T022 Run `cargo test` and full `specs/009-external-tool-runtime/quickstart.md` validation

**Total**: 22 tasks complete