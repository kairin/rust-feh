# Feature Specification: External Tool Runtime

**Feature Branch**: `009-external-tool-runtime`

**Created**: 2026-06-22

**Status**: Implemented

**Input**: Unified runtime detection and re-check for external tools on PATH — **feh** (required) and **ImageMagick** (optional). Merges outstanding issue **A5** and feature **002** scope with the shipped **Recheck tools on PATH** control in the capabilities panel. Single source of truth for startup detection, on-demand recheck, and stale-state recovery after install/remove.

**Parent**: [001-persistent-ui-virtual-browsing](../001-persistent-ui-virtual-browsing/spec.md) (FR-008a, FR-008b)  
**Supersedes**: [002-feh-runtime-detection](../002-feh-runtime-detection/spec.md) for implementation (002 remains historical traceability)  
**Related**: [008-tool-capabilities-panel](../008-tool-capabilities-panel/spec.md)

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Install feh Mid-Session (Priority: P1)

A user launches rust-feh before installing feh. Capabilities panel and feh actions show unavailable. They run `sudo apt install feh` while the app stays open, then recheck tools. They open an image without restarting.

**Why this priority**: Same as former feature 002 — common Linux workflow; panel recheck already exists but feh button enablement and status must stay in sync.

**Independent Test**: Start without feh, install feh, recheck, confirm Open in feh works and panel + buttons update.

**Acceptance Scenarios**:

1. **Given** rust-feh started with feh absent, **When** the user rechecks tools and feh is now on PATH, **Then** feh-dependent controls become enabled per feature 001 FR-008a.
2. **Given** feh was missing and is now detected, **When** recheck succeeds, **Then** the capabilities panel shows feh installed and operation timings reflect feh as the view handler.
3. **Given** feh is still absent after recheck, **When** recheck completes, **Then** status and panel remain in degraded state and no feh process is spawned.

---

### User Story 2 - Install ImageMagick Mid-Session (Priority: P1)

A user works without ImageMagick, then installs it. They recheck and the capabilities panel updates format routing for exotic types without restart.

**Why this priority**: Panel already detects magick; runtime state must drive FR-008 of feature 008 consistently.

**Independent Test**: Start without magick, install imagemagick, recheck, confirm panel shows magick installed and exotic format notes change.

**Acceptance Scenarios**:

1. **Given** ImageMagick was absent at startup, **When** user installs it and rechecks, **Then** `magick` or `convert` is detected and optional dependency shows installed.
2. **Given** only `convert` is on PATH (legacy ImageMagick), **When** recheck runs, **Then** ImageMagick is treated as available.
3. **Given** ImageMagick is removed from PATH, **When** user rechecks, **Then** optional dependency shows not installed and format routing reflects reduced exotic support.

---

### User Story 3 - Recover After feh Spawn Failure (Priority: P2)

A user had feh at startup; feh is later removed or PATH changes. Open in feh fails. The UI must not keep showing feh as available.

**Why this priority**: Feature 002 US2 — prevents stale enable state.

**Independent Test**: Simulate missing feh at spawn; confirm `feh_available` flips false and messaging matches FR-008a.

**Acceptance Scenarios**:

1. **Given** feh was marked available, **When** spawning feh fails because the executable is not found, **Then** feh availability becomes false and install messaging is shown.
2. **Given** spawn fails for a non-availability reason (e.g. display permission), **When** handling the error, **Then** feh is not marked unavailable unless PATH lookup also fails.
3. **Given** availability flips false after spawn failure, **When** the capabilities panel is visible, **Then** it reflects not installed on next render without restart.

---

### User Story 4 - Discover Recheck From Menu (Priority: P2)

A user expects tool actions under the **Tools** menu (per feature 002). Recheck must be discoverable from menu and/or capabilities panel without duplication confusion.

**Why this priority**: Closes gap between 002 FR-002 and current panel-only button.

**Independent Test**: Tools menu contains recheck entry; same action as panel button; one log line per recheck.

**Acceptance Scenarios**:

1. **Given** the menu bar is visible, **When** the user opens **Tools**, **Then** a **Recheck tools on PATH** action is available (or equivalent label documented in quickstart).
2. **Given** recheck is triggered from menu or panel, **When** it completes, **Then** both feh and ImageMagick detection run in one operation.
3. **Given** multiple rapid rechecks, **When** triggered, **Then** behavior is idempotent and logs at most one result line per user action.

---

### Edge Cases

- Recheck during folder scan — MUST NOT block scan; lookup only.
- feh present but not executable — treat as unavailable; distinguish in status if possible.
- Partial PATH change (feh yes, magick no) — recheck updates each tool independently in one snapshot.
- User never rechecks after install — degraded state persists until recheck or restart (no background polling).

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: At startup, the application MUST detect **feh** and **ImageMagick** (`magick` or `convert`) on PATH and store a single **ToolCapabilities** snapshot.
- **FR-002**: `feh_available` used by feature 001 feh controls MUST derive from the ToolCapabilities snapshot.
- **FR-003**: The user MUST be able to trigger **on-demand recheck** from the capabilities panel.
- **FR-004**: The user MUST be able to trigger the **same recheck** from the **Tools** menu.
- **FR-005**: Recheck MUST refresh feh and ImageMagick detection in one operation and update panel content, button enablement, and relevant status text.
- **FR-006**: Recheck MUST append one debug log line summarizing feh and ImageMagick found/not found.
- **FR-007**: Recheck MUST complete without requiring application restart.
- **FR-008**: When feh spawn fails because the executable is missing, the application MUST set feh unavailable and surface feature 001 install messaging.
- **FR-009**: Detection logic MUST live in a core module testable without egui (constitution §III); GUI only triggers detect and renders results.
- **FR-010**: No background polling — recheck is on-demand (startup, menu, panel, spawn-failure recovery) only.
- **FR-011**: Feature 002 requirements FR-001–FR-006 are satisfied by this feature's unified recheck (002 is not implemented separately).

### Key Entities

- **ToolCapabilities**: feh_available, magick_available, magick_binary — single runtime snapshot.
- **ReCheckResult**: Per-tool found/not found; drives log, panel, and `feh_available`.
- **FehAvailability**: Runtime flag consumed by feature 001 open/wallpaper flows.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: User can go from "feh not found" to successful feh launch within 30 seconds of installing feh, without restarting rust-feh.
- **SC-002**: Recheck completes in under 500ms on a typical workstation.
- **SC-003**: After feh spawn failure due to missing binary, UI reflects unavailable within one user-visible update cycle.
- **SC-004**: Installing ImageMagick mid-session and rechecking updates capabilities panel format routing in the same session.
- **SC-005**: Unit tests cover detect with feh/magick presence logic without GUI.

## Assumptions

- Custom binary paths (non-PATH feh) are out of scope.
- ImageMagick recheck does not invoke convert — detection only.
- Config persistence of tool paths deferred to later feature.
- Depends on 008 panel for primary display of detection results.
- Implement phase may be largely **verify + gap-fill** (panel recheck and `ToolCapabilities::detect` exist); gap-fill likely includes Tools menu entry and spawn-failure → unavailable sync.

## Traceability

| Source | ID |
|--------|-----|
| outstanding-issues.md | A5 |
| feature 002 | FR-001–FR-006 (superseded by 009) |
| feature 001 | FR-008a, FR-008b |
| feature 008 | FR-008 dynamic content after recheck |
| adversarial-review.md | #8 feh startup-only |