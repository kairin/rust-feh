# Data Model: Multi-Feh Instance Launcher & Image Clipboard

**Feature**: 014-multi-feh-clipboard

## Entities

### FehLaunchEntry

Represents a single configurable feh launch profile.

| Field | Type | Required | Description |
|---|---|---|---|
| `id` | `String` | Yes | Stable unique identifier (UUID-like, e.g., "a1b2c3d4"). Survives label/folder edits. |
| `label` | `Option<String>` | No | User-provided display name. When `None`, the folder's basename is shown. |
| `folder_path` | `Option<PathBuf>` | No | Absolute path to the target folder. When `None`, entry is unconfigured (launch disabled). |
| `created_at` | `u64` | Yes | Unix timestamp of creation (for stable ordering). |

**Validation Rules**:
- `id` must be non-empty and unique within the list
- `folder_path`, if set, must be an absolute path (starts with `/`)
- `label`, if set, must be non-empty after trimming
- `created_at` is set once at creation and never modified

**State Transitions**:
```
[Unconfigured] -- folder assigned --> [Configured] -- feh available & images exist --> [Launchable]
                                                                                          |
                                                                                          v (launch)
                                                                                     [Launched]
                                                                                          |
                                                                                          v (feh window closes independently)
                                                                                     [Launchable]
[Any state]    -- remove           --> [Deleted] (removed from list, not persisted)
[Configured]   -- folder reassigned --> [Configured] (new folder)
[Configured]   -- folder deleted   --> [Stale] (folder no longer exists; show warning, launch disabled)
```

### FehLaunchList

Ordered collection of launch entries with persistence support.

| Field | Type | Required | Description |
|---|---|---|---|
| `version` | `u32` | Yes | Schema version for forward compatibility (currently 1) |
| `entries` | `Vec<FehLaunchEntry>` | Yes | Ordered list of entries, sorted by `created_at` ascending |

**Serialization**: JSON file at `~/.config/rust-feh/launch-entries.json`

**Operations**:
- `add_entry(folder_path) -> &FehLaunchEntry` — creates new entry at end of list, persists
- `remove_entry(id) -> bool` — removes by id, persists; returns false if id not found
- `update_label(id, label) -> bool` — sets optional label, persists; false if id not found
- `update_folder(id, path) -> bool` — changes folder assignment, persists; false if id not found
- `load() -> Result<Self>` — reads from config file; returns empty list on missing/corrupt file
- `save(&self) -> Result<()>` — writes to config file atomically (write to temp, rename)

**Error Handling**:
- Missing config directory: create it (`~/.config/rust-feh/`)
- Corrupt JSON: log warning, return empty list
- Write failure (disk full, permissions): log error, show status message, don't crash

### Persistence Schema (JSON)

```json
{
  "version": 1,
  "entries": [
    {
      "id": "c1a2b3d4",
      "label": "Wallpapers",
      "folder_path": "/home/user/Pictures/wallpapers",
      "created_at": 1719440000
    },
    {
      "id": "e5f6g7h8",
      "label": null,
      "folder_path": "/home/user/Pictures/screenshots",
      "created_at": 1719440100
    }
  ]
}
```

### Rust Type Definitions (src/types.rs)

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FehLaunchEntry {
    pub id: String,
    pub label: Option<String>,
    pub folder_path: Option<PathBuf>,
    pub created_at: u64,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct FehLaunchList {
    pub version: u32,
    pub entries: Vec<FehLaunchEntry>,
}
```

### Derived State (computed at runtime, not persisted)

These are computed fields added to the App state for UI rendering:

| Field | Source | Description |
|---|---|---|
| Entry display label | `label.unwrap_or(folder_basename)` | Shown in the entry list |
| Entry is stale | Check if `folder_path` exists on filesystem + has images | Controls launch button enable/disable |
| Folder candidates | `build_folder_tree()` output | List of scanned folders for the ComboBox |

## Entity Relationships

```
ScanResult (existing)           FehLaunchList (new)
     │                               │
     │ images: Vec<ImageEntry>       │ entries: Vec<FehLaunchEntry>
     │                               │
     └───── folder_path ─────────────┘
              (shared key)
```

- `FehLaunchEntry.folder_path` references folders discovered by the scanner
- No foreign key enforcement — entries can reference folders not currently in the scan tree (stale entries)
- Each entry's folder images are resolved at launch time via `list_indices()` with `root = entry.folder_path`

## No New Data for Clipboard Feature

The clipboard feature has no persistent data model — it's a transient operation:
1. User right-clicks image → path extracted from `ImageEntry.path` (existing type)
2. Image data read from disk → placed on system clipboard
3. No entries, no persistence, no additional types needed
