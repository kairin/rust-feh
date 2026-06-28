# Feature Specification: Image Tools with Magick Cache

**Feature Branch**: `013-image-tools-magick-cache`

**Created**: 2026-06-24

**Status**: Clarified (Q1 integrated from /speckit-clarify; ready for plan)

**Input**: User description: Advance rust-feh by implementing a cohesive, professional set of image tools (resize, crop, format conversion, batch rename) while deeply integrating ImageMagick’s Magick Cache for performance-oriented persistent storage, fast repeated operations, folder pre-caching, and a "Prepare Fast feh Viewing Cache" capability that produces optimized versions (auto-orient, stripped, standard format) so the external feh viewer loads large or exotic image collections faster on repeated use. All operations safe-by-default (new files), support single + batch, update the visible inventory, log everything. Must follow existing thin-wrapper, minimal-dep, graceful-degradation, responsive-UI, and Linux/feh-centric principles.

**Parent**: [008-tool-capabilities-panel](../008-tool-capabilities-panel/spec.md), [009-external-tool-runtime](../009-external-tool-runtime/spec.md), [005-image-list-presentation](../005-image-list-presentation/spec.md), [011-browsing-experience-round](../011-browsing-experience-round/spec.md)

**Specification Quality Checklist**: [checklists/requirements.md](checklists/requirements.md) (all items pass; re-validated during clarify)

## Clarifications

### Session 2026-06-24 (from feature request analysis)

- Q: Is interactive draggable crop rectangle required for the initial release? → A: **No** — numeric WxH+X+Y (+repage semantics) + a live preview image is sufficient and in scope; graphical crop is explicitly noted as optional later work.
- Q: Must cache configuration and passkey be persisted to disk in v1? → A: **No** — in-memory settings (toggle, root dir, passkey path, TTL) are acceptable following the project's current "no persistent config file yet" stance. Persistence can be added later without breaking the feature.
- Q: Does "Prepare Fast feh Viewing Cache" require modifying how feh is launched at the core level, or can it work via filelists/paths? → A: **Filelist or explicit path hand-off to existing feh launch is sufficient**; the feature enhances the inputs fed to feh rather than replacing feh's viewer behavior.
- Q: When the "Prepare Fast feh Viewing Cache" completes and the user chooses to "launch the viewer on optimized versions", what is the precise relationship between the optimized versions and the feh launch + the rust-feh browser list / inventory? → A: Option A (recommended) — The prepare step materializes real image files on disk (temp location or managed "fast" area, via magick export from cache). A temporary feh-compatible filelist is generated pointing to those real files; feh is launched using the *existing* launch mechanism with that list. The materialized optimized files are also registered in the in-memory inventory and appear in the rust-feh browser list as first-class selectable ImageAssets (with an "optimized"/"fast" status indicator, analogous to current "native" vs. "magick" tags). The user can treat them like any other file (further processing, selection, etc.). This ensures consistency with existing processed-file behavior and makes "usable as regular images" directly actionable.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Safe Single-Image Resize, Crop, or Convert (Priority: P1)

A user browsing a large folder selects one image in the list, chooses a simple edit (resize to specific dimensions or percent, crop by exact geometry, or convert to another common format with quality control), reviews the effect via preview where relevant, selects a safe output option (new file in subfolder or alongside with suffix), applies, and immediately sees the new resulting file appear in the browser list ready to open in the viewer. The original is untouched.

**Why this priority**: This is the minimal independently valuable slice. It directly extends the existing single "Quick resize (50%)" demo into useful, controllable tools while proving the safe-output + inventory update contract. Delivers immediate value even before batch or cache.

**Independent Test**: With a 5-10 image test folder, select one photo, perform a 50% resize to a "processed" subfolder; confirm original still present and unchanged, new file appears in list within seconds, double-clicking the new entry launches it successfully in the viewer, and the action is recorded in the activity log.

**Acceptance Scenarios**:

