# Tasks: GUI Performance Validation

**Input**: Design documents from `specs/003-gui-performance-validation/`  
**Status**: Automated tier complete; **manual GUI T013–T017 pending human session**

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/validation-run.md, quickstart.md

## Phase 1: Setup (Infrastructure)

- [x] T001 Create `scripts/generate-perf-fixture.sh` with default 10000 minimal `.jpg` files under `/tmp/rust-feh-perf-*`
- [x] T002 [P] Create `scripts/sample-rss.sh` printing RSS KB for newest `rust-feh` process via `pgrep`/`ps`
- [x] T003 [P] Create `scripts/validate-gui-performance.sh` calling `validate-feature-001.sh` and printing manual checklist
- [x] T004 Ensure scripts are executable (`chmod +x scripts/generate-perf-fixture.sh scripts/sample-rss.sh scripts/validate-gui-performance.sh`)
- [x] T005 Verify fixture generator: `COUNT=$(./scripts/generate-perf-fixture.sh 10000)` and `ls "$COUNT" | wc -l` equals 10000

---

## Phase 2: Foundational (Artifacts & Contract)

- [x] T006 Create `specs/003-gui-performance-validation/quickstart.md` per FR-001 (V1/V2/V4 cross-ref to 001)
- [x] T007 [P] Create `specs/003-gui-performance-validation/contracts/validation-run.md` per data-model ValidationRun fields
- [x] T008 [P] Create stub `specs/003-gui-performance-validation/validation-results.md` matching contract template
- [x] T009 Run `./scripts/validate-gui-performance.sh` — automated tier MUST pass (build, clippy, cargo test, SC-003)
- [x] T010 [P] Verify `quickstart.md` Step 0–8 commands match live script paths and thresholds (150MB RSS, 500ms freeze) in `data-model.md`

---

## Phase 3: User Story 1 — Evidence Large Lists Stay Smooth (Priority: P1) 🎯 MVP

- [x] T011 [US1] Build release binary: `./build-and-place.sh` or `cargo build --release` in project root
- [x] T012 [US1] Generate fixture: `FIXTURE=$(./scripts/generate-perf-fixture.sh 10000)` and record path for validation-results.md
- [x] T013 [US1] Launch `./rust-feh`, load `$FIXTURE`, wait for scan complete — confirmed counter shows 10000/10000 (2026-06-28, fixture `/tmp/rust-feh-perf-OCjDyZ`)
- [x] T014 [US1] Execute V2 scroll protocol (5s rapid scrollbar drag) — **pass** 2026-06-28: user confirmed smooth scrolling, no freeze during rapid full-range drag on 10k list
- [x] T015 [US1] Sample RSS — **pass** 2026-06-28: `./scripts/measure-resources.sh 10000 15` peak RSS 142.8 MB (< 150 MB threshold); recorded in `validation-results.md`
- [x] T016 [US1] Fill all mandatory ValidationRun fields in `specs/003-gui-performance-validation/validation-results.md` per `contracts/validation-run.md`
- [x] T017 [US1] Update `specs/001-persistent-ui-virtual-browsing/gap-audit.md` SC-002 and SC-004 `validated` columns — done 2026-06-28 (both pass)

---

## Phase 4: User Story 2 — Repeatable Runbook (Priority: P1)

- [x] T018 [US2] Add "Automated vs manual tier" table to top of `specs/003-gui-performance-validation/quickstart.md` if missing — explicit SC-002/SC-004 manual label (FR-003 two-tier)
- [x] T019 [P] [US2] Cross-link 003 `quickstart.md` from `specs/001-persistent-ui-virtual-browsing/quickstart.md` Automated Validation section (line ~33)
- [x] T020 [US2] Verify `scripts/validate-gui-performance.sh` output lists exact paths to `quickstart.md` and does not claim GUI metrics in CI
- [x] T021 [US2] Dry-run timing: document actual minutes for Steps 0–8 in `validation-results.md` Notes to validate SC-003 (<45 min) for 003 spec

---

## Phase 5: User Story 3 — Automate What Can Be Automated (Priority: P2)

- [x] T022 [US3] Confirm `tests/feature_001_validation.rs` `sc003_filter_10k_under_200ms` passes — no changes unless regression
- [x] T023 [P] [US3] Confirm `scripts/validate-feature-001.sh` unchanged behavior — SC-002/SC-004 rows remain "manual GUI only" in `specs/001-persistent-ui-virtual-browsing/validation-results.md`
- [x] T024 [US3] Add link from 001 `validation-results.md` to 003 `validation-results.md` for GUI tier results (append "## GUI tier (feature 003)" section)

---

## Phase 6: Polish & Cross-Cutting

- [x] T025 [P] Update `specs/OUTSTANDING-ISSUES-ROADMAP.md` — mark 003 automated tier pass; manual validated tier pending T017
- [ ] T026 [P] If scroll/RSS inconclusive (VM/software GL), add Clarifications session to `specs/003-gui-performance-validation/spec.md` per 007 FR-006 advisory rule
- [x] T027 [P] Mention `./scripts/validate-gui-performance.sh` in `README.md` validation section (if section exists; add brief bullet if not)
- [x] T028 Run full closure: `./scripts/validate-gui-performance.sh` && review all `[ ]` tasks above for completion

---

## Phase 7: Convergence (2026-06-21) — superseded

*Stale cross-feature queue; 005/008/009 completed 2026-06-22. Kept for traceability.*

- [x] T029 Mark T005 complete (duplicate of Phase 1 T005)
- [x] T030 [P] ~~Run `/speckit-plan` for 005~~ — **done** 2026-06-22
- [x] T031 [P] ~~Run `/speckit-plan` for 006~~ — deferred; 006 not started
- [x] T032 [P] ~~`/speckit-tasks` for 005~~ — **done**
- [x] T033 [P] ~~`/speckit-tasks` for 006~~ — deferred
- [x] T034 ~~005 gap-audit~~ — **done** (`specs/005-image-list-presentation/gap-audit.md`)
- [x] T035 ~~006 gap-audit~~ — deferred
- [x] T036 ~~009 implement~~ — **done** (T001–T022)
- [ ] T037 Implement 004 FR-002–FR-006: `Scan skip:` warnings in `src/scanner.rs` — **next feature after 003 MVP**
- [x] T038 ~~001 data-model sync~~ — partial; 005 data-model covers list entities
- [x] T039 ~~README update~~ — done via T027
- [x] T040 ~~ROADMAP status~~ — done via T025

**Total**: 28 tasks + Phase 7 | **Open**: T013–T017 (manual GUI), T026 (if inconclusive), T037 (004)

**MVP blocker**: Human GUI session ~30 min — see `validation-results.md` Manual handoff section.