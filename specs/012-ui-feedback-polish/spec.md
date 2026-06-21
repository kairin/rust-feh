# Feature Specification: UI Feedback & Network Scan Polish

**Feature Branch**: `012-ui-feedback-polish`

**Created**: 2026-06-22

**Status**: Complete (T001–T032; manual V1–V5 pending)

**Input**: Post-011 dogfood screenshots — dependencies panel should show clear OK state and collapse when verified; status bar feels static during activity and needs live feedback; speed/timing tips belong in the bottom bar with rotation; NAS/GVFS scan triggers window-manager "not responding"; central panel sections lack visual separation; activity log should detach as its own closable window.

**Parent**: [008-tool-capabilities-panel](../008-tool-capabilities-panel/spec.md), [011-browsing-experience-round](../011-browsing-experience-round/spec.md), [004-scanner-resilience](../004-scanner-resilience/spec.md)

## Clarifications

### Session 2026-06-22

- Q: Should speed/timing tips remain in the right Tools panel? → A: **No** — move to the bottom status bar beside count/status; rotate on a fixed interval.
- Q: How should "all OK" dependencies present? → A: **Collapsed header with success indicator** (e.g. checkmark in title); user may expand manually; auto-collapse again after a successful recheck.
- Q: Detachable activity log close behavior? → A: **Standard window close (X)** dismisses the detached window; user can reattach from main window or reopen via Detach again.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Scan Network Folders Without App Freeze (Priority: P1)

A user opens a folder on a network mount (GVFS/SMB/NFS). The application stays responsive; the window manager does not report the app as "not responding"; scan results appear when complete.

**Why this priority**: Confirmed in dogfood — synchronous or heavy per-file work on NAS paths caused WM freeze dialogs despite background scan from 011.

**Independent Test**: Choose SMB share with 500+ images; interact with menus and resize window during scan; confirm no freeze dialog within 30s.

**Acceptance Scenarios**:

1. **Given** a network-mounted folder path, **When** scan starts, **Then** the user can still use menus, resize the window, and read status within 1 second of starting.
2. **Given** a network-mounted folder path, **When** scan runs, **Then** the activity log records that network-optimized scan behavior is active.
3. **Given** scan completes on a network path, **When** results apply, **Then** image count and inventory match a local scan of the same tree (minus optional format-discovery differences documented in assumptions).
4. **Given** the window manager monitors responsiveness, **When** scanning a large network tree, **Then** no "application not responding" dialog appears during normal use.

---

### User Story 2 - Live Activity Feedback in Status Bar (Priority: P1)

A user starts a folder scan or other long-running activity. The bottom status area visibly indicates work in progress so the interface does not feel frozen or broken.

**Why this priority**: Dogfood — static "Showing X / Y" and status text during scan felt alarming even when background scan was active.

**Independent Test**: Start scan; observe bottom bar for animated or pulsing indicators; confirm feedback stops when scan completes.

**Acceptance Scenarios**:

1. **Given** scan in progress, **When** the user views the bottom status bar, **Then** a distinct in-progress indicator is visible (e.g. animated label, color pulse, or progress glyph).
2. **Given** scan in progress on a network path, **When** viewing status, **Then** messaging clarifies that the UI remains responsive.
3. **Given** scan completes, **When** viewing status, **Then** in-progress animation stops and normal status text is shown.
4. **Given** idle state (no scan), **When** viewing the bottom bar, **Then** indicators are calm/static without unnecessary animation.

---

### User Story 3 - Dependencies OK State and Collapse (Priority: P2)

A user opens the Tools panel. When all required dependencies are installed, the section shows a clear success state, stays collapsed by default, and can be expanded for detail.

**Why this priority**: Reduces noise in the right panel; success should be glanceable without scrolling through green rows.

**Independent Test**: With feh and required tools on PATH, open app — dependencies header shows success and is collapsed; expand manually; click Recheck — stays collapsed if still OK.

**Acceptance Scenarios**:

1. **Given** all required dependencies installed, **When** the Tools panel renders, **Then** the dependencies header displays a success indicator in its title.
2. **Given** all required dependencies installed, **When** the panel first loads, **Then** the dependencies body is collapsed.
3. **Given** collapsed dependencies with all OK, **When** the user clicks the header, **Then** per-dependency detail expands.
4. **Given** missing required dependency, **When** the panel renders, **Then** the header shows a warning indicator and the section is expanded.
5. **Given** user clicks Recheck and all required tools pass, **When** recheck completes, **Then** the section collapses automatically.

---

### User Story 4 - Speed and Timing Tips in Bottom Bar (Priority: P2)

A user wants to understand operation speed tiers without hunting in the right panel. Tips appear in the bottom bar and rotate over time.

**Why this priority**: Dogfood — speed/timing lived in Tools panel but belongs with status/summary information.

**Independent Test**: Load app with tools detected; read bottom bar; wait one rotation interval; confirm tip text changes.

**Acceptance Scenarios**:

1. **Given** tool capabilities are detected, **When** the bottom bar is visible, **Then** a speed/timing tip is shown adjacent to count/status (not in the right Tools panel).
2. **Given** multiple operation tips exist, **When** time advances, **Then** the displayed tip rotates on a fixed interval (default 4 seconds).
3. **Given** rotation is active, **When** viewing the tip area, **Then** a subtle activity glyph indicates rotation (e.g. spinner character).
4. **Given** no capability tips available, **When** the bottom bar renders, **Then** the tip area is empty without layout breakage.

