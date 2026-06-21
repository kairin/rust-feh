# Data Model: Image List Presentation (005)

## Existing (shipped)

### SortMode
| Variant | Sort key |
|---------|----------|
| Path | Full path lowercase |
| Name | Filename lowercase |
| Folder | Relative folder + filename |

### ImageEntry (extended)
| Field | Type | Notes |
|-------|------|-------|
| path | PathBuf | Absolute path |
| size_bytes | Option<u64> | Still None at scan (Area 6) |
| status | FileStatus | **NEW** — default NativeListed for scanner natives |

## New enums

### ListViewMode
| Variant | UI |
|---------|-----|
| FlatList | Folder + Filename + Status columns |
| FolderTree | Expandable hierarchy per UX Reference |

### FileStatus
| Variant | Display tag | Selectable |
|---------|-------------|------------|
| NativeListed | `native` | Yes |
| MagickDetected | `magick · awaiting convert` | Yes when magick on PATH |
| Converted | `converted` | Yes |

## ScanInventory

Snapshot after each `scan_images` completion.

| Field | Type | Rule |
|-------|------|------|
| native_listed | usize | Count of `FileStatus::NativeListed` entries |
| magick_detected | usize | `awaiting_convert` + `Converted` entries with non-native extension (magick-origin) |
| converted | usize | All entries with `FileStatus::Converted` (native or magick) |
| awaiting_convert | usize | Count of `FileStatus::MagickDetected` entries (FR-011) |
| non_image_skipped | usize | Files walked, not native-listed, not magick-detected |
| magick_identify_truncated | bool | True if R2 identify cap hit |

**Rules** (implemented in `ScanInventory::from_entries`):
- `awaiting_convert` = `entries.filter(|e| e.status == MagickDetected).count()`
- `magick_detected` = `awaiting_convert` + converted magick-origin count
- Native `*_processed.*` conversion increments `converted` only, not `magick_detected`

## FolderTreeNode

| Field | Type |
|-------|------|
| relative_path | String | e.g. `2024/vacation` or `.` |
| listed_count | usize | `NativeListed` entries in subtree (matches inventory `native_listed` policy, SC-005) |
| magick_count | usize | `MagickDetected` entries in subtree |
| skipped_count | usize | Subtree non-image (root uses scan-wide total; per-folder deferred to **004**) |
| children | BTreeMap<String, FolderTreeNode> | Sorted child folders |
| expanded | — | UI state in `tree_expanded_paths: HashSet<String>` on app, not on node |

## ScanResult (scanner output)

| Field | Type |
|-------|------|
| entries | Vec<ImageEntry> | All selectable images |
| warnings | Vec<String> | Permission denied etc. |
| inventory | ScanInventory | Aggregate counts |

## App state additions (main.rs)

| Field | Type |
|-------|------|
| list_view_mode | ListViewMode | Default FlatList |
| scan_inventory | Option<ScanInventory> | None until first scan |
| tree_expanded_paths | HashSet<String> | Session UI state |

## Relationships

```text
scan_images() → ScanResult
  → entries[] → list_indices() / build_folder_tree()
  → inventory → inventory summary bar
ToolCapabilities.magick_available → gates magick identify in scanner
```