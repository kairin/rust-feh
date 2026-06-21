# Feature Specification: feh Runtime Detection

**Feature Branch**: `002-feh-runtime-detection`

**Created**: 2026-06-21

**Status**: Draft

**Input**: Outstanding issue **A5** from feature 001 — `feh_available` is detected once at startup; if the user installs feh while rust-feh is running, buttons stay disabled until restart.

**Superseded by**: [009-external-tool-runtime](../009-external-tool-runtime/spec.md) (2026-06-22) — unified feh + ImageMagick recheck. This spec retained for traceability only; implement via **009**, not **002**.

**Parent**: [001-persistent-ui-virtual-browsing](../001-persistent-ui-virtual-browsing/spec.md) (FR-008a, FR-008b)

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Install feh Without Restarting (Priority: P1)

A user launches rust-feh before installing feh. The status bar shows the install message and feh actions are degraded. They run `sudo apt install feh` in a terminal while rust-feh stays open. They return to rust-feh and want to open an image without restarting the application.

**Why this priority**: Common Linux workflow — install missing tools mid-session. Without this, the app appears broken until quit/relaunch.

**Independent Test**: Start rust-feh with feh absent, install feh, trigger re-detection, confirm "Open in feh" works and status updates.

**Acceptance Scenarios**:

1. **Given** rust-feh started with feh not installed, **When** the user installs feh and chooses "Re-check feh" from the Tools menu, **Then** `feh_available` becomes true and feh buttons become active.
2. **Given** feh was missing and is now detected, **When** re-check succeeds, **Then** the primary status line no longer shows the install message (unless another error applies).
3. **Given** feh is still not installed, **When** the user chooses "Re-check feh", **Then** status remains the install message and no feh process is spawned.

---

### User Story 2 - Recover After Transient feh Removal (Priority: P2)

A user had feh available at startup. feh is later removed or PATH changes. The user clicks "Open in feh" and spawn fails. They want the UI to reflect that feh is no longer available without guessing.

**Why this priority**: Less common than install-mid-session but prevents stale "available" state.

**Independent Test**: Mock or rename feh binary, attempt open, confirm UI flips to unavailable and message is shown.

**Acceptance Scenarios**:

1. **Given** `feh_available` is true, **When** spawning feh fails with "not found" class errors, **Then** the application sets `feh_available` to false and shows the install/status message.
2. **Given** spawn fails for a non-availability reason (e.g. permission), **When** handling the error, **Then** feh is not marked unavailable unless PATH lookup also fails.

---

### Edge Cases

- Re-check while a folder scan is in progress — MUST NOT block scan; re-check is instant PATH lookup.
- Multiple rapid re-check clicks — idempotent; no duplicate log spam.
- feh present but not executable — treated as unavailable with a distinct status hint if distinguishable.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The application MUST detect feh availability at startup (unchanged from 001).
- **FR-002**: The application MUST provide a user-visible **Re-check feh** action in the Tools menu.
- **FR-003**: Re-check MUST use PATH lookup (`which feh` equivalent) and update `feh_available` and status accordingly.
- **FR-004**: When re-check finds feh, feh-dependent controls MUST become enabled per FR-008a behavior from feature 001.
- **FR-005**: When feh spawn fails because the executable is missing, the application MUST set `feh_available` to false and surface FR-008a messaging.
- **FR-006**: Re-check MUST append a single line to the debug log indicating result (found / not found).

### Key Entities

- **FehAvailability**: Boolean runtime flag plus optional last-checked timestamp (in-memory only).
- **ReCheckResult**: Found | NotFound | Error(message) — drives status and log.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: User can go from "feh not found" to successful feh launch in under 30 seconds after installing feh, without restarting rust-feh.
- **SC-002**: Re-check completes in under 500ms on a typical workstation.
- **SC-003**: After failed spawn due to missing binary, UI reflects unavailable state within one user action (no restart).

## Assumptions

- feh is installed on PATH when "available"; no custom binary path UI in this feature.
- No background polling — re-check is **on-demand** only (menu + failed spawn).
- Config persistence of feh path deferred to a later feature.
- Depends on feature 001 `feh_button` / `try_open_in_feh` patterns.

## Traceability

| Source | ID |
|--------|-----|
| outstanding-issues.md | A5 |
| adversarial-review.md | #8 |
| feature 001 | FR-008a, FR-008b |