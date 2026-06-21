# Quickstart: External Tool Runtime (009)

**Feature**: [spec.md](./spec.md) | **Plan**: [plan.md](./plan.md) (see **Convergence consolidation**) | **Contract**: [contracts/tool-runtime-ui.md](./contracts/tool-runtime-ui.md) | **Tasks**: [tasks.md](./tasks.md) T001–T020

## Prerequisites

- Built `./rust-feh`
- Ability to temporarily hide/restore `feh` on PATH for failure simulation

## V1: Install feh mid-session (US1)

1. Ensure feh not on PATH; launch `./rust-feh`
2. **Verify** Open in feh / wallpaper buttons disabled (greyed)
3. **Verify** capabilities panel shows feh ✗ not installed
4. Install feh: `sudo apt install feh` (or restore PATH)
5. Click **Recheck tools on PATH** (panel)
6. **Verify** feh buttons enabled; panel shows feh ✓; log `Rechecked tools: feh=true, …`
7. Select image → **Open in feh** succeeds

**Pass**: SC-001, FR-003, FR-005, FR-007

## V2: Tools menu recheck (US4)

1. With feh absent, open **Tools** menu
2. **Verify** **Recheck tools on PATH** entry present (gap-fill target)
3. Install feh; trigger recheck from **menu** (not panel)
4. **Verify** same behavior as panel recheck; one log line per click

**Pass**: FR-004, FR-006 | **Tasks**: T008–T009

## V3: Spawn failure recovery (US3)

1. Start with feh on PATH; confirm available
2. Temporarily move feh binary: `sudo mv $(which feh) /tmp/feh.bak`
3. Click **Open in feh**
4. **Verify** `feh_available` becomes false; buttons disable; panel shows feh ✗; status shows install message
5. Restore: `sudo mv /tmp/feh.bak $(dirname $(which feh 2>/dev/null || echo /usr/bin/feh))/feh` — or reinstall
6. **Recheck tools on PATH** → feh available again

**Pass**: FR-008, SC-003 | **Tasks**: T010–T012, T018–T019

## V4: ImageMagick mid-session (US2)

1. Start without ImageMagick on PATH
2. **Verify** panel exotic format notes mention reduced capability
3. `sudo apt install imagemagick`
4. **Recheck tools on PATH**
5. **Verify** ImageMagick ✓; heic format note changes to magick-detected wording
6. Optional: **Rescan** folder to update inventory counts (recheck alone does not rescan)

**Pass**: SC-004

## Automated

```fish
cargo clippy -- -D warnings
cargo test tool_caps
```

Expected: 9+ `tool_caps` tests pass; clippy clean.

## Gap-audit output

Record pass/partial per FR in `gap-audit.md`; note 002 superseded (FR-011).