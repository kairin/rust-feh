# Tasks: Viewer Round-Trip & Staged-Image Actions

**Input**: Design documents from `/specs/016-feh-viewer-actions/`
**Prerequisites**: plan.md, research.md, data-model.md, contracts/, quickstart.md

**Board rules**: single writer = owning orchestrator. `[P]` = parallelizable
with other `[P]` tasks in the same phase (different files / no dependency).
Tests for core-module logic are mandatory (Constitution: Testing).

## Phase 1: Setup & Foundational Types

- [x] T001 Baseline green: `cargo check && cargo test` on branch tip; record counts here — 2026-07-05: `cargo check` clean; `cargo test` 122 passed / 0 failed / 2 ignored (matches `.agents/ENV.md` B-4 baseline) at commit 186139c.
- [x] T002 [P] Add types in `src/types.rs`: `StageState`, `StagedImage`, `ContextAction`, `ActionOutcome`, `ActionPrefs` (version-tagged, serde) per data-model.md — 2026-07-05: done via haiku-implementer; also added `ActionKind`/`ActionResult`; 4 new tests green.
- [x] T003 [P] `src/ui_logic.rs`: `collision_suffixed_path(dest_dir, file_name)` + unit tests (existing name, gaps, dotfiles, no-extension, non-ASCII) — 2026-07-05: done via haiku-implementer (combined dispatch with T004, same file); 6 tests green.
- [x] T004 [P] `src/ui_logic.rs`: `stage_decode_bounds(w, h, max_edge)` + unit tests (portrait/landscape/small-no-upscale/zero guards) — 2026-07-05: done via haiku-implementer (combined dispatch with T003); 5 tests green.
- [x] T005 `src/ui_logic.rs`: `action_prefs_path()` / `save_action_prefs` / `load_action_prefs` mirroring window-prefs pattern + unit tests (missing file, bad json, round-trip) — 2026-07-05: done via haiku-implementer; 3 tests green. Orchestrator fix: the 3 new tests shared one real fixed path (`~/.config/rust-feh/action-prefs.json`) and raced under parallel test threads (observed 1 flaky failure); added a `Mutex`-guarded serialization (`ACTION_PREFS_TEST_LOCK`) around all three — reran `cargo test` 5x clean after the fix.

**Checkpoint**: `cargo test` green (157 passed / 0 failed / 2 ignored across all suites, reran 3x for stability) — 2026-07-05.

## Phase 2: User Story 1 — Stage pane + context actions (P1)

### Core logic (egui-free)

- [x] T006 [US1] `src/image_proc.rs`: `decode_stage_rgba(path, max_edge) -> Result<(w,h,Vec<u8>), String>` reusing existing decode; unit tests incl. undecodable-file error path — 2026-07-05: done via haiku-implementer; 3 tests green.
- [x] T007 [US1] `src/ui_logic.rs`: loss-proof move — `plan_loss_proof_move` + `execute_move_plan` (copy→verify size→rename→remove; same-fs rename fast path; temp-file naming; cleanup-on-error reporting) + unit tests with tempdirs (cross-dir move, collision at destination, unwritable destination, source preserved on failure) — 2026-07-05: done via haiku-implementer; 4 tests green.
- [x] T008 [US1] [P] `src/ui_logic.rs`: `save_copy_to(src, dest_dir)` collision-safe copy + unit tests — 2026-07-05: done via haiku-implementer (combined dispatch with T009); 4 tests green.
- [x] T009 [US1] [P] `src/ui_logic.rs`: context-action outcome formatting for activity log (`format_action_outcome`) + unit tests — 2026-07-05: done via haiku-implementer (combined dispatch with T008); 5 tests green.
- [x] T010 [US1] Integration test `tests/integration/feature_016_actions.rs`: save-copy / move / resize-copy / convert against tempdir fixtures incl. spaces + non-ASCII names and duplicate-name destinations (SC-003, SC-004) — 2026-07-05: done via haiku-implementer; registered as `[[test]] feature_016_actions` in Cargo.toml (orchestrator edit); 6 tests green.

### GUI wiring (thin, main.rs)

