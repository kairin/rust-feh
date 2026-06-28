# Quickstart Validation Guide: Image Tools with Magick Cache

**Feature**: 013-image-tools-magick-cache
**Purpose**: Runnable end-to-end scenarios that prove the feature works from the user's perspective. Use these to validate after implementation and in CI/manual test runs. Do not duplicate implementation internals (see tasks.md and the service code for that).

**Validation (2026-06-24 implement)**: Automated coverage via `cargo test` (unit + integration + perf). Timing smoke tests in `tests/perf/sc_timing.rs`: SC-001 4000×3000 resize &lt;30s, SC-002 repeat without cache, SC-004 50-image prepare-fast &lt;60s, SC-007 rename preview 500 files &lt;2s. Manual GUI scenarios 1–8 remain recommended for feh launch and cache setup with passkey.

**Prerequisites** (for full cache + fast-fe h scenarios):
- Linux (or WSL) with `feh` installed.
- ImageMagick 7+ with `magick` on PATH.
- `magick-cache` built and on PATH — see **[docs/MAGICK-CACHE-SETUP.md](../../docs/MAGICK-CACHE-SETUP.md)** (install from source, dev headers, autotools, `ldconfig`, passkey, cache root).
- rust-feh cache configured in **Image Tools → Cache** (enable, root, passkey, Apply).
- A test folder with 50–500+ images (mix of sizes, orientations, at least one exotic format if possible). Example: `~/Pictures/test-images/`.

**Build & Run**:
```bash
cargo build --release
./target/release/rust-feh
# or the build-and-place.sh convenience
```

## Scenario 1: Single Image Safe Resize + Inventory Update (P1 MVP)
1. Open the app, choose a folder with images.
2. Select one image in the list.
3. Open the new "Image Tools" panel (detachable inspector).
4. Choose Resize, set 50% or specific dimensions + quality.
5. Choose output policy "Save to subfolder 'processed'".
6. Click Apply.
7. **Expected**:
   - New file appears in `processed/` (or the chosen subfolder) with correct size/quality.
   - The new asset immediately appears in the browser list (no full re-scan required).
   - Activity Log contains a clear entry with before/after paths and op details.
   - Original file is untouched.
   - You can select the new file and launch it in feh via the existing "Open in feh" action.

**Validation command / script hook**: `cargo test feature_013_single_resize` (or equivalent manual timing < 30s per SC-001).

## Scenario 2: Numeric Crop with Live Preview + Safe Output
1. Select an image.
2. In Image Tools → Crop, enter geometry e.g. `800x600+120+80`.
3. Observe the live preview pane updates to show the cropped region (low-res proxy).
4. Apply with default safe policy.
5. **Expected**: Cropped result created, appears in list, preview accurately reflected the geometry, log entry, original safe.

## Scenario 3: Batch Processing with Progress + Summary (P2)
1. Select 30–100 images (use shift/ctrl or existing selection model).
2. In Image Tools (Batch section), choose Convert to high-quality JPEG + quality 90.
3. Output policy: subfolder.
4. Trigger batch.
5. **Expected**:
   - Confirmation dialog appears first with count + policy summary.
   - Visible progress (status pulse / bottom bar / per-op log lines) while UI remains responsive (you can scroll/filter elsewhere).
   - On completion: summary dialog ("28 succeeded, 2 skipped (format)").
   - All successful new files appear in the list immediately.
   - Activity Log has individual entries for every item (success or skip).
   - No data loss on originals.

## Scenario 4: Batch Rename with Live Preview + Confirmation
1. Select several images with varied names.
2. Open rename UI (pattern field).
3. Type e.g. `vacation-{date:YYYYMMDD}-{counter:03}`.
4. **Expected**: Preview table instantly shows correct proposed names for the current ordering/selection.
5. Adjust pattern → table updates live.
6. Apply → confirmation → rename happens.
7. **Expected**: Files on disk (or list) reflect new names, log has before/after, no partial damage on error, collision cases are highlighted before apply.

## Scenario 5: Enable Cache + Repeated Op (Cache Hit Speed) (P3)

**Setup**: Complete [docs/MAGICK-CACHE-SETUP.md](../../docs/MAGICK-CACHE-SETUP.md) before this scenario.

1. In Image Tools → Cache: enable cache, pick cache root + passkey file, set TTL, **Apply cache settings**.
2. Perform a resize on a 10–20MP image (note wall time or use timing script).
3. Immediately repeat the *exact same* resize on the same source.
4. **Expected**:
   - First run: full compute + put to cache (logged).
   - Second run: fast path (< 3s per SC-002), logged as cache hit.
   - Result file identical (bitwise or visually).
5. Disable cache → repeat is slow again.

## Scenario 6: Prepare Fast feh Viewing Cache + Launch (Killer Feature, P3, per clarify Q1)
1. Open a 500–2000+ image folder (mix of orientations/formats).
2. Trigger "Pre-cache entire folder" (background).
3. Trigger "Prepare Fast feh Viewing Cache".
4. When complete, use the "Launch viewer on optimized versions" action.
5. **Expected** (per clarified Option A):
   - Real .jpg/.png files are materialized (temp or managed location) from the cache.
   - feh launches (via existing mechanism + temp filelist) showing the optimized versions (first image appears noticeably faster; navigation smooth).
   - The same materialized optimized files appear as selectable entries in the rust-feh browser list (with "optimized"/"fast" status indicator).
   - You can further select one of those "fast" entries and run another tool on it.
   - Originals untouched.
   - Log records preparation + materialization steps.
6. Time comparison (optional but recommended for SC-004): launch on raw folder vs. "fast" path.

## Scenario 7: Graceful Degradation (Missing Tools)
1. Rename `magick-cache` temporarily or run without it on PATH.
2. Attempt cache features and a basic resize.
3. **Expected**:
   - Cache-related UI shows "not available" + clear install/setup guidance (points to README "How to use Magick Cache").
   - Basic single/batch resize/convert still works via `image` crate fallback.
   - No crashes, no silent failures, no data loss.
4. Restore binary → recheck (tool_caps) → cache features become available.

## Scenario 8: In-Place with Backup (Dangerous Path)
1. Select image(s).
2. Choose operation + the explicit "Backup originals + modify in place" policy.
3. Apply.
4. **Expected**: Multi-step confirmation (at least two explicit clicks), backups created (with visible suffix or in backup subdir), originals updated, new "version" appears in list, full log trail, easy to verify backups exist and originals were only touched after confirm.

## Cleanup / Teardown Notes
- Temp fast materialization dirs should be cleaned on app exit or via explicit action.
- Cache contents are user-managed (TTL + `magick-cache` commands); app does not auto-purge in v1.
- Use `cargo test` + the feature's quickstart validation scripts for repeatable runs.

These scenarios directly map to the prioritized User Stories (P1–P3) and the measurable Success Criteria (SC-001 through SC-008) in `spec.md`. They are the primary acceptance tests for the implementation.

See `research.md` for the technical decisions that make these flows possible (especially materialization for Q1 and Command usage). Implementation tasks live in `tasks.md` (produced by `/speckit-tasks`).