# Feature Specification: Multi-Feh Instance Launcher & Image Clipboard

**Feature Branch**: `014-multi-feh-clipboard`

**Created**: 2026-06-26

**Status**: Draft

**Input**: User description: "we need the ability where we can add multiple instances of feh to launch, currently only 1 open in feh button available. we need to have a + that can allow us to add multiple instances of feh, with the goal of opening feh in different folders that has been scanned. also when feh open, one of the allowed abilities is when right click an open photo we can copy the photo directly to copy so that we can paste the photo directly to terminal or to an upload to a post on browser etc."

## Clarifications

### Session 2026-06-26

- Q: Clipboard Target Format (FR-008) → A: Standardize on `image/png` (lossless, widely supported by terminals, browsers, editors, and most clipboard managers on Linux).
- Q: "Currently active scanned folder" definition (FR-002) → A: Resolve in priority order — (1) the folder of the currently selected folder-tree node, else (2) the parent folder of the currently selected image, else (3) the active scan root. If none can be resolved, the entry is created with no folder assigned and the user must pick one.
- Q: SC-006 fidelity (FR-008 conflict) → A: Clipboard fidelity is defined as pixel/content equivalence at native resolution, NOT byte-for-byte file identity (PNG transcoding cannot preserve original encoded bytes for non-PNG sources).
- Q: Clipboard performance vs single-threaded UI (SC-003) → A: v1 performs the copy on the main thread and MAY briefly block the render loop for large images; the <3s target applies to images up to 50 MB on typical hardware. A non-blocking/background implementation is deferred to a future iteration and explicitly out of scope for v1.
- Q: "Viewable images" definition (FR-013) → A: An image is "viewable" if it appears in the folder's scanned inventory with a supported image extension; FR-013 checks the scanned inventory for the assigned folder, independent of any active text/name filter.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Launch Multiple Feh Instances for Different Folders (Priority: P1)

A user has scanned several image folders (e.g., "vacation-photos", "wallpapers", "screenshots") and wants to open each one in its own feh window simultaneously so they can compare images across directories or work on different sets of images in parallel.

**Why this priority**: This is the primary feature request. Currently only one feh instance can be launched at a time, forcing users to close one feh window before opening another folder. Multi-instance support unlocks side-by-side comparisons and parallel workflows that the single-launch model cannot support.

**Independent Test**: Scan two different folders, add each as a separate feh launch entry via the "+" button, and verify both feh windows open simultaneously with their respective folder contents. Close one feh window and confirm the other remains unaffected and functional.

**Acceptance Scenarios**:

1. **Given** rust-feh is open with one or more scanned folders available in the folder tree, **When** the user clicks the "+" button next to the feh launch area, **Then** a new feh launch entry is created using FR-002's default-folder resolution order, and the entry appears in a numbered list of launch entries.
2. **Given** at least one feh launch entry exists, **When** the user clicks the "Launch" or "Open" button on that entry, **Then** feh opens in a new window displaying images from the folder associated with that entry, without interfering with any already-running feh instances.
3. **Given** multiple feh launch entries exist pointing to different folders, **When** the user launches two or more of them, **Then** all specified feh windows open concurrently, each showing the correct folder's contents, and all windows remain independently operable.
4. **Given** a launch entry is no longer needed, **When** the user clicks the "Remove" (×) button on that entry, **Then** the entry is deleted from the list and does not affect any already-running feh instances.

---

### User Story 2 - Per-Instance Folder Assignment (Priority: P1)

A user wants each feh launch entry to remember which folder it is associated with, and be able to change the target folder of an existing entry without deleting and recreating it.

**Why this priority**: Without per-instance folder assignment, the multi-launch feature degenerates into duplicate single-launch buttons, defeating the purpose of parallel folder workflows.

**Independent Test**: Create two launch entries, assign each to a different scanned folder using the folder selector, launch both, and verify each feh window displays images from its assigned folder. Reassign one entry to a third folder, relaunch, and verify the change takes effect.

**Acceptance Scenarios**:

1. **Given** a new launch entry is created, **When** viewed by the user, **Then** it displays the name of its currently assigned folder and provides a way to select a different scanned folder.
2. **Given** a launch entry is assigned to folder A, **When** the user changes its assignment to folder B, **Then** subsequent launches of that entry open feh showing folder B's images.
3. **Given** a launch entry is assigned to a folder that no longer exists or has been removed from the scan tree, **When** the user attempts to launch it, **Then** the system shows a clear message indicating the folder is unavailable and suggests reassigning or removing the entry.

---

