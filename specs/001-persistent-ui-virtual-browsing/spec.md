# Feature Specification: Persistent UI Layout & Virtual Browsing

**Feature Branch**: `001-persistent-ui-virtual-browsing`

**Created**: 2026-06-21

**Status**: Draft

**Input**: Phase 1 priorities from ISSUES_AND_CORE_AREAS.md — the foundational UI refactor needed
before thumbnails, async scanning, and richer interactions can be built.

## Clarifications

### Session 2026-06-21

Automated `/speckit-clarify` pass after gap-analysis update (no interactive questions — all
items resolved from spec review, constitution, and codebase audit):

- Q: What format should the gap audit (T004) use? → A: Markdown table with columns `FR-ID`, `Status` (pass/gap/partial), `Gap Description`, `Fix Location`, `Task ID`.
- Q: How should feh-dependent buttons be disabled when feh is absent? → A: Use `feh_button` helper (inactive fill + `Sense::click`) or `ui.add_enabled(feh_available, …)` — both acceptable; click without spawn shows install message.
- Q: How should filter scroll reset be implemented? → A: On `search` change, increment `scroll_generation` and apply `ScrollArea::id_salt(scroll_generation)` (track `prior_search` on `RustFehApp`). Equivalent to offset reset per FR-005.
- Q: Where should scanner permission-denied warnings be logged? → A: Extend `scanner.rs` to collect skipped-path messages; `main.rs` forwards them to `debug_logs` via `log()` after scan completes.
- Q: Where do new state fields (`feh_available`, `scanning`, `scroll_generation`, `prior_search`) live? → A: On `RustFehApp` in `src/main.rs` only (Constitution §III — no GUI types in core modules).

### Session 2026-06-21 (adversarial-review)

Automated `/speckit-clarify` pass integrating `adversarial-review.md` findings (no interactive
questions — all items pre-resolved from live code audit):

- Q: Where is authoritative implementation gap list? → A: `gap-audit.md` (T004 deliverable); decomposed fix steps in `remediation.md`.
- Q: How should `research.md` report implementation status? → A: Per-FR pass/partial/gap — never blanket "✅ Done" unless FR verified in code.
- Q: Where must auto-select-first run? → A: Only in `scan_directory` completion — remove duplicate from folder-pick handler (FR-012).
- Q: FR-009 empty debug log vs startup message? → A: Startup notice via `eprintln!` only; `debug_logs` stays empty until user actions (V6 passes).
- Q: FR-010 responsiveness during sync scan? → A: Controls remain visible in panels; status shows "Scanning…"; render loop MAY briefly stall — not a violation if indicator present.
- Q: View menu recursive toggle behavior? → A: MUST trigger `scan_directory` on change — same as toolbar checkbox (FR-011).

### Session 2026-06-21 (post-implementation converge)

Automated `/speckit-clarify` after post-implementation adversarial review — classifies
review findings; see `outstanding-issues.md` for full table.

- Q: Is missing `list_scroll_offset` a bug? → A: **No** — `scroll_generation` + `ScrollArea::id_salt` satisfies FR-005; data-model updated to match code (T066).
- Q: Is `feh_button` instead of `ui.add_enabled` a bug? → A: **No** — equivalent FR-008a behavior (visually disabled, click shows install message without spawn).
- Q: Does gap-audit "15 pass" mean done? → A: **No** — use two tiers: `code` (static review) and `validated` (quickstart V1–V10 + perf notes). SC-002–004 require validated tier.
- Q: FR-008a "persistent" feh message after folder load? → A: **Yes, fix required** — post-scan status MUST append feh-not-found when `!feh_available` (T065).
- Q: Duplicate recursive checkbox menu + toolbar? → A: **Intentional** — menu parity per FR-011; not consolidated.
- Q: Re-detect feh at runtime? → A: **Deferred** Area 6.
- Q: Non-permission walkdir errors silent? → A: **Deferred** Area 6 optional logging.
- Q: How to consolidate 10 open validation tasks? → A: Single **T067** quickstart V1–V10 session; supersedes T026, T036–T039, T050, T051, T058.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Always-Visible Controls on Large Collections (Priority: P1)

A user opens a directory containing 10,000 images. The folder picker, filter/search box,
recursive toggle, and primary action buttons (Open in feh, Set as wallpaper) remain
visible and reachable without scrolling through the image list. Previously, all controls
shared a single scrollable panel, so loading a large folder buried the controls behind
thousands of list entries. The user had to scroll back to the top just to change folders
or type a filter.

**Why this priority**: Foundational UX — every other feature (thumbnails, menu bar, image
tools) depends on a layout where controls are guaranteed visible. Without this, the app
is frustrating to use with any non-trivial image collection.

