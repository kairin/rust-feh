# Contract: Scan Inventory & List UI (005)

**Updated**: 2026-06-22 (adversarial remediation)

## Inventory summary bar

**Location**: Central panel, above column headers (filter/sort/view controls remain in top panel).

**Fields** (all non-negative integers):

| Label | Key | Rule |
|-------|-----|------|
| Images listed (native) | `native_listed` | Count of `FileStatus::NativeListed` |
| Magick-detected (unlisted) | `magick_detected` | `awaiting_convert` + magick-origin `Converted` |
| Converted | `converted` | All `FileStatus::Converted` (native or magick) |
| Awaiting convert | `awaiting_convert` | Count of `FileStatus::MagickDetected` (FR-011) |
| Non-image files skipped | `non_image_skipped` | Walk total; not per-folder in tree (deferred **004**) |

**When magick absent**: `magick_detected` and `awaiting_convert` are `0`; hint: "Install ImageMagick to detect more formats (Tools panel)."

**When identify cap hit**: `magick_identify_truncated` true; UI may show cap message.

**After Quick resize**: `refresh_entry_and_inventory` updates selected row + inventory without full rescan when `{stem}_processed.*` appears.

## Flat list columns

| Column | Content |
|--------|---------|
| Folder | Relative folder path |
| Filename | Base name |
| Status | `native`, `magick · awaiting convert`, or `converted` |

## Folder tree row types

| Row kind | Prefix | Selectable |
|----------|--------|------------|
| Folder collapsed | `▶ path/` | No (expand only) |
| Folder expanded | `▼ path/` | No |
| Native file | `● name` | Yes |
| Magick file | `○ name` | Yes |
| Converted file | `● name [converted]` | Yes |

**Folder line suffix** (SC-005):
- `{listed} listed` = `NativeListed` count in subtree (matches inventory `native_listed` policy at root)
- `{magick} magick` = `MagickDetected` count in subtree (omit if 0)
- `{skipped} skipped` = scan-wide total on **root** only; subfolders omit until **004**

## View toggle

| Control | Values | Default |
|---------|--------|---------|
| View mode | `Flat list` \| `Folder tree` | `Flat list` |

Changing mode MUST NOT clear scan data or selection.

## Filter interaction

When filter non-empty:
- Flat list: `list_indices`
- Tree: matching entries + ancestor folders visible (`effective_expanded_paths`); folder counts reflect filtered subset