1. **Given** an image is selected and required external processing capability is present, **When** the user configures a resize (width/height or percentage, with quality) and chooses "save to subfolder", **Then** a new file is created in the chosen location without touching the original, the browser list updates to include it, and the log records the operation.
2. **Given** an image is selected, **When** the user enters a numeric crop geometry (e.g. 800x600+100+50) and applies with default safe output, **Then** the resulting cropped image is added to the list; the preview reflected the geometry before apply.
3. **Given** conversion parameters are set for a source image to a target format (e.g. large TIFF or exotic to high-quality JPEG), **When** applied, **Then** the new file appears with the requested format and the original remains.
4. **Given** the user attempts an in-place destructive edit, **When** the mode is offered, **Then** it requires explicit confirmation and creates a backup of the original before modification.

---

### User Story 2 - Batch Operations on Multiple Selected Images (Priority: P2)

A user selects 50–500 images in the browser (via the existing selection model), chooses the same processing operation and parameters for all of them, picks a consistent output policy (subfolder or suffixed new files), triggers the batch, sees visible non-blocking progress, receives a clear success/failure summary at the end, and finds all successfully created new assets (or updated names) immediately visible and usable in the list.

**Why this priority**: Batch is the natural next step after single; essential for real-world use on large collections. Still independently testable and valuable without the cache story.

**Independent Test**: Select 100 images from a test set; run a batch 75% resize to a subfolder; verify progress indicator is visible and UI remains responsive, summary shows e.g. "98 succeeded, 2 skipped (permission)", the 98 new files appear in the list, originals are intact, and every action (including skips) is in the activity log. Re-select the new files and open one in the viewer.

**Acceptance Scenarios**:

1. **Given** a multi-selection of N images and a valid batch resize/convert/crop configuration with safe new-file output, **When** the batch is started, **Then** a progress indicator is shown, the UI stays interactive for other actions (filter, scroll, open viewer on other images), and upon completion a summary dialog reports counts of success/skipped/error.
2. **Given** batch completes successfully, **When** the user views the folder contents or filtered list, **Then** the newly created files from the batch are present and selectable without requiring a manual re-scan of the entire directory.
3. **Given** some images in the selection cannot be processed (missing permission, unsupported format for the op), **When** the batch finishes, **Then** those are reported in the summary and log without aborting the rest of the batch.
4. **Given** the user chooses the explicit "backup originals + modify in place" policy for a batch, **When** confirming the dangerous action (twice), **Then** backups are created for each and the originals are updated in place; the list reflects the changes.

---

### User Story 3 - Safe Batch Rename with Live Preview (Priority: P2)

A user selects a set of images and opens the rename tool. They define a renaming pattern using available tokens (prefix, suffix, sequential counter with padding, date components, original basename). A live preview table shows every old name → proposed new name. They can adjust the pattern and see updates instantly. Only after explicit confirmation does the rename happen; originals are never touched without the confirmation step; name collisions are detected and handled gracefully (user warned or names adjusted).

**Why this priority**: Rename is a common companion to processing workflows (e.g. "vacation-001.jpg"). Preview + safety makes it trustworthy for bulk work. Valuable independently of pixel processing.

**Independent Test**: Select 20 images with varied names; configure pattern "trip-{date:YYYYMMDD}-{counter:03}"; preview shows correct proposed names; apply; verify on-disk names changed (or list reflects), activity log has entries, and selecting one opens the (now renamed) file in the viewer. Repeat with a pattern that would collide and confirm warning + no partial damage.

**Acceptance Scenarios**:

1. **Given** images selected, **When** the rename dialog opens and the user types a pattern containing counter and date tokens, **Then** the preview table immediately reflects the exact new names that would be produced for the current selection and ordering.
2. **Given** a preview is shown, **When** the user changes the pattern (e.g. adds a prefix or changes padding), **Then** the entire preview list updates consistently without applying any changes yet.
3. **Given** the user clicks Apply after reviewing the preview, **When** confirmation is given, **Then** the renames are performed atomically per file (or best-effort with rollback on error), the browser list reflects the new names, and the log records before/after for each.
4. **Given** a potential name collision is detected in preview or at apply time, **When** the situation arises, **Then** the operation is blocked or the colliding items are highlighted with options (skip, auto-number, cancel) and no files are partially renamed.

---

### User Story 4 - Enable and Automatically Benefit from Persistent Image Cache (Priority: P3)

A user installs the external cache tool if needed, enables the cache feature in the app (with basic settings for location, passkey, retention), performs processing operations, and on the second identical operation on the same image experiences dramatically faster results because the processed pixels are retrieved from the cache instead of recomputing. The cache also stores originals for quick retrieval. All of this happens transparently behind the existing tool operations.