**Independent Test**: Open a directory with 5,000+ images. Verify that the folder picker,
filter/search field, recursive checkbox, rescan button, and the primary action buttons
(Open in feh, Set as wallpaper) are visible and clickable at all times without scrolling
the image list. Scroll the image list to the bottom — controls must still be visible.

**Acceptance Scenarios**:

1. **Given** the app is running with no folder loaded, **When** the user views the top
   panel, **Then** the "Choose folder" button and menu bar (File/View/Tools) remain
   visible and functional. Filter, recursive toggle, rescan, and action buttons are
   hidden or disabled until a folder is loaded.
2. **Given** the app is running with no folder loaded, **When** the user opens a directory
   with 5,000 images, **Then** the folder path, filter/search box, recursive checkbox, and
   action buttons become visible above the image list without requiring any scroll.
3. **Given** a loaded directory with 5,000 images, **When** the user scrolls the image
   list to the bottom, **Then** the controls at the top remain fixed and clickable.
4. **Given** a loaded directory, **When** the user types a filter term, **Then** the
   filter input is always accessible (not scrolled away) and the image list updates to
   show matching entries.
5. **Given** a loaded directory, **When** the user selects "Choose folder..." from the
   File menu, **Then** the folder picker opens and behaves identically to the toolbar
   "Choose folder" button.

---

### User Story 2 - Smooth Browsing on Large Collections (Priority: P1)

A user browses a directory with 10,000+ images. The image list scrolls smoothly without
lag or freezing, and filtering narrows the displayed entries instantly. Previously, every
image in the directory created a full UI widget (selectable label), causing the application
to slow down and consume excessive memory on large collections even when most items were
off-screen.

**Why this priority**: The application's core purpose is browsing image collections. If
browsing breaks down at large sizes, the tool fails its primary use case. This is also a
blocker for thumbnails (Area 4) — a virtualized list is prerequisite for thumbnail grid.

**Independent Test**: Load a directory with 10,000 images. Scroll rapidly through the
list with the scrollbar. Verify that scrolling remains smooth (no perceptible stutter)
and the application memory usage is proportional to visible items, not the total count.
Apply a filter — results must appear within 200ms.

**Acceptance Scenarios**:

1. **Given** a directory with 10,000 images loaded, **When** the user scrolls the image
   list, **Then** scrolling remains smooth at 60fps with no noticeable lag or freeze.
2. **Given** 10,000 images loaded and the filter/search box empty, **When** the user
   types a filter term (e.g., "vacation"), **Then** the list updates to show only
   matching entries within 200ms.
3. **Given** 10,000 images loaded, **When** the user clicks the recursive toggle to
   include subfolders, **Then** the list rescans and updates with the new results. The
   status bar shows a scanning indicator and controls remain visible and clickable
   throughout (the render loop may briefly stall during sync scan; see FR-010).
4. **Given** 10,000 images loaded and the user scrolled to the bottom, **When** the user
   changes the filter term, **Then** the list scroll position resets to the top so
   filtered results are visible from the beginning.

---

### User Story 3 - Clear Selection vs. Open Model (Priority: P2)

A user opens a folder of images. The first image is selected (highlighted in the list)
and the status shows its path, but feh does NOT automatically launch. The user can then
choose to open the selected image in feh with an explicit button click, double-click, or
keyboard shortcut. Previously, loading a folder automatically launched feh for the first
image, which felt surprising and spammy — especially when the user just wanted to browse.

**Why this priority**: Fixes a concrete user pain point ("loads folder but doesn't open"
was ambiguous — the real complaint is that auto-open is surprising). Clarifying the
selection/launch model is needed before adding double-click, context menus, and keyboard
navigation (Area 2 + Area 5).

**Independent Test**: Open a folder with images. Verify that the first image is selected
and highlighted, the status bar shows its name, but no external feh window appears. Click
"Open in feh" — feh launches for the selected image.

**Acceptance Scenarios**:

1. **Given** the user opens a folder with images, **When** the folder finishes loading,
   **Then** the first image is selected and highlighted in the list, the status shows
   "Selected: <filename>", and no feh window appears.
2. **Given** an image is selected in the list, **When** the user clicks "Open in feh",
   **Then** feh launches and displays the selected image.
3. **Given** an image is selected in the list, **When** the user clicks
   "Set as wallpaper (feh --bg-fill)", **Then** the wallpaper is set to that image.
4. **Given** no image is selected (e.g., after a rescan clears selection or an empty
   directory), **When** the user clicks "Open in feh" or "Set as wallpaper", **Then** the
   status shows "No image selected" and no action is taken.
5. **Given** feh is not installed on the system, **When** the application starts,
   **Then** the status bar shows a persistent "feh not found" message and feh-dependent
   buttons are visually disabled (grayed out). Clicking them shows the same error without
   attempting to spawn a process.
