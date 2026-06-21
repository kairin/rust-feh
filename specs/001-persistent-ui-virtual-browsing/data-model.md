# Data Model: Persistent UI Layout & Virtual Browsing

**Feature**: 001-persistent-ui-virtual-browsing
**Date**: 2026-06-21

## Entities

### RustFehApp (Application State)

The central state container for the egui application. GUI state fields added in this
phase per Clarifications 2026-06-21: `feh_available`, `scanning`, `scroll_generation`,
`prior_search`.

| Field | Type | Description |
|-------|------|-------------|
| `current_dir` | `Option<PathBuf>` | Currently loaded directory, or None if no folder chosen |
| `images` | `Vec<ImageEntry>` | All discovered image files in current_dir |
| `selected` | `Option<PathBuf>` | Currently selected image path, or None |
| `status` | `String` | Status message displayed in bottom bar |
| `debug_logs` | `Vec<String>` | Ring buffer of debug messages (max 100, oldest dropped) |
| `search` | `String` | Filter/search text for filename matching |
| `recursive` | `bool` | Whether to include subdirectories when scanning |
| `feh_available` | `bool` | Whether `feh` was found on PATH at startup (FR-008a) |
| `scanning` | `bool` | Whether a directory scan is in progress (FR-010, FR-013) |
| `scroll_generation` | `u64` | Incremented when `search` changes; passed to `ScrollArea::id_salt` to reset list scroll (FR-005) |
| `prior_search` | `String` | Previous filter value; compared to `search` to detect change and bump `scroll_generation` |
| `sort_mode` | `SortMode` | Active list sort (Path / Name / Folder) — see [005 data-model](../005-image-list-presentation/data-model.md) |
| `prior_sort_mode` | `SortMode` | Previous sort; bump `scroll_generation` on change (FR-006, extended in 005) |
| `tool_caps` | `ToolCapabilities` | Detected feh/ImageMagick on PATH (008 panel) |
| `scan_inventory` | `Option<ScanInventory>` | Post-scan counts snapshot (005 US5); `None` until first scan completes |
| `list_view_mode` | `ListViewMode` | `FlatList` (default) or `FolderTree` (005 US4) |
| `tree_expanded_paths` | `HashSet<String>` | Session-only expanded folder keys (e.g. `"."`, `"sub"`) |

**State transitions**:

```
INITIAL → (app starts, detect feh) → INITIAL [feh_available set]
INITIAL → (user clicks "Choose folder") → LOADING [images cleared, selected=None, scanning=true]
LOADING → (scan completes, images found) → LOADED [first image auto-selected]
LOADING → (scan completes, zero images) → LOADED [selected=None, status="No images found"]
LOADED  → (user changes recursive toggle) → LOADING → LOADED
LOADED  → (user clicks "Choose folder" again) → LOADING → LOADED
LOADED  → (user types in search) → FILTERED [scroll resets to top]
FILTERED → (user clears search) → LOADED [full list restored, scroll resets to top]
```

**Selection lifecycle** (FR-012):
- `selected` MUST be set to `None` when `scan_directory` begins (before scan runs).
- After scan completes with `images.len() > 0`, `selected` MUST be set to `images[0].path`.
- After scan completes with `images.is_empty()`, `selected` MUST remain `None`.
- Filter changes do NOT clear `selected`; if the selected image is filtered out, it
  remains selected but may not be visible in the list until the filter is cleared.

### ImageEntry (Domain Type)

Represents a single discovered image file. Extended in feature **005**.

| Field | Type | Description |
|-------|------|-------------|
| `path` | `PathBuf` | Absolute path to the image file |
| `size_bytes` | `Option<u64>` | File size in bytes (None = not yet queried) |
| `status` | `FileStatus` | `NativeListed` (scanner default), `MagickDetected`, or `Converted` (005) |

**Constraints**:
- `path` MUST be a valid, absolute filesystem path
- Native scanner extensions: jpg, jpeg, png, webp, gif, bmp
- Additional formats MAY appear when ImageMagick `identify` succeeds during scan (005)
- `size_bytes` is lazy — populated when metadata is needed, not during scan

**Future enrichment** (Area 6): dimensions (width, height), modification time,
image format enum.

### FilteredView (Logical Construct)

Not a struct — a transient `Vec<usize>` from `ui_logic::list_indices()` each frame.

| Attribute | Description |
|-----------|-------------|
| Source | `self.images` |
| Filter | `self.search` — case-insensitive substring on filename, relative folder, full path (005 FR-003) |
| Sort | `self.sort_mode` — Path, Name, or Folder (005 FR-002) |
| Result | `Vec<usize>` of indices into `images` |
| When empty search | All indices (then sorted) |
| When filter active | Matching indices only (then sorted) |

**Recomputation trigger**: Every `update()` frame via `compute_list_indices()`.
Flat list and folder tree both derive visible rows from the same filter/sort logic.

**Display mapping**: Flat list uses Folder + Filename + Status columns; tree mode
uses `tree_visible_rows()` with lazy expand state in `tree_expanded_paths`.

### Selection (Existing, Unused)

`Selection` struct in `types.rs` with `selected: Option<PathBuf>`. Marked
`#[allow(dead_code)]`. Not used in current implementation (selection state
is inline in RustFehApp). Removal or integration deferred to Area 8.

### SortMode (Shipped — feature 005)

`SortMode` in `types.rs`: `Path` (default), `Name`, `Folder`. Applied in
`ui_logic::list_indices()` after filter. Scanner still sorts by path on ingest;
display order follows user-selected `sort_mode`.

### FileStatus, ListViewMode, ScanInventory (feature 005)

Defined in `types.rs`; see [005 data-model](../005-image-list-presentation/data-model.md).

| Type | Role |
|------|------|
| `FileStatus` | Row tag: native / magick-detected / converted |
| `ListViewMode` | Flat list vs folder tree presentation |
| `ScanInventory` | `native_listed`, `magick_detected`, `converted`, `awaiting_convert`, `non_image_skipped` |

`ScanResult` from `scanner::scan_images()` returns `entries`, `warnings`, and
initial `inventory`; `ui_logic::finalize_scan_entries()` applies converted detection
and rebuilds inventory before GUI display.

## Relationships

```
RustFehApp
  ├── images: Vec<ImageEntry>           (owned, scanned from current_dir)
  ├── scan_inventory: Option<ScanInventory>  (005 summary bar)
  ├── selected: Option<PathBuf>         (reference into images[].path)
  ├── list_view_mode: ListViewMode      (flat vs tree)
  └── [per-frame] list_indices / tree_visible_rows
```

- `selected` always points to a path that exists in `images` (or is None).
  The invariant is maintained: clearing `images` (via rescan or new folder)
  always sets `selected = None`.
- `FilteredView` is purely derived — it never owns data and is never stored
  across frames. It is a rendering optimization, not a persistent state.

## Validation Rules

1. `selected` MUST be None after `images.clear()` (enforced in `scan_directory`).
2. `search` is case-insensitive UTF-8 substring match on filename as displayed (`to_lowercase`).
3. `debug_logs` maintains max 100 entries (oldest removed on overflow).
4. `recursive` toggle triggers immediate rescan with updated flag.
5. Bottom status bar always reflects `images.len()` (total) and `filtered.len()` (shown)
   in format "Showing X / Y images" (FR-006). Counter MUST NOT appear in CentralPanel.
6. `feh_available == false` disables feh-dependent buttons and prevents spawn attempts.
7. `scanning == true` sets status message to "Scanning…" (overridden by post-scan status).
