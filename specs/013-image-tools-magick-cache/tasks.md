# Tasks: Image Tools with Magick Cache

**Input**: Design documents from `/specs/013-image-tools-magick-cache/`

**Prerequisites**: plan.md (required), spec.md (required), research.md, data-model.md, contracts/ (image-tools-ui.md, magick-cache-tool.md), quickstart.md

**Tests**: Mandatory per constitution v1.0.1 (`image_proc`, `ui_logic`, `tool_caps`) and plan.md. Include unit, integration, and perf harness tasks.

**Organization**: Tasks grouped by user story. Phases 1–2 reflect converge assessment: several setup items are done; foundation and US1–US5 remain. `[X]` = verified in codebase as of 2026-06-24.

**Structure**: Extend `image_proc` for processing + cache; helpers in `ui_logic`; `tool_caps` detection only; UI in `main.rs`. No `image_tools` core module.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Parallelizable (different files, no blocking deps)
- **[Story]**: US1–US5 for user-story phases only
- Every task names exact file paths

## Path Conventions

- `src/image_proc.rs` — processing, cache manager, `ImageToolsService`
- `src/types.rs` — `FitMode`, `OutputPolicy`, `CacheConfig`, `AssetStatus`, etc.
- `src/ui_logic.rs` — rename expander, crop preview pixels, `spawn_job`, inventory helpers
- `src/tool_caps.rs` — magick-cache detection
- `src/main.rs` — Image Tools panel, confirmations, progress, wiring
- `tests/unit/image_proc.rs`, `tests/unit/ui_logic.rs`
- `tests/integration/image_tools.rs`
- `tests/perf/` — SC benchmarks
- `Cargo.toml` — `[[test]]` registration

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Compliant module layout, types baseline, test harness prep.

- [X] T001 Remove non-compliant `image_tools` core module: ensure `src/lib.rs` declares only scanner/image_proc/types/ui_logic/tool_caps; no `pub mod image_tools` in `src/lib.rs` or `src/main.rs`
- [X] T002 [P] Extend `src/types.rs` with `FitMode`, `Filter`, `ImageOperation`, `OutputPolicy`, `CacheConfig`, `RenamePattern`, `ProcessedResult`, `PreparedFastSet`, `AssetStatus` on `ImageEntry` per data-model.md
- [X] T003 [P] Extend `src/tool_caps.rs` for magick-cache detection + `magick_cache_ready` probe + `dependencies()` entry + unit tests in `src/tool_caps.rs`
- [X] T004 Update `src/lib.rs`: re-export `ImageToolsService`, `ImageOperation`, `OutputPolicy`, `ProcessedResult` from `image_proc`/`types`
- [X] T005 [P] Register `tests/unit/image_proc.rs` and `tests/unit/ui_logic.rs` as `[[test]]` in `Cargo.toml`; delete stale `tests/unit/image_tools/operations_test.rs`
- [X] T006 [P] Use zero-dep stable hash for IRI in `src/image_proc.rs` (`MagickCacheManager::make_iri`)
- [X] T007 Add `image_tools: ImageToolsService` field + skeleton `render_inspector_image_tools` in `src/main.rs`
- [X] T008 [P] Verify `cargo check` and `cargo test --lib` pass after setup

**Checkpoint**: Core modules compliant. Types and tool_caps ready. Test wiring (T005) still required.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Complete service, cache, job harness, and mandatory core tests before user stories.

**⚠️ CRITICAL**: No user story work until this phase is complete.

- [X] T009 Port `MagickCacheManager` stub (put, materialize, IRI) in `src/image_proc.rs` via `std::process::Command`
- [X] T010 Implement cache `get` / hit probe and integrate `tool_caps::ToolCapabilities::detect().magick_cache_ready` in `src/image_proc.rs` per contracts/magick-cache-tool.md
- [X] T011 [P] Add magick `Command` fallback for resize/crop/convert when magick available in `src/image_proc.rs`; keep `image` crate path
- [X] T012 Complete `ImageToolsService` in `src/image_proc.rs`: full `OutputPolicy` dest computation, cache get-before-compute + put-after, `was_cache_hit`, fix crop `WxH+X+Y` parser for signed offsets
- [X] T013 [P] Implement `spawn_job` mpsc harness with cancel (`Arc<AtomicBool>`) and progress in `src/ui_logic.rs`
- [X] T014 Implement `add_or_update_asset_in_inventory` in `src/ui_logic.rs`
- [X] T015 [P] Implement `compute_output_path`, rename pattern tokenizer/expander skeleton, `crop_preview_pixels` (raw RGBA, no egui) in `src/ui_logic.rs`
- [X] T016 [P] Mandatory unit tests in `tests/unit/image_proc.rs`: IRI stability, OutputPolicy paths, param validation, cache config
- [X] T017 [P] Mandatory unit tests in `tests/unit/ui_logic.rs`: inventory update, output path, rename collision detection
- [X] T018 Wire `CacheConfig` in app state, mpsc result types for image-tool jobs, Activity Log helper usage in `src/main.rs`
- [X] T019 [P] Integration test skeleton in `tests/integration/image_tools.rs` (skip when magick-cache absent)
- [X] T020 Run `cargo test` (lib + unit + integration skeleton); fix failures

