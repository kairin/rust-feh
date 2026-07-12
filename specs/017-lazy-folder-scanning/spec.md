# Feature Specification: Lazy Folder Scanning (drill-down navigation, per-folder default, perf)

**Feature Branch**: `017-lazy-folder-scanning`

**Created (retroactively)**: 2026-07-12, during feature 018 Batch 0

**Status**: Shipped (unmerged into `main` as of this writing — 6 commits on this branch)

**Input**: This spec was back-filled after the fact. Feature 017 shipped directly from
implementation without a SpecKit spec/tasks folder (a process gap). This document
reconstructs the shipped scope from the 6 commits and code for system-of-record
purposes, per `.agents/GUARDRAILS.md` "Resume durability" (`specs/<feature>/` is the
source of truth for class-A work).

## Clarifications

### Session 2026-07-12 (retro back-fill)

- Q: Why does this spec exist after the code shipped? → A: A Fable-5 validation pass over
  feature 017 (ahead of starting feature 018) found the SpecKit folder was never created.
  This spec plus `tasks.md` back-fill it from the shipped commits so 017 has a proper
  system-of-record home before 018 (which builds directly on top of it) begins.

## Shipped Scope

### `a7f6230` — Perf: once-per-frame indices, cached sort keys, memoized tree, lean scan partials

Optimized critical rendering paths to eliminate redundant per-frame recomputations.
The sort key generation (a freshly-allocated String per comparison) is now cached via
`sort_by_cached_key`, avoiding O(n log n) allocations during list sorting. The
`compute_list_indices` result is memoized across frames using an `images_revision`
counter (bumped at every image mutation site) plus a RefCell cache keyed on
(revision, current_dir, search, sort_mode), so multiple per-frame call sites reuse
a single result instead of filtering and sorting repeatedly. Tree rendering output
is similarly cross-frame-memoized. Redundant `ScanMsg::Partial` messages queued
in a single drain are coalesced to apply only the latest one, cutting useless
`self.images` replaces.

### `0f2db2a` — Perf: cancellable scans — stop superseded walk, magick probes, converted stat-storm

Added per-scan Arc<AtomicBool> cancellation tokens so that when a new scan begins,
the prior scan's token is set to trigger cancellation at all its checkpoints
(walkdir loop, ImageMagick identify probe, background converted-sibling detection).
Superseded scans now stop promptly instead of burning CPU/IO producing results
nobody will see, improving UI responsiveness and resource use during rapid
folder navigation.

### `a4fe807` — Fix: per-folder scan default + absolute/cleared selected_tree_folder

Changed recursive scanning default from true to false, establishing per-folder
drill-down semantics where users opt-in to subfolders explicitly via "Include
subfolders" checkboxes rather than exploring recursively by default. Fixed a
bug where `toggle_tree_folder` was misinterpreting relative folder paths as
absolute PathBufs; paths are now resolved against `current_dir` before storage.
`selected_tree_folder` is cleared on every fresh `scan_directory` to prevent
stale state across rescans.

### `5d04722` — Refactor: decouple launch entries from flat image list

Decoupled entry-launch logic from the flat image list, because a launch entry's
assigned folder need not exist in the currently-scanned (now non-recursive-by-default)
folder tree. `build_entry_filelist(entry)` now shallow-scans only the entry's own
folder directly. `entry_is_launchable` eliminated its per-frame O(images) disk walk
(was redundantly recomputed in 3 render call sites) and now only checks folder
assignment/existence with a static result. Launchable entry status text switched
from a live count to the fixed string "Ready" (the per-entry count cache was
deferred as noted in tasks.md Handback). `launch_entry_feh` adds a launch-time
empty-folder guard.

### `e44d5a3` — Feat: drill-down folder navigation + cross-folder round-trip landing

Introduced off-thread, non-recursive subfolder listing (`list_subfolders`) and
a unified `navigate_to_folder` entry point for all folder changes (folder picker,
dev hook, subfolder-row clicks, breadcrumb control). Folder navigation uses the
existing `request_subfolders`/`poll_subfolders` pattern (generation-guarded,
coalesced, spinner while pending). Cross-folder round-trip landing: when a
round-trip handoff (feature 016) lands on an image outside the currently-loaded
folder, `stage_selection_from_round_trip` now calls `navigate_to_folder(parent)`
first, ensuring the intended landing target survives the async rescan instead of
being clobbered by the default `images[0]` selection.

### `8689292` — Feat: fixed-width inspector auto-sized to widest static label

The inspector SidePanel becomes non-resizable and auto-sizes to the widest
whitelisted static label (section-header identities, fixed intro text, button/checkbox
captions), clamped to [280, inspector_max_width]. The whitelist is a const &[&str]
so no dynamic self-state can ever peg the panel wide or jitter it frame to frame.
Width is cached and recomputed only when pixels_per_point changes. Dynamic path/status
rows switch from Wrap to Truncate + on_hover_text so long paths never force width.

## Functional Requirements

- **FR-001**: Scanning MUST default to non-recursive per-folder listing with subfolder
  drill-down available, rather than recursive by default; all "Include subfolders"
  checkboxes remain opt-in.
- **FR-002**: Superseded scans (initiated before a prior scan completes) MUST be cancelled
  promptly, stopping the walkdir loop, ImageMagick probes, and converted-sibling detection
  to avoid wasting CPU/IO on results that will not be used.
- **FR-003**: Folder navigation changes MUST route through a unified `navigate_to_folder`
  entry point, supporting folder picker, dev hooks, subfolder clicks, and breadcrumb
  controls.
- **FR-004**: Cross-folder round-trip landing (a closed feh viewer showing an image from
  a different folder) MUST cause the app to navigate to that image's folder first, then
  select and stage the landed image, rather than discarding the handoff.
- **FR-005**: Critical per-frame computations (list indices, sort keys, tree rendering,
  scan partials) MUST be memoized and reused across frames, not recomputed on every
  render pass.
- **FR-006**: Entry-launch logic MUST be decoupled from the flat list and able to discover
  and list images in a folder independently, even if that folder is not in the current
  recursive scan.
- **FR-007**: The inspector SidePanel MUST be fixed-width (non-resizable) and auto-size to
  the widest static label with a hard floor of 280px, never auto-sizing based on dynamic
  paths or status text.

## Assumptions

- This spec documents already-shipped, already-decided behavior; there was no separate
  maintainer clarification session — decisions were made during implementation and are
  recorded here for traceability only.
