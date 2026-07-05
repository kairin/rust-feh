# Feature Specification: In-Viewer Image Actions (feh)

**Feature Branch**: `016-feh-viewer-actions`

**Created**: 2026-07-05

**Status**: Draft

**Input**: User description: "When I'm viewing an image in a feh window launched from rust-feh, I want to act on the image I'm looking at — save a copy of it somewhere else, move it to another folder, make a resized copy, convert its format, copy its path or the image itself to the clipboard — without going back to the rust-feh window to tinker with settings. I don't want distracting labels drawn over the image. Keep feh's normal navigation as it is."

## Clarifications

### Session 2026-07-05

- Q: What should the mouse scroll wheel do in rust-feh-launched viewers? → A: **Keep feh default** (scroll = previous/next image). No navigation rebinding.
- Q: Which actions in v1? → A: **Save a copy to another folder; move to another folder; resize copy; convert format; copy path to clipboard; copy image to clipboard.** Set-as-wallpaper from the viewer was NOT selected. Maintainer emphasized: "I always find that I need to save a copy of the same image elsewhere, or copy and paste the image I am viewing."
- Q: On-screen action list inside the viewer? → A: **No** — "the numbering on the side of the image distracts me." Shortcut discoverability lives in the rust-feh UI instead.
- Q: Relationship to the user's personal feh configuration? → A: **Fully isolated** — rust-feh-launched viewers use rust-feh-managed viewer settings only; the personal feh config applies only when the user runs feh directly, and is never read or modified by rust-feh.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Act on the image I'm viewing (Priority: P1)

While viewing an image in a feh window that rust-feh launched, I press a
shortcut to act on exactly that image: save a copy to a folder I choose, move
it to a folder I choose, create a resized copy, convert its format, or copy
its path or the image itself to the clipboard — without switching back to the
rust-feh window.

**Why this priority**: This is the core workflow pain — image triage decisions
happen while looking at the image, and today every action requires returning
to the browser window and re-finding the image.

**Independent Test**: Launch a viewer from rust-feh on a folder of images,
press each action shortcut on a chosen image, and verify the outcome on disk /
clipboard — no interaction with the rust-feh window required after launch.

**Acceptance Scenarios**:

1. **Given** a viewer launched from rust-feh showing image X, **When** the user
   triggers "save a copy", **Then** a native folder chooser appears, and on
   confirmation a copy of X exists in the chosen folder with its original name
   (auto-suffixed if a name collision exists — never overwriting).
2. **Given** a viewer showing image X, **When** the user triggers "move to
   folder" and picks a destination, **Then** X is moved there (collision-safe),
   the viewer advances to the next image, and the rust-feh list no longer shows
   X after its next refresh.
3. **Given** a viewer showing image X, **When** the user triggers "resize copy"
   or "convert format", **Then** the derived file is produced using the same
   rules as the rust-feh Image Tools panel, and the original is untouched.
4. **Given** a viewer showing image X, **When** the user triggers "copy path"
   or "copy image", **Then** the path (or image data) is on the system
   clipboard and pasteable into another application.
5. **Given** any action fails (permission denied, disk full, unsupported
   format), **When** the failure occurs, **Then** the user sees a native error
   message naming the image and the reason — without opening a terminal.

---

### User Story 2 - Viewer settings isolated from my personal feh (Priority: P2)

As a feh user with my own customized configuration, I want rust-feh-launched
viewers to run with rust-feh's own viewer settings, fully separate from my
personal configuration, so the two never interfere.

**Why this priority**: The maintainer keeps a personal feh setup (custom
actions for external editors); rust-feh must neither depend on it nor disturb
it, and rust-feh's shortcuts must not leak into manually-started feh sessions.

**Independent Test**: Compare the personal feh config before/after using
rust-feh viewers (byte-identical); verify rust-feh action shortcuts work in
launched viewers and do NOT work in a manually started feh.

**Acceptance Scenarios**:

1. **Given** a personal feh configuration exists, **When** rust-feh launches
   viewers and actions are used, **Then** the personal configuration files are
   never read, modified, or shadowed for manually-started feh sessions.
2. **Given** a rust-feh-launched viewer, **When** the user navigates with the
   usual feh keys and scroll wheel, **Then** behavior matches stock feh
   defaults (navigation is NOT rebound), plus the rust-feh action shortcuts.
3. **Given** no personal feh configuration exists at all, **When** rust-feh
   launches a viewer, **Then** everything works identically.

---

### User Story 3 - I can discover the shortcuts without on-image clutter (Priority: P3)

I want to find out which shortcut does what from the rust-feh interface — not
from labels drawn over the image I'm viewing.

**Why this priority**: Requested explicitly (on-image action labels are
distracting); without some discoverability the feature is invisible.

**Independent Test**: Locate the complete shortcut reference in the rust-feh
UI; confirm the viewer image area contains no action labels or numbering.

**Acceptance Scenarios**:

1. **Given** rust-feh is open, **When** the user looks at the viewer-related
   UI (e.g. near the open-in-viewer controls), **Then** a concise list of the
   in-viewer action shortcuts is visible or one interaction away.
2. **Given** a launched viewer, **When** an image is displayed, **Then** no
   action list, numbering, or labels are drawn over the image.