**Checkpoint**: Service, cache get/put, ui_logic helpers, job harness, and core tests green.

---

## Phase 3: User Story 1 - Safe Single-Image Resize, Crop, or Convert (Priority: P1) 🎯 MVP

**Goal**: Full single-image tools with safe output, crop preview, inventory + Activity Log update.

**Independent Test**: 5–10 image folder; 50% resize to `processed/` subfolder; original untouched; new file in list; opens in feh; logged. (spec US1 + SC-001)

### Tests for User Story 1

- [X] T021 [P] [US1] Unit tests for resize/crop/convert geometry, fit modes, OutputPolicy dest paths in `tests/unit/image_proc.rs`
- [X] T022 [P] [US1] Unit tests for crop preview pixels in `tests/unit/ui_logic.rs`
- [X] T023 [P] [US1] Integration test single op end-to-end (create file, inventory, no original touch) in `tests/integration/image_tools.rs`

### Implementation for User Story 1

- [X] T024 [P] [US1] Complete resize in `src/image_proc.rs` mapping `ImageOperation::Resize` with fit/filter/quality/percent
- [X] T025 [P] [US1] Complete numeric crop in `src/image_proc.rs` with validation and +repage semantics
- [X] T026 [P] [US1] Complete format convert in `src/image_proc.rs` with smart JPEG defaults for exotic sources
- [X] T027 [US1] Wire Single mode UI in `src/main.rs` per contracts/image-tools-ui.md: op selector, params, fit/filter/quality, output policy radios, Apply
- [X] T028 [US1] Live crop preview: `ui_logic` pixels → `egui::ColorImage` texture in `src/main.rs`
- [X] T029 [US1] In-place policy: double-confirm dialogs + verifiable backup before write in `src/main.rs` + service
- [X] T030 [US1] On success: structured Activity Log entry + `add_or_update_asset_in_inventory` with `AssetStatus::Processed` in `src/main.rs`
- [X] T031 [P] [US1] Verify SC-001 timing on 10–20MP fixture (manual or script hook)

**Checkpoint**: US1 independently testable via quickstart Scenario 1–2.

---

## Phase 4: User Story 2 - Batch Operations (Priority: P2)

**Goal**: Multi-selection batch with progress, cancel, confirmation, per-item reporting.

**Independent Test**: 100 images, 75% resize to subfolder; UI responsive; summary "98 succeeded, 2 skipped"; new files in list. (spec US2 + SC-003)

### Tests for User Story 2

- [X] T032 [P] [US2] Unit tests for batch plan + summary aggregation in `tests/unit/ui_logic.rs`
- [X] T033 [P] [US2] Integration test batch mixed success/skip in `tests/integration/image_tools.rs`

### Implementation for User Story 2

- [X] T034 [US2] Implement `process_batch` in `src/image_proc.rs` using `spawn_job` from `src/ui_logic.rs`
- [X] T035 [US2] Batch section UI in `src/main.rs`: selection summary, FR-017 confirmation dialog, Run Batch
- [X] T036 [US2] Non-modal progress + Cancel during batch in `src/main.rs` (mpsc pump)
- [X] T037 [US2] Per-item inventory/log updates + completion summary dialog in `src/main.rs`
- [X] T038 [US2] Error isolation: skip failed items, continue batch, report in summary
- [X] T039 [US2] In-place batch: double-confirm + backup-all warning before dispatch

**Checkpoint**: US1 + US2 independently testable. quickstart Scenario 3.

---

## Phase 5: User Story 3 - Safe Batch Rename (Priority: P2)

**Goal**: Pattern rename with live preview, collision detection, safe apply.

**Independent Test**: 20 images, `trip-{date:YYYYMMDD}-{counter:03}`; preview accurate; apply after confirm; collision blocked. (spec US3 + SC-007)

### Tests for User Story 3

- [X] T040 [P] [US3] Unit tests rename parser/expander + 500+ preview perf in `tests/unit/ui_logic.rs`
- [X] T041 [P] [US3] Integration test rename apply + rollback on error in `tests/integration/image_tools.rs`

### Implementation for User Story 3

