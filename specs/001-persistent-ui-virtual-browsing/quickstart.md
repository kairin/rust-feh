# Quickstart: Persistent UI Layout & Virtual Browsing

**Feature**: 001-persistent-ui-virtual-browsing
**Date**: 2026-06-21

## Prerequisites

- Rust stable toolchain (rustc 1.75+, cargo)
- System dependencies: `build-essential`, `pkg-config`, `libssl-dev`, `libxcb1`,
  `libxcb-render0`, `libxcb-shape0`, `libxcb-xfixes0`
- `feh` installed (`sudo apt install feh`) for viewing and wallpaper features
- A test directory with many images (5,000+ recommended for performance validation)

## Build

```fish
cd /home/kkk/Apps/rust-feh
cargo build --release
```

Expected: Successful compile with no warnings from `cargo clippy -- -D warnings`.

Binary location: `target/release/rust-feh`

## Run

```fish
cargo run --release
# or directly:
./target/release/rust-feh
```

## Automated Validation (preferred)

Replaces most manual steps in V1–V10:

```fish
./scripts/validate-feature-001.sh
```

Runs `cargo build --release`, `cargo clippy`, `cargo test` (including `tests/feature_001_validation.rs`), and static FR checks. Results: `validation-results.md`.

**Not automated** (optional manual): SC-002 smooth 60fps scroll feel, SC-004 RSS while GUI is open.

**GUI performance runbook (feature 003):** [specs/003-gui-performance-validation/quickstart.md](../003-gui-performance-validation/quickstart.md) — 10k fixture, scroll protocol, RSS sampling, `validation-results.md`.

```fish
./scripts/validate-gui-performance.sh
```

## Validation Scenarios

### V1: Persistent Controls (US1, FR-001, FR-002, FR-003)

1. Launch the application.
2. Click "Choose folder" and select a directory with 5,000+ images.
3. **Verify**: The top panel with folder path, filter box, recursive checkbox,
   rescan button, and action buttons (Open in feh, Set as wallpaper, Quick resize)
   remains visible.
4. Scroll the image list to the bottom using the scrollbar.
5. **Verify**: Top controls remain visible and clickable. The bottom status bar
   remains visible showing "Showing X / Y images" (counter MUST be in bottom bar, not
   in the scrollable list area).
6. Type a filter term in the search box.
7. **Verify**: The search box is accessible (not scrolled away). The list updates
   to show matching entries.

### V2: Virtualized Scrolling (US2, FR-004, FR-005)

1. Load a directory with 10,000+ images.
2. Scroll rapidly using the scrollbar (drag the thumb).
3. **Verify**: Scrolling is smooth — no perceptible stutter or freeze.
4. In a separate terminal, check memory: `ps aux | grep rust-feh`.
5. **Verify**: RSS is under 150MB (baseline for metadata-only list, no thumbnails).
6. Type a filter term (e.g., part of a filename).
7. **Verify**: Results appear instantly (within 200ms). The "Showing X / Y images"
   counter updates correctly.
8. Clear the filter (delete all text).
9. **Verify**: Full list restores. Counter shows all images.

### V3: No Auto-Feh Launch (US3, FR-007, FR-008)

1. Launch the application fresh.
2. Click "Choose folder" and select a directory with images.
3. **Verify**: After loading, the first image is highlighted in the list. The
   status bar shows "Selected: <filename>" or similar. **No feh window appears.**
4. Click "Open in feh".
5. **Verify**: feh launches with the selected image.
6. Close feh. Click a different image in the list.
7. **Verify**: Selection changes (highlight moves). No feh window appears.
8. Click "Set as wallpaper (feh --bg-fill)".
9. **Verify**: Wallpaper changes to the selected image. Check with `cat ~/.fehbg`.

### V4: Filter + Display Counter (FR-005, FR-006)

1. Load a directory with mixed image types (jpg, png, webp, gif, bmp).
2. **Verify**: Counter shows total count of supported images.
3. Type a filter that matches a subset.
4. **Verify**: Counter updates to "Showing M / N images" where M < N.
5. Type a filter that matches nothing.
6. **Verify**: List is empty. Counter shows "Showing 0 / N images".
7. Empty directory edge case: load a directory with no images.
8. **Verify**: Status shows "Loaded 0 images." List is empty.

### V5: Recursive Toggle (FR-010)

1. Load a directory with subdirectories containing images.
2. Note the image count.
3. Toggle "Include subfolders" ON.
4. **Verify**: Rescan triggers. Count increases (subdirectory images included).
5. Toggle "Include subfolders" OFF.
6. **Verify**: Rescan triggers. Count returns to original (top-level only).

### V6: Debug Log

1. Launch fresh — before any user action, expand "Debug Log (click to expand)".
2. **Verify**: Shows "(no debug messages yet)" — startup banner is terminal-only (`eprintln!`), not in `debug_logs` (FR-009 clarify).
3. Load a folder and perform actions.
4. **Verify**: Log entries show folder loads, image counts, feh launches, and selection events.
5. Click "Clear logs".
6. **Verify**: Log area empties and shows "(no debug messages yet)".

### V7: Menu Bar Actions (FR-011)

1. Launch the application with no folder loaded.
2. Select File → "Choose folder..." from the menu bar.
3. **Verify**: Folder picker opens (same behavior as toolbar button).
4. Load a folder. Select File → "Rescan".
5. **Verify**: Directory rescans. Select Tools → "Open in feh" with an image selected.
6. **Verify**: feh launches.

### V8: Feh Missing (FR-008a, SC-007)

1. Temporarily rename or hide the `feh` binary: `sudo mv /usr/bin/feh /usr/bin/feh.bak`
2. Launch the application.
3. **Verify**: Status bar shows "feh not found — install with `sudo apt install feh`".
4. **Verify**: "Open in feh" and "Set as wallpaper" buttons are grayed out.
5. Click a disabled button.
6. **Verify**: Same error message shown; no process spawn attempted.
7. Restore feh: `sudo mv /usr/bin/feh.bak /usr/bin/feh`

### V9: Scanning State & Folder Change (FR-010, FR-012, FR-013)

1. Load a directory with images. Note the selection.
2. Toggle "Include subfolders".
3. **Verify**: Status bar briefly shows "Scanning…". Selection clears during scan,
   then first image is re-selected after completion.
4. Load a different folder while the first is displayed.
5. **Verify**: List clears immediately. Status shows "Scanning…". New results appear
   when scan completes.

### V10: Filter Scroll Reset (FR-005)

1. Load a directory with 1,000+ images.
2. Scroll to the bottom of the list.
3. Type a filter term.
4. **Verify**: List scroll resets to top showing filtered results from the beginning.

## Expected Outcomes

| Scenario | Expected |
|----------|----------|
| V1 | Controls always reachable, even scrolled to bottom of 5k+ list |
| V2 | Smooth 60fps scroll, <150MB RSS, filter <200ms |
| V3 | No feh auto-launch; explicit "Open in feh" works |
| V4 | Filter narrows list, counter accurate, empty-dir handled |
| V5 | Recursive toggle rescans correctly |
| V6 | Debug log functional, clearable |
| V7 | Menu bar actions functional (not stubs) |
| V8 | Feh missing: disabled buttons, no spawn, clear message |
| V9 | Scanning state shown; selection cleared and restored on rescan |
| V10 | Filter change resets scroll to top |