### User Story 3 - Copy Image to System Clipboard (Priority: P2)

A user is viewing images in rust-feh's image list and wants to copy a specific image to the system clipboard so they can paste it directly into a terminal (e.g., with tools that accept image paste), upload it to a form in a web browser, or insert it into a document.

**Why this priority**: The clipboard copy feature enables a common workflow — grabbing an image from the scanned collection and pasting it elsewhere — without needing to open the image in feh, locate it in a file manager, or manually copy the file path. It directly reduces friction in content-sharing tasks.

**Independent Test**: Select an image in rust-feh's image list, right-click and choose "Copy image to clipboard", then paste (Ctrl+V) into an application that accepts image pastes (e.g., a rich text editor, image upload field in a browser) and confirm the pasted content is the expected image.

**Acceptance Scenarios**:

1. **Given** at least one image is displayed in the filtered list, **When** the user right-clicks on an image row, **Then** a context menu appears with a "Copy image to clipboard" option.
2. **Given** the context menu is open, **When** the user clicks "Copy image to clipboard", **Then** the full image data (not just the file path) is placed on the system clipboard, and a confirmation message appears in the status bar (e.g., "Copied image to clipboard: vacation01.jpg").
3. **Given** an image has been copied to clipboard, **When** the user pastes into a target application that supports image data, **Then** the pasted content is the original image at its native resolution and quality.
4. **Given** clipboard operations are attempted on an image that cannot be read (e.g., corrupt file, deleted between scan and copy), **When** the user clicks "Copy image to clipboard", **Then** an error message is displayed in the status bar and the clipboard is not modified.

---

### User Story 4 - Launch Entry Labels (Priority: P3)

A user with several feh launch entries wants to label them meaningfully (e.g., "Wallpapers - 2560x1440", "Client Project A") rather than relying on raw folder paths to tell them apart.

**Why this priority**: Labels improve usability when working with many entries but are not essential to the core multi-launch functionality. Users can work with folder-path-based identification initially.

**Independent Test**: Create a launch entry, set a custom label, and verify the label appears in the entry list instead of (or alongside) the folder path.

**Acceptance Scenarios**:

1. **Given** a launch entry exists, **When** the user edits its label field, **Then** the new label is displayed in the entry list.
2. **Given** a launch entry has no custom label set, **When** displayed in the entry list, **Then** the assigned folder's display name is used as the default identifier.

---

### Edge Cases

