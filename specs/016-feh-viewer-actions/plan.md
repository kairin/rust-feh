# Implementation Plan: Viewer Round-Trip & Staged-Image Actions

**Branch**: `016-feh-viewer-actions` | **Date**: 2026-07-05 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/016-feh-viewer-actions/spec.md`

## Summary

Selecting an image displays it in a new rust-feh **stage pane** (lazy,
off-thread, bounded decode → egui texture) with a **right-click context menu**
(save-copy-to, move-to, resize copy, convert, copy path, copy image). feh
becomes the cycling engine via a **round-trip**: rust-feh spawns feh with an
isolated config profile and a per-image `--info` trail hook writing the
displayed image's path to a rust-feh-owned handoff file; rust-feh retains the
`Child`, polls `try_wait()` each frame, and on viewer exit reads + validates
the handoff and re-selects/stages that image. All file operations are
collision-safe and moves are loss-proof (copy-verify-remove). Verified on
target machine 2026-07-05: `--info 'echo %F > handoff'` runs per displayed
image with no on-screen output (command output redirected), and
`XDG_CONFIG_HOME` isolation gives stock-default, overlay-free viewers.

## Technical Context

**Language/Version**: Rust stable, edition 2021

**Primary Dependencies**: existing only — egui/eframe 0.30 (glow), `image` 0.25
(decode/resize/convert), `arboard` 3 (clipboard text + image), `rfd` 0.15
(native folder chooser), `serde`/`serde_json` (persisted prefs). **No new
dependencies** (FR-011).

**Storage**: handoff files under the existing `runtime_cache_dir()`
(`~/.cache/rust-feh/`), pid+viewer-scoped; isolated viewer profile dir under
`~/.config/rust-feh/viewer-profile/` (may be empty — its existence redirects
feh away from the personal config); last-used destination persisted following
the window-prefs pattern (`~/.config/rust-feh/action-prefs.json`).

**Testing**: `cargo test` — unit tests in `ui_logic.rs`/`image_proc.rs` for all
new pure logic (collision suffixing, loss-proof move, handoff parse/validate,
spawn arg construction, decode bounding math); integration tests under
`tests/integration/` for file ops + handoff round-trip (no GUI needed);
scripted GUI validation per quickstart.md.

**Target Platform**: Linux desktop (X11/XWayland for feh; rust-feh itself
Wayland-native or X11)

**Project Type**: Desktop GUI application (existing single crate)

**Performance Goals**: selection→displayed < 500 ms typical (SC-005); zero
frame stalls from decode (off-thread, latest-wins); 10k-image scan/filter
timings within 10% of baseline.

**Constraints**: no network; no new external tools; main.rs growth minimized —
new logic lives in core modules; Codacy 50-line/complexity caps; clippy
`-D warnings`.

**Scale/Scope**: single staged image at a time; folders up to 10k+ images;
up to a handful of concurrent viewer round-trips.

## Constitution Check

*GATE: evaluated against constitution v1.0.1 — PASS (with one documented
wording clarification).*

- **I. Thin-Wrapper**: PASS with clarification. feh remains the cycling/
  slideshow/wallpaper engine; the stage displays only the current selection
  (no navigation, zoom workbench, or editing — explicitly out of scope in the
  spec). This plan ships a `docs/POSITIONING.md` wording clarification
  ("selection stage ≠ viewer replacement"). No principle redefinition; if
  review disagrees, STOP and escalate to the maintainer (amendment procedure).
- **II. Pure Rust, Minimal Dependencies**: PASS — zero new dependencies.
- **III. Clean Module Separation**: PASS — all new logic (collision naming,
  loss-proof move, handoff protocol, spawn args, decode bounding) lands in
  `ui_logic.rs`/`image_proc.rs`/`types.rs` (egui-free, unit-tested); `main.rs`
  gets only thin rendering/wiring (stage texture, context menu, per-frame
  viewer polling).
- **IV. Linux-First, feh-Centric**: PASS — feh delegation is the round-trip's
  core; graceful degradation when feh is missing is unchanged; wallpaper stays
  `feh --bg-fill` (not in this feature's menu).
- **V. Performance Awareness**: PASS — decode is lazy, off-thread, bounded
  (max texture edge ~2048 px), latest-selection-wins; listing never reads
  image bytes (FR-001/SC-005).

**Post-design re-check (Phase 1)**: PASS — no violations introduced;
Complexity Tracking left empty.

## Project Structure

### Documentation (this feature)

```text
specs/016-feh-viewer-actions/
├── plan.md              # This file
├── research.md          # Phase 0: verified feh mechanics + decisions
├── data-model.md        # Phase 1: entities/state
├── quickstart.md        # Phase 1: manual validation scenarios
├── contracts/
│   ├── viewer-roundtrip.md      # spawn/handoff protocol contract
│   └── stage-context-menu.md    # stage pane + menu UI contract
└── tasks.md             # Phase 2 (/speckit-tasks)
```

### Source Code (repository root)

```text
src/
├── main.rs        # thin: stage pane render + texture cache, context menu wiring,
│                  #  per-frame ViewerRoundTrip polling, error dialogs (rfd)
├── ui_logic.rs    # NEW logic: collision_suffixed_path(), loss-proof move plan,
│                  #  viewer spawn arg/env construction (XDG_CONFIG_HOME, --info),
│                  #  handoff read+validate (path ∈ launched list), stage decode
│                  #  bounding math, action outcome formatting
├── image_proc.rs  # reuse: resize/convert routing for context actions;
│                  #  decode-to-RGBA for stage + clipboard image
├── types.rs       # NEW types: StagedImage state, ViewerRoundTrip, ContextAction,
│                  #  ActionOutcome, ActionPrefs (persisted last destination)
├── scanner.rs     # unchanged
└── tool_caps.rs   # unchanged (feh presence already handled)

tests/
├── unit/ui_logic.rs                      # suffixing, handoff validation, spawn args, bounding
└── integration/feature_016_roundtrip.rs  # file ops + handoff round-trip fixtures
```

**Structure Decision**: existing single-crate layout; no new modules. GUI-free
logic goes to `ui_logic.rs`/`image_proc.rs`/`types.rs` per Constitution III;
`main.rs` additions are rendering and event wiring only, kept within Codacy
function limits by delegating to core helpers. Note: current feh spawns drop
the `Child` handle; round-trip spawns retain it (also fixes zombie reaping for
these spawns via `try_wait`).

## Phase 0 → research.md (complete)

All unknowns resolved by direct verification on the target machine — see
`research.md`. No NEEDS CLARIFICATION remain.

## Phase 1 → data-model.md, contracts/, quickstart.md (complete)

Agent context refreshed via the agent-context extension after plan.

## Complexity Tracking

*(empty — no constitution violations to justify)*
