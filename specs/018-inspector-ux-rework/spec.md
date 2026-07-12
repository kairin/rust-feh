# Feature Specification: Inspector UX Rework

**Feature Branch**: `018-inspector-ux-rework`

**Created**: 2026-07-12

**Status**: In progress (Batch 0 — pre-work fixes + this scaffolding)

**Input**: Maintainer request: make the right-hand inspector panel the primary, intuitive control surface — fold every section by default and expand on interaction, move folder navigation and the file list into the inspector, leave the central panel showing only the viewed/staged image, remove duplicated menu-bar actions, and make inspector section panels reusable/pinnable in detached windows for consistency.

## Clarifications

### Session 2026-07-12

- Q: How should browse controls behave on startup? → A: **Collapsed by default; auto-expand ONLY when no folder is loaded.** No section opens "just for convenience." When a folder is already set at launch, the Browse section stays folded until the user opens it.

- Q: Where do the file list and folder navigation live? → A: **Inside the inspector panel.** The central panel becomes ONLY the viewed/staged image — it fills the entire space, with no list, subfolder navigation controls, or inventory overlay. The file list (both flat and tree views, fully virtualized) and the folder drill-down move into the inspector's persistent zones.

- Q: How are duplicate menu actions handled? → A: **Removed entirely.** File → Choose folder / Rescan and View → Include subfolders / Detect exotic formats are deleted from the menu bar (removing empty menu shells if any result). The inspector's Browse section becomes the single source of truth for these four actions. The logic was already single-sourced; only the duplicate entry points are removed from the menus.

- Q: How does pinning work for detached windows? → A: **One shared Image-actions detached window, modeled as PanelPin::Image(path).** Pinning is resolved via `PanelContext` (which holds pinned vs. live state), so the detached window can keep acting on a pinned image even while the main window's selection moves. Unpinning makes it follow the live selection again.

## User Scenarios & Testing

### User Story 1 — Sections fold by default and auto-expand when relevant (Priority: P1)

Browse and other inspector sections are collapsed by default, giving the file list maximum vertical space. The sections auto-expand when needed (no folder loaded → Browse opens; scan starts → Session status opens; required tool missing → Dependencies opens), and manual collapse persists across interactions at the same scope.

**Why this priority**: A cramped drawer makes the list hard to scan; auto-expand on necessity ensures the user discovers features without hunting through collapsed sections, while keeping the default state focused.

**Independent Test**: Launch rust-feh with no folder set; verify Browse is open and others are collapsed. Load a folder; verify all sections return to collapsed. Start a scan; verify Session status opens automatically. Close all drawers; verify they stay closed until the next relevant event.

**Acceptance Scenarios**:

1. **Given** a fresh rust-feh launch with no folder set, **When** the inspector renders, **Then** the Browse section is open (expanding the drawer if needed) and Image actions, Feh instances, Session status, Activity log, Dependencies, and Format discovery are all collapsed.
2. **Given** a folder is selected in Browse, **When** the selection settles and a scan completes, **Then** all sections return to collapsed state.
3. **Given** a scan is in progress (user initiated or automatic re-scan), **When** the scan status updates, **Then** the Session status section auto-opens (expanding the drawer) without user action.
4. **Given** a scan or action detects a missing required tool, **When** the detection occurs, **Then** the Dependencies or Format discovery section auto-opens with a clear message, and manually closing it stops it from re-opening on the same event.

---

### User Story 2 — File list and folder navigation live in the inspector; central panel is image-only (Priority: P1)

The file list (flat or tree, virtualized, scrollable) and the folder drill-down navigator move from various locations into the inspector's persistent top/middle zones. The central panel shrinks to show only the viewed/staged image, filling the available space without any text overlay or secondary controls.

**Why this priority**: Today's central panel mixes the image with list, controls, and inventory, fragmenting attention. Moving the list into the fixed-width inspector keeps navigation and actions in one column while the image gets focused display. Virtualization at inspector width ensures the list remains fast even at 10k+ images.

**Independent Test**: Load a folder with 100+ images; verify the file list is fully visible in the inspector and virtualized (scroll through 1000+ rows smoothly); select images and verify each is displayed centrally; toggle Flat/Tree and verify both work from the inspector; use the Up/Folder drill-down and verify navigation from the inspector.

**Acceptance Scenarios**:

1. **Given** a loaded folder and a selected image, **When** the central panel renders, **Then** only the image is displayed (no list, no subfolder nav, no action buttons); the image fills the available central panel area.
2. **Given** the file list in the inspector, **When** the user selects a row, **Then** that image is displayed centrally within one frame, and no text overlay, breadcrumb, or secondary controls appear in the central panel.
3. **Given** a virtualized list of 10k images at inspector width (fixed), **When** the user scrolls through 100+ rows, **Then** scroll latency is imperceptible (<50 ms frame time) and matches the pre-rework baseline within 10%.
4. **Given** a flat list view and a tree view, **When** toggling between them from the inspector Flat/Tree control, **Then** both views render fully in the inspector's primary zone, selection is preserved across the toggle, and both remain virtualized.
5. **Given** the inspector's fold/unfold drawer, **When** the drawer expands and contracts, **Then** the file list height adjusts proportionally (no jitter, no shrink-oscillate), with a floor of row_height × 4.