3. **Given** an action ran from the viewer, **When** the user returns to
   rust-feh, **Then** the activity log records what ran, on which image, and
   the outcome.

---

### Edge Cases

- Name collision at the destination (copy/move/derive): the new file gets a
  distinct suffixed name; nothing is ever silently overwritten.
- Moving or converting the image currently displayed: the viewer must not
  crash or show a stale/broken frame; on move, it advances past the removed
  file.
- Destination unwritable, disk full, or on a disconnected network share:
  per-action native error message; the original file is never lost or
  half-written (copy-then-verify before any removal on move).
- Filenames with spaces, quotes, and non-ASCII characters work in every action.
- Rapid repeated shortcut presses do not queue duplicate dialogs or corrupt
  the destination (second invocation while one is pending is refused or
  ignored, predictably).
- Two viewers open on different folders: each action applies to the image in
  the viewer where the shortcut was pressed.
- Cancelling the folder chooser aborts the action with no side effects.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Viewers launched by rust-feh MUST expose these actions on the
  image currently displayed: save-copy-to-folder, move-to-folder, resize copy,
  convert format, copy path to clipboard, copy image to clipboard.
- **FR-002**: Destination-taking actions (save-copy, move) MUST present a
  native folder chooser at action time and remember the last-used destination
  as the chooser's starting point.
- **FR-003**: All file-producing actions MUST be collision-safe: existing
  files are never overwritten; collisions produce a distinct suffixed name
  that is reported to the user.
- **FR-004**: Move MUST be loss-proof: the original is removed only after the
  destination copy verifiably exists (byte-complete), including across
  filesystems.
- **FR-005**: Resize and convert MUST produce results consistent with the
  existing Image Tools rules (same quality/naming conventions), without
  launching or requiring the rust-feh window.
- **FR-006**: Action failures MUST surface as a native, human-readable error
  naming the image and cause; every action (success or failure) MUST also be
  recorded in the rust-feh activity log with image path and outcome.
- **FR-007**: The viewer image area MUST remain free of action labels,
  numbering, or overlays introduced by this feature.
- **FR-008**: rust-feh-launched viewers MUST use rust-feh-managed viewer
  settings, fully isolated from the user's personal feh configuration: the
  personal configuration is never read, modified, or shadowed for sessions the
  user starts manually, and stock navigation defaults (including scroll-wheel
  previous/next) are preserved in launched viewers.
- **FR-009**: Every action MUST handle arbitrary valid filenames (spaces,
  quotes, non-ASCII) without misinterpretation.
- **FR-010**: The rust-feh UI MUST present a concise reference of the
  in-viewer action shortcuts, discoverable in at most one interaction from the
  viewer-launch controls.
- **FR-011**: Actions MUST operate without network access and MUST NOT add
  new external tool dependencies beyond what rust-feh already uses.

### Key Entities

- **In-viewer action**: a named operation (save-copy, move, resize copy,
  convert, copy path, copy image) bound to a shortcut inside launched viewers,
  targeting the currently displayed image.
- **Managed viewer settings**: the rust-feh-owned configuration under which
  launched viewers run — isolated from, and never touching, the user's
  personal feh configuration.
- **Action outcome record**: activity-log entry per action run — image path,
  action, destination (if any), success/failure and reason.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: From a launched viewer, saving a copy of the current image to a
  chosen folder takes at most 3 user interactions (shortcut → pick folder →
  confirm), with zero interactions in the rust-feh window.
- **SC-002**: 100% of collision cases produce a suffixed file; 0 overwrites
  across an adversarial fixture (same names, repeated runs).
- **SC-003**: 100% of actions work on an adversarial-filename fixture (spaces,
  quotes, non-ASCII).
- **SC-004**: 0 modifications to the personal feh configuration across a full
  action-suite session (byte-identical before/after), and 0 rust-feh shortcuts
  active in a manually-started feh.
- **SC-005**: 0 pixels of action labels/overlays in the viewer image area.
- **SC-006**: On action failure, a human-readable error is visible within 2
  seconds, and the activity log contains the corresponding record.
- **SC-007**: Move operations lose 0 files across a fault-injection fixture
  (unwritable destination, interrupted copy, cross-filesystem move).

## Assumptions

- Viewers are feh instances launched by rust-feh (the existing delegation
  model); manually-started feh sessions are explicitly out of scope.
- The viewer already receives the full browsing context at launch (the
  current filtered list), so "browse the rest of the folder" is covered by
  shipped behavior; this feature adds actions, not navigation changes.
- Shortcut assignments (which key triggers which action) are decided at
  planning; they avoid feh's stock navigation keys and are shown in the
  rust-feh shortcut reference (FR-010).
- "Copy image to clipboard" targets standard desktop clipboard image formats;
  best-effort fidelity for exotic formats (fallback: copy as path with a clear
  message).
- The last-used destination persists across sessions alongside existing
  persisted preferences.
- Set-as-wallpaper from the viewer is out of scope for v1 (not selected);
  trivially addable later as another action.
- A security review of the implementation diff is mandatory before merge
  (engagement guardrail): shell-facing argument construction and untrusted
  path handling get explicit attention.
