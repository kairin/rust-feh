# Contract — Stage Pane & Context Menu (UI)

## Stage pane

- Location: central panel area, below the scan-inventory banner, sharing the
  central space with the image list (list keeps priority; stage takes the
  remaining height, minimum sensible height, collapsible). Exact split decided
  at implementation; the contract is: **list usability at 10k images is
  unchanged** (SC-005) and the stage is visible without scrolling at the
  default window size.
- Content: the currently selected image, scaled to fit (no crop, no upscale
  beyond 1:1), centered. While decoding: lightweight "Loading…" state (no
  spinner storms). Undecodable: placeholder with filename + reason (FR-012).
- The stage NEVER draws text over the image itself (labels live outside the
  image rectangle) — parity with the no-overlay requirement.
- Selection changes always win: a stale decode result is discarded
  (generation check), never displayed.

## Context menu (right-click on the stage image)

Exact items, in order (FR-002):

1. Save a copy… → rfd folder chooser (starts at last destination) → collision-safe copy
2. Move to… → rfd folder chooser → loss-proof move → stage advances to next surviving image
3. Resize copy → derived file per existing Image Tools rules
4. Convert format ▸ → submenu of supported targets per existing routing
5. Copy path → clipboard text
6. Copy image → clipboard image (fallback: path + explanatory message)

Rules:
- Items that cannot apply (e.g. Copy image / Resize on an undecodable file)
  are disabled, not hidden (FR-012); Copy path / Move / Save copy stay enabled.
- While a chooser dialog is open, re-invoking a dialog-bearing item is refused
  (no queued duplicates).
- Every item completes or fails with an ActionOutcome logged; failures also
  raise an rfd error dialog naming image + cause within 2 s (FR-010/SC-007).
- Menu contains EXACTLY these six items in v1 — no wallpaper, no editing.

## Shortcut/discoverability surface

- The existing viewer-launch controls area gets one line/tooltip describing
  the round-trip ("browse in feh; closing it stages the image you were on")
  — the FR-010 (spec) discoverability requirement carries to this feature's
  UI copy, not on-image text.
