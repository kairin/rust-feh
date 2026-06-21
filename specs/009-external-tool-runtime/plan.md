# Implementation Plan: External Tool Runtime

**Branch**: `009-external-tool-runtime` | **Date**: 2026-06-22 (speckit-plan + converge consolidation) | **Spec**: [spec.md](./spec.md) | **Status**: Implemented (T001–T022)

**Input**: Unified PATH detection + on-demand recheck for feh and ImageMagick  
**Supersedes**: [002-feh-runtime-detection](../002-feh-runtime-detection/spec.md)  
**Session index**: [SESSION-2026-06-22-TRACEABILITY.md](../SESSION-2026-06-22-TRACEABILITY.md)  
**Related**: [008-tool-capabilities-panel](../008-tool-capabilities-panel/spec.md), [005-image-list-presentation](../005-image-list-presentation/spec.md)  
**Tasks**: [tasks.md](./tasks.md) — **22 open tasks** (converge items woven into US3/US4 phases)

## Summary

Feature **009** unifies external tool runtime state in `ToolCapabilities`. Most behavior is **already shipped** from 001/008: startup `detect()`, panel **Recheck tools on PATH**, `feh_available` mirror, debug log line.

**Remaining work** (verified by `/speckit-converge`):
1. **FR-004** — Tools menu recheck (T008)
2. **FR-008** — spawn-failure unavailable sync in **both** `open_in_feh` and `set_wallpaper`, including `tool_caps.feh_available` + `feh_missing_status()` (T010–T012, T018–T019)
3. **FR-009/SC-005** — `is_feh_not_found` classifier + unit tests (T020)
4. **FR-011** — `gap-audit.md` + mark 002 superseded (T015)

**Next step**: `/speckit-implement` using the consolidated task order below.

## Convergence consolidation

*How to roll `/speckit-converge` outcomes into plan → implement:*

| Step | Command | What it writes | Role |
|------|---------|----------------|------|
| 1 | `/speckit-plan` | `plan.md`, contracts, research | Intent + architecture |
| 2 | `/speckit-tasks` | `tasks.md` Phases 1–7 | Initial task breakdown |
| 3 | `/speckit-implement` | Code + checked tasks | First implementation pass |
| 4 | `/speckit-converge` | **Appends** `## Phase N: Convergence` to `tasks.md` only | Finds code↔spec gaps; **never** rewrites plan/spec/tasks |
| 5 | `/speckit-plan` (re-run) | **This section** in `plan.md` | Consolidates converge findings + open tasks into one implement handoff |
| 6 | `/speckit-implement` | Completes all open `[ ]` tasks | Closes gaps including Convergence phase |

**Rule**: Convergence is append-only on `tasks.md`. Re-planning absorbs its findings here so implement has a single ordered backlog — do **not** duplicate converge tasks elsewhere; trace via the table below.

### Findings → tasks → code (2026-06-22 converge)

| ID | Gap | Severity | Source | Task(s) | Code touch-point |
|----|-----|----------|--------|---------|------------------|
| F1 | Tools menu recheck missing | HIGH | FR-004 | T008, T009 | `main.rs` Tools menu L416–428 |
| F2 | `open_in_feh` spawn doesn't flip unavailable | HIGH | FR-008 | T010–T012 | `main.rs` `open_in_feh` L860–888 |
| F3 | `set_wallpaper` spawn same gap | HIGH | FR-008 | **T018** | `main.rs` `set_wallpaper` L891–907 |
| F4 | `tool_caps.feh_available` not synced on failure | HIGH | FR-008, plan:R4 | **T019**, T012 | `main.rs` both spawn paths |
| F5 | Generic error vs `feh_missing_status()` | MEDIUM | contract | **T019** | `ui_logic::feh_missing_status()` |
| F6 | No `is_feh_not_found` classifier/tests | MEDIUM | FR-009, SC-005 | T011, **T020** | `tool_caps.rs` or `ui_logic.rs` |
| F7 | `gap-audit.md` absent | MEDIUM | FR-011 | T015 | `specs/009-.../gap-audit.md` |
| F8 | feh on PATH but not executable | LOW | edge case | Defer | Note in gap-audit only |

**Shipped (no task):** FR-001–FR-003, FR-006–FR-007, FR-010; panel recheck; `refresh_tool_caps`; 9 `tool_caps` tests from 008.

### Consolidated implement order (for `/speckit-implement`)

Execute in this order — convergence tasks are woven into the spawn-failure slice, not a separate pass:

```text
P0  Verify     T001–T006                baseline + classifier
P1  Menu       T012 → T014              FR-004 (MVP #1)
P2  Spawn      T015 → T019              FR-008 (MVP #2 + converge)
P3  Recheck    T007–T011                US1/US2 verify
P4  Polish     T020 → T022              gap-audit, full pass
```

**P3 dependency**: Implement T020 before T010/T019 so spawn paths call a tested helper.

## Technical Context