---

### User Story 5 - Separated Panels and Detachable Activity Log (Priority: P2)

A user works in the central area with scan inventory, image list, and activity log. Each functional block is visually distinct; the activity log can open in a separate window and close with the window X.

**Why this priority**: Dogfood — sections blended together; activity log competed for space with the image list.

**Independent Test**: Load folder; confirm bordered groups for inventory, list, and log; detach log; close detached window with X; reattach.

**Acceptance Scenarios**:

1. **Given** scan inventory is shown, **When** viewing the central panel, **Then** inventory, image list, and activity log each have visible boundaries (border or grouped frame).
2. **Given** activity log in main window, **When** user clicks Detach window, **Then** a separate floating window opens with the same log content and actions.
3. **Given** detached activity log window, **When** user clicks the window close control, **Then** the detached window closes and the main window offers reattach or placeholder text.
4. **Given** detached window was closed, **When** user clicks Reattach (or Detach again from collapsed placeholder), **Then** log returns to the main panel layout.
5. **Given** detached window open, **When** new log lines arrive, **Then** both views stay in sync (or detached view updates on next frame).

---

### Edge Cases

- All dependencies OK but user keeps section expanded — must not force-close while user is reading; auto-collapse only on recheck transition or initial load.
- Network path with zero images — responsive scan, empty list, no freeze.
- Detach activity log then quit app — detached window closes with application (no orphan process).
- Very narrow window — bottom bar wraps count/status and tips without clipping Copy buttons.
- Recheck tools while scan running — both activity indicators may show; no conflicting crash.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Scan on network-mounted paths MUST avoid per-file format-discovery work that blocks responsiveness on slow mounts (see assumptions for scope).
- **FR-002**: Network path detection MUST cover GVFS, SMB share URIs, NFS paths, and UNC-style paths visible to the user.
- **FR-003**: When scan runs, the bottom status bar MUST show a visible in-progress state distinct from idle status.
- **FR-004**: In-progress status MUST update continuously during scan (not a single static frame for the entire scan duration).
- **FR-005**: Network scans MUST surface user-facing copy that the UI stays responsive during scan.
- **FR-006**: Dependencies section MUST show success vs warning in the collapsible header title when required tools are OK vs missing.
- **FR-007**: Dependencies section MUST default to collapsed when all required tools are OK at startup.
- **FR-008**: Successful Recheck MUST collapse the dependencies section when all required tools pass.
- **FR-009**: Speed/timing capability tips MUST render in the bottom status bar, not the right Tools panel.
- **FR-010**: Speed/timing tips MUST rotate through available operations on a fixed interval (default 4 seconds).
- **FR-011**: Central panel MUST visually separate scan inventory, image list, and activity log regions.
- **FR-012**: Activity log MUST offer Detach window opening a separate closable window.
- **FR-013**: Closing the detached window MUST return the user to an in-panel reattach affordance.
- **FR-014**: Copy log, Copy status, and Clear logs behaviors from feature 011 MUST remain available in both attached and detached modes.

### Key Entities

- **NetworkPathPolicy**: Classification of folder paths that trigger reduced scan work for responsiveness.
- **StatusBarState**: Idle vs scanning presentation including count label, status text, and optional tip rotation.
- **DependenciesSectionState**: Collapsed/expanded plus header severity (OK vs action needed).
- **ActivityLogView**: Attached (in-panel) or detached (floating window) presentation of the same log buffer.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: On SMB/GVFS dogfood path, user interacts with menus within 1s of scan start; no WM "not responding" dialog in a 60s observation window.
- **SC-002**: During scan, status bar in-progress indicator changes visually at least once per 2 seconds until completion.
- **SC-003**: With all required deps present, dependencies section is collapsed on cold start in 100% of manual quickstart runs.
- **SC-004**: Speed/timing tip text changes at least once within 8 seconds when multiple tips exist.
- **SC-005**: User detaches activity log, closes with X, and reattaches without restart in under 30 seconds (quickstart V-detach).
- **SC-006**: Existing feature 011 tests continue to pass; no regression in feh filelist or copy-log behavior.

## Assumptions

- Feature 011 background scan and activity log copy/select remain in place; this feature polishes presentation and network scan policy only.
- "Network-optimized scan" means skipping optional ImageMagick identify on classified network paths; native extension listing still runs.
- Animation uses lightweight visual feedback (color pulse, spinner glyph, label dots) — no new dependencies or progress-bar infrastructure.
- Detached activity log is an in-app floating panel, not a separate OS process.
- Speed/timing content reuses existing capability metadata from feature 008; only placement and rotation change.
- Default rotation interval is 4 seconds unless quickstart documents an override.

## Traceability

| Source | ID |
|--------|-----|
| Dogfood screenshots 2026-06-22 | deps OK/collapse, status animation, bottom-bar tips, NAS freeze, segment borders, detach log |
| 011-browsing-experience-round | US-2 responsive scan (extends with network policy + visual feedback) |
| 008-tool-capabilities-panel | dependencies panel, operation timings |
| 004-scanner-resilience | partial results on walk errors |
| Constitution §V | UI responsiveness during scans |