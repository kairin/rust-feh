# Implementation Plan: Tool Capabilities Panel

**Branch**: `008-tool-capabilities-panel` | **Date**: 2026-06-22 | **Spec**: [spec.md](./spec.md) | **Status**: Implemented (T001–T010)

**Input**: Retroactive formalization of shipped **Tools & capabilities** side panel  
**Session index**: [SESSION-2026-06-22-TRACEABILITY.md](../SESSION-2026-06-22-TRACEABILITY.md)  
**Parent**: [001-persistent-ui-virtual-browsing](../001-persistent-ui-virtual-browsing/spec.md)  
**Related**: [005-image-list-presentation](../005-image-list-presentation/spec.md) (inventory labels), [009-external-tool-runtime](../009-external-tool-runtime/spec.md) (recheck)

## Summary

Feature **008** is a **retroactive gap-audit** of code already shipped in `src/tool_caps.rs` and `render_tool_caps_panel` in `src/main.rs`. The panel exposes dependency status (feh required, ImageMagick optional), install commands with clipboard copy, operation speed tiers, and format routing — aligned with [docs/POSITIONING.md](../../docs/POSITIONING.md).

**005 is complete** — format discovery notes and scan legend already use inventory terminology (`native listed`, `magick-detected`). **009** owns Tools menu recheck parity and spawn-failure sync; 008 requires the panel to reflect state after `refresh_tool_caps()`.

Implement phase: **verify FR-001–FR-013 → `gap-audit.md` → fix any honesty/layout gaps → quickstart + POSITIONING cross-check**.

## Technical Context

**Language/Version**: Rust stable, edition 2021  
**Primary Dependencies**: `which` (PATH detect), `egui` 0.30 SidePanel + clipboard via `ctx.copy_text`  
**Storage**: None — `ToolCapabilities` snapshot on app struct; refreshed at startup and recheck  
**Testing**: `cargo test tool_caps` (5 unit tests); optional `tests/feature_008_panel.rs` for clipboard/status contracts; manual quickstart  
**Target Platform**: Linux desktop (glow)  
**Project Type**: desktop-app retroactive spec (logic in `tool_caps`, render in `main`)  
**Performance Goals**: Panel render O(1) per frame; `ToolCapabilities::detect()` only on startup/recheck (not per frame)  
**Constraints**: No ImageMagick convert subprocess in 008; constitution §III — all routing tables in `tool_caps.rs`  
**Scale/Scope**: 4 user stories; 13 FRs; ~10 tasks (gap-audit type)

## Constitution Check

*GATE: Pre-implementation. Re-check after gap-audit.*

| Principle | Status | Evidence |
|-----------|--------|----------|
| I. Thin-Wrapper Architecture | ✅ PASS | Panel documents feh/magick roles; does not replace feh viewer |
| II. Pure Rust, Minimal Dependencies | ✅ PASS | No new crates; `which` already in use |
| III. Clean Module Separation | ✅ PASS | `tool_caps.rs` pure logic + tests; `main.rs` renders only |
| IV. Linux-First, feh-Centric | ✅ PASS | apt install strings; feh required, magick optional |
| V. Performance Awareness | ✅ PASS | Detect on recheck only; static route tables |

**Gate result**: ALL PASS.

## Shipped code baseline (pre-gap-audit)

| FR area | Shipped? | Location |
|---------|----------|----------|
| FR-001–FR-005 Dependencies + copy | ✅ Likely | `main.rs` `render_tool_caps_panel` L210–252 |
| FR-006–FR-008 Speed + format routing | ✅ Likely | `tool_caps.rs` `operation_timings`, `format_routes` |
| FR-009 Missing required warning | ✅ Likely | `has_missing_required()` L300–306 |
| FR-010 Testable logic | ✅ | `tool_caps` unit tests |
| FR-011 Status pointer | ✅ | Bottom panel small text L777 |
| FR-012–FR-013 Scroll + resize | ✅ Likely | `ScrollArea` wrapper L589–593; SidePanel `resizable(true)` L584–587 min 240px |
| Recheck button | ✅ Panel only | `refresh_tool_caps` L201–208 — **009** adds Tools menu parity |

## Project Structure

### Documentation (this feature)

```text
specs/008-tool-capabilities-panel/
├── spec.md
├── plan.md                 # This file
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   └── capabilities-panel-ui.md
├── gap-audit.md            # T003–T010 output (complete)
└── tasks.md
```

### Source Code (repository root)

```text
src/
├── tool_caps.rs            # ToolCapabilities, DependencyStatus, routes, tests
└── main.rs                 # SidePanel, render_tool_caps_panel, refresh_tool_caps

tests/                      # Optional feature_008_panel.rs (gap-audit follow-up)
```

**Structure Decision**: No new modules. Gap-fill edits stay in `tool_caps.rs` or `main.rs` only.

## Complexity Tracking

> No violations.

## Phase 0: Research

Consolidated in [research.md](./research.md).

**Updates (2026-06-22)**: R2 revised — **005 landed**; scan notes use inventory bar terminology; magick identify during scan documented in format routes.

## Phase 1: Design Artifacts

| Artifact | Path | Status |
|----------|------|--------|
| Data model | [data-model.md](./data-model.md) | Updated — entities + refresh lifecycle |
| UI contract | [contracts/capabilities-panel-ui.md](./contracts/capabilities-panel-ui.md) | Updated — 005 inventory alignment |
| Validation | [quickstart.md](./quickstart.md) | Updated — automated + manual |

## Implementation Phases (for /speckit-tasks)

| Phase | Deliverable | FRs / SC |
|-------|-------------|----------|
| P0 | Baseline `cargo test tool_caps` + clippy | — |
| P1 | `gap-audit.md` vs FR-001–FR-013 | All FRs |
| P2 | Fix gaps from audit (if any) | FR-005, FR-008 honesty |
| P3 | quickstart validation | SC-001–SC-002 |
| P4 | POSITIONING + 005 inventory cross-check | SC-005, T010 |

## Gap-audit focus areas

| Area | Question | Likely pass? |
|------|----------|--------------|
| Copy → status | FR-005 status confirms dependency name | ✅ |
| Dynamic routing | `format_routes()` varies with `magick_available` | ✅ |
| Scan honesty | Exotic formats note magick-detect vs native listed | ✅ post-005 T046 |
| Recheck | Panel button calls `refresh_tool_caps` | ✅ |
| 009 boundary | Tools menu recheck **not** in 008 scope | N/A → 009 T008 |

## Dependencies on other features

| Feature | Relationship |
|---------|--------------|
| **001** | Persistent layout; FR-008a status complements panel |
| **005** | Inventory bar labels; format notes aligned (T046 done) |
| **009** | Recheck in Tools menu; spawn failure → `feh_available` sync |
| **010** (future) | Convert pipeline execution — panel documents only |

## Deferred / out of scope

- ImageMagick convert subprocess execution (010)
- Non-apt install strings (other distros)
- Scanner extension expansion beyond 005 identify
- Moving panel logic into separate egui widget crate

## Verification

```fish
cargo clippy -- -D warnings
cargo test tool_caps
# Manual: specs/008-tool-capabilities-panel/quickstart.md
```

## Advisory

If gap-audit finds format routing contradicts live scanner behavior (005 `scan_images`), fix `tool_caps.rs` notes before marking 008 complete — POSITIONING doc requires panel/doc/code agreement.