**Why this priority**: This is the "smart performance layer" that makes the tools scale to large collections and repeated use. Still independently valuable once the basic tools exist.

**Independent Test**: With cache enabled and a passkey configured, process (resize) a 20MP image once (note time); immediately repeat the exact same parameters on the same source; observe the second run completes in a small fraction of the time and is logged as a cache hit. Disable cache and confirm the operation becomes slow again.

**Acceptance Scenarios**:

1. **Given** the cache capability is detected and the user has enabled it with valid settings, **When** any supported processing operation (resize/crop/convert) succeeds, **Then** the result (and the original) is stored in the cache under a stable identifier derived from the source and operation.
2. **Given** a prior processed result exists in the cache for the exact source + parameters, **When** the user requests the same operation again, **Then** the result is served from cache (fast path) and the activity log indicates cache usage.
3. **Given** cache is enabled but the specific result is not present (first time or expired), **When** processing runs, **Then** it falls back to full computation and then stores the result for future use.
4. **Given** the user disables the cache feature, **When** subsequent operations run, **Then** they behave exactly as without cache (no storage or lookup attempts) and the UI indicates cache is off.

---

### User Story 5 - Pre-Cache Folder and Prepare Fast Viewer Experience (Priority: P3)

A user with a large folder (thousands of images, some exotic formats or unoriented) runs a "Pre-cache entire folder" action in the background. Later, or as a dedicated step, they run "Prepare Fast feh Viewing Cache". This produces optimized, viewer-friendly versions (auto-oriented, metadata-stripped, converted to high-quality JPEG/PNG where beneficial) by materializing real files from the cache (in a temp or managed location). The user is then offered the choice to launch the external viewer against these optimized versions (via a generated temporary feh-compatible filelist pointing to the materialized files, using the existing launch mechanism) instead of the raw originals, resulting in noticeably quicker first-image display and smoother navigation for the session. The materialized optimized files are registered in the in-memory inventory and appear as first-class selectable entries in the browser list (with an "optimized" or "fast" status indicator).

**Why this priority**: This is the killer integration that directly fulfills the project's core promise ("feh loads large/exotic images much faster"). It is the concrete reason to have the cache and processing tools together. Highest leverage for the "Linux-first, feh-centric" value prop.

**Independent Test**: On a 2000+ image folder containing a mix of orientations and formats, run Prepare Fast feh Viewing Cache (background); when complete, use the "launch fast" option; time from feh launch to first image being fully displayed and navigable vs. launching on the raw folder. Confirm the fast path is faster, originals are untouched, the log records the preparation steps (including materialization), and the optimized files appear as selectable entries in the rust-feh browser list (with status indicator) and can be further processed/selected like any other asset.

**Acceptance Scenarios**:

1. **Given** a folder with many images and the cache + processing tools available, **When** the user triggers "Pre-cache entire folder", **Then** a background job runs without freezing the UI, progress is visible, and upon completion the cache contains entries for the images (or a useful subset).
2. **Given** the prepare fast action is invoked, **When** it completes, **Then** the system has produced (in the cache) optimized versions for the folder and offers the user a "Launch viewer on optimized versions" action.
3. **Given** the user chooses to launch the viewer on the prepared fast versions, **When** the launch happens, **Then** the viewer receives a temporary filelist pointing to real materialized optimized image files on disk (not the raw originals or internal cache references); the first image displays and subsequent navigation feels responsive even on a large collection. The same materialized files are also visible and selectable in the rust-feh browser list as regular ImageAssets.
4. **Given** cache or magick tools are missing or disabled, **When** the user attempts pre-cache or prepare-fast, **Then** a clear message explains the missing capability and offers install guidance; the core browser and single-image tools remain fully functional.

---

### Edge Cases