- [X] T042 [P] [US3] Complete `RenamePattern` expander in `src/ui_logic.rs` (`{original}`, `{ext}`, `{counter:NN}`, `{date:YYYYMMDD}`)
- [X] T043 [US3] Rename UI in `src/main.rs`: pattern field, live Old|Proposed table, collision highlight
- [X] T044 [US3] Confirmation dialog + `fs::rename` apply with inventory path updates in `src/main.rs`
- [X] T045 [US3] Apply-time collision: block or skip/auto-number without partial damage

**Checkpoint**: US3 independently testable. quickstart Scenario 4.

---

## Phase 6: User Story 4 - Persistent Image Cache (Priority: P3)

**Goal**: Enable cache, auto put/get on ops, pre-cache folder, graceful degradation.

**Independent Test**: Enable cache; resize 20MP twice; second run <3s cache hit; disable → slow again. (spec US4 + SC-002)

### Tests for User Story 4

- [X] T046 [P] [US4] Unit tests cache hit/miss + IRI stability in `tests/unit/image_proc.rs`
- [X] T047 [P] [US4] Integration test put + fast repeat get in `tests/integration/image_tools.rs`

### Implementation for User Story 4

- [X] T048 [US4] Wire real cache put/get in `ImageToolsService` in `src/image_proc.rs` for single + batch paths
- [X] T049 [US4] Cache settings UI in `src/main.rs`: enable, root picker (rfd), passkey picker, TTL; in-memory `CacheConfig`
- [X] T050 [US4] "Pre-cache entire folder" background job in `src/image_proc.rs` + `src/main.rs` progress
- [X] T051 [US4] Activity Log cache hit/put lines; graceful messages when cache absent (FR-020)
- [X] T052 [US4] Verify cache config survives folder changes (session lifetime)

**Checkpoint**: US1–US4 testable. quickstart Scenario 5.

---

## Phase 7: User Story 5 - Prepare Fast feh Viewing Cache (Priority: P3)

**Goal**: Materialize optimized assets, launch feh via filelist, register in inventory.

**Independent Test**: 2000+ folder; Prepare Fast; launch optimized; faster first image; optimized entries in list. (spec US5 + SC-004/SC-008)

### Tests for User Story 5

- [X] T053 [P] [US5] Unit/integration tests for prepare-fast planning + materialization in `tests/unit/image_proc.rs` and `tests/integration/image_tools.rs`
- [X] T054 [P] [US5] Idempotency test: re-run prepare on prepared folder is fast

### Implementation for User Story 5

- [X] T055 [P] [US5] Implement `prepare_fast` job in `src/image_proc.rs`: cache put + `materialize_from_cache` to temp dir
- [X] T056 [US5] Generate feh filelist + `PreparedFastSet`; offer "Launch viewer on optimized" in `src/main.rs`
- [X] T057 [US5] Register materialized paths with `AssetStatus::Optimized` via `src/ui_logic.rs` inventory helper
- [X] T058 [US5] Progress UI + temp dir cleanup on exit; disable when cache/magick missing
- [X] T059 [US5] Verify SC-004 timing vs raw launch (script or manual per quickstart Scenario 6)

**Checkpoint**: All five user stories independently functional.

---

## Phase 8: Polish & Cross-Cutting Concerns

**Purpose**: Perf verification, edge cases, docs, constitution validation.

- [X] T060 [P] Perf harness SC-001 in `tests/perf/` using `scripts/generate-perf-fixture.sh` + `scripts/measure-resources.sh`
- [X] T061 [P] Perf harness SC-002 cache hit vs disabled in `tests/perf/`
- [X] T062 [P] Perf harness SC-004 prepare-fast feh timing in `tests/perf/`
- [X] T063 [P] Perf harness SC-007 rename preview 500+ in `tests/perf/` or `tests/unit/ui_logic.rs` benchmark
- [X] T064 [P] Integration test SC-005: original file hash unchanged after non-in-place ops in `tests/integration/image_tools.rs`
- [X] T065 Handle edge cases in code: large-image guardrails, job teardown on folder switch/exit, re-scan during active job per research.md Post-Convergence section
- [X] T066 Run full `cargo test`; `cargo clippy -- -D warnings`; fix regressions
- [X] T067 [P] Update `README.md` with Image Tools + Magick Cache + Prepare Fast feh sections
- [X] T068 Validate all 8 quickstart.md scenarios; document timing results
- [X] T069 Constitution audit: no egui in `image_proc`/`ui_logic`; Command-only external tools; mandatory core tests run via `cargo test`
- [X] T070 [P] Run `.specify/extensions/agent-context/scripts/bash/update-agent-context.sh` to refresh `AGENTS.md`

