# Contract: External Tool Runtime UI (009)

**Updated**: 2026-06-22 (post-008)

## Recheck triggers

| Trigger | Location | Handler | Status |
|---------|----------|---------|--------|
| Panel button | `render_tool_caps_panel` | `refresh_tool_caps()` | ✅ Shipped |
| Tools menu | `Tools` menu bar | `refresh_tool_caps()` | ❌ Gap-fill FR-004 |

**Label** (both): `Recheck tools on PATH`

**Effect**:
- `tool_caps = ToolCapabilities::detect()`
- `feh_available = tool_caps.feh_available`
- Debug log: `Rechecked tools: feh={bool}, magick={bool}`
- Panel re-renders with updated deps, timings, format routes (008 FR-008)

## feh control enablement (001 FR-008a)

| Control | Enabled when | Location |
|---------|--------------|----------|
| Open in feh (toolbar) | `feh_available` | `main.rs` toolbar |
| Open in feh (Tools menu) | `feh_available` | `main.rs` L417 |
| Set wallpaper | `feh_available` | toolbar |

Disabled controls use `feh_button()` styling; click sets `feh_missing_status()`.

## Spawn failure (FR-008)

| Function | On `NotFound` (+ PATH confirm) |
|----------|-------------------------------|
| `open_in_feh` | `feh_available = false`, `tool_caps.feh_available = false`, `status = feh_missing_status()` |
| `set_wallpaper` | Same as above |

**Do not** flip on non-NotFound errors when `which feh` still succeeds.

## Status messaging

| Condition | Status text source |
|-----------|-------------------|
| feh missing (guard) | `feh_missing_status()` from `ui_logic.rs` |
| feh spawn failed (not found) | `feh_missing_status()` after sync |
| feh spawn failed (other) | Error detail; keep `feh_available` if PATH ok |

## Cross-feature

| Feature | Contract |
|---------|----------|
| 008 | Panel displays snapshot; no duplicate detect logic |
| 005 | `magick_available` at scan time from `self.tool_caps`; rescan needed for inventory re-classify |
| 002 | Tools menu recheck supersedes "Re-check feh" |