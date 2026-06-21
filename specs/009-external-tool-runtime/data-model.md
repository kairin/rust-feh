# Data Model: External Tool Runtime (009)

**Module**: `src/tool_caps.rs` (detect)  
**Triggers**: `RustFehApp::refresh_tool_caps`, startup, spawn-failure handler in `src/main.rs`  
**Consumers**: `feh_button`, capabilities panel, `scan_images(..., magick_available)`

## Entities

### ToolCapabilities (runtime snapshot)

| Field | Type | Rule |
|-------|------|------|
| feh_available | bool | `which("feh").is_ok()` at last detect |
| magick_available | bool | `which("magick")` or `which("convert")` |
| magick_binary | Option<String> | `magick` preferred over `convert` |

**Lifecycle**: Created at startup; replaced on recheck; `feh_available` field may be forced `false` on spawn failure without full recheck.

### FehAvailability (derived flag)

| Field | Location | Rule |
|-------|----------|------|
| feh_available | `RustFehApp` L87 | MUST match `tool_caps.feh_available` after every `refresh_tool_caps()`; may be set `false` on spawn failure in sync with `tool_caps` |

**Consumers**: `feh_button()`, `try_open_in_feh`, `try_set_wallpaper`, `post_scan_status`.

### ReCheckResult (log contract)

| Field | Format |
|-------|--------|
| log line | `Rechecked tools: feh={bool}, magick={bool}` |

One line per user-initiated recheck (panel or menu). Idempotent — repeated clicks log each action (user expectation).

## State transitions

```text
startup
  → ToolCapabilities::detect()
  → feh_available = tool_caps.feh_available

recheck (panel L254–256 | menu [gap-fill])
  → refresh_tool_caps()
  → tool_caps = detect()
  → feh_available = tool_caps.feh_available
  → log ReCheckResult

feh spawn Err (NotFound + which confirms)
  → feh_available = false
  → tool_caps.feh_available = false
  → status = feh_missing_status()
  → panel reflects on next render (dependencies, timings)

no transition
  → background polling (forbidden FR-010)
  → auto-rescan on magick change (out of scope R5)
```

## Relationships

```text
ToolCapabilities::detect()
  └── refresh_tool_caps() [main.rs]
        ├── feh_available (app field)
        ├── render_tool_caps_panel (008)
        └── scan_images(magick_available) [on next Rescan only]

open_in_feh / set_wallpaper
  └── on NotFound → sync feh_available + tool_caps.feh_available
```

## Validation rules

1. Recheck MUST update feh and magick in one `detect()` call (FR-005).
2. Menu and panel recheck MUST call the same function (FR-004/FR-005).
3. Spawn failure MUST NOT mark unavailable unless PATH confirms feh absent (R3).
4. Detection logic MUST remain in `tool_caps.rs` (FR-009).
5. Feature 002 FR-001–FR-006 satisfied by this unified model (FR-011).

## Persistence

None. Snapshot is session-only.

## Supersession

| Legacy | Replacement |
|--------|-------------|
| 002 `Re-check feh` menu only | 009 unified recheck (feh + magick) |
| 002 startup-only detect | 009 on-demand recheck + spawn recovery |