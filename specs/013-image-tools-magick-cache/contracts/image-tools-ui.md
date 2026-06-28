# Image Tools UI Contract

**For**: The new detachable "Image Tools" inspector panel (Single + Batch sections) and associated dialogs (crop preview, batch confirmation, rename preview).

**Note (plan revision)**: Implementation of the panel and logic will be in main.rs + extended image_proc (per constitution C1 fix in plan). No change to UI contract itself.

## Single Mode
- Input: Currently selected ImageAsset (or none).
- Controls:
  - Operation selector: Resize | Crop | Convert (rename is Batch-only in v1).
  - Resize: numeric width/height or percent slider, fit mode (contain/cover/stretch?), filter choice (Lanczos etc. or high-quality toggle), quality slider for lossy formats.
  - Crop: text field for "WxH+X+Y" (live validation + parse), live preview image pane (low-res crop region rendered).
  - Convert: target format dropdown (jpeg, png, webp, and magick-discovered), quality/compression.
- Output policy: radio or segmented: "Save to subfolder (name)", "Save alongside with suffix", "Backup originals + modify in place" (the last one visually warns and requires extra confirm later).
- Actions: "Apply" (enabled only when valid selection + params). For in-place policy, Apply triggers a confirmation dialog first.
- Preview: For crop, a resizable image view showing the current geometry applied to a proxy of the source. Updates on field edit.
- Feedback: After apply, result appears in main browser list (with status), Activity Log entry, status bar update.

## Batch Mode
- Input: Current multi-selection (N >= 2) from the browser.
- Same operation controls as Single (applied uniformly).
- Output policy applied to all.
- "Prepare summary" button (or live): shows "Will create N new files in subfolder X" or equivalent.
- Confirmation dialog (always for batch): "About to process N images. Policy: XXX. Estimated time: Y. Proceed?" with [Cancel] [Confirm].
- Progress: Non-modal indicator (reuse existing session status + bottom bar). Per-item or aggregate. Cancel button.
- Result: Summary dialog ("N succeeded, K skipped, M errors") + links or log references. All new assets appear in list.

## Rename Preview (Batch-focused)
- Pattern input field with live parsing.
- Large table: Old Name | Proposed New Name (updates instantly on pattern change).
- Collision highlighting (red rows or warning banner).
- Tokens helper (inline or tooltip): lists supported {counter:NN}, {date:...}, etc.
- Apply requires the same batch confirmation.

## Common
- All tools respect the current filter/sort (operate only on visible selected? or all selected? — spec implies current selection).
- Cache toggle/status visible (if enabled in settings).
- "Pre-cache folder" and "Prepare Fast feh Viewing Cache" as prominent actions (possibly top-level or in a Cache sub-section). Show progress + "Launch feh on optimized" affordance when ready.
- Graceful: If magick-cache or magick missing, tools still work for basic cases (image crate fallback); cache features disabled with clear "Install guidance" text + link to README section.
- Accessibility: Keyboard reachable, labels, live regions for progress/summary.

This contract is implemented in `main.rs` (panel + widgets + egui textures) + `ui_logic` (preview pixel calc, pattern expansion, plan building). Core execution stays in `image_proc::ImageToolsService`.

See `quickstart.md` for runnable validation of the flows.