**Language/Version**: Rust stable, edition 2021  
**Primary Dependencies**: `which` (PATH detect), `std::process::Command` (feh spawn)  
**Storage**: None — session-only `ToolCapabilities` snapshot on `RustFehApp`  
**Testing**: `cargo test tool_caps` + `is_feh_not_found` unit tests; manual quickstart V1–V4  
**Target Platform**: Linux desktop (glow)  
**Project Type**: desktop-app gap-fill  
**Performance Goals**: Recheck &lt;500ms (SC-002)  
**Constraints**: No background polling (FR-010); no custom feh paths  
**Scale/Scope**: 11 FRs; **22 tasks** (converge woven into US3/US4 phases)

## Constitution Check

*GATE: Pre-implementation. Re-check after gap-audit.*

| Principle | Status | Evidence |
|-----------|--------|----------|
| I. Thin-Wrapper Architecture | ✅ PASS | Detect + spawn feh; no in-app viewer |
| II. Pure Rust, Minimal Dependencies | ✅ PASS | No new crates |
| III. Clean Module Separation | ✅ PASS | Classifier in core module; GUI triggers only |
| IV. Linux-First, feh-Centric | ✅ PASS | PATH lookup; graceful degradation |
| V. Performance Awareness | ✅ PASS | On-demand detect only |

**Gate result**: ALL PASS.

## Shipped code baseline

| FR area | Status | Location |
|---------|--------|----------|
| FR-001 startup detect | ✅ Shipped | `main.rs` L38–39 |
| FR-002 `feh_available` derive | ✅ Shipped | `main.rs` L39, L203 |
| FR-003 panel recheck | ✅ Shipped | L254–256 → `refresh_tool_caps` L201–208 |
| FR-004 Tools menu recheck | ❌ **Outstanding** | T008 |
| FR-005 unified refresh | ⚠️ **Partial** | Panel only; T008 completes |
| FR-006 debug log | ✅ Shipped | L204–207 |
| FR-007 no restart | ✅ Shipped | In-session `detect()` |
| FR-008 spawn failure sync | ❌ **Outstanding** | T010–T012, T018–T019 |
| FR-009 core testable | ⚠️ **Partial** | 9 tests; T020 adds classifier |
| FR-010 no polling | ✅ Shipped | On-demand only |
| FR-011 002 superseded | ⏳ **Outstanding** | T015 |

## Project Structure

### Documentation

```text
specs/009-external-tool-runtime/
├── spec.md
├── plan.md                 # This file (convergence consolidation)
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/tool-runtime-ui.md
├── gap-audit.md            # T015 output
└── tasks.md                # T001–T020 (Phase 8 = converge append)
```

### Source Code

```text
src/
├── tool_caps.rs            # detect(), is_feh_not_found() [T020]
├── ui_logic.rs             # feh_missing_status()
└── main.rs                 # menu recheck [T008], spawn sync [T010,T018,T019]
```

## Phase 0: Research

Consolidated in [research.md](./research.md) — includes R7 converge workflow.

## Phase 1: Design Artifacts

| Artifact | Path | Status |
|----------|------|--------|
| Data model | [data-model.md](./data-model.md) | Current |
| UI contract | [contracts/tool-runtime-ui.md](./contracts/tool-runtime-ui.md) | Current — spawn both paths |
| Validation | [quickstart.md](./quickstart.md) | Updated — maps to T007/T014/T017 |

## Implementation sketches

### P1 — Tools menu (T008)

```rust
if ui.button("Recheck tools on PATH").clicked() {
    ui.close_menu();
    self.refresh_tool_caps();
}
```

### P2 — Classifier (T020)

```rust
// tool_caps.rs or ui_logic.rs
pub fn is_feh_not_found(err: &std::io::Error) -> bool { ... }
pub fn feh_confirmed_missing() -> bool { which::which("feh").is_err() }
```

### P3 — Spawn failure (T010, T018, T019)

```rust
Err(e) if is_feh_not_found(&e) && feh_confirmed_missing() => {
    self.feh_available = false;
    self.tool_caps.feh_available = false;
    self.status = feh_missing_status();
}
```

Apply in **both** `open_in_feh` and `set_wallpaper`.

## Dependencies on other features

| Feature | Relationship |
|---------|--------------|
| **001** | `feh_available` drives feh controls |
| **002** | Superseded — document in T015 gap-audit |
| **008** | Panel displays snapshot after recheck/spawn sync |
| **005** | `magick_available` at scan time; Rescan after magick install |

## Deferred / out of scope

- feh present but not executable (F8 — gap-audit note only)
- Custom binary paths, background polling, auto-rescan on magick change

## Verification

```fish
cargo clippy -- -D warnings
cargo test tool_caps
# Manual: specs/009-external-tool-runtime/quickstart.md V1–V4
```

## Advisory

Run `/speckit-implement` once with consolidated order P0→P5. After implement, run `/speckit-converge` again; if clean, no new Phase 9 appears in `tasks.md`.