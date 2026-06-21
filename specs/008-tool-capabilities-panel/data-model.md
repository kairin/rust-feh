# Data Model: Tool Capabilities Panel (008)

**Module**: `src/tool_caps.rs`  
**GUI**: `RustFehApp::render_tool_caps_panel` in `src/main.rs`

## Entities

### ToolCapabilities

Snapshot of detected tools on PATH. Refreshed at app startup and on **Recheck tools on PATH**.

| Field | Type | Rule |
|-------|------|------|
| feh_available | bool | `which("feh").is_ok()` |
| magick_available | bool | `which("magick")` or `which("convert")` |
| magick_binary | Option<String> | Resolved path; `magick` preferred over `convert` |

**Consumers**: `main.rs` (`feh_available`, panel render), `scan_images(..., magick_available)` (005).

### DependencyStatus

| Field | Type | Notes |
|-------|------|-------|
| name | &'static str | `feh`, `ImageMagick` |
| binaries | &'static [&'static str] | Search order |
| kind | DepKind | Required / Optional |
| role | &'static str | One-line capability description |
| install_cmd | &'static str | `sudo apt install …` |
| installed | bool | From snapshot |
| resolved_binary | Option<String> | Display name when installed |

### OperationTiming

| Field | Type |
|-------|------|
| operation | &'static str |
| handler | Handler |
| speed | SpeedTier |
| note | &'static str |

**Required operations** (FR-006): browse/filter/sort, open/slideshow, wallpaper, quick resize, exotic format view.

### FormatRoute

| Field | Type |
|-------|------|
| extensions | &'static str |
| scan | Handler |
| view | Handler |
| resize | Handler |
| view_speed | SpeedTier |
| note | &'static str |

**Dynamic rules** (FR-008): `view`/`scan` handlers and notes vary with `feh_available` and `magick_available`.

### Enums

| Enum | Variants |
|------|----------|
| SpeedTier | Instant, Fast, Medium, Slow |
| Handler | RustFeh, Feh, ImageCrate, ImageMagick |
| DepKind | Required, Optional |

## App state (main.rs)

| Field | Type | Notes |
|-------|------|-------|
| tool_caps | ToolCapabilities | Refreshed via `refresh_tool_caps()` |
| feh_available | bool | Mirrored from `tool_caps.feh_available` (001 FR-008a) |

## Refresh lifecycle

```text
startup → ToolCapabilities::detect()
user clicks Recheck → refresh_tool_caps() → detect() again → feh_available sync
(009) Tools menu Recheck → same refresh_tool_caps()
```

## Relationships

```text
ToolCapabilities
  ├── dependencies() → [DependencyStatus]
  ├── operation_timings() → [OperationTiming]
  └── format_routes() → [FormatRoute]

render_tool_caps_panel
  └── reads snapshot only (no detect per frame)
```

## Validation rules

1. feh MUST always appear as `DepKind::Required`.
2. ImageMagick MUST always appear as `DepKind::Optional`.
3. `has_missing_required()` true only when feh absent.
4. Format route notes MUST NOT claim convert pipeline is live (010 deferred).
5. Scan notes MUST reference 005 inventory terminology when describing listed files.

## Persistence

None. Snapshot is session-only.