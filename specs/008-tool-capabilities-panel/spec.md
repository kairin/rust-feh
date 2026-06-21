# Feature Specification: Tool Capabilities Panel

**Feature Branch**: `008-tool-capabilities-panel`

**Created**: 2026-06-22

**Status**: Implemented

**Input**: Retroactive spec for the shipped **Tools & capabilities** side panel — dependency status, install commands, operation speed tiers, and format routing (which tool handles scan/view/resize). Formalizes positioning claims in `docs/POSITIONING.md` and runtime transparency promised vs nfeh.

**Parent**: [001-persistent-ui-virtual-browsing](../001-persistent-ui-virtual-browsing/spec.md) (thin-wrapper, feh-centric UX)  
**Related**: [docs/POSITIONING.md](../../docs/POSITIONING.md), [docs/NFEH-COMPARISON-AND-MIGRATION.md](../../docs/NFEH-COMPARISON-AND-MIGRATION.md)

## User Scenarios & Testing *(mandatory)*

### User Story 1 - See What Tools Are Installed (Priority: P1)

A user opens rust-feh on a fresh Linux install. They need to know whether feh and optional ImageMagick are available before trying to view images or set wallpaper. The capabilities panel shows each dependency as installed or missing, with a one-line role description.

**Why this priority**: Core positioning — honest dependency transparency; replaces scattered status-bar hints.

**Independent Test**: Launch with feh present/absent and with ImageMagick present/absent; panel reflects each combination without opening a terminal.

**Acceptance Scenarios**:

1. **Given** rust-feh starts, **When** the user views the capabilities panel, **Then** **feh** appears as a **required** dependency with installed or not-installed state.
2. **Given** rust-feh starts, **When** the user views the panel, **Then** **ImageMagick** appears as an **optional** dependency with installed or not-installed state.
3. **Given** a dependency is installed, **When** displayed, **Then** the resolved command name on PATH is shown (e.g. `feh`, `magick`, or `convert`).
4. **Given** a dependency is missing, **When** displayed, **Then** a Linux package install command is shown (`sudo apt install feh` or `sudo apt install imagemagick`).

---

### User Story 2 - Install Missing Tools Quickly (Priority: P1)

A user sees feh is missing. They want the exact install command without searching the README.

**Why this priority**: Reduces "app appears broken" support friction; pairs with FR-008a from feature 001.

**Independent Test**: With feh absent, copy install command from panel; paste in terminal (manual); recheck updates state.

**Acceptance Scenarios**:

1. **Given** feh is not installed, **When** the user activates **Copy** beside the install command, **Then** the install command is placed on the system clipboard.
2. **Given** copy succeeds, **When** the action completes, **Then** the primary status line confirms which dependency was copied.
3. **Given** required dependencies are missing, **When** the panel is visible, **Then** a prominent reminder directs the user to install and use **Recheck tools**.

---

### User Story 3 - Understand Speed and Format Routing (Priority: P1)

A user with a mixed folder (jpg, heic, svg) wants to know which operations are fast, which tool opens which format, and what resize support exists — without reading source code.

**Why this priority**: Differentiates rust-feh from nfeh; supports "feh orchestrator" positioning.

**Independent Test**: Panel shows operation timing table and format groups with Scan / View / Resize handlers; content updates when optional ImageMagick is present vs absent.

**Acceptance Scenarios**:

1. **Given** the panel is open, **When** the user reads **Speed / timing**, **Then** at least these operations are listed: browse/filter/sort, open/slideshow, wallpaper, quick resize, exotic format view — each with a speed tier (Instant / Fast / Medium / Slower) and handling component.
2. **Given** the panel is open, **When** the user reads **Format discovery**, **Then** extension groups show which component handles **scan**, **view**, and **resize** for that group.
3. **Given** ImageMagick is absent, **When** format routing is shown, **Then** exotic-format notes indicate reduced capability (no false claim that convert is already running).
4. **Given** feh is absent, **When** operation timings are shown, **Then** view/slideshow notes indicate feh is required to enable those operations.

---

### User Story 4 - Panel Uses Empty Layout Space (Priority: P2)

A user resizes the window wider than the image list needs. The capabilities panel occupies the side area so the layout does not leave a large dead zone.

**Why this priority**: UX motivation for the panel placement; keeps central list focused on browsing.