6. **Given** feh is installed but spawn fails (e.g., permission denied), **When** the user
   clicks "Open in feh", **Then** the status bar shows a descriptive error message and
   no crash occurs.

---

### Edge Cases

- What happens when the user opens a directory with zero supported images? The status
  shows "No images found" and the list area is empty. Controls remain visible. No image
  is selected. The "Open in feh" and "Set as wallpaper" buttons show "No image selected"
  when clicked.
- What happens when the user opens a very deep directory tree with recursive enabled
  (e.g., a home directory)? Sync scanning may briefly block the render loop. Controls
  remain visible and the status bar shows "Scanning…" during the scan. The scan MUST
  complete in a reasonable time for typical photo directories (under 10 seconds for
  20,000 images on a modern workstation). Async scanning is deferred to Area 6.
- What happens when the list is empty due to a filter that matches nothing? The bottom
  status bar shows "Showing 0 / N images" where N is the total loaded count. Controls
  remain visible so the user can adjust or clear the filter.
- What happens when the filter term changes while the user is scrolled down the list?
  The list updates to show matching results, and the scroll position resets to the top
  so the user sees the filtered results starting from the beginning.
- What happens when the user changes folders while a previous folder is loaded? The
  image list clears immediately, selection is cleared, and the status bar shows
  "Scanning…" until the new folder's scan completes.
- What happens when a subdirectory is inaccessible during scan (permission denied)?
  The scanner skips the inaccessible path, logs a warning to the debug log, and
  continues scanning accessible paths. The scan does not abort.
- What happens when "Quick resize" is clicked on a corrupt or unsupported image? The
  status bar shows a descriptive process error (e.g., "Process error: …") and no crash
  occurs.
- What happens when the window is resized with a large collection loaded? egui panels
  adapt automatically; persistent top/bottom panels remain visible and the virtualized
  list continues to render only visible rows. No explicit resize handling is required
  beyond standard egui layout behavior.
- What happens when a filename contains Unicode or special characters? Filter matching
  uses case-insensitive substring comparison on the UTF-8 filename as displayed; no
  regex metacharacter interpretation.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The application MUST display folder controls in a persistent non-scrolling
  top region. The "Choose folder" button and menu bar MUST always be visible and
  functional. When a folder is loaded, the region MUST also show: path label,
  search/filter input, recursive checkbox, and rescan button — all always visible
  regardless of image list length.
- **FR-002**: When a folder is loaded, the application MUST display primary action buttons
  (Open in feh, Set as wallpaper, Quick resize) in the persistent non-scrolling top
  region, always visible regardless of image list length.
- **FR-003**: The bottom status bar MUST display the current status message and the
  image counter (FR-006). When an image is selected, the top panel MAY also show the
  selection path for convenience. All status information MUST be in a persistent
  non-scrolling region.
- **FR-004**: The image list MUST render only the items currently visible in the
  scrollable viewport (via egui `show_rows` virtualization), not all loaded images.
  Rendering cost MUST NOT scale with total image count. Buffer rows above/below the
  viewport are managed by egui's `show_rows` and are acceptable.
- **FR-005**: The filter/search input MUST filter the image list by filename (case-
  insensitive UTF-8 substring match, no regex interpretation). Filtering MUST complete
  in under 200ms for collections up to 20,000 images, measured from the last keystroke.
  When the filter term changes, the list scroll position MUST reset to the top.
- **FR-006**: The bottom status bar MUST always show the count of displayed vs. total
  images in the format "Showing X / Y images" (e.g., "Showing 42 / 10000 images"),
  including the zero-match case ("Showing 0 / N images"). The counter MUST NOT be placed
  in the scrollable central panel.
- **FR-007**: When a folder finishes loading with one or more images, the first image
  MUST be selected and highlighted in the list, but the application MUST NOT
  automatically launch feh. When a folder finishes loading with zero images, no image
  is selected.
- **FR-008**: The "Open in feh" button MUST only launch feh when an image is selected.
  When no image is selected, it MUST display a "No image selected" status message.
- **FR-008a**: At application startup, the application MUST detect whether `feh` is on
  the system PATH (via `which`). If feh is not found, the status bar MUST show
  "feh not found — install with `sudo apt install feh`", and the "Open in feh" and
  "Set as wallpaper" buttons MUST be visually disabled (grayed out). Clicking them MUST
  show the same error message without attempting to spawn a process.
- **FR-008b**: When feh spawn fails at runtime (process not found, permission denied, or
  other I/O error), the status bar MUST show a descriptive error message. The
  application MUST NOT crash.
- **FR-009**: The debug log viewer MUST remain accessible but MUST NOT compete with
  primary controls for persistent screen space. It MUST be a collapsible panel in the
  central (scrollable) area, collapsed by default. When expanded with zero log entries,
  it MUST display "(no debug messages yet)".
