# Quickstart: Tool Capabilities Panel (008)

**Feature**: [spec.md](./spec.md) | **Plan**: [plan.md](./plan.md) | **Contract**: [contracts/capabilities-panel-ui.md](./contracts/capabilities-panel-ui.md)

## Prerequisites

- Built `./rust-feh`
- Optional: temporarily hide `feh` from PATH to test missing-required UX

## V1: Dependencies (US1–US2)

1. Launch `./rust-feh`
2. Open right **Tools & capabilities** panel
3. **Verify** feh row: required, installed or not, role text, `On PATH` or install cmd
4. **Verify** ImageMagick row: optional, same pattern
5. If feh missing: click **Copy** beside install cmd → paste in terminal (manual) → **Recheck tools on PATH** after install

**Pass**: SC-001, SC-002, FR-001–FR-005

## V2: Speed and format routing (US3)

1. With default PATH, read **Speed / timing** — five operation rows with tiers
2. Read **Format discovery** — legend mentions inventory bar (005 alignment)
3. **Verify** jpg group: native listed note; heic group changes when magick present vs absent

**Pass**: FR-006–FR-008, SC-003 (unit tests), SC-005 (vs POSITIONING.md)

## V3: Layout (US4)

1. Widen window — panel resizes horizontally (min ~240px)
2. Narrow height — scroll panel to reach format groups and bottom warning

**Pass**: FR-012–FR-013

## V4: Recheck (panel; 009 extends to menu)

1. Click **Recheck tools on PATH**
2. **Verify** debug log: `Rechecked tools: feh=…, magick=…`
3. If feh was installed mid-session, feh buttons enable without restart

**Pass**: SC-004 (with 009 menu parity optional)

## Automated

```fish
cargo clippy -- -D warnings
cargo test tool_caps
```

Expected: 5 `tool_caps` tests pass; clippy clean.

## Gap-audit output

Record pass/partial per FR in `gap-audit.md` after manual V1–V4.