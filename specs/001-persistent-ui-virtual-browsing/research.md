# Research: Persistent UI Layout & Virtual Browsing

**Feature**: 001-persistent-ui-virtual-browsing
**Date**: 2026-06-21

## Research Questions

### RQ1: How to implement persistent top controls in egui?

**Decision**: Use `egui::TopBottomPanel::top("controls")` for all persistent controls.

**Rationale**:
- `TopBottomPanel` is a built-in egui container that renders in a fixed,
  non-scrolling region. The CentralPanel scrolls independently.
- This is the standard egui pattern for toolbars, menu bars, and
  always-visible controls.
- No third-party crate required. No custom layout code.
- The panel can contain nested horizontal layouts, menu bars, text
  inputs, and buttons — all standard egui widgets.

**Alternatives considered**:
- `SidePanel` — useful for left/right tool palettes but unnecessary for
  a horizontal toolbar.
- `Window` (floating) — would overlap content and feel non-native.
- Custom `Area` — overengineered for a simple toolbar.
- `egui_dock` / `egui_tiles` — tab/docking systems, too heavy for this
  use case.

### RQ2: How to virtualize a large list in egui?

**Decision**: Use `ScrollArea::vertical().show_rows(...)` with a pre-computed
filtered index vector.

**Rationale**:
- `show_rows` is built into egui's `ScrollArea`. It takes a row height, a
  total logical row count, and a closure that renders only the visible range.
- Only widgets for on-screen rows are created — O(visible) cost, not O(total).
- Row height of ~18px works for single-line `selectable_label` items.
- Pre-computing filtered indices (a `Vec<usize>` of indices into `images`)
  is O(n) per frame for n=10k, which completes in <1ms in Rust. This is
  fast enough to recompute every frame without caching.

**Alternatives considered**:
- `egui_virtual_list` crate — external dependency, and egui 0.30's built-in
  `show_rows` is sufficient.
- `egui_extras::TableBuilder` — designed for tabular data with columns;
  overkill for a simple list.
- Custom scroll area with manual range culling — possible but reinvents
  what `show_rows` already provides.

### RQ3: How to structure the filtered-index computation?

**Decision**: Pre-compute `Vec<usize>` of indices at the start of each
`CentralPanel` render, before calling `show_rows`.

**Rationale**:
- Filtering is a pure function: `images + search_string → Vec<usize>`.
- Computing it once per frame outside the `show_rows` closure keeps the
  closure simple and avoids re-filtering on every visible-row paint.
- The `show_rows` closure receives a `row_range` and indexes into the
  pre-computed `filtered` vec, then looks up `images[filtered[row]]`.
- This separates concerns: filtering logic (what to show) from rendering
  logic (how to paint each row).

### RQ4: How to remove auto-feh-launch on folder load?

**Decision**: Remove the `self.open_in_feh(&p)` call from the folder-load
block. Update the status message to guide the user to click "Open in feh".

**Rationale**:
- The original code auto-opened feh as a convenience but user feedback
  indicated it was surprising/spammy.
- The fix is a one-line removal — no structural changes needed.
- The first image is still auto-selected and highlighted, so the user
  can immediately act on it.
- The status text changes from the implicit path-only display to
  an explicit prompt: "Loaded N images. First selected (click Open in
  feh to view)."

### RQ5: Where to place the debug log viewer?

**Decision**: Keep it as a `CollapsingHeader` inside the CentralPanel,
below the list.

**Rationale**:
- The debug log is a developer tool, not a primary user control.
- Placing it in the CentralPanel (scrollable with the list) is acceptable
  because it's collapsible and the user must explicitly expand it.
- Moving it to the bottom status bar would clutter the status line.
- A floating window is an option for Area 8 but not needed now.
- The spec (FR-009) requires it to not compete with primary controls —
  collapsing it by default satisfies this.

## Implementation Summary

Research decisions are **designed**; FR satisfaction is tracked per `gap-audit.md` (2026-06-21
adversarial audit). Scaffold ≠ complete.

| Element | Scaffold | FR status | Gap / task |
|---------|----------|-----------|------------|
| Top panel | `TopBottomPanel::top("controls")` + toolbar | FR-001 partial | T019, T027 |
| Bottom panel | status text only (no counter) | FR-006 **gap** | T021 |
| Virtual list | `show_rows` + filtered indices | FR-004 pass | — |
| Filter + scroll | filter works; no scroll reset | FR-005 partial | T033 |
| No auto-feh | removed from folder-pick path | FR-007 partial | T049 (scan_directory) |
| Debug log | `CollapsingHeader` collapsed | FR-009 partial | T053 (startup log) |
| Menu bar | `menu::bar` present | FR-011 **gap** | T022–T025 |
| feh degradation | not implemented | FR-008a **gap** | T010, T047 |
| Scanning state | not implemented | FR-010/FR-013 **gap** | T011, T034 |
| Scanner warnings | silent skip | FR-015 **gap** | T013, T014 |

Decomposed fix instructions: `remediation.md`. Verify via quickstart.md V1–V10.