- What happens when a crop rectangle extends outside the image bounds or uses invalid geometry syntax?
- How does the system handle a batch rename pattern that would produce duplicate names for different source files?
- What happens on permission errors, read-only media, or disk-full during output creation or cache put?
- How are very large images (hundreds of MB) or unsupported source formats handled for cache vs. direct processing?
- What occurs if the user triggers a long-running batch or pre-cache and then closes the app or switches folders?
- How does "Prepare Fast feh" behave on a folder that has already been prepared (idempotent / incremental)?
- What happens when magick-cache is installed but has not been initialized with `create` + passkey (user has not run the one-time setup)?
- How are network-mounted or slow storage paths treated for cache put/get vs. direct file output (performance and reliability)?
- Does an in-progress cache population or prepare-fast job survive a re-scan or filter change in the browser?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST allow a user to select one or more images and apply resize, crop (numeric geometry), or format conversion operations.
- **FR-002**: The system MUST default all image processing operations to creating new output files; originals MUST never be overwritten without an explicit, confirmed "modify in place with backup" mode.
- **FR-003**: The system MUST provide output location choices for new files (dedicated subfolder or alongside originals with a user-visible suffix) and apply the choice consistently for single and batch operations.
- **FR-004**: The system MUST log every processing or rename action (success, skip, error) to the activity log with sufficient detail for the user to understand what changed.
- **FR-005**: After any successful create-new-file or rename operation, the in-memory image inventory MUST be updated so the new or renamed assets appear in the browser list without requiring a full manual directory re-scan.
- **FR-006**: The system MUST support batch application of a single operation (resize/crop/convert) across a user selection of images, including visible non-blocking progress feedback and a final summary of outcomes.
- **FR-007**: For batch rename, the system MUST support pattern-based renaming using at minimum prefix, suffix, zero-padded counter, and date-derived tokens; it MUST show a live, accurate preview of every resulting name before any files are changed.
- **FR-008**: The system MUST detect the presence/absence of the external cache capability and surface its status alongside other tools; when absent, core single/batch tools remain usable.
- **FR-009**: When the cache capability is enabled by the user, the system MUST automatically store originals and processing results (resize/crop/convert) into the cache using stable identifiers so that identical subsequent operations can be satisfied quickly.
- **FR-010**: The system MUST provide a user-invokable "Pre-cache entire folder" action that populates the cache for images in the current view (background, interruptible, with progress).
- **FR-011**: The system MUST provide a "Prepare Fast feh Viewing Cache" action that, using available processing and cache capabilities, generates optimized viewer-friendly versions (auto-orient, metadata stripped, preferred standard formats at good quality) for a folder and stores them via the cache mechanism.
- **FR-012**: After a successful "Prepare Fast feh Viewing Cache", the system MUST offer the user the ability to launch the external viewer against the prepared optimized versions (via a temporary filelist to the materialized real files, using the existing launch path) instead of raw originals, while still supporting normal launch behavior for the raw collection. The materialized optimized files MUST also be registered in the in-memory inventory so they appear as first-class selectable entries in the browser list (with a visible "optimized"/"fast" status) and can be further acted upon by other tools.
- **FR-013**: All cache-related features (put, get, pre-cache, prepare-fast) MUST degrade gracefully with clear user messages when the cache tool is not installed or not initialized; the rest of the image tools and browser must continue to function.
- **FR-014**: Crop operations MUST accept standard geometry of the form WxH+X+Y (with +repage semantics) and provide a visual preview of the crop region on the source before the operation is applied.
- **FR-015**: Resize operations MUST support absolute pixel dimensions, percentage scaling, and basic quality/compression controls for output formats that support them; filter choice (e.g. high-quality resampling) SHOULD be exposed for user control.
- **FR-016**: Format conversion MUST allow choosing a target format from common options with appropriate quality or compression settings and smart defaults (e.g. large/exotic sources default toward high-quality JPEG).
- **FR-017**: Every batch operation MUST present a clear summary + confirmation dialog before execution, showing the number of affected images, the operation, and the output policy.
- **FR-018**: The "in-place with backup" policy, when chosen, MUST create a backup copy of each original before modification and MUST require at least two explicit user confirmations.
- **FR-019**: Cache configuration (enable/disable, cache root, passkey location, default retention/TTL) MUST be adjustable by the user at runtime (in-memory is acceptable for the initial version of this feature).
- **FR-020**: The system MUST surface install or setup guidance for the external cache and processing tools when they are detected as missing, without blocking other functionality.

*No clarification markers remain after the clarifications section above.*

### Key Entities *(include if feature involves data)*

