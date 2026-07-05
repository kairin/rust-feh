# Feature Specification: Viewer Round-Trip & Staged-Image Actions

**Feature Branch**: `016-feh-viewer-actions`

**Created**: 2026-07-05 (reworked same day — see Clarifications)

**Status**: Draft

**Input**: User description: "rust is the wrapper where the interaction layer lives and feh is really just the engine underneath. rust launches with the image in display; when we need to cycle images, it will launch a separate feh; when I land on the image I want in feh and close the feh, that last image will be loaded in the rust window — and then I act on the image there (save a copy elsewhere, move it, convert, copy path or the image itself) by right-clicking it, without tinkering with settings elsewhere."

## Clarifications

### Session 2026-07-05 (initial)

- Q: Which actions in v1? → A: **Save a copy to another folder; move to another folder; resize copy; convert format; copy path to clipboard; copy image to clipboard.** Set-as-wallpaper not selected. Maintainer: "I always find that I need to save a copy of the same image elsewhere, or copy and paste the image I am viewing."
- Q: On-image action labels? → A: **No** — "the numbering on the side of the image distracts me." (Verified: that numbering comes from the maintainer's personal feh theme; profile isolation removes it in launched viewers.)
- Q: Personal feh configuration? → A: **Fully isolated** — launched viewers never read, modify, or shadow it; stock feh navigation (incl. scroll = previous/next) preserved.

### Session 2026-07-05 (rework — intended behavior changed by maintainer)

- Q: Where does interaction live? → A: **In the rust-feh window.** Maintainer: the original feh-side shortcut design was "the problematic part — I thought rust is the wrapper where the interaction layer lives and feh is really just the engine underneath." The delivery surface moves from feh key hooks to a rust-feh **staged-image pane with a right-click context menu**. feh-side action shortcuts are dropped from scope.
- Q: How do viewing and cycling relate? → A: **Round-trip flow** (maintainer design): rust-feh displays the current image; cycling happens in a launched feh window; on feh close, the image the user landed on is automatically selected and displayed in rust-feh. Feasibility of the close-handoff verified live on the target machine 2026-07-05.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - The image I selected is displayed in rust-feh, and I act on it there (Priority: P1)

When I select an image in the rust-feh list, the image itself is displayed in
the rust-feh window, and right-clicking it gives me the actions I need: save a
copy to a folder I choose, move it to a folder I choose, create a resized
copy, convert its format, copy its path, or copy the image itself to the
clipboard.

**Why this priority**: This is the interaction layer the product was expected
to be — triage decisions happen while looking at the image, in the window that
owns the interaction.

**Independent Test**: Select images in the list, confirm each is displayed;
right-click and run every action; verify outcomes on disk/clipboard. No feh
involvement required.

**Acceptance Scenarios**:

1. **Given** an image is selected in the list, **When** the selection settles,
   **Then** the image is displayed in the rust-feh window (fit-to-pane), and
   list browsing performance is unchanged.
2. **Given** the displayed image, **When** the user right-clicks it, **Then** a
   context menu offers exactly: save a copy…, move to…, resize copy, convert
   format, copy path, copy image.
3. **Given** "save a copy…" or "move to…" is chosen, **When** the native
   folder chooser is confirmed, **Then** the file appears at the destination
   (collision-safe, never overwriting; move is loss-proof), and the action and
   outcome are recorded in the activity log.
4. **Given** "copy path" or "copy image" is chosen, **When** the menu closes,
   **Then** the path (or image data) is pasteable in another application.
5. **Given** any action fails (permissions, disk full, unsupported format),
   **When** the failure occurs, **Then** a human-readable error names the
   image and cause, within 2 seconds.

---

### User Story 2 - Cycle in feh, land, close — rust-feh picks up where I stopped (Priority: P1)

When I want to flip through many images quickly, I open the feh viewer from
rust-feh and browse with feh's normal keys and scroll wheel. When I land on
the image I want and close feh, that exact image is now selected and displayed
in the rust-feh window, ready for right-click actions.

**Why this priority**: This is the round-trip that makes the two windows one
workflow: feh is the fast cycling engine, rust-feh is where decisions land.

**Independent Test**: Open feh from rust-feh on a folder, navigate several
images, quit feh (q / Escape / window close); verify rust-feh's selection and
displayed image match the last image shown in feh.

**Acceptance Scenarios**:

1. **Given** a viewer launched from rust-feh, **When** the user navigates to
   any image and closes feh by any normal means, **Then** within one second
   rust-feh selects and displays that image, and the list scrolls to it.
2. **Given** the user closes feh without navigating, **Then** rust-feh keeps
   showing the image it launched with (no spurious change).
3. **Given** the landed image no longer passes the active filter, **When** the
   handoff occurs, **Then** rust-feh still stages the image and makes the
   situation visible (e.g. clears or annotates the filter) rather than
   silently ignoring the handoff.
4. **Given** multiple rust-feh-launched viewers are open, **When** one closes,
   **Then** the handoff uses that viewer's last image (most recent close wins),
   and this is recorded in the activity log.

---

### User Story 3 - Launched viewers are clean and isolated (Priority: P2)

Viewers launched from rust-feh run with rust-feh-managed settings: stock feh
navigation, no on-image text or numbering, and no interaction with my personal
feh configuration in either direction.

**Why this priority**: The maintainer's personal feh theme currently draws
filename/action overlays that they find distracting; isolation removes them in
launched viewers while leaving manually-started feh exactly as the user built
it.

**Independent Test**: Byte-compare the personal feh config before/after a full
session; verify launched viewers show no overlay text and stock navigation;
verify manually-started feh still shows the personal theme.

**Acceptance Scenarios**:

1. **Given** a launched viewer, **When** an image is displayed, **Then** no
   filename, action list, or other text overlays appear on the image.
2. **Given** the personal feh configuration, **When** any number of launched
   viewer sessions run, **Then** it is byte-identical before and after, and
   manually-started feh behavior is unchanged.

---

### Edge Cases

- Name collision at destination (copy/move/derive): distinct suffixed name;
  nothing silently overwritten.
- Moving the currently displayed image: the stage advances predictably (next
  image or empty state), never a stale/broken frame.
- Destination unwritable / disk full / disconnected share: per-action native
  error; originals never lost or half-written (copy-verify-remove on move).
- Filenames with spaces, quotes, and non-ASCII work in every action and in the
  close-handoff.
- feh killed abnormally (crash, kill signal): rust-feh treats it as a close —
  stages the last known image if one was recorded, otherwise keeps state.
- Very large image selected: the stage shows a scaled rendition promptly and
  must not block the UI or balloon memory (lazy, bounded decode).
- Unsupported/exotic format selected (not decodable in-process): the stage
  shows a clear placeholder + reason, actions that can't apply are disabled;
  the feh round-trip still works for such files.
- Rapid selection changes while a decode is in flight: the stage always ends
  showing the latest selection.
- Cancelling the folder chooser aborts with no side effects; double-invoking
  an action while its dialog is open is refused predictably.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Selecting an image in the list MUST display it in a rust-feh
  stage pane, scaled to fit, without degrading list/browse performance
  (lazy, bounded, off-thread decode; latest selection wins).
- **FR-002**: Right-clicking the staged image MUST open a context menu with
  exactly these v1 actions: save a copy to folder, move to folder, resize
  copy, convert format, copy path to clipboard, copy image to clipboard.
- **FR-003**: Destination-taking actions MUST use a native folder chooser and
  remember the last-used destination as the starting point (persisted).
- **FR-004**: All file-producing actions MUST be collision-safe (never
  overwrite; distinct suffixed name, reported to the user).
- **FR-005**: Move MUST be loss-proof: original removed only after the
  destination copy verifiably exists, including across filesystems.
- **FR-006**: Resize and convert MUST produce results consistent with the
  existing Image Tools rules, and MUST work from the context menu without
  visiting the Image Tools panel.
- **FR-007**: Closing a rust-feh-launched viewer MUST, within one second,
  select and stage the image that viewer last displayed (any close path:
  quit key, window close, abnormal termination after at least one image was
  recorded). If the user never navigated, the selection is unchanged.
- **FR-008**: The close-handoff MUST be correct for arbitrary valid filenames
  and MUST record the round-trip in the activity log (launched-with,
  landed-on).
- **FR-009**: Launched viewers MUST use rust-feh-managed viewer settings,
  fully isolated from the personal feh configuration (never read, modified,
  or shadowed for manual sessions), with stock navigation defaults and no
  on-image text overlays.
- **FR-010**: Action failures MUST surface as native, human-readable errors
  naming the image and cause; every action and every round-trip MUST be
  recorded in the activity log.
- **FR-011**: The feature MUST NOT add network access or new external tool
  dependencies; in-process image handling uses the existing always-available
  processor.
- **FR-012**: When the staged image cannot be decoded in-process, the stage
  MUST show an explicit placeholder with the reason; inapplicable context
  actions are disabled; path/clipboard-path and move/copy actions remain
  available.

### Key Entities

- **Staged image**: the image currently displayed in the rust-feh window —
  always the current selection; the target of all context-menu actions.
- **Viewer round-trip**: a launched feh session tied to its originating
  rust-feh state — launched-with image, per-image trail, landed-on image at
  close.
- **Context action**: a named operation on the staged image (save-copy, move,
  resize copy, convert, copy path, copy image) with a per-run outcome record.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: From noticing an image in the stage, saving a copy to a chosen
  folder takes at most 3 interactions (right-click → save a copy… → choose),
  with zero visits to other panels.
- **SC-002**: The close-handoff stages the correct image in 100% of trials
  across a 50-image navigation fixture, within 1 second of feh closing.
- **SC-003**: 100% of collision cases produce a suffixed file; 0 overwrites;
  0 files lost across a move fault-injection fixture.
- **SC-004**: 100% of actions and handoffs work on an adversarial-filename
  fixture (spaces, quotes, non-ASCII).
- **SC-005**: With a 10,000-image folder loaded, enabling the stage changes
  scan/filter timings by no more than 10% vs. the pre-feature baseline, and
  selection-to-displayed latency is under 500 ms for typical photos.
- **SC-006**: 0 modifications to the personal feh configuration across a full
  session; 0 text overlays in launched viewers.
- **SC-007**: On action failure, a human-readable error is visible within 2
  seconds and the activity log contains the corresponding record.

## Assumptions

- feh remains the cycling/slideshow engine and the wallpaper mechanism; the
  stage displays the single current image and is NOT a slideshow, zoom
  workbench, or editor (no reimplementation of feh navigation features).
  A positioning-document clarification recording this boundary ("stage pane ≠
  viewer replacement; feh remains the viewer engine") ships with the plan —
  wording change only, no principle redefinition.
- The launched viewer carries the current filtered list (shipped behavior);
  the round-trip adds state handoff, not navigation changes.
- The per-image trail mechanism relies on the viewer's own per-image hook and
  a rust-feh-owned handoff location; verified working on the target machine
  2026-07-05 (silent, no overlays, correct path emitted per displayed image).
- "Copy image to clipboard" targets standard desktop clipboard image formats;
  best-effort for exotic formats with a clear fallback message (copy as path).
- The last-used destination persists across sessions alongside existing
  persisted preferences.
- Set-as-wallpaper from the context menu is out of scope for v1 (not
  selected); trivially addable later.
- A security review of the implementation diff is mandatory before merge
  (engagement guardrail): subprocess argument construction, handoff-file
  trust (paths read back from it are validated against the launched list),
  and untrusted path handling get explicit attention.
