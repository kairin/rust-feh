# Contract: Multi-Feh Instance Panel UI

**Feature**: 014-multi-feh-clipboard
**Type**: UI Component Contract

## Overview

The multi-feh instance panel is a new inspector section in the right sidebar of rust-feh, below the existing "Image actions" section. It allows users to manage multiple feh launch configurations.

## Layout

```
┌─ Feh Instances ─────────────────────── [Detach] ─┐
│ [+ Add] [Launch All]                              │
│                                                    │
│ ┌─ #1: Wallpapers ──────── [×] ─────────────────┐ │
│ │ Folder: [/home/user/Pics/walls  ▾]             │ │
│ │ [Launch]                                        │ │
│ └────────────────────────────────────────────────┘ │
│                                                    │
│ ┌─ #2: /home/user/Pics/ss ── [×] ───────────────┐ │
│ │ Folder: [/home/user/Pics/ss    ▾]              │ │
│ │ [Launch]                                        │ │
│ └────────────────────────────────────────────────┘ │
│                                                    │
│ ┌─ #3: (new) ─────────────── [×] ────────────────┐ │
│ │ Folder: [none selected       ▾]                 │ │
│ │ [Launch] (disabled)                              │ │
│ └────────────────────────────────────────────────┘ │
└────────────────────────────────────────────────────┘
```

## States

| State | Conditions | UI Behavior |
|---|---|---|
| Empty | No entries exist | Show "[+ Add]" button with hint text "Add folders to launch in feh" |
| Configured | Entry has folder + feh available + images exist | Launch button enabled, green tint |
| Unconfigured | Entry has no folder assigned | Launch button disabled; show "Select a folder" |
| Feh missing | feh not on PATH | All Launch buttons disabled; per-entry badge "feh not found" |
| Folder stale | Folder path no longer exists | Launch button disabled; warning icon + "Folder not found" |
| Folder empty | Folder exists but has 0 images | Launch button disabled; "No images" label |

## Widget Contract

### "+" (Add) Button

- **Location**: Top-left of the panel, always visible
- **Action**: Creates new `FehLaunchEntry` with default folder resolved per FR-002 (selected folder-tree node → selected image's parent → active scan root → unassigned), appends to list, persists
- **Feedback**: New entry appears instantly; scrolls to bottom if needed
- **Error**: If no folder can be resolved, entry is still created but unassigned and shows "Select a folder"

### Entry Row

Each entry is a framed group containing:
1. **Header row**: Entry number/label (bold) + [×] remove button (right-aligned)
   - Number is the 1-based index in the list
   - Label is editable inline (click to edit text field)
   - [×] removes entry with no confirmation dialog (undo not supported in v1)
2. **Folder selector**: egui `ComboBox` populated from `tree_visible_rows()` folder nodes
   - Shows folder path relative to scan roots when possible
   - "None selected" option at top for clearing assignment
3. **Launch button**: Full-width, enabled only when launchable

### "Launch All" Button

- **Location**: Next to [+ Add], top of panel
- **Action**: Iterates all configured entries and spawns feh for each (sequentially, not parallel — Command::spawn is non-blocking)
- **Feedback**: Status bar: "Launched N feh instances"
- **Skip**: Entries that are unconfigured, stale, or empty are silently skipped

### Remove [×] Button

- **Location**: Top-right of each entry frame
- **Style**: Small, subtle (no border, faint X symbol)
- **Action**: Removes entry from list, persists, does NOT kill running feh process

## Detach Support

- The panel supports the same detach pattern as other inspector sections
- Detached state: `feh_instances_detached: bool`
- Open state: `feh_instances_section_open: bool`
- Uses `render_segment_detach_toolbar()` helper
- When detached, renders in a separate egui window via `ctx.show_viewport_deferred()`

## Interaction with Existing "Open in feh"

- The existing single "Open in feh" button in Image actions is NOT modified (FR-014)
- New instances operate on the same `open_in_feh` code path (extracted to accept a folder root)
- The multi-instance panel is additive; users can still use the single button for quick one-off launches

## Persistence Contract

- **Save triggers**: Entry add, remove, label change, folder change
- **Load trigger**: `RustFehApp::new()` (application startup)
- **File**: `~/.config/rust-feh/launch-entries.json`
- **Atomicity**: Write to temp file, rename (std::fs::rename is atomic on Linux)
- **Corruption recovery**: If load fails, start with empty list; log warning
