# Implementation Plan: GUI Performance Validation

**Branch**: `003-gui-performance-validation` | **Date**: 2026-06-21 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `specs/003-gui-performance-validation/spec.md`

**Parent**: [001-persistent-ui-virtual-browsing](../001-persistent-ui-virtual-browsing/spec.md) — closes validated tier for SC-002, SC-004

## Summary

Produce a **repeatable GUI performance validation runbook** and supporting scripts that prove (or disprove) feature 001 scroll smoothness and memory targets on a real Linux session. Automated tier (`validate-feature-001.sh`, `cargo test`) stays unchanged; this feature adds fixture generation, RSS sampling helpers, a structured `ValidationRun` artifact, and updates feature 001 `gap-audit.md` when complete.

No application feature code required unless validation exposes a regression.

## Technical Context

**Language/Version**: Rust stable, edition 2021; bash for validation scripts  
**Primary Dependencies**: Existing rust-feh binary (eframe/glow); `ps`, `pgrep`, standard Unix tools  
**Storage**: Ephemeral temp dirs for 10k fixtures; `validation-results.md` artifacts in spec dirs  
**Testing**: `cargo test` (SC-003 automated); manual GUI for SC-002/SC-004; new script `scripts/validate-gui-performance.sh` orchestrates helpers  
**Target Platform**: Linux workstation (glow OpenGL backend)  
**Project Type**: desktop-app validation harness (docs + scripts, minimal Rust changes)  
**Performance Goals**: Document thresholds from 001 — scroll smooth (subjective), RSS &lt;150MB @10k, filter &lt;200ms (already in CI)  
**Constraints**: No new Cargo dependencies; no FPS overlay in app v1; two-tier audit preserved  
**Scale/Scope**: One maintainer runbook; 10k image fixture; ~45 min end-to-end per spec SC-003  

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Evidence |
|-----------|--------|----------|
| I. Thin-Wrapper Architecture | ✅ PASS | Validation only; no feh or viewer logic added |
| II. Pure Rust, Minimal Dependencies | ✅ PASS | Scripts use bash + existing binary; no new crates |
| III. Clean Module Separation | ✅ PASS | No changes to scanner/image_proc unless perf regression found |
| IV. Linux-First, feh-Centric | ✅ PASS | Linux-only validation; feh not required for V2 perf |
| V. Performance Awareness | ✅ PASS | Feature exists to **measure** principle V on feature 001 |

**Gate result**: ALL PASS. No Complexity Tracking needed.

## Project Structure

### Documentation (this feature)

```text
specs/003-gui-performance-validation/
├── plan.md              # This file
├── research.md          # Phase 0
├── data-model.md        # ValidationRun entity
├── quickstart.md        # GUI perf runbook (FR-001)
├── contracts/           # validation-run schema
├── validation-results.md # Filled after manual run (FR-004)
└── tasks.md             # /speckit-tasks (next)
```

### Source Code (repository root)

```text
scripts/
├── validate-feature-001.sh      # Existing automated tier (unchanged)
├── generate-perf-fixture.sh     # NEW — 10k image temp dir (PerfFixture)
├── sample-rss.sh                # NEW — RSS helper while GUI running
└── validate-gui-performance.sh  # NEW — orchestrates fixture + checklist

tests/
└── feature_001_validation.rs    # SC-003 already covered (no change required)

specs/001-persistent-ui-virtual-browsing/
├── gap-audit.md                 # UPDATE validated column when 003 completes
└── validation-results.md        # Automated tier (preserve; link to 003 results)
```

**Structure Decision**: Validation lives in `scripts/` and spec artifacts — not in `src/`. Keeps constitution §III intact.

## Complexity Tracking

> No violations. Section intentionally empty.

## Phase 0: Research Notes

Consolidated in [research.md](./research.md).

## Phase 1: Design Artifacts

| Artifact | Path |
|----------|------|
| Data model | [data-model.md](./data-model.md) |
| Contract | [contracts/validation-run.md](./contracts/validation-run.md) |
| Runbook | [quickstart.md](./quickstart.md) |

## Implementation Phases (for /speckit-tasks)

| Phase | Deliverable |
|-------|-------------|
| P1 | `scripts/generate-perf-fixture.sh` |
| P2 | `scripts/sample-rss.sh` + `scripts/validate-gui-performance.sh` |
| P3 | `quickstart.md` cross-ref 001 V1/V2/V4/V5 spot checks |
| P4 | Execute manual run → `validation-results.md` |
| P5 | Update 001 `gap-audit.md` SC-002/SC-004 validated column |

## Advisory (007 FR-006)

If scroll pass/fail is ambiguous (VM, software GL), consult Codex/Grok/Hermes/DeepSeek 4 Pro or human; record in this feature `spec.md` Clarifications before marking gap-audit pass/fail.