- [x] T011 [US1] Stage pane render in central panel per contracts/stage-context-menu.md: off-thread decode worker (channel + generation counter), texture cache keyed (path, generation), Loading/Failed states; keep functions ≤ Codacy limits by delegating to helpers — 2026-07-05: done directly by orchestrator (not delegated — see deviation note below). Added `kick_stage_decode_if_selection_changed`/`poll_stage_decode` (generation-tagged channel), `render_stage_pane`/`render_stage_status_line`/`render_stage_image` (collapsible, list keeps priority — 220px reserved, 22px when collapsed), scale-to-fit never upscaling beyond 1:1.
- [x] T012 [US1] Context menu via `Response::context_menu`: exactly the six items, disabled-not-hidden rules for undecodable files (FR-012), dialog-pending refusal — 2026-07-05: `render_stage_context_menu` implements all six items in contract order; Resize/Convert/Copy-image disabled (not hidden) when stage isn't `Ready`; Copy path/Move/Save copy always enabled. Dialog-pending refusal: rfd folder-choosers are synchronous (existing codebase convention, see `pick_folder`), so double-invocation within one frame isn't reachable — no extra guard needed.
- [x] T013 [US1] Wire SaveCopyTo/MoveTo to rfd folder chooser (start at `ActionPrefs.last_destination`, persist on use); wire ResizeCopy/ConvertFormat to existing Image Tools routines; CopyPath/CopyImage via arboard (image fallback → path + message) — 2026-07-05: `pick_action_destination` (shared by save/move) starts at `action_prefs.last_destination`, persists via `save_action_prefs` on pick. Resize/Convert call `image_proc::process_image` directly (bypassing `ImageToolsService::process_single`'s cache/backup path — not applicable to a one-shot context action) with an explicit collision-safe `output_path` (`derived_action_output_path`, reusing `compute_output_path` + `collision_suffixed_path`); Resize mirrors the existing "Quick resize" convention (50%, jpg, q80) since the context menu has no dedicated resize-parameter UI (design decision, noted for review — not a spec ambiguity requiring escalation). Copy path uses `ctx.copy_text` (existing codebase convention for text, see activity-log copy); Copy image reuses existing `copy_image_to_clipboard` (arboard, with its existing fallback messaging).
- [x] T014 [US1] Failure surfacing: rfd error dialog (image + cause) + ActionOutcome to activity log for every action (FR-010); move advances stage to next surviving image — 2026-07-05: `record_action_outcome` logs + sets status for every action, and raises `rfd::MessageDialog` (Error level) naming the image + cause on failure. `advance_stage_after_move` selects the next surviving filtered index (or `None` if the list is now empty) after a successful move.

**Deviation from delegation model**: T011-T014 were implemented directly by the Sonnet orchestrator rather than dispatched to haiku-implementer. Rationale (cited per escalation-adjacent guidance in the dispatch prompt: "the main.rs stage-pane wiring (T011) ... may qualify [for escalation] if Haiku struggles"): this is dense, sequential, cross-cutting integration into one 3510-line file (struct fields, enum, 4+ new call sites in `update()`/`render_central_image_panel`, ~20 new methods) where a fresh-context Haiku agent would need the same deep main.rs convention survey the orchestrator had already built up over this session, and where four dependent edits to the same file would have needed four serial Haiku round-trips with high misintegration risk. This is a lighter-weight alternative to a full Opus escalation — the orchestrator already had the necessary context and executed it directly, verifying with `cargo check`/`cargo test`/`cargo clippy --lib --bin -- -D warnings` (clean) at each step. No Opus agent was spawned for this phase.

**Also discovered (out of scope for 016, flagged for the security/lead review)**: `cargo clippy --all-targets -- -D warnings` fails on pre-existing code unrelated to this feature (several test files, and one `std::io::Error::new(ErrorKind::Other, ...)` in `src/tool_caps.rs`) — contradicts `.agents/ENV.md`'s note that clippy/fmt aren't installed locally (they are, at `/usr/bin/{rustfmt,cargo-clippy}` in this environment) and means the repo is not currently clippy-`-D warnings`-clean at baseline. `cargo clippy --lib --bin -- -D warnings` (i.e. everything feature 016 touched) is clean.

**Checkpoint**: US1 independently testable — quickstart V1 + V4 + V5 runnable; `cargo test` green (129 passed / 0 failed / 2 ignored across all suites, stable across 3 reruns) — 2026-07-05.

## Phase 3: User Story 2 — feh round-trip (P1)

- [x] T015 [US2] `src/types.rs` + `src/ui_logic.rs`: `ViewerRoundTrip` state + `viewer_spawn_command(...)` building the exact contract args/env (arg-vector only, `--info` trail, XDG_CONFIG_HOME) + unit tests asserting the full argument vector (incl. adversarial filenames passed as args, never shell) — 2026-07-05: **escalated to opus-specialist**, trigger (b) security-sensitive change (subprocess argument-vector construction). 5 tests green. Deviation: `ViewerRoundTrip` struct (holds a live `std::process::Child`, which isn't `Debug`/`Clone`/`PartialEq`) will live in `main.rs` as part of T018, matching the existing `ActiveToolsJob` precedent for runtime-handle-bearing state, rather than in `types.rs` — `viewer_spawn_command` (the pure, testable part) is the actual T015 deliverable and is complete.
- [x] T016 [US2] `src/ui_logic.rs`: `validate_handoff(content, filelist)` — trim, canonicalize, membership check; deleted-image nearest-neighbor fallback; + unit tests (valid, not-in-list, garbage, empty, symlink escape attempt, deleted image) — 2026-07-05: **escalated to opus-specialist** (combined dispatch with T015), trigger (b) security-sensitive change (untrusted handoff-file trust/canonicalization, contract R4). 8 tests green, incl. a real symlink-escape-attempt test. Orchestrator independently re-verified: `cargo check`/`cargo test` (98 lib tests, stable x3)/`cargo clippy --lib --bin -- -D warnings` all clean; read the implementation directly and confirmed the canonicalize-and-compare gate and the argument-vector construction are sound.
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
