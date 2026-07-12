# Implementation Plan: Inspector UX Rework

**Branch**: `018-inspector-ux-rework` | **Date**: 2026-07-12 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/018-inspector-ux-rework/spec.md` and maintainer request

## Summary

Restructure rust-feh's right-hand inspector panel as the primary, intuitive control surface: fold every section by default and expand only on interaction (no folder loaded → Browse opens; scan starts → Session status opens; missing tool → Dependencies opens). Relocate the file list (flat/tree, virtualized) and folder navigation into the inspector's persistent zones, leaving the central panel for the viewed/staged image alone. Remove duplicated menu-bar actions (File→Choose folder/Rescan, View→Include subfolders/Detect exotic formats) — their only home is now the inspector's Browse section. Generalize detached-window state to a HashMap keyed by `InspectorSection` enum (7 variants), rendering deterministically via `InspectorSection::ALL`. Introduce `PanelPin::Image(PathBuf)` and `PanelContext` to model a single shared Image-actions detached window that can be pinned to a specific image, independent of the main selection.

## Resolved Inspector Layout

| Zone | Persistent? | Content | Height | Notes |
|------|-------------|---------|--------|-------|
| A. Nav strip | persistent | Up + truncated breadcrumb (hover=full) + Flat/Tree toggle + scan spinner | 1 row | relocated from existing subfolder-nav and Browse-controls code |
| B. Subfolder drill-down | persistent (hidden when none) | selectable subfolder rows | capped scroll, max_height 120 | relocated verbatim |
| C. File list | persistent, PRIMARY | flat OR tree image list, virtualized | flexible bulk = available − drawer height, floor row_h×4 | relocated verbatim including its virtualization/caching |
| D. Controls & panels drawer | collapsed by default | one meta-toggle → bounded ScrollArea of 7 CollapsingHeaders, each folded (Browse controls, Image actions, Feh instances, Session status, Activity log, Dependencies, Format discovery) | 24px collapsed / clamped 180-360px expanded | auto-expand-when-relevant preserved (scanning → Session status; no folder → Browse; missing tools → Dependencies/Format) |

Note: the outer infinite `ScrollArea` wrapping the whole inspector today is REMOVED — it makes `available_height` unbounded and breaks the list's row virtualization; heights become fixed/deterministic instead so the panel never jitters.

## Technical Context

**Language/Version**: Rust stable, edition 2021

**Primary Dependencies**: existing only — egui/eframe 0.30 (glow), same stack. **No new dependencies** (FR-012).

**Storage**: panel open/closed state lives in `App.inspector_open: HashSet<InspectorSection>`; detached-window metadata in `App.detached: HashMap<InspectorSection, DetachedWindow>`; persisted panel-toggle prefs alongside existing window-prefs (`~/.config/rust-feh/prefs.json`).

**Testing**: `cargo test` — unit tests in `ui_logic.rs`/`types.rs` for new pure logic (`InspectorSection` enum + `ALL`, `PanelPin`, `DetachedWindow`, `PanelContext` state resolution); integration tests for drawer-height math and list-virtualization stability; GUI validation per quickstart.md.

**Target Platform**: Linux desktop (X11/XWayland; rust-feh Wayland-native or X11)

**Project Type**: Desktop GUI application (existing single crate)

**Performance Goals**: list scroll latency <50 ms (10% of pre-rework baseline); drawer toggle ≤ 1 frame; no jitter during resize.

**Constraints**: no network; no new external tools; main.rs additions minimal (layout restructuring, menu deletion, pin wiring); core logic in `ui_logic.rs`/`types.rs`. Codacy 50-line/complexity caps; clippy `-D warnings`.

**Scale/Scope**: single inspector panel at fixed width; up to 10k+ images in the list; one shared Image-actions detached window; up to a handful of concurrent detached instances of other sections (Feh instances, Activity log, etc. if detach is enabled for all sections).

## Constitution Check

*GATE: evaluated against constitution v1.0.1 — PASS.*

- **I. Thin-Wrapper**: PASS. Inspector rework is a layout/UX reorganization of existing controls and logic; no new feh-side behavior or reimplementation of viewer features. The staged image pane (feature 016) remains unchanged; this feature reorders existing UI, not features.
- **II. Pure Rust, Minimal Dependencies**: PASS — zero new dependencies. Layout redesign uses existing egui primitives (CollapsingHeader, ScrollArea, dynamic height calculation).
- **III. Clean Module Separation**: PASS — all new logic (`InspectorSection` enum, `PanelPin`, `PanelContext`, state resolution) lands in `ui_logic.rs`/`types.rs` (egui-free, unit-tested); `main.rs` gets layout rendering + event wiring only, thin and delegation-heavy.
- **IV. Linux-First, feh-Centric**: PASS — feh delegation unchanged; this is UI layering only.
- **V. Performance Awareness**: PASS — removing the infinite outer ScrollArea and fixing heights enable list virtualization to work correctly; no new scanning or decode work.

**Post-design re-check (Phase 1)**: PASS — no violations introduced.

## Project Structure

### Documentation (this feature)

```text
specs/018-inspector-ux-rework/
├── plan.md                       # This file
├── spec.md                        # Feature specification
├── tasks.md                       # Batch-by-batch task list
└── (additional docs may be added per Batch 1+)
```

### Source Code (repository root)

```text
src/
├── main.rs        # restructure: layout rendering (three zones + drawer), menu dedup,
│                  #  pin wiring, per-frame render loop
├── ui_logic.rs    # NEW: InspectorSection enum + ALL, initial_open_sections,
│                  #  PanelPin, PanelContext, panel_context() resolver, toggle_inspector_section
├── types.rs       # NEW: DetachedWindow struct (pin, id, visible)
├── image_proc.rs  # unchanged
├── scanner.rs     # unchanged
└── tool_caps.rs   # unchanged

tests/
├── unit/ui_logic.rs   # open-state generalization tests + PanelContext resolution
└── integration/       # (if needed for list-virtualization stability regression)
```

**Structure Decision**: reuse existing single-crate layout; no new modules. New enums/structs land in `ui_logic.rs`/`types.rs` (pure, testable); `main.rs` refactoring is layout-heavy but delegation-based, keeping function sizes within Codacy limits by using helpers from `ui_logic.rs`.

## Phase Overview

**Batch 0** (this batch, Handback section of 017 tasks.md): prerequisite fixes (F1–F6) + this SpecKit scaffolding. Delivers no rework, only gating fixes and planning.

**Batches 1–6**: incremental, gated rework as detailed in `tasks.md`. Each batch delivers measurable progress (Batch 1: open-state generalization + width floor; Batch 2: the three-zone layout; Batch 3: menu dedup; Batch 4: detached-window generalization; Batch 5: pinning + security review; Batch 6: audit + verification).

## Complexity Tracking

*(empty — no constitution violations to justify)*