- **FR-010**: The recursive subfolder toggle MUST trigger a rescan when changed (toolbar
  and View menu checkbox MUST behave identically). During sync scanning, controls in
  top/bottom panels MUST remain visible. The status bar MUST show "Scanning…" while
  `scanning == true`. Brief render-loop stall during sync scan is acceptable; async
  scanning is deferred to Area 6. Scanning MUST complete in under 10 seconds for
  20,000 images on a modern workstation.
- **FR-011**: The menu bar (File, View, Tools) MUST provide functional equivalents of
  toolbar actions: File → "Choose folder..." opens the folder picker; File → "Rescan"
  rescans the current folder; View → "Include subfolders" toggles recursive mode;
  Tools → "Open in feh" opens the selected image. Menu actions MUST not be stubs.
- **FR-012**: When a scan begins (new folder, rescan, or recursive toggle), the
  application MUST clear `selected` and the image list MUST clear immediately. After
  scan completes with images, `scan_directory` MUST set `selected` to the first image
  path — this logic MUST NOT live only in the folder-pick handler. After scan with zero
  images, `selected` MUST remain None.
- **FR-013**: When the user chooses a new folder while one is already loaded, the image
  list MUST clear immediately, selection MUST be cleared, and the status bar MUST show
  "Scanning…" until the new scan completes.
- **FR-014**: When "Quick resize" fails (corrupt file, unsupported format, I/O error),
  the status bar MUST show a descriptive error message (e.g., "Process error: …"). The
  application MUST NOT crash.
- **FR-015**: When the scanner encounters inaccessible paths (permission denied on a
  subdirectory), it MUST skip them, log a warning to the debug log, and continue
  scanning accessible paths. The scan MUST NOT abort.

### Key Entities

- **ImageEntry**: Represents a discovered image file. Attributes: file path, file size
  (bytes). Future: dimensions, modification time, format. (Existing type, may be
  enriched later.)
- **FilteredView**: A logical view over the loaded image list. Computed from the full
  image list by applying the current filter term. Used to determine which items to
  render in the visible viewport without duplicating the image data.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: With 10,000 images loaded, the user can access and interact with the
  filter/search box and action buttons without scrolling the image list.
- **SC-002**: Scrolling a 10,000-image list remains smooth (no visible stutter,
  frame drops below 30fps are imperceptible in normal use).
- **SC-003**: Filtering a 10,000-image list by filename substring returns results and
  updates the display within 200ms of the last keystroke.
- **SC-004**: Application memory usage when displaying a 10,000-image directory does
  not exceed 150MB RSS (baseline: egui + image list metadata, no thumbnails loaded).
- **SC-005**: After loading a folder, the user sees the first image selected and
  highlighted with its filename in the status, and no external feh window appears
  until the user explicitly clicks "Open in feh". Verification is scoped to the
  application's own spawn behavior, not pre-existing feh windows from other processes.
- **SC-006**: The "Showing X / Y images" counter is visible in the bottom status bar
  without scrolling, for all folder states (loaded, filtered, empty).
- **SC-007**: With feh absent from PATH, feh-dependent buttons are disabled at startup
  and no process spawn is attempted when they are clicked.

## Assumptions

- The existing `walkdir`-based scanner (`scanner.rs`) remains in place for this phase;
  async scanning is deferred to Area 6 (Scanning, Data Model & Core Logic).
- The existing `ImageEntry` type (`types.rs`) is sufficient for this phase; enrichment
  with dimensions and mtime is deferred to Area 6.
- The existing `process_image` function (`image_proc.rs`) remains available for the
  "Quick resize" demo button; a full image tools dialog is deferred to Area 7.
- The debug log collapsible panel is kept in CentralPanel, collapsed by default. Startup
  banner text uses `eprintln!` only — not `debug_logs` — so the empty-state message
  "(no debug messages yet)" is reachable before user actions (FR-009, V6).
- The application remains a single Cargo crate (no workspace split) for this phase;
  module extraction into separate files (e.g., `widgets/`) is acceptable but a full
  workspace refactor is deferred to Area 8.
- `feh` is the required external dependency for "Open in feh" and wallpaper features.
  Graceful degradation when feh is missing is required per Constitution §IV and encoded
  in FR-008a (proactive PATH detection, disabled buttons, no spawn attempts).
- The auto-select-first behavior on folder load is preserved (users expect a selection
  after loading); only the auto-launch of feh is removed.
- Window resize behavior relies on standard egui layout; no custom resize logic is
  required for this phase.
- Stale image entries (files deleted from disk between scans) are not addressed; the
  list reflects the last scan snapshot until the user rescans.
- Deferred roadmap areas (thumbnails Area 4, async scanning Area 6, image tools Area 7,
  code organization Area 8) are out of scope for this feature. References to them in user
  stories indicate future dependencies, not partial delivery expectations.
