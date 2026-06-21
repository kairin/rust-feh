# Implementation Plan: Persistent UI Layout & Virtual Browsing

**Branch**: `001-persistent-ui-virtual-browsing` | **Date**: 2026-06-21 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `specs/001-persistent-ui-virtual-browsing/spec.md`

**Note**: This feature is partially implemented — core structural changes (panels, show_rows,
auto-open removal) are already in the codebase. This plan documents the design decisions and
identifies remaining verification and polish work.

**Updated**: 2026-06-21 — `/speckit-clarify` added Clarifications session; `/speckit-tasks`
regenerated tasks.md (64 tasks, FR-001–FR-015 traceability table).

## Summary

Refactor the single-CentralPanel egui layout into a multi-panel structure with persistent
top controls (folder picker, filter, actions), virtualized image list via `show_rows`,
a persistent bottom status bar, and a clarified selection model where folder-load selects
the first image but does not auto-launch feh. This is the foundational UX layer that
thumbnails, menu bars, and richer interactions will build upon.

## Technical Context

**Language/Version**: Rust stable, edition 2021 (rustc 1.75+)
**Primary Dependencies**: egui 0.30, eframe 0.30 (glow backend), walkdir 2, rfd 0.15, image 0.25, which 6
**Storage**: N/A (in-memory state only; persistence deferred to Area 8)
**Testing**: `cargo test`, `cargo clippy -- -D warnings`
**Target Platform**: Linux (X11/Wayland via glow OpenGL backend)
**Project Type**: desktop-app (single Cargo crate, monolithic `src/main.rs` with modules)
**Performance Goals**: 60fps scroll on 10k images, <200ms filter response, <150MB RSS
**Constraints**: Single crate (no workspace split), sync scanning retained (async deferred to Area 6)
**Scale/Scope**: 10k-20k image directories, single binary, developer tool for Linux workstations

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Evidence |
|-----------|--------|----------|
| I. Thin-Wrapper Architecture | ✅ PASS | No feh features reimplemented. GUI delegates to feh via subprocess for viewing/wallpaper. Core modules (scanner, image_proc, types) unchanged. |
| II. Pure Rust, Minimal Dependencies | ✅ PASS | No new dependencies added. egui built-in `TopBottomPanel`, `menu::bar`, `show_rows` used. |
| III. Clean Module Separation | ✅ PASS | GUI refactored within `main.rs` only. scanner, image_proc, types modules untouched and independent. |
| IV. Linux-First, feh-Centric | ✅ PASS | No cross-platform changes. feh integration unchanged (same subprocess spawn). ImageMagick detection unchanged. |
| V. Performance Awareness | ✅ PASS | `show_rows` virtualization directly targets performance for large collections. LTO/strip release profile unchanged. |

**Gate result**: ALL PASS. No violations. No Complexity Tracking needed.

## Implementation Status (2026-06-21 adversarial audit)

Scaffold present: `TopBottomPanel`, `show_rows`, no auto-feh on folder pick.

**Gaps** (see `gap-audit.md`, fixes in `remediation.md`):

| FR | Status |
|----|--------|
| FR-006 | gap — counter in CentralPanel |
| FR-008a | gap — no feh PATH detection |
| FR-010, FR-013 | gap — no scanning indicator |
| FR-011 | gap — menu stubs |
| FR-012 | partial — selection not in `scan_directory` |
| FR-015 | gap — scanner silent on permission denied |

Code landed 2026-06-21. **Convergence phase 7** (T065–T069): 2 code fixes, 1 consolidated
validation session, artifact sync. See `outstanding-issues.md`.

## Project Structure

### Documentation (this feature)

```text
specs/001-persistent-ui-virtual-browsing/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── gap-audit.md         # T004 — FR pass/partial/gap table
├── remediation.md       # Decomposed implementation steps
├── adversarial-review.md
├── contracts/           # Phase 1 output (empty — no external interfaces)
└── tasks.md             # Phase 2 output (/speckit-tasks)
```

### Source Code (repository root)

```text
src/
├── main.rs              # egui App: panels, menu, toolbar, virtualized list, bottom status
├── scanner.rs           # Directory scanner (walkdir, unchanged)
├── image_proc.rs        # Image processor (image crate, unchanged)
└── types.rs             # Domain types (ImageEntry, Selection, SortMode)

archive/
└── original-nfeh/       # Archived legacy code (not referenced by build)
```

**Structure Decision**: Single crate retained per project conventions. The egui app
lives in `src/main.rs` with methods distributed across `impl` blocks. Module extraction
(e.g., `widgets/`, `state.rs`) deferred to Area 8 (Code Organization).

## Complexity Tracking

> No violations. Section intentionally empty.

## Phase 0: Research Notes

Research findings consolidated in [research.md](./research.md). Key decisions:

| Decision | Rationale |
|----------|-----------|
| `TopBottomPanel::top` for controls | Standard egui pattern for persistent toolbars. Non-scrolling by definition. Controls always visible. |
| `TopBottomPanel::bottom` for status | Keeps status line fixed while image list scrolls. Standard UX pattern. |
| `show_rows` for virtualization | Built into egui ScrollArea. Only renders visible rows. No external crate needed. |
| Pre-computed filtered indices | Compute once per frame, pass to show_rows. Avoids re-filtering inside the render closure. O(n) per frame for 10k items is acceptable (<1ms). |
| No auto-feh on load | Removes the `open_in_feh(&p)` call from the folder-load path. Status message updated to say "click Open in feh to view". |
| Menu bar in top panel | Starts Area 2 (menu/navigation). File/View/Tools structure. Menu actions MUST be functional per FR-011 (not stubs). |
| Counter in bottom status bar | "Showing X / Y images" lives in `TopBottomPanel::bottom`, not CentralPanel (FR-006). |
| Proactive feh detection | `which::which("feh")` at startup; disable buttons when absent (FR-008a, Constitution §IV). |
| Sync scan responsiveness | Controls stay visible; status shows "Scanning…"; render loop may briefly stall (FR-010). Async deferred to Area 6. |
| Filter scroll reset | Scroll position resets to top on filter change (FR-005). |

## Phase 1: Design

### Data Model

See [data-model.md](./data-model.md) for full entity definitions. Summary:

- **RustFehApp** (state container): adds `feh_available`, `scanning`, `list_scroll_offset` per data-model.md
- **ImageEntry**: unchanged (path, size_bytes)
- **FilteredView**: logical construct — a `Vec<usize>` of indices into `images`, recomputed when `search` or `images` changes

### Contracts

No external interfaces. This is a desktop GUI application with no API, CLI contract, or
network boundary. Contracts directory is intentionally empty.

### Quickstart

See [quickstart.md](./quickstart.md) for build, run, and validation procedures.

### Agent Context Update

AGENTS.md updated to reference this plan file via the managed SPECKIT section.
