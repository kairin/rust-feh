# Quickstart: Window & Viewer Stability (006)

This feature is partly retroactive: FR-001–FR-007 are already implemented. The remaining implementation target is FR-008 / SC-005 window preference persistence.

## Preconditions

- Linux desktop session for manual GUI checks (X11 or Wayland compositor).
- `feh` installed for feh viewer checks.
- Rust toolchain installed.
- Note: `cargo clippy` is not installed in the current Hermes environment; use `cargo check && cargo test` as the automated gate here.

## Automated verification

Run from repo root:

```bash
cargo check
cargo test
```

Expected in this environment before FR-008 implementation:

- `cargo check`: passes.
- `cargo test`: passes with ignored/manual tests unchanged.
- Existing shipped-window tests:
  - `clamp_window_size_enforces_floor`
  - `window_preset_dimension_values`

After FR-008 implementation, add/expect tests equivalent to:

- missing `~/.config/rust-feh/window-prefs.json` returns `WindowPreferences::default()`.
- corrupt `window-prefs.json` returns default and does not panic.
- saved prefs round-trip (`Large`, `resizable=false`, etc.).

## Manual GUI validation (requires working desktop; not possible in broken computer_use env)

### SC-001 / SC-002 — list uses available space, no large blank gap

1. Start the app:

   ```bash
   cargo run
   ```

2. Choose a folder with several images.
3. Confirm the image list fills the central panel vertically after scan.
4. Resize the app larger and smaller.
5. Confirm the list expands/contracts with the central panel and does not leave a large unused blank area below visible rows.

Expected: top controls stay stable; the list scroll area owns available height.

### SC-003 / SC-004 — feh viewer has stable visible geometry

1. Prepare a folder containing at least one tiny image (for example 5×5) and one normal image.
2. Select the tiny image in rust-feh and open it in feh.
3. Inspect the spawned process command if needed:

   ```bash
   ps -ef | grep '[f]eh'
   ```

4. Confirm feh opens with `--geometry 1280x960 --scale-down --zoom max` behavior: the window is visible/stable and the tiny image is zoomed enough to inspect.
5. Navigate to the normal image and back to the tiny one.

Expected: feh window does not collapse to tiny-image dimensions; small image remains inspectable.

### SC-005 — window preferences persist after restart (post-implementation)

1. Start the app.
2. View ▸ Window size ▸ Large.
3. Disable View ▸ Resizable window.
4. Quit the app.
5. Confirm config was written:

   ```bash
   jq . ~/.config/rust-feh/window-prefs.json
   ```

6. Restart the app.
7. Confirm the Large preset and locked/resizable=false state are restored.
8. Re-enable resizable and/or choose Default, quit, restart, confirm updated value persists.

Expected JSON shape:

```json
{
  "version": 1,
  "preset": "Large",
  "resizable": false
}
```

## Failure recovery checks

### Missing file

```bash
rm -f ~/.config/rust-feh/window-prefs.json
cargo run
```

Expected: app starts with Default preset and Resizable window enabled.

### Corrupt file

```bash
mkdir -p ~/.config/rust-feh
printf 'not-json' > ~/.config/rust-feh/window-prefs.json
cargo run
```

Expected: app starts with defaults, does not crash, emits a warning to stderr.

## Cleanup

If manual checks wrote local preferences you do not want to keep:

```bash
rm -f ~/.config/rust-feh/window-prefs.json
```