---

### User Story 3 — Menu duplicates removed; Browse is the single home (Priority: P2)

File menu entries (Choose folder, Rescan) and View menu entries (Include subfolders, Detect exotic formats) are deleted from the menu bar. Every action is reachable from the Browse section in the inspector, and no action is accessible only via the menu.

**Why this priority**: Menu duplication creates confusion (where do I click for this?) and wastes screen real estate. Centralizing browse and discovery in the inspector panel (the primary control surface) reduces cognitive load and reinforces the panel's role.

**Independent Test**: Verify File/View menus no longer contain the four deleted entries; verify each action is reachable and functional from the inspector's Browse section.

**Acceptance Scenarios**:

1. **Given** the File menu, **When** opened, **Then** it does not contain "Choose folder" or "Rescan" (only other file-level actions remain, or the menu is deleted if empty).
2. **Given** the View menu, **When** opened, **Then** it does not contain "Include subfolders" or "Detect exotic formats" (only other view-level actions remain, or the menu is deleted if empty).
3. **Given** the inspector's Browse section, **When** opened or expanded, **Then** it contains at least "Choose folder," "Rescan," "Include subfolders," and "Detect exotic formats," all functional and performing their original actions.
4. **Given** a user workflow (choose folder → rescan → toggle subfolders), **When** all actions are executed from the inspector Browse, **Then** every step completes successfully and the activity log records each action.

---

### User Story 4 — Single pinnable Image-actions detached window (Priority: P2)

Detaching the Image-actions section opens one shared window (not one per detached image). The window can be pinned to a specific image via `PanelPin::Image(path)`, and `PanelContext` resolves whether it's acting on the pinned image or the live selection. Unpinning makes it follow the live selection again.

**Why this priority**: Multiple detached windows clone state and create confusion about which window acts on which image. A single shared window with explicit pinning (visual, title-indicated) is clearer and lighter. Pinning lets power users keep one window on a reference image while browsing others.

**Independent Test**: Detach Image-actions; verify one window opens. Pin it to an image; change the main window's selection and verify the detached window still acts on the pinned image (title shows pin, actions target the pinned file). Unpin; verify it follows the live selection again.

**Acceptance Scenarios**:

1. **Given** the Image-actions section, **When** the user clicks Detach, **Then** one Image-actions window opens (not multiple copies); clicking Detach again returns focus to the existing window rather than opening a second.
2. **Given** the detached window, **When** the user clicks Pin (or a 📌 button), **Then** the window title updates to show the pinned image's filename, and subsequent actions act on the pinned image.
3. **Given** a pinned detached window, **When** the main window's file selection changes to a different image, **Then** the detached window continues acting on the pinned image and its title remains unchanged.
4. **Given** a pinned window with its pinned file deleted or moved, **When** an action is attempted, **Then** the action is disabled and a clear message (e.g., "Pinned image no longer available") is shown.
5. **Given** a pinned detached window, **When** the user clicks Unpin, **Then** the window title reverts to a generic "Image actions," and subsequent actions follow the live selection.

---

### Edge Cases

- First-run (no folder, no prefs): Browse opens, drawer expands, all other sections folded.
- Switching folders via Browse: current list is replaced, selection resets to index 0, Session status auto-opens if a scan is triggered, other sections stay as the user left them.
- Detached window closed by OS (window manager × button): treated as Unpin + close; the section can be re-detached.
- Pinned image's folder changes (moved, renamed): the path record becomes stale; next interaction shows "pinned image no longer available."
- Rapid folder changes while a scan is in-flight: the Session status section remains auto-open, list indices update in-place, the detached window (if pinned to a different folder) disables actions gracefully.
- List selection during drawer toggle: selection is not lost; scrolling to the selected row (if it was off-screen) may occur when the drawer shrinks back to full list height.
- Very tall sections (e.g., Activity log with 100+ entries): the drawer's bounded ScrollArea prevents overflow; users scroll within the section independently.

## Requirements

### Functional Requirements

- **FR-001**: Inspector MUST have a three-zone persistent layout: (A) nav strip with Up, breadcrumb, Flat/Tree toggle, spinner; (B) optional subfolder drill-down (capped scroll, hidden when none); (C) primary virtualized file list (flat or tree, auto-height 80% of available, floor row_height × 4).

- **FR-002**: All 7 inspector sections (Browse controls, Image actions, Feh instances, Session status, Activity log, Dependencies, Format discovery) MUST default to collapsed (folded CollapsingHeaders); exactly one meta-toggle expands/collapses the drawer, bounding its height to 180–360 px when expanded, 24 px when collapsed.