- **ImageAsset**: Represents an image file discovered by the browser (path, metadata, status such as native/magick-processed). The primary thing the user selects and operates on.
- **ProcessingOperation**: A user request to transform one or more ImageAssets (type: resize/crop/convert/rename, parameters, output policy, target selection).
- **OutputPolicy**: The rules governing where results are written (new subfolder, suffixed sibling, or in-place with backup).
- **RenamePlan**: A pattern plus the concrete before/after name mapping for a selection; includes preview state and collision detection.
- **CacheConfiguration**: User-controlled settings for the persistent image cache (enabled flag, storage root, credential reference, retention policy).
- **CachedAsset**: A stored original or processing result retrievable by stable key for fast subsequent use.
- **PreparationJob**: Background work (pre-cache or prepare-fast) that populates the cache for a folder or collection and can report progress and completion.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A user can complete a single-image resize, crop, or conversion (including preview where applicable) and see the resulting new asset appear in the browser list, ready to be opened in the viewer, in under 30 seconds for a typical 10–20 megapixel photograph on ordinary workstation hardware.
- **SC-002**: With the image cache enabled, repeating an identical processing operation (same source image + same parameters) on a previously processed image completes in under 3 seconds (perceived as near-instant retrieval); repeating the operation with the cache disabled takes materially longer.
- **SC-003**: A user can select 200 images, configure and confirm a batch resize or conversion with safe new-file output, and receive a completion summary while the rest of the browser (scrolling, filtering, launching the viewer on other images) remains usable throughout; at least 95% of the selected images succeed on a typical run.
- **SC-004**: For a collection of at least 2000 images containing a realistic mix of formats and orientations, a user can run the "Prepare Fast feh Viewing Cache" action to completion and then launch the viewer against the prepared versions such that the first image displays and is navigable in less than half the time (or subjectively "much faster") compared with launching directly against the raw originals on the same hardware and storage.
- **SC-005**: 100% of normal (non-explicit-in-place) processing and rename operations leave the original files on disk completely unmodified; any use of the in-place-with-backup path creates verifiable backups and requires explicit multi-step confirmation.
- **SC-006**: When the external cache or processing tools are not installed or not configured, the core single-image and batch tools (resize/crop/convert/rename) remain fully functional with clear, actionable guidance shown to the user; no crashes, hangs, or silent data loss occur.
- **SC-007**: A batch rename preview for 500+ files using counter and date tokens renders the proposed names accurately within 2 seconds of the user finishing the pattern; applying the rename updates the visible list with zero data loss on successful completion and clear reporting on any skipped items.
- **SC-008**: Pre-cache and prepare-fast background jobs can be started, observed for progress via status/activity, and produce usable cached/optimized assets without requiring the user to keep the folder selected or the app in the foreground the entire time.

## Assumptions

- The initial implementation treats cache settings (root directory, passkey file path, default TTL such as "90 days" or "never") as in-memory only; a future increment can add persistence without invalidating user work.
- Numeric crop geometry (WxH+X+Y) plus a rendered preview image satisfies the crop story for v1; a draggable rectangular region editor inside a dedicated dialog is explicitly future work.
- "Prepare Fast feh Viewing Cache" produces assets that the existing viewer launch mechanism can consume (via temporary file list or path substitution); it does not change feh's own viewing, zooming, or wallpaper behavior.
- When both the external processor and the in-process image library are available, the system prefers the external for quality and cache compatibility but can fall back for basic operations when the external tool is absent (consistent with current graceful degradation for magick).
- Output file names for new assets use straightforward, user-visible suffixes or subfolder names; advanced deduplication/collision strategies beyond warning + skip are out of scope for the first version.
- The feature is justified under the thin-wrapper and feh-centric principles because the tools and especially the "fast viewing" preparation directly improve the experience of handing large/exotic collections to feh without attempting to replace feh's viewer role.
- All background work (batch, pre-cache, prepare-fast) must keep the primary UI responsive; this is consistent with the existing background scan contract.
- One-time user setup of the external cache (e.g. running `create` with a passkey) is documented but is a prerequisite outside the application's control; the app detects and guides rather than automating privileged cache initialization.
- No changes are required to the core low-memory virtualized list or the ~126 MiB target for 10k-image sessions; new processed assets participate in the same list model.
- The scope is limited to the four named operations (resize, crop, convert, rename) plus the cache integration layer; thumbnail grids, non-destructive adjustment layers, or watermarking are not included.
