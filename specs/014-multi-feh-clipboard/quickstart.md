# Quickstart: Multi-Feh Instance Launcher & Image Clipboard

**Feature**: 014-multi-feh-clipboard

## Prerequisites

- Rust stable toolchain (edition 2021)
- `feh` installed (`sudo apt install feh` or equivalent)
- Linux with X11 or Wayland display server running
- A clipboard manager running (for clipboard tests: `wl-paste` on Wayland, `xclip` on X11)

## Setup

```bash
cd /home/kkk/Apps/rust-feh
cargo build --release
```

## Validation Scenarios

### VS-1: Multi-Instance — Add, Configure, Launch

**Goal**: Verify the multi-instance panel allows adding entries and launching multiple feh windows.

**Steps**:
1. Launch rust-feh: `./target/release/rust-feh`
2. Choose a folder with images (File → Choose folder...)
3. Locate the "Feh Instances" panel in the inspector (collapsible section)
4. Click **[+ Add]** — a new entry appears with its folder resolved from the selected folder-tree node, selected image parent, active scan root, or left unassigned if none is available
5. Click **[Launch]** on the new entry — a feh window opens with the folder's images
6. Click **[+ Add]** again, use the folder selector ComboBox to pick a different scanned folder
7. Click **[Launch]** on the second entry — a second feh window opens concurrently
8. Close both feh windows independently (they should not interfere)

**Expected**: Two feh windows open simultaneously, each showing images from its assigned folder.

### VS-2: Multi-Instance — Persistence

**Goal**: Verify launch entries survive application restart.

**Steps**:
1. Add 2-3 launch entries with different folders and optional labels
2. Close rust-feh completely
3. Reopen rust-feh
4. Open the "Feh Instances" panel

**Expected**: All 2-3 entries are restored with their original folder assignments and labels.

**Check**: `cat ~/.config/rust-feh/launch-entries.json` — should contain the entries as JSON.

### VS-3: Multi-Instance — Remove and Stale Handling

**Goal**: Verify entry removal and stale folder detection.

**Steps**:
1. Add an entry pointing to a specific folder
2. Click **[×]** on that entry — it disappears from the list
3. Add another entry, change its folder to a path that doesn't exist on disk
4. Observe the launch button is disabled with "Folder not found" indicator
5. Change folder back to a valid one — launch button re-enables

**Expected**: Entries can be removed cleanly. Stale folders are detected and communicated to the user.

### VS-4: Clipboard — Right-Click Copy

**Goal**: Verify image data lands on the system clipboard correctly.

**Steps**:
1. Scan a folder with image files (JPG or PNG)
2. In the image list (FlatList view), right-click on any image row
3. Click "Copy image to clipboard" in the context menu
4. Status bar shows "Copied image to clipboard: [filename]"
5. Open a terminal and verify clipboard content:
   - **Wayland**: `wl-paste --type image/png > /tmp/clipboard-test.png`
   - **X11**: `xclip -selection clipboard -t image/png -o > /tmp/clipboard-test.png`
6. Verify the pasted PNG has the same dimensions and visual/pixel content as the original. For PNG originals, exact byte identity is not required because clipboard data may be re-encoded; prefer pixel comparison tooling when available (e.g., ImageMagick `compare -metric AE ORIGINAL_IMAGE /tmp/clipboard-test.png null:`).

**Expected**: The clipboard contains the image as PNG; the pasted image preserves the original's native dimensions and pixel/content fidelity. Byte-for-byte identity with the original encoded file is not required.

### VS-5: Clipboard — Error Handling

**Goal**: Verify graceful failure on unreadable images.

**Steps**:
1. Create a fake image file: `echo "not an image" > /tmp/fake.jpg`
2. Scan that folder and locate the fake entry
3. Right-click "fake.jpg" → "Copy image to clipboard"

**Expected**: Status bar shows "Failed to decode image: ...". Clipboard is not modified (previous clipboard content preserved).

### VS-6: Clipboard — Context Menu Dismissal

**Goal**: Verify context menu closes properly.

**Steps**:
1. Right-click an image row → context menu appears
2. Click elsewhere in the application window → menu closes
3. Right-click again → menu reappears

**Expected**: Exactly one context menu is visible at a time; it dismisses on outside click.

### VS-7: Multi-Instance — Launch All

**Goal**: Verify "Launch All" spawns feh for all configured entries.

**Steps**:
1. Add 3 entries pointing to different folders (all with images)
2. Click **[Launch All]**
3. Observe status bar: "Launched 3 feh instances"

**Expected**: 3 feh windows open, one per configured entry. Unconfigured/empty entries are skipped.

### VS-8: Single "Open in feh" Not Regressed

**Goal**: Verify existing single-launch button still works.

**Steps**:
1. Scan a folder
2. Click the existing "Open in feh" button in the Image actions panel

**Expected**: feh opens normally, showing the selected folder's images. Status and log messages are unchanged from current behavior.

## Run Tests

```bash
# Unit tests for new types and ui_logic functions
cargo test --test unit_ui_logic

# Full test suite
cargo test

# Ad-hoc: check clipboard integration compiles
cargo check
```
