# Feature Specification: Browsing Experience Round

**Feature Branch**: `011-browsing-experience-round`

**Created**: 2026-06-22

**Status**: Draft

**Input**: Dogfood session on SMB/GVFS share — feh stops at subfolder boundaries; network scan freezes UI; debug log not copyable. Implements feh filelist launch + background scan + copyable Activity log (consolidates some exploratory dinner notes).

**Parent**: [001-persistent-ui-virtual-browsing](../001-persistent-ui-virtual-browsing/spec.md), [004-scanner-resilience](../004-scanner-resilience/spec.md), [006-window-viewer-stability](../006-window-viewer-stability/spec.md)

## Clarifications

### Session 2026-06-22

- Q: Should feh navigation span the full scan or only the filtered list? → A: **Filtered + sorted list** — matches what the user sees in the central panel.
- Q: Async scanning technology? → A: Background worker thread with message channel; UI polls each frame (constitution: minimal deps, no tokio yet).
- Q: Copyable log vs external terminal (tmux)? → A: **In-app selectable log + Copy buttons** — no embedded terminal.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Browse All Images in feh Across Subfolders (Priority: P1)

A user loads a recursive folder tree (local or network) with images in many subdirectories. They open one image in feh and use feh's next/previous keys to move through **all** images in the current rust-feh list, crossing subfolder boundaries without reopening feh.

**Why this priority**: Confirmed in dogfood — feh was launched per-parent-folder only; browsing stopped at each `images/` batch folder.

**Independent Test**: Load multi-subfolder fixture; open feh on image in folder A; advance past last image in A into folder B.

**Acceptance Scenarios**:

1. **Given** a recursive scan with images in ≥2 subfolders, **When** the user opens feh on any listed image, **Then** feh receives a filelist containing every path in the current filtered, sorted list.
2. **Given** a filelist launch, **When** the user presses next in feh at the last image of subfolder A, **Then** the first image of subfolder B appears (order matches rust-feh sort).
3. **Given** an active search filter, **When** Open in feh runs, **Then** the filelist contains only filtered images.
4. **Given** feh launch, **When** inspecting the activity log, **Then** the command documents `--filelist` and entry count.

---

### User Story 2 - Responsive Scanning on Slow Paths (Priority: P1)

A user chooses a folder on a network mount (GVFS/SMB/NFS). The window stays interactive during scan; status shows progress; results apply when complete.

**Why this priority**: Synchronous scan blocked the UI; network paths felt like immediate crash/dispose.

**Independent Test**: Start scan on large or slow path; interact with menus and resize window while "Scanning…" shows.

**Acceptance Scenarios**:

1. **Given** a folder pick, **When** scan starts, **Then** the UI remains responsive (menus, window resize, cancel not required).
2. **Given** scan in progress, **When** viewing status bar, **Then** "Scanning…" is shown until completion.
3. **Given** scan completes, **When** results arrive, **Then** image list and inventory update as today (first image selected, counts correct).
4. **Given** user picks a new folder while scan runs, **When** the new scan starts, **Then** only the latest scan result is applied.

---

### User Story 3 - Visible Scan Skip Reasons (Priority: P1)

A user scans a tree with permission issues, stale mount entries, or other walk errors. Warnings appear in the activity log; scan still returns partial results.

**Why this priority**: Extends FR-015 / feature 004 — non-permission errors were silent.

**Independent Test**: Mixed-error fixture; permission line preserved; at least one `Scan skip:` line for other errors.

**Acceptance Scenarios**:

1. **Given** permission denied on a subdirectory, **When** scan completes, **Then** warning contains "Permission denied" (unchanged format).
2. **Given** a non-permission walkdir error, **When** scan completes, **Then** warning uses prefix `Scan skip:` with reason.
3. **Given** more than 50 warnings, **When** scan completes, **Then** log shows capped summary plus count of omitted lines.

---

### User Story 4 - Copyable Activity Log (Priority: P2)

A user wants to copy scan paths, feh commands, or warnings for debugging or sharing. The bottom activity area supports selection and one-click copy.

**Why this priority**: Status bar is read-only; dogfood requested paste-friendly output (tmux-like persistence without a terminal).

**Independent Test**: Run scan + feh open; select text in log or click Copy log; paste elsewhere.

**Acceptance Scenarios**:

1. **Given** log entries exist, **When** the user expands Activity log, **Then** text is selectable in a multiline read-only view.
2. **Given** log entries exist, **When** the user clicks "Copy log", **Then** full log text is on the system clipboard.
3. **Given** a status message, **When** the user clicks "Copy status", **Then** current status line is on the clipboard.

---

### Edge Cases

- Empty filtered list — Open in feh disabled or shows "No image selected".
- feh filelist with GVFS paths containing special characters — one path per line, UTF-8.
- Scan of nonexistent root — empty result, optional warning.
- feh not installed — filelist path still logged; spawn shows existing unavailable handling.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Open in feh MUST launch with a filelist covering the current filtered, sorted image paths.
- **FR-002**: feh MUST start at the selected image via `--start-at` matching a filelist entry.
- **FR-003**: feh MUST retain fixed geometry and zoom policy from feature 006 (`--geometry`, `--scale-down`, `--zoom`).
- **FR-004**: Directory scan MUST run off the UI thread; GUI MUST poll for completion each frame.
- **FR-005**: Only the latest scan request MUST apply when multiple scans are triggered.
- **FR-006**: Scanner MUST log non-permission walkdir errors with prefix `Scan skip:` (feature 004).
- **FR-007**: Permission denied warnings MUST retain existing message format.
- **FR-008**: Warning volume MUST cap at 50 lines with summary when exceeded.
- **FR-009**: Activity log MUST be selectable (read-only multiline) and offer Copy log / Copy status actions.
- **FR-010**: feh launch and scan warnings MUST continue to mirror to stderr and in-app log.

### Key Entities

- **FehFilelist**: Ordered paths written to a temp file for feh `--filelist`.
- **ScanJob**: Background scan with generation id and channel-delivered `ScanResult`.
- **ActivityLog**: Ring buffer of timestamped messages with joined text for copy.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: User can advance feh across ≥2 subfolders without relaunching (dogfood SMB tree).
- **SC-002**: UI accepts input within 1s of starting a 10k local scan (no multi-second freeze).
- **SC-003**: 100% of walkdir errors in mixed fixture produce a warning (none silent).
- **SC-004**: `t068_permission_denied_warning` continues to pass.
- **SC-005**: User copies full log to clipboard in one click during quickstart validation.

## Assumptions

- feh distro build supports `--filelist` and `--start-at` (standard on Ubuntu feh).
- Temp filelist lives under system temp dir; rewritten on each feh open.
- No feh process management (single launch per click, as today).
- Async uses `std::thread` + `std::sync::mpsc` — no new Cargo dependencies.

## Traceability

| Source | ID |
|--------|-----|
| Dogfood SMB session | feh single-folder spawn |
| SESSION-2026-06-22 | feh filelist + responsive scan (dinner topics 10/11 partial; no 010/011 named features promised) |
| 004-scanner-resilience | FR-002–FR-004, SC-001–SC-003 |
| Constitution §V | responsive scanning |