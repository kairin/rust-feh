# Tasks: Viewer Round-Trip & Staged-Image Actions

**Input**: Design documents from `/specs/016-feh-viewer-actions/`
**Prerequisites**: plan.md, research.md, data-model.md, contracts/, quickstart.md

**Board rules**: single writer = owning orchestrator. `[P]` = parallelizable
with other `[P]` tasks in the same phase (different files / no dependency).
Tests for core-module logic are mandatory (Constitution: Testing).

## Phase 1: Setup & Foundational Types

- [ ] T001 Baseline green: `cargo check && cargo test` on branch tip; record counts here
- [ ] T002 [P] Add types in `src/types.rs`: `StageState`, `StagedImage`, `ContextAction`, `ActionOutcome`, `ActionPrefs` (version-tagged, serde) per data-model.md
- [ ] T003 [P] `src/ui_logic.rs`: `collision_suffixed_path(dest_dir, file_name)` + unit tests (existing name, gaps, dotfiles, no-extension, non-ASCII)
- [ ] T004 [P] `src/ui_logic.rs`: `stage_decode_bounds(w, h, max_edge)` + unit tests (portrait/landscape/small-no-upscale/zero guards)
- [ ] T005 `src/ui_logic.rs`: `action_prefs_path()` / `save_action_prefs` / `load_action_prefs` mirroring window-prefs pattern + unit tests (missing file, bad json, round-trip)

**Checkpoint**: `cargo test` green; no GUI changes yet.

## Phase 2: User Story 1 — Stage pane + context actions (P1)

### Core logic (egui-free)

- [ ] T006 [US1] `src/image_proc.rs`: `decode_stage_rgba(path, max_edge) -> Result<(w,h,Vec<u8>), String>` reusing existing decode; unit tests incl. undecodable-file error path
- [ ] T007 [US1] `src/ui_logic.rs`: loss-proof move — `plan_loss_proof_move` + `execute_move_plan` (copy→verify size→rename→remove; same-fs rename fast path; temp-file naming; cleanup-on-error reporting) + unit tests with tempdirs (cross-dir move, collision at destination, unwritable destination, source preserved on failure)
- [ ] T008 [US1] [P] `src/ui_logic.rs`: `save_copy_to(src, dest_dir)` collision-safe copy + unit tests
- [ ] T009 [US1] [P] `src/ui_logic.rs`: context-action outcome formatting for activity log (`format_action_outcome`) + unit tests
- [ ] T010 [US1] Integration test `tests/integration/feature_016_actions.rs`: save-copy / move / resize-copy / convert against tempdir fixtures incl. spaces + non-ASCII names and duplicate-name destinations (SC-003, SC-004)

### GUI wiring (thin, main.rs)

- [ ] T011 [US1] Stage pane render in central panel per contracts/stage-context-menu.md: off-thread decode worker (channel + generation counter), texture cache keyed (path, generation), Loading/Failed states; keep functions ≤ Codacy limits by delegating to helpers
- [ ] T012 [US1] Context menu via `Response::context_menu`: exactly the six items, disabled-not-hidden rules for undecodable files (FR-012), dialog-pending refusal
- [ ] T013 [US1] Wire SaveCopyTo/MoveTo to rfd folder chooser (start at `ActionPrefs.last_destination`, persist on use); wire ResizeCopy/ConvertFormat to existing Image Tools routines; CopyPath/CopyImage via arboard (image fallback → path + message)
- [ ] T014 [US1] Failure surfacing: rfd error dialog (image + cause) + ActionOutcome to activity log for every action (FR-010); move advances stage to next surviving image

**Checkpoint**: US1 independently testable — quickstart V1 + V4 + V5 runnable; `cargo test` green.

## Phase 3: User Story 2 — feh round-trip (P1)

- [ ] T015 [US2] `src/types.rs` + `src/ui_logic.rs`: `ViewerRoundTrip` state + `viewer_spawn_command(...)` building the exact contract args/env (arg-vector only, `--info` trail, XDG_CONFIG_HOME) + unit tests asserting the full argument vector (incl. adversarial filenames passed as args, never shell)
- [ ] T016 [US2] `src/ui_logic.rs`: `validate_handoff(content, filelist)` — trim, canonicalize, membership check; deleted-image nearest-neighbor fallback; + unit tests (valid, not-in-list, garbage, empty, symlink escape attempt, deleted image)
- [ ] T017 [US2] Handoff lifecycle helpers: handoff path naming (`handoff-<pid>-<id>`), stale-handoff cleanup at startup + unit tests
- [ ] T018 [US2] main.rs: spawn round-trip viewer from the existing Open-in-feh path (retain Child), per-frame `try_wait` polling loop with idle repaint tick; on exit: validate → select+scroll+stage → filter-mismatch surfacing (US2-3) → delete handoff → log (launched_with → landed_on)
- [ ] T019 [US2] Integration test `tests/integration/feature_016_roundtrip.rs`: simulate viewer exit by writing handoff files + invoking the exit-handling logic directly (no GUI): correct image, unchanged case, tampered content ignored, most-recent-close-wins with two round-trips (SC-002 logic tier)

**Checkpoint**: US2 logic green; quickstart V2 runnable end-to-end with real feh.

## Phase 4: User Story 3 — Isolated viewer profile (P2)

- [ ] T020 [US3] `src/ui_logic.rs`: `viewer_profile_dir()` (create-if-missing, empty) + spawn env injection already covered by T015; unit test: profile dir creation idempotent
- [ ] T021 [US3] Verify non-round-trip feh launches (wallpaper, Launch All) are byte-identical in args/env to pre-feature behavior (regression assertions in existing tests)

**Checkpoint**: quickstart V3 runnable (stock nav, no overlays, personal config untouched).

## Phase 5: Polish & Cross-Cutting

- [ ] T022 [P] `docs/POSITIONING.md`: add the "selection stage ≠ viewer replacement; feh remains the viewer engine" clarification (plan's Constitution I note)
- [ ] T023 [P] README: brief stage + round-trip description; discoverability line near viewer-launch controls (contracts/stage-context-menu.md)
- [ ] T024 [P] Activity-log copy review: round-trip and action messages readable and consistent with existing log style
- [ ] T025 Full gates: `cargo test`, `cargo clippy -- -D warnings` (CI), `cargo fmt --check` (CI), release build; Codacy limits respected
- [ ] T026 Scripted GUI validation per quickstart V1–V6; record results in `validation-results.md` (env: synthetic input unavailable — use start-folder hook + screenshots + real feh close; V6 uses perf fixture + `.agents/ENV.md` baseline)
- [ ] T027 **Security review (hard gate)**: `/security-review` on the full diff — explicit attention: spawn arg-vector construction (no shell), handoff-file trust + canonicalization (R4), symlink/path traversal in save/move destinations, temp-file handling in loss-proof move. A failed review blocks merge.
- [ ] T028 Update `.agents/BOARD.md` + BACKLOG status for 016; PR with green CI + Codacy

**Dependencies**: T001 → all. T002 → T003–T005. Phase 2 core (T006–T010) before GUI (T011–T014). T015–T017 before T018–T019. T025–T028 last, in order.
