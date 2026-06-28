# Contract: Image Clipboard Copy

**Feature**: 014-multi-feh-clipboard
**Type**: UI Component Contract

## Overview

The image clipboard feature adds a right-click context menu to image rows in the browse panel, allowing users to copy the full image data to the system clipboard.

## Trigger

- **Gesture**: Right-click (secondary click) on an image row in the browse panel
- **Target**: The `selectable_label` widget for each image entry in both FlatList and FolderTree view modes
- **Response**: Opens a floating context menu at the click position

## Context Menu

```
┌──────────────────────────────┐
│ Copy image to clipboard      │
└──────────────────────────────┘
```

- Single action only in v1 (future: "Copy file path", "Open with…")
- Menu closes when:
  - User clicks "Copy image to clipboard"
  - User clicks anywhere outside the menu
  - User presses Escape

## Implementation Contract

### Capture Right-Click

```rust
// In the image row rendering loop (both FlatList and FolderTree):
let response = ui.selectable_label(is_selected, label);
if response.secondary_clicked() {
    self.clipboard_context_menu = Some(ClipboardContextMenu {
        image_path: path.clone(),
        anchor_pos: response.rect.right_bottom(),
    });
}
```

### Render Context Menu

```rust
if let Some(ref menu) = self.clipboard_context_menu {
    egui::Area::new("clipboard_context_menu".into())
        .fixed_pos(menu.anchor_pos)
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            egui::Frame::popup(ui.style()).show(ui, |ui| {
                if ui.button("📋 Copy image to clipboard").clicked() {
                    self.copy_image_to_clipboard(&menu.image_path);
                    self.clipboard_context_menu = None;
                }
            });
        });
}
// Close menu on any click outside:
if ctx.input(|i| i.pointer.button_clicked(egui::PointerButton::Primary)) {
    self.clipboard_context_menu = None;
}
```

### Copy Operation

```rust
fn copy_image_to_clipboard(&mut self, path: &Path) {
    // 1. Read file into memory
    let data = match std::fs::read(path) {
        Ok(d) => d,
        Err(e) => {
            self.status = format!("Failed to read image: {}", e);
            return;
        }
    };

    // 2. Decode with image crate, convert to RGBA8
    let img = match image::load_from_memory(&data) {
        Ok(i) => i.to_rgba8(),
        Err(e) => {
            self.status = format!("Failed to decode image: {}", e);
            return;
        }
    };

    // 3. Copy to system clipboard via arboard
    let mut clipboard = match arboard::Clipboard::new() {
        Ok(c) => c,
        Err(e) => {
            self.status = format!("Clipboard unavailable: {}", e);
            return;
        }
    };

    let image_data = arboard::ImageData {
        width: img.width() as usize,
        height: img.height() as usize,
        bytes: std::borrow::Cow::Owned(img.into_raw()),
    };

    match clipboard.set_image(image_data) {
        Ok(()) => {
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            self.status = format!("Copied image to clipboard: {}", name);
        }
        Err(e) => {
            self.status = format!("Clipboard copy failed: {}", e);
        }
    }
}
```

## Error States

| Error | User Message | Clipboard Modified? |
|---|---|---|
| File not found / permission denied | "Failed to read image: [reason]" | No |
| Corrupt / unreadable image | "Failed to decode image: [reason]" | No |
| Clipboard manager not running | "Clipboard unavailable: [reason]" | No |
| Image too large (OOM) | App may slow/freeze — see edge case handling below | No (crash before set) |
| General clipboard error | "Clipboard copy failed: [reason]" | Undefined |

## Edge Case: Large Images

For images >50 MB decoded size:
1. No special handling in v1 — the operation blocks the UI thread
2. The image crate's `load_from_memory` + `to_rgba8` for a 200MP image (~800 MB in memory) could cause OOM
3. Mitigation (future): async clipboard worker thread; for v1, document this as a known limitation
4. The status bar feedback ("Copied image to clipboard: name") confirms operation completion, providing implicit progress feedback

## State Management

- `clipboard_context_menu: Option<ClipboardContextMenu>` — tracks open context menu
- Menu is ephemeral: closes on any interaction, no persistence

## New App State Fields

```rust
struct RustFehApp {
    // ... existing fields ...
    clipboard_context_menu: Option<ClipboardContextMenu>,
}

struct ClipboardContextMenu {
    image_path: PathBuf,
    anchor_pos: egui::Pos2,
}
```