- What happens when feh is not installed and the user tries to add launch entries? The "+" button should be available (entries can be created for future use), but the launch button should be disabled with a clear "feh not installed" indicator per entry.
- What happens when all images in a folder's filtered list are deleted between creating the entry and launching? Launch should fail gracefully with a status message indicating the folder contains no viewable images.
- How does the system handle extremely large folders (10,000+ images) when launching multiple feh instances simultaneously? Each feh instance operates independently on its own filelist; system resource limits (memory, file handles) are the user's responsibility. The application must not crash or hang when generating large filelists.
- What happens to clipboard content when the user closes rust-feh? System clipboard behavior follows the operating system's default — clipboard content persists after application exit on Linux (unless a clipboard manager clears it).
- How does the clipboard copy handle very large images (e.g., 200MP raw files)? The copy operation should complete within a reasonable time (under 5 seconds for typical images). For extremely large files, a progress indication or timeout with a clear message is preferred over an application freeze.
- What if the user launches the same folder in multiple feh instances? This is allowed; each instance operates independently. There is no coordination between instances — the user accepts that two feh windows may show the same images.
- How are launch entries persisted across application restarts? Launch entries must survive application close and reopen so users do not need to reconfigure their multi-folder setup each session.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST display a list of feh launch entries, each representing a configurable feh instance that can be launched independently.
- **FR-002**: The system MUST provide a "+" (add) button that creates a new feh launch entry, defaulting to the currently active scanned folder (resolved in priority order: selected folder-tree node → selected image's parent folder → active scan root → unassigned).
- **FR-003**: Each launch entry MUST allow the user to select which scanned folder it targets, from the set of folders present in the folder tree.
- **FR-004**: Each launch entry MUST have a "Launch" button that spawns a feh subprocess displaying the entry's assigned folder images, without interfering with any other running feh instances.
- **FR-005**: Each launch entry MUST have a "Remove" (×) button that deletes the entry from the list, without affecting any already-running feh instances.
- **FR-006**: Launch entries MUST persist across application restarts (survive close and reopen).
- **FR-007**: The system MUST allow the user to right-click an image row in the filtered image list and present a context menu containing a "Copy image to clipboard" action.
- **FR-008**: The "Copy image to clipboard" action MUST read the full image file data, decode it, and place it on the system clipboard as `image/png` (lossless format widely supported by terminals, browsers, editors, and clipboard managers on Linux). Fidelity is defined as pixel/content equivalence at native resolution (see SC-006); byte-for-byte preservation of the original encoded file is NOT required.
- **FR-009**: The system MUST display a confirmation message in the status bar after a successful clipboard copy, including the image file name.
- **FR-010**: The system MUST display an error message in the status bar if a clipboard copy fails (e.g., unreadable file, permissions error), and MUST NOT modify the clipboard in that case.
- **FR-011**: When feh is not installed, launch buttons on all entries MUST be visibly disabled with an indicator showing "feh not installed".
- **FR-012**: Launch entries MUST support an optional user-editable label that overrides the default folder-name display.
- **FR-013**: When a launch entry's assigned folder contains no viewable images at launch time, the system MUST show a status message indicating the folder is empty and MUST NOT launch feh. "Viewable images" are entries present in the folder's scanned inventory with a supported image extension, evaluated independently of any active text/name filter.
- **FR-014**: The existing single "Open in feh" button in the image actions panel MUST be retained as a convenience shortcut (it remains the primary single-launch flow); the multi-entry launcher is an additional panel for power users.
- **FR-015**: The launcher panel MUST provide a "Launch All" action that launches every configured (folder-assigned, launchable) entry in one click; entries that are not launchable (no folder, missing folder, empty folder, or feh not installed) are skipped without aborting the batch.
- **FR-016**: The launcher panel MAY be detachable into its own window (consistent with the existing inspector-section detach pattern); this is a UX convenience and MUST NOT change launch behavior.

### Key Entities

- **FehLaunchEntry**: Represents a single configurable feh launch profile. Key attributes: assigned folder path, optional user label, and stable ID. It is passive persisted configuration only; `ui_logic` resolves launchability and generates feh filelist data, while `main.rs` spawns the feh subprocess as UI orchestration glue.
- **FehLaunchList**: An ordered collection of FehLaunchEntry instances. Persisted between sessions. Supports add and remove operations. (Reorder is not part of v1 scope; entries are displayed in insertion order with a stable "#N" index.)

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can configure and launch at least 5 concurrent feh instances pointing to different folders without the application freezing or becoming unresponsive.
- **SC-002**: Creating a new launch entry and assigning it to a folder takes fewer than 3 user interactions (clicks/selections).
- **SC-003**: Copying an image to clipboard completes in under 3 seconds for images up to 50 MB in size on typical hardware, with a visible status confirmation. v1 may briefly block the render loop during decoding/clipboard transfer; background copy/progress UI is deferred unless validation shows typical 50 MB images exceed this target.
- **SC-004**: 100% of launch entries configured before closing the application are restored correctly on next launch (no lost entries).
- **SC-005**: A user who has never used the multi-launch feature can discover and add their first launch entry within 10 seconds of looking at the UI (the "+" button is visually obvious and placed near the existing "Open in feh" button).
- **SC-006**: Clipboard-pasted images preserve the original image's pixel content, native dimensions, and visual quality when pasted into a target application that supports `image/png`; byte-for-byte identity with the original encoded file is not required because clipboard data is standardized as PNG.

## Assumptions

- Feh instances are independent — there is no cross-instance coordination or state sharing. Each feh window operates on its own filelist and its own `--start-at` image.
- The system clipboard on Linux uses the standard X11/Wayland clipboard protocols. The clipboard target format is `image/png` (or a common format supported by clipboard managers). If format conversion is needed, it uses the existing `image` crate (already a project dependency) to decode and re-encode as PNG for clipboard placement.
- Launch entries persist via a simple configuration file stored alongside the application's existing state, not via a database.
- The user's Linux desktop environment has a functioning clipboard (X11 `CLIPBOARD` selection or Wayland equivalent). Clipboard failure due to missing clipboard manager is not the application's responsibility to fix, but a clear error message is provided.
- The "Copy image to clipboard" feature applies to the currently-selected or right-clicked image in rust-feh's image list, not to images opened in external feh windows (modifying feh itself is out of scope per project constitution).
- The existing single "Open in feh" button behavior and code path remain unchanged; the multi-entry launcher is an additive feature that does not regress the current single-launch flow.
- Launch entry labels are plain text strings with no formatting or special characters restrictions beyond what the UI text field enforces.
- Folder assignment for launch entries uses the folder tree already populated by the scanning system; no new scanning infrastructure is required.