- **FR-003**: Auto-expand logic MUST run per-frame: no folder loaded → Browse opens; scan status changes → Session status opens; missing tool detected → Dependencies or Format discovery opens. Manual close of a section MUST prevent it from auto-opening again during that event scope (same folder, same scan attempt).

- **FR-004**: Central panel MUST display ONLY the staged image (full fit-to-pane scaling), with zero menu controls, list overlay, subfolder breadcrumb, or inventory text; the image selection is always driven from the inspector's file list.

- **FR-005**: File list (flat and tree views) MUST be relocated into the inspector's Zone C; virtualization and caching MUST remain in place; list height MUST adapt dynamically (available_height − drawer_reserved_height, floor row_height × 4); selection, scroll position, and sort state MUST persist across view toggles.

- **FR-006**: Menu bar MUST have duplicates removed: File→Choose folder/Rescan and View→Include subfolders/Detect exotic formats are deleted; empty menu shells are removed; the inspector's Browse section is the single authoritative home for these four actions.

- **FR-007**: Inspector sections and their open/collapsed states MUST use a `HashSet<InspectorSection>` enum (7 variants + ALL for deterministic iteration); this replaces the existing 7 open-state bools and deps_section_open plumbing.

- **FR-008**: Detached windows MUST be generalized via `detached: HashMap<InspectorSection, DetachedWindow>`; the render loop MUST iterate `InspectorSection::ALL` (deterministic order, never a HashMap) with one shared window-chrome helper; every section gains detach/re-attach capability.

- **FR-009**: Image-actions detached window (when detached) MUST render both Image Tools and Actions subsections; a single shared `PanelPin::Image(PathBuf)` model tracks the pin target; `PanelContext { image, folder, pinned }` resolves the target before any `&mut self` body.

- **FR-010**: Menu-bar deletions and inspector width floor (raised to ~440 px for list-bearing inspector) are safe only after F3 cache-staleness prerequisite fix (Batch 0, task 0.6) lands; this feature's batches MUST NOT land before that.

- **FR-011**: Inspector MUST NOT use an infinite outer `ScrollArea` (which breaks list virtualization); heights MUST be fixed/deterministic so list virtualization never jitters, and the panel is always stable during resize and drawer toggle.

- **FR-012**: No new dependencies, no network access, no regression to virtualized-list scroll performance at 10k+ images (within 10% of pre-rework baseline); no in-process image data read during listing, no UI block during browse/scan.

### Key Entities

- **InspectorSection**: enum of 7 variant names (Browse, ImageActions, FehInstances, SessionStatus, ActivityLog, Dependencies, FormatDiscovery), plus a static `ALL` list for deterministic iteration.

- **PanelPin**: enum modeling pin state — `None` (follow live selection) or `Image(PathBuf)` (pinned to a specific file).

- **PanelContext**: struct holding `image: PathBuf`, `folder: PathBuf`, `pinned: bool` — the resolved state used by image-action code paths, cloned before any mutable borrow.

- **DetachedWindow**: struct holding `pin: PanelPin`, `id: egui::Window::Id`, `visible: bool` — metadata for each detached section's window.

## Success Criteria

- **SC-001**: Drawer stays collapsed by default across 5 fresh launches with a folder already set; Browse auto-opens 100% of the time when no folder is loaded at startup.

- **SC-002**: List scroll performance at 10k+ images within 10% of pre-rework baseline (measured via `.agents/ENV.md`'s performance fixture).

- **SC-003**: Central panel never displays list, subfolder nav, or control overlays in 100% of observed frames across a 50-image selection pass.

- **SC-004**: All four menu duplicates (File→Choose folder/Rescan, View→Include subfolders/Detect exotic) absent in 100% of trials; every action reachable from Browse section.

- **SC-005**: Detached Image-actions window opens once; subsequent detach clicks focus the existing window (zero duplicates); pinning and unpinning reflect title and behavior changes within 1 frame.

- **SC-006**: Pinned detached window's actions target the pinned image in 100% of trials, even when main selection differs by >10 rows.

- **SC-007**: Security review (Batch 5, task 5.5) on the pinning diff passes with zero HIGH/MEDIUM findings.

## Assumptions

- Batch 0 (this batch) is prerequisite-only and delivers none of the four decisions yet. Batch 0 creates the spec scaffolding and lands Batch 0 fixes (F1–F6) from feature 017; the actual rework happens in Batches 1–6 per tasks.md, each checkpointing `cargo test` green before advancing.

- The inspector's three-zone layout (nav strip, drill-down, file list) is relocated verbatim from today's various code homes; no algorithmic changes to folder scanning, filtering, or file-tree logic.

- The pinning model (`PanelPin` + `PanelContext`) ensures a detached window can act on a pinned image without holding a live reference; all image-action code will be threaded with `&PanelContext` before any `&mut self`.

- Auto-expand logic uses the same per-frame event queue that drives Session status and tool-detection alerts; Batch 3+ integrates with that queue.

- Central panel image display (the "staged image" from feature 016) continues unchanged; this feature only removes the list and subfolder controls from the central space.
