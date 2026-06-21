# Feature Specification: Scanner Resilience & Error Visibility

**Feature Branch**: `004-scanner-resilience`

**Created**: 2026-06-21

**Status**: Draft

**Input**: Outstanding **Bucket D** — `scanner.rs` only surfaces `PermissionDenied` walkdir errors; other failures (loops, I/O, corrupt paths) are silently skipped. Adversarial review #6.

**Parent**: [001-persistent-ui-virtual-browsing](../001-persistent-ui-virtual-browsing/spec.md) (FR-015)

## User Scenarios & Testing *(mandatory)*

### User Story 1 - See Why Files Were Skipped (Priority: P1)

A user scans a large tree with a few problematic entries (broken symlinks, unreadable mount points, permission issues on siblings). They need to know what was skipped and why, without the scan failing entirely.

**Why this priority**: Silent skips erode trust — the image count looks wrong and debugging is guesswork.

**Independent Test**: Fixture with permission-denied dir plus one other walkdir error type; confirm warnings appear in debug log.

**Acceptance Scenarios**:

1. **Given** a subdirectory with permission denied, **When** recursive scan runs, **Then** a warning containing "Permission denied" appears in debug log (existing FR-015 behavior preserved).
2. **Given** a walkdir error that is not permission denied, **When** scan runs, **Then** a warning describing the skip reason appears in debug log.
3. **Given** multiple errors, **When** scan completes, **Then** all warnings are forwarded to debug log via existing `log()` path in main.

---

### User Story 2 - Scan Still Returns Partial Results (Priority: P1)

Errors on one branch must not abort the entire scan — user still gets images from accessible paths.

**Why this priority**: Matches walkdir best-effort semantics users expect from file browsers.

**Independent Test**: One bad entry in tree; scan returns other images and non-zero warning count.

**Acceptance Scenarios**:

1. **Given** 100 valid images and one unreadable directory entry, **When** scan completes, **Then** valid images are listed and warnings count ≥1.
2. **Given** scan warnings exist, **When** status line updates, **Then** primary status still reports load count; warnings are not silently dropped.

---

### Edge Cases

- Nonexistent root path — empty result with optional warning (platform-dependent walkdir behavior).
- Very large warning volume — cap or summarize if &gt;50 messages (document limit in plan phase).
- Symlink loops — `follow_links(false)` already; loop errors should surface as warnings.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Scanner MUST continue scanning after non-fatal walkdir errors (unchanged).
- **FR-002**: Scanner MUST collect a human-readable warning string for every walkdir `Err` not already handled.
- **FR-003**: Permission denied errors MUST retain current message format for compatibility with T068 test.
- **FR-004**: Non-permission errors MUST use a distinct prefix e.g. `"Scan skip: {reason}"` to distinguish from permission lines.
- **FR-005**: `main.rs` MUST forward all warnings to debug log after scan (unchanged pipeline).
- **FR-006**: Unit or integration test MUST cover at least one non-permission error path (platform-gated if needed).

### Key Entities

- **ScanWarning**: message string collected during walk; returned alongside `Vec<ImageEntry>`.
- **ScanResult**: (entries, warnings) tuple — API unchanged from feature 001.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 100% of walkdir errors in a mixed-error fixture produce a warning line (none silent).
- **SC-002**: Existing `t068_permission_denied_warning` test continues to pass.
- **SC-003**: New test for non-permission skip passes on Linux CI or is documented as `cfg(unix)` only.
- **SC-004**: Scan throughput on 10k clean directory unchanged within 10% (no catastrophic slowdown).

## Assumptions

- Warnings go to debug log, not modal dialogs (consistent with FR-015).
- No retry logic in this feature.
- Async scanning out of scope (Area 6).
- Error message text may include OS-specific detail.

## Traceability

| Source | ID |
|--------|-----|
| outstanding-issues.md | Bucket D |
| adversarial-review.md | #6, scanner.rs section |
| feature 001 | FR-015, T055, T068 |