**Independent Test**: Wide window shows resizable right panel; narrow window still allows scrolling panel content.

**Acceptance Scenarios**:

1. **Given** the main window is open, **When** layout renders, **Then** the capabilities panel appears in a dedicated side region separate from the virtualized image list.
2. **Given** panel content exceeds viewport height, **When** the user scrolls the panel, **Then** all sections remain reachable.
3. **Given** the user drags the panel edge, **When** resizing, **Then** panel width adjusts within documented minimum width.

---

### Edge Cases

- Both `magick` and `convert` on PATH — show whichever is resolved first per detection rules; mark ImageMagick installed.
- Neither feh nor ImageMagick — panel shows both missing; required warning for feh only.
- User copies install command without sudo rights — copy still succeeds; install is user's responsibility outside the app.
- Format routing describes capabilities **beyond** current scanner extensions — notes MUST clarify when scan does not yet list exotic types (honest routing vs scan scope).
- Panel visible while folder not loaded — dependency and routing sections still useful; no dependency on loaded images.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The application MUST provide a persistent **Tools & capabilities** panel visible during normal use.
- **FR-002**: The panel MUST list **feh** as a required external tool with installed/not-installed state and role (view, slideshow, wallpaper).
- **FR-003**: The panel MUST list **ImageMagick** as an optional external tool with installed/not-installed state and role (exotic formats, convert fallback).
- **FR-004**: For each missing dependency, the panel MUST show a **Linux apt install command** appropriate to that tool.
- **FR-005**: The user MUST be able to **copy** each install command to the clipboard from the panel.
- **FR-006**: The panel MUST include a **Speed / timing** section mapping core operations to speed tier and handling component.
- **FR-007**: The panel MUST include a **Format discovery** section grouping extensions with Scan, View, and Resize handlers.
- **FR-008**: Format and timing content MUST reflect whether feh and ImageMagick are currently detected (dynamic, not static brochure text).
- **FR-009**: When a required dependency is missing, the panel MUST show an explicit install-and-recheck reminder.
- **FR-010**: Capability logic (dependency list, timings, format routes) MUST be testable without the GUI (constitution §III).
- **FR-011**: The bottom status region MUST NOT duplicate full dependency prose; a short pointer to the panel is sufficient.
- **FR-012**: Panel MUST be scrollable when content exceeds available height.
- **FR-013**: Panel MUST be user-resizable horizontally within a reasonable minimum width.

### Key Entities

- **ToolCapabilities**: Snapshot of detected tools (feh available, ImageMagick available, resolved binary path).
- **DependencyStatus**: Name, required/optional kind, role, install command, installed flag, resolved binary.
- **OperationTiming**: Operation label, handler component, speed tier, explanatory note.
- **FormatRoute**: Extension group, scan/view/resize handlers, view speed tier, honesty note (including scan-scope limitations).

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A new user can determine whether feh is installed within 5 seconds of opening the panel.
- **SC-002**: A user can copy a missing dependency install command in one click from the panel.
- **SC-003**: Unit tests cover dependency list shape, operation timings presence, and format route coverage without launching the GUI.
- **SC-004**: When ImageMagick install state changes and the user rechecks tools, format routing content updates in the same session without restart.
- **SC-005**: Positioning doc claims about "Tools & capabilities panel" remain accurate — panel shows deps, speed tiers, and format routing as documented.

## Assumptions

- **Retroactive**: Panel and `tool_caps` module largely shipped — implement phase is **verify + gap-fill + gap-audit**.
- Linux apt install strings are the default; other distros are out of scope for install copy text.
- ImageMagick **subprocess convert** is out of scope for this feature — panel documents routing; execution is feature 010+.
- Scanner extension expansion for exotic formats is out of scope — format section may describe planned routing with honest scan notes.
- Recheck behavior is owned by feature **009**; this feature requires panel to **display** updated state after recheck.
- Depends on feature 001 layout (persistent panels, central list).

## Traceability

| Source | ID |
|--------|-----|
| docs/POSITIONING.md | Messaging emphasis, claims today/avoid |
| docs/NFEH-COMPARISON-AND-MIGRATION.md | Tool comparison matrix |
| user session | Capabilities panel implementation |
| feature 001 | FR-008a status messaging (complementary, not replaced) |