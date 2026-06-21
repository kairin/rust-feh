# Feature Specification: Window & Viewer Stability

**Feature Branch**: `006-window-viewer-stability`

**Created**: 2026-06-21

**Status**: Draft

**Input**: Dogfood gaps — (1) rust-feh window jumped/shrank when loading folders; huge empty space in list; (2) no user control over window size; (3) feh viewer resizing to tiny images (5px icons → "vanished" window). Post-001 code changes exist; this spec formalizes and completes the feature (including persistence if missing).

**Parent**: [001-persistent-ui-virtual-browsing](../001-persistent-ui-virtual-browsing/spec.md) (layout US1)

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Stable Application Window (Priority: P1)

A user opens rust-feh, loads a folder, and resizes the window. The layout does not jump unexpectedly; the image list fills available space instead of leaving a large blank area.

**Why this priority**: Window chaos undermines trust in the whole tool — reported as more annoying than missing sort.

**Independent Test**: Load folder before/after; toolbar height stable; list area uses remaining central panel height.

**Acceptance Scenarios**:

1. **Given** no folder loaded, **When** user views toolbar, **Then** filter/sort/button rows are visible but disabled (no layout jump when folder loads).
2. **Given** a loaded folder with few images, **When** viewing central panel, **Then** list area expands to fill space above debug log header (no tiny list + huge empty gap).
3. **Given** default launch, **When** window opens, **Then** size is at least the Default preset (960×720) and not smaller than absolute floor (640×480).

---

### User Story 2 - User-Controlled Window Size (Priority: P1)

A user wants a larger or smaller window, or to lock size while working.

**Why this priority**: Replacing hard-coded min=max lock from interim fix.

**Independent Test**: View → Window size presets and Resizable toggle behave as documented.

**Acceptance Scenarios**:

1. **Given** View menu, **When** user picks Compact / Default / Large preset, **Then** window resizes to that preset immediately.
2. **Given** Resizable enabled, **When** user drags edges, **Then** window resizes freely down to floor 640×480.
3. **Given** Resizable disabled, **When** toggled off, **Then** current size is locked (min=max) until re-enabled.
4. **Given** preset change while locked, **When** applied, **Then** window moves to preset and stays locked if resizable off.

---

### User Story 3 - Predictable feh Viewer Window (Priority: P1)

A user opens a 5×5 pixel image in feh. The feh window stays a sensible size; the image is visible (upscaled), not a invisible speck or microscopic window.

**Why this priority**: Classic feh `--auto-zoom` failure mode; users blamed rust-feh for "window disappeared."

**Independent Test**: Open tiny PNG via Open in feh; feh window ≥ documented geometry; image visible.

**Acceptance Scenarios**:

1. **Given** any image including very small dimensions, **When** Open in feh runs, **Then** feh is launched with fixed geometry and zoom policy documented in quickstart (no per-image window shrink).
2. **Given** a large image, **When** opened in feh, **Then** image scales down to fit fixed window.
3. **Given** feh launch, **When** inspecting debug log, **Then** command line documents geometry and zoom flags for support.

---

### User Story 4 - Remember Window Preferences (Priority: P2)

A user sets window preset and resizable preference; next launch restores them.

**Why this priority**: Area 8 persistence deferred in 001; natural completion of window settings.

**Independent Test**: Change settings, restart app, preferences restored.

**Acceptance Scenarios**:

1. **Given** user changed preset to Large and disabled resizable, **When** app restarts, **Then** same preset and lock state apply.
2. **Given** first run, **When** no config file, **Then** Default preset and resizable=true apply.

---

### Edge Cases

- Tiling WM forces geometry — document that WM may override; rust-feh re-applies min floor.
- feh not installed — viewer story N/A; no crash.
- Multi-monitor — preset sizes are logical pixels; no position memory in P1.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Central panel list MUST use available vertical space (`auto_shrink` disabled for list scroll area).
- **FR-002**: Toolbar layout MUST NOT change height when folder loads (disabled vs enabled controls).
- **FR-003**: Application MUST enforce absolute minimum window size 640×480 at all times.
- **FR-004**: View menu MUST offer window size presets: Compact (720×540), Default (960×720), Large (1280×960).
- **FR-005**: View menu MUST offer **Resizable window** toggle controlling drag resize and lock behavior.
- **FR-006**: feh launch MUST use fixed viewer geometry and zoom policy preventing window shrink-to-image (document exact flags in plan).
- **FR-007**: feh launch MUST upscale tiny images within fixed window (user-visible, not 1:1 pixel).
- **FR-008**: Window preference persistence MUST be implemented in P2 (config file path defined in plan); until then in-memory session only is acceptable with FR-004–FR-005 still required.

### Key Entities

- **WindowSizePreset**: Compact | Default | Large.
- **WindowPolicy**: preset, resizable, clamped dimensions.
- **FehLaunchProfile**: geometry string, zoom mode, scale-down flag.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Loading a folder does not change window outer dimensions unless user action caused it.
- **SC-002**: With 3 images loaded, list scroll area height is ≥50% of central panel (no dominant empty gap).
- **SC-003**: Opening 5×5 test image in feh produces viewer window ≥640×480 effective visible area.
- **SC-004**: User can switch presets and lock size in under 10 seconds via View menu.
- **SC-005**: (P2) Preferences survive restart on same machine.

## Assumptions

- Retroactive: FR-001–FR-007 partially implemented — plan starts with code audit.
- feh flags are subprocess contract; feh version is distro `apt install feh`.
- Persistence format deferred to plan (JSON in XDG config dir assumed).
- Does not cover feh keyboard shortcuts inside viewer.

## Traceability

| Source | ID |
|--------|-----|
| user dogfood | window jump, empty space, size setting |
| user dogfeed | 5px image / feh vanish |
| lessons-learned.md | Shadow work §5 |
| feature 001 | layout US1 (extended) |