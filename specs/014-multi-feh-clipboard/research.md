# Research: Multi-Feh Instance Launcher & Image Clipboard

**Feature**: 014-multi-feh-clipboard
**Date**: 2026-06-26

## 1. System Clipboard for Images on Linux

### Decision
Use the `arboard` crate (v3.x) for placing image data on the system clipboard.

### Rationale
- `arboard` is the de facto Rust clipboard library: 3k+ GitHub stars, actively maintained, pure Rust for Linux (uses x11rb on X11 and wlr-data-control on Wayland).
- Supports `image::ImageData` encoding — can pass raw RGBA bytes with dimensions, which gets converted to `image/png` for the clipboard target.
- Works on both X11 and Wayland transparently; matches the project's Linux-first requirement.
- Minimal dependency footprint — only pulls in `x11rb` + `wayland-client` on Linux (no heavy GUI toolkit).
- No need for a separate X11-only or Wayland-only clipboard solution.

### Alternatives Considered
| Alternative | Rejected Because |
|---|---|
| `copypasta` | Older, less maintained; Wayland support is experimental. |
| `x11-clipboard` | X11 only; no Wayland support. Would require separate code paths. |
| Manual X11/Wayland FFI | Massive implementation burden; violates "minimal dependencies" principle. |
| `smithay-clipboard` | Wayland-only; no X11 fallback. |

### Image Format Strategy
- Read the source image file with the `image` crate (already a dependency)
- Convert to RGBA8 (arboard's required format)
- Pass `ImageData { width, height, bytes: rgba }` to `arboard::Clipboard::set_image()`
- arboard internally encodes as PNG for the clipboard MIME type `image/png`
- This preserves lossless pixel/content fidelity at native resolution (SC-006). It does **not** preserve byte-for-byte identity with the original encoded file, especially for JPEG/WebP sources or re-encoded PNGs.

### Edge Case: Large Images
- The `image` crate's decode + convert to RGBA can be slow for large files
- For images >50 MB, the operation may exceed the 3-second target
- Mitigation: Parse image dimensions/metadata first; if decoded pixel count exceeds a threshold (e.g., 8192×8192 = 64MP), show a warning but proceed. No timeout mechanism needed in v1 — the egui render loop is single-threaded, so the copy is inherently blocking. The status bar confirms completion.

## 2. Persistence for Launch Entries

### Decision
Use a JSON file at `~/.config/rust-feh/launch-entries.json` with `serde` + `serde_json`.

### Rationale
- JSON is human-readable and debuggable (users can inspect/edit config)
- `~/.config/` is the XDG standard config location on Linux
- The project currently has zero persistence; this is the first persistent state
- Minimal dependencies: `serde` (derive macros) + `serde_json` = ~0 additional transitive deps over what `image`/`eframe` already pull
- The launch entry list is small (typically <20 entries), so JSON parsing overhead is negligible

### Alternatives Considered
| Alternative | Rejected Because |
|---|---|
| `toml` + `serde` | More Rust-idiomatic but less user-friendly for hand-editing; `serde` same dep cost. |
| SQLite (`rusqlite`) | Massive dependency; overkill for a simple entry list. |
| Memory-only (no persistence) | Violates FR-006 (must persist across restarts). |
| `directories` crate for XDG paths | Not needed — project already hardcodes paths; `std::env::var("HOME")` + `Path::join` is sufficient. |

### Config Path
```
$HOME/.config/rust-feh/launch-entries.json
```
Schema:
```json
{
  "version": 1,
  "entries": [
    {
      "id": "entry-1",
      "label": "Wallpapers",
      "folder": "/home/user/Pictures/wallpapers",
      "sort_mode": "Path"
    }
  ]
}
```

### Error Handling
- If the config dir doesn't exist, create it on first save
- If the config file is missing or corrupted on load, start with empty list (no crash)
- Log warnings for parse errors; never block startup on bad config

## 3. Clipboard Dependency in eframe

### Decision
arboard requires a running event loop on Linux. Since rust-feh uses eframe (which provides the event loop), arboard will work within the `update()` method (which runs on the main thread).

### Verification
- arboard's Linux backend uses X11 `CLIPBOARD` selection on X11 and `wl_data_device_manager` on Wayland
- Both require a running display connection — eframe's `glow` backend already initializes this
- Tested pattern: call `arboard::Clipboard::new()` inside `update()`, set image, drop clipboard

## 4. Right-Click Context Menu in egui

### Decision
Use egui's built-in `response.context_menu()` with a custom popup.

### Rationale
- egui 0.30 has `Ui::menu_button` and `Area::new` for context menus
- For right-click on image rows: capture `response.secondary_clicked()`, then open a floating menu with "Copy image to clipboard"
- This avoids external dependencies for context menus

### Implementation Pattern
```rust
// In the image row rendering loop:
let response = ui.selectable_label(is_selected, label);
if response.secondary_clicked() {
    self.context_menu_target = Some((path.clone(), response.rect));
}
// Later, render the context menu:
if let Some((ref path, ref anchor_rect)) = self.context_menu_target {
    egui::Area::new("image_context_menu".into())
        .fixed_pos(anchor_rect.right_bottom())
        .show(ctx, |ui| {
            if ui.button("Copy image to clipboard").clicked() {
                self.copy_image_to_clipboard(path);
                self.context_menu_target = None;
            }
        });
}
// Close menu on any click outside
```

## 5. Multi-Instance Panel UI Architecture

### Decision
Add a new inspector section ("Feh Instances") below the existing "Image actions" section, with its own CollapsingHeader and detach support.

### Layout
- A button bar: [+ Add Instance] [Launch All]
- A scrollable vertical list of entries, each showing:
  - Entry index (#1, #2, ...)
  - Editable label text field (optional, small)
  - Folder selector (ComboBox from scanned tree folders)
  - [Launch] button (green when feh available, disabled gray when not)
  - [×] remove button (small, top-right of entry row)

### Folder Assignment
- The folder selector populates from `self.tree_expanded_paths` and `build_folder_tree()` — the same data already used for the browse tree
- Each entry stores a `folder_path: Option<PathBuf>` (None = "no folder selected")
- New entries default to the folder resolved per FR-002 (selected folder-tree node → selected image's parent → active scan root → None)

### Persistence Hook
- Save on: entry add, entry remove, entry label change, entry folder change, app exit
- Load on: `RustFehApp::new()` — read `launch-entries.json`, populate `self.launch_entries`

## 6. Module Responsibilities (Constitution Principle III)

| Component | Home | Rationale |
|---|---|---|
| `FehLaunchEntry` struct | `src/types.rs` | Domain type, no GUI dependency |
| `FehLaunchList` (save/load) | `src/ui_logic.rs` | I/O logic; pure functions, no egui |
| Copy-to-clipboard function | `src/ui_logic.rs` | I/O function; no egui dependency; reusable |
| Multi-instance panel UI | `src/main.rs` | egui widget rendering |
| Right-click context menu | `src/main.rs` | egui widget rendering |
| `open_in_feh` for entries | `src/main.rs` | Extends existing `open_in_feh` method |

## 7. Dependency Justification (Constitution Principle II)

| Dependency | Justification |
|---|---|
| `arboard` 3.x | Required for system clipboard image paste (core feature, no Rust stdlib equivalent) |
| `serde` 1.x (derive) | Required for JSON persistence of launch entries (FR-006); industry standard |
| `serde_json` 1.x | JSON encoder/decoder; minimal, audited, no unsafe known issues |

All three are crates.io, actively maintained, widely used in the Rust ecosystem.