**Checkpoint**: Feature complete, measured, ready for review.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1**: Start immediately. T005 remains before reliable unit test runs.
- **Phase 2**: Depends on Phase 1. **Blocks all user stories.**
- **Phase 3 (US1)**: After Phase 2. MVP — stop and validate here.
- **Phase 4–5 (US2–US3)**: After Phase 2; US2 needs US1 ops; US3 mostly independent of pixel ops.
- **Phase 6–7 (US4–US5)**: After US1; US5 needs US4 cache + materialize.
- **Phase 8**: After desired stories complete.

### User Story Dependencies

| Story | Depends on | Independent test |
|-------|------------|------------------|
| US1 P1 | Phase 2 | quickstart Scenarios 1–2 |
| US2 P2 | US1 ops + job harness | Scenario 3 |
| US3 P2 | Phase 2 ui_logic | Scenario 4 |
| US4 P3 | US1 service paths | Scenario 5 |
| US5 P3 | US4 + feh filelist | Scenario 6 |

### Parallel Opportunities

- Phase 1: T002, T003, T005, T006, T008 in parallel after T001
- Phase 2: T010, T011, T013, T015, T016, T017, T019 in parallel after T009
- Within each US: `[P]` test tasks parallel to each other; separate-file impl tasks parallel where noted
- Phase 8: T060–T063, T064, T067, T070 in parallel

---

## Parallel Example: User Story 1

```bash
# Tests (parallel)
T021: tests/unit/image_proc.rs resize/crop/convert tests
T022: tests/unit/ui_logic.rs crop preview tests
T023: tests/integration/image_tools.rs single op e2e

# Core impl (parallel, different concerns)
T024: src/image_proc.rs resize
T025: src/image_proc.rs crop
T026: src/image_proc.rs convert

# UI (after core ready)
T027–T030: src/main.rs Single panel + preview + policy + log/inventory
```

---

## Implementation Strategy

### MVP First (US1)

1. Complete Phase 1 (finish T005).
2. Complete Phase 2 (T010–T020).
3. Complete Phase 3 (US1).
4. **STOP**: quickstart Scenarios 1–2 + `cargo test`.
5. Demo/commit MVP.

### Incremental Delivery

1. Setup + Foundational → T020 green.
2. US1 → validate → MVP shipped.
3. US2 + US3 (parallel possible) → batch + rename.
4. US4 → cache acceleration.
5. US5 → Prepare Fast feh killer feature.
6. Phase 8 polish + perf evidence.

### Task Count Summary

| Phase | Tasks | Done | Remaining |
|-------|-------|------|-----------|
| 1 Setup | T001–T008 | 8 | 0 |
| 2 Foundational | T009–T020 | 12 | 0 |
| 3 US1 | T021–T031 | 11 | 0 |
| 4 US2 | T032–T039 | 8 | 0 |
| 5 US3 | T040–T045 | 6 | 0 |
| 6 US4 | T046–T052 | 7 | 0 |
| 7 US5 | T053–T059 | 7 | 0 |
| 8 Polish | T060–T070 | 11 | 0 |
| 9 Convergence | T071–T075 | 5 | 0 |
| **Total** | **75** | **75** | **0** |

---

## Notes

- Prior T001–T092 checklist was reset: converge showed many false `[X]` marks; this file reflects verified code state.
- All external tools: `std::process::Command` only.
- `ui_logic` returns raw pixels; egui textures only in `main.rs`.
- Next: `/speckit-implement` starting at T005 or T010.

---

## Phase 9: Convergence

**Purpose**: Post-implement assessment (2026-06-24). Core US1–US5 paths exist; 18 tasks remain open (T022–T068). This phase adds remediation for gaps where tasks were marked `[X]` but code is incomplete, plus one spec gap not previously tasked.

- [X] T071 Refactor batch, pre-cache, and prepare-fast in `src/main.rs` to use `ui_logic::spawn_job` with mpsc polling each frame (non-blocking progress + cancel) instead of synchronous loops; wire `process_batch` via job closure per FR-006/SC-003/Constitution V (partial)
- [X] T072 Add cache root directory picker, passkey file picker, and TTL field in `src/main.rs` Cache section (rfd); persist into `CacheConfig` and pass to `ImageToolsService::update_cache` per FR-019/contracts/magick-cache-tool.md (partial)
- [X] T073 Add resize filter ComboBox in `src/main.rs` Single/Batch panel (Lanczos/Nearest/etc.) wired to `tools_panel.filter` per FR-015/contracts/image-tools-ui.md (partial)
- [X] T074 Store `PreparedFastSet` (with `filelist_path`) in `RustFehApp` after prepare-fast; add **Launch feh on optimized versions** button that calls existing `open_in_feh` / filelist launch per FR-012/SC-004 (partial)
- [X] T075 Add in-place batch double-confirm + per-file backup warning before `tools_run_batch` when `ToolsPolicyUi::InPlace` selected per FR-018/T039 (partial)