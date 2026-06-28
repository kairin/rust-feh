<!--
  Sync Impact Report
  ==================
  Version change: 1.0.0 → 1.0.1
  Reason: PATCH — clarify module inventory in Principle III and Technical Standards
          to reflect actual crate layout after features 005/008/009/011/012
          (ui_logic + tool_caps added as core egui-independent modules via lib.rs;
          matches src/, README, Cargo profile, and all recent plans/specs).
          No principles added, removed, or re-defined. Rules and governance unchanged.

  Modified principles: None (text refinements inside I, III, Technical Standards, and Development Workflow for module accuracy; no rule or section changes)
  Added sections: none
  Removed sections: none
  Templates requiring updates:
    - .specify/templates/plan-template.md      ✅ no changes needed (generic Constitution Check section; gates always derived from live .specify/memory/constitution.md at plan time)
    - .specify/templates/spec-template.md      ✅ no changes needed (no hardcoded constitution references)
    - .specify/templates/tasks-template.md     ✅ no changes needed
    - .specify/templates/checklist-template.md ✅ no changes needed
    - .specify/templates/constitution-template.md ✅ source of placeholders; no drift introduced
  Runtime guidance & command files reviewed (no edits required):
    - README.md, docs/POSITIONING.md, docs/NFEH-COMPARISON-AND-MIGRATION.md (only reference principles by §I/§III/§IV etc.; module lists are descriptive, not contractual)
    - AGENTS.md (only SPECKIT markers)
    - .specify/extensions/agent-context/commands/speckit.agent-context.update.md (uses generic "coding agent context file" + e.g. examples including CLAUDE.md; no CLAUDE-only guidance)
    - No .specify/templates/commands/*.md directory in this repo (commands provided via extensions/ when needed; checked per skill)
  Follow-up TODOs: none
-->
# rust-feh Constitution

## Core Principles

### I. Thin-Wrapper Architecture
rust-feh is a GUI *frontend*, not a replacement for feh. feh handles viewing, zoom,
navigation, and wallpaper-setting; rust-feh provides folder browsing, image selection,
file listing, and launch orchestration.

- Core modules (scanner, image_proc, types, ui_logic, tool_caps) MUST remain independent of the egui GUI.
- NEVER reimplement feh features in Rust — delegate to feh via subprocess spawn.
- The GUI layer MUST NOT contain business logic; it drives core modules and renders
  results.
- New features MUST be justified by a concrete user need that feh does not address.

### II. Pure Rust, Minimal Dependencies
The application is built entirely in the Rust ecosystem, targeting a small, fast binary.

- All new dependencies MUST be in the crates.io ecosystem and be actively maintained.
- Adding a dependency requires a clear justification in the plan or PR description.
- Prefer the standard library over third-party crates when the functionality is simple.
- Avoid heavyweight frameworks (no Qt, no Electron, no GTK bindings). The egui/eframe
  stack is the approved GUI toolkit.
- The `Cargo.toml` release profile MUST use LTO, single codegen unit, and panic=abort
  to keep the binary small and fast.

### III. Clean Module Separation
Every module has a single, well-defined responsibility. GUI code never leaks into
core logic; core modules never depend on GUI types.

- `scanner` — filesystem traversal and image discovery. No GUI awareness.
- `image_proc` — image processing (resize, format conversion). Pure `image` crate.
  No GUI awareness.
- `types` — domain types (`ImageEntry`, `Selection`, `SortMode`, `ListViewMode`,
  `ScanInventory`, `TreeRowKind`, etc.). No dependencies on any other crate module.
- `ui_logic` — testable UI orchestration and presentation logic (filtering,
  sorting, tree computation, network/GVFS mount policy, feh filelist emission,
  status formatting, inventory aggregation). Pure functions; zero egui widget
  types or immediate-mode rendering code. Invoked by `main` for data prep only.
- `tool_caps` — external tool detection (`which` for feh/magick), runtime
  capability flags, operation timing tiers, per-format routing tables (native vs
  magick). Standalone, heavily unit-tested, no GUI, no persistent side effects.
- `main` — egui/eframe application, UI rendering, user interaction, feh
  subprocess spawn + filelist handoff. Delegates to core modules (via
  `rust_feh::*` re-exports from `lib.rs`) for all non-UI work. Uses `std::sync::mpsc`
  channels today to keep the render loop responsive during background scans.
- Async/threading concerns MUST be extracted from the GUI into dedicated modules
  when introduced (current channel usage already follows the rule; future
  tokio/flume adoption would live outside `main.rs`).

### IV. Linux-First, feh-Centric
rust-feh targets Linux workstations as its primary platform. feh integration is
the central value proposition.

- `feh` MUST be detected or installable; the application MUST degrade gracefully
  with a clear status message if feh is missing.
- Wallpaper-setting uses `feh --bg-fill` exclusively — no alternative backends.
- ImageMagick (`magick`/`convert`) is optional and MUST only enhance format support;
  the `image` crate is the always-available processor.
- Other platforms (macOS, Windows via WSL) are NOT primary targets. Contributions
  for cross-platform support are welcome but MUST not regress Linux behavior.

### V. Performance Awareness
The application starts fast, scans fast, and stays responsive under large directories.

- Directory scanning MUST use `walkdir` with symlink-follow disabled.
- Image metadata (size, dimensions) MUST be loaded lazily; never read full image
  data during listing.
- The UI MUST remain responsive during scans. When async scanning is introduced,
  it MUST use channels and never block the egui render loop.
- Release builds MUST optimize for binary size and runtime speed (LTO, opt-level=3,
  codegen-units=1, panic=abort, strip).

## Technical Standards

**Language / Edition**: Rust stable, edition 2021.
**GUI Toolkit**: egui 0.30 / eframe 0.30 with `glow` backend (Linux OpenGL).
**Key Dependencies**: `walkdir` (scanning), `image` 0.25 (processing), `rfd` 0.15
(file dialogs), `which` (binary detection).
**License**: MIT — all new code uses SPDX-License-Identifier: MIT headers.
**Build**: `cargo build --release` produces the binary at `target/release/rust-feh`.
The `build-and-place.sh` script copies it to the project root.
**Core modules** (egui-independent, re-exported by `src/lib.rs`): `scanner`,
`image_proc`, `types`, `ui_logic`, `tool_caps`. `src/main.rs` contains only the
eframe `App` implementation, egui rendering, and orchestration glue.
**Archive**: The original nfeh code lives in `archive/original-nfeh/` and MUST remain
there until the new tool is fully verified. No old maintainer artifacts exist in the
active source tree.

## Development Workflow

- **Branching**: Feature branches from `main`, named `###-feature-name`.
- **Specs**: Feature specifications live in `specs/###-feature-name/`.
- **Plan**: Implementation plans are written before coding; the plan includes a
  Constitution Check section verifying alignment with all core principles.
- **Testing**: `cargo test` runs all unit and integration tests. Tests for core
  modules (scanner, image_proc, types, ui_logic, tool_caps) are mandatory. GUI
  tests (in main or integration) are optional but encouraged for critical user
  flows.
- **Review**: All PRs MUST verify constitution compliance. The plan template's
  Constitution Check is the gate — any violation MUST be justified in the
  Complexity Tracking table or rejected.
- **Docs**: README.md is the primary user-facing documentation. Module-level
  rustdoc comments document public APIs.

## Governance

This constitution supersedes all other development practices and conventions.
Any amendment requires:

1. A documented proposal (PR or issue) explaining the change and rationale.
2. Review against all existing specs and plans for consistency.
3. A version bump following semantic versioning:
   - **MAJOR**: Principle removal or backward-incompatible redefinition.
   - **MINOR**: New principle or section added.
   - **PATCH**: Clarifications, wording, non-semantic refinements.
4. Propagation of changes to affected templates and active plans.

All PRs and code reviews MUST verify compliance with the constitution. Any
complexity or dependency that violates a principle MUST be justified in the
plan's Complexity Tracking table and explicitly approved.

**Version**: 1.0.1 | **Ratified**: 2026-06-21 | **Last Amended**: 2026-06-24
