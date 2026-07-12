# Tasks: Lazy Folder Scanning (retro, tasks-as-shipped)

**Input**: back-filled from 6 shipped commits on `017-lazy-folder-scanning`, see spec.md

## Shipped (retro)

- [x] T001 perf: once-per-frame indices, cached sort keys, memoized tree, lean scan partials — a7f6230
- [x] T002 perf: cancellable scans — stop superseded walk, magick probes, converted stat-storm — 0f2db2a
- [x] T003 fix: per-folder scan default + absolute/cleared selected_tree_folder — a4fe807
- [x] T004 refactor: decouple launch entries from flat image list — 5d04722
- [x] T005 feat: drill-down folder navigation + cross-folder round-trip landing — e44d5a3
- [x] T006 feat: fixed-width inspector auto-sized to widest static label — 8689292

## Handback — follow-ups for feature 018 Batch 0

The following gaps were identified by a Fable-5 validation pass over 017 and are being
fixed as prerequisite work in feature 018's Batch 0 (see `specs/018-inspector-ux-rework/`):

- **Deferred per-entry image-count cache**: commit `5d04722` replaced the live per-frame
  "{n} images" status text for Feh launch entries with the fixed string "Ready", because
  computing a live count required an O(images) disk walk every frame. A per-entry count
  cache (computed off the hot path, invalidated on folder change) was deferred rather than
  built inline. Not yet scheduled; noted here for whoever picks it up next.
- **F1** — `ScanMsg::Converted`'s handling in `main.rs` wholesale-overwrote `self.images`
  with the background-converted snapshot, discarding in-place user mutations (rename/move/
  processed-add) that happened between the scan's Complete arm and the async Converted
  arm arriving. Fixed in 018 Batch 0 via a by-path merge (`merge_converted_statuses` in
  `ui_logic.rs`).
- **F3** — the auto-sized inspector width cache (`inspector_width`, from commit `8689292`)
  was keyed only on `pixels_per_point` but clamped to the live half-viewport, so it went
  stale on window resize. Fixed in 018 Batch 0 (cache only the measured text, re-clamp
  every frame) — this was also a hard prerequisite for 018 Batch 1's width-floor increase.
- **F4** — `launch_entry_feh`'s success status message printed `"Launched feh on {status}"`
  where `status` is now the fixed string "Ready" (see the count-cache deferral above),
  producing the meaningless "Launched feh on Ready". Fixed in 018 Batch 0 with a
  count-based message.
- **F5** — cross-folder round-trip landing (commit `e44d5a3`) could lose the "scroll list
  to the landed image" behavior: `pending_flat_scroll_offset` consumed (`take()`d) the
  pending scroll target even on frames where the target row didn't exist in the list yet.
  Fixed in 018 Batch 0 (peek-then-take).
- **F6** — a round-trip landing in the *same* folder while a rescan was already in flight
  could be clobbered by the scan's default-to-`images[0]` selection once it completed.
  Fixed in 018 Batch 0 by also arming `pending_select_path` when a scan is in flight.
