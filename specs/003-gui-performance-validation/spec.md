# Feature Specification: GUI Performance Validation

**Feature Branch**: `003-gui-performance-validation`

**Created**: 2026-06-21

**Status**: In progress (automated tier pass; SC-004 RSS **pass** 2026-06-22; SC-002 scroll pending)

**Input**: Outstanding **Bucket C** from feature 001 — SC-004 (RSS &lt;150MB @10k) **validated: pass** (~126 MB peak @10k). SC-002 (60fps scroll) still **pending**. Evidence: [validation-results.md](./validation-results.md), [README](../../README.md) Resource usage.

**Parent**: [001-persistent-ui-virtual-browsing](../001-persistent-ui-virtual-browsing/spec.md) (US2, SC-002, SC-003, SC-004)

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Evidence That Large Lists Stay Smooth (Priority: P1)

A maintainer or release reviewer needs proof that feature 001's virtualized list meets scroll and memory targets on a real GUI session — not only static code checks and filter micro-benchmarks.

**Why this priority**: Feature 001's primary value is browsing 10k+ images. Without validated SC-002/SC-004, "15/15 FR pass" overclaims readiness.

**Independent Test**: Run documented quickstart perf scenarios; record pass/fail and metrics in a validation artifact.

**Acceptance Scenarios**:

1. **Given** a directory of ≥10,000 images, **When** the reviewer runs quickstart V2 scroll protocol, **Then** scroll is subjectively smooth (no sustained stutter) and result is recorded.
2. **Given** 10,000 images loaded, **When** RSS is sampled per protocol, **Then** resident set is under 150MB or a documented exception with environment notes is filed.
3. **Given** validation completes, **When** updating feature 001 `gap-audit.md`, **Then** SC-002 and SC-004 `validated` columns move from pending to pass or fail with evidence links.

---

### User Story 2 - Repeatable Validation Runbook (Priority: P1)

A new contributor can validate performance without reading the entire feature 001 spec — one script or doc lists exact steps, commands, and pass thresholds.

**Why this priority**: Prevents one-off manual sessions that cannot be reproduced (original T026–T039 problem).

**Independent Test**: Another person follows `quickstart.md` perf section only and produces the same artifact format.

**Acceptance Scenarios**:

1. **Given** `quickstart.md` perf section in this feature directory, **When** followed on Linux with test data, **Then** outputs include scroll verdict, RSS sample, filter timing reference, and date.
2. **Given** `scripts/validate-feature-001.sh` passes, **When** GUI validation is run, **Then** results are appended to `validation-results.md` (or feature 003 equivalent) without contradicting automated tier.

---

### User Story 3 - Automate What Can Be Automated (Priority: P2)

Where GUI metrics can be approximated without human scroll judgment (filter timing, scan duration), tests remain in CI; GUI-only metrics stay explicitly labeled manual.

**Why this priority**: Honors two-tier audit from outstanding-issues Bucket A3.

**Independent Test**: `cargo test` and validate script still pass; manual section clearly separated.

**Acceptance Scenarios**:

1. **Given** CI runs unit/integration tests, **When** SC-003 filter &lt;200ms test runs, **Then** it passes independently of GUI session.
2. **Given** SC-002 requires human judgment, **When** documented, **Then** checklist states "manual" and does not claim CI coverage.

---

### Edge Cases

- VM / remote desktop / software rendering — document environment; may fail SC-002 without being a product bug.
- Directory with &lt;10k images — use generated temp fixture per feature 001 tests.
- Debug log expanded — note whether RSS measurement uses collapsed debug panel (default layout).

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: This feature MUST produce a **GUI performance quickstart** with steps for V1 layout check, V2 scroll, and V4/V5 spot checks cross-referenced to feature 001.
- **FR-002**: This feature MUST define an **RSS sampling procedure** (when to sample, command, expected threshold 150MB).
- **FR-003**: This feature MUST define **scroll smoothness criteria** (e.g. rapid scrollbar drag 5s without freeze &gt;500ms).
- **FR-004**: Results MUST be written to a dated validation artifact linked from feature 001 `gap-audit.md`.
- **FR-005**: The feature MUST preserve existing automated validation (`validate-feature-001.sh`, `cargo test`) without regression.
- **FR-006**: gap-audit SC-002 and SC-004 MUST use `validated: pass|fail|pending` with evidence date after this feature completes.

### Key Entities

- **ValidationRun**: date, environment, tester, scroll_verdict, rss_mb, notes.
- **PerfFixture**: 10k image temp directory generation instructions.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: At least one completed ValidationRun document exists with all mandatory fields filled.
- **SC-002**: SC-002 and SC-004 in feature 001 gap-audit are no longer `pending` (pass or fail with reason).
- **SC-003**: A new contributor can execute the runbook in under 45 minutes including fixture generation.
- **SC-004**: Automated tier (SC-003 filter) remains green in CI without manual steps.

## Assumptions

- Validation is run on Linux with glow backend (same as feature 001).
- No embedded FPS counter in app required for v1 — human judgment acceptable if documented.
- Hardware variance acknowledged; failures may become environment-specific notes, not code changes.
- Does not require rewriting virtualization — only **proves** feature 001 claims.

## Traceability

| Source | ID |
|--------|-----|
| outstanding-issues.md | Bucket C, D (perf) |
| adversarial-review.md | ST-01, ST-10, #1, #5 |
| feature 001 | SC-002, SC-003, SC-004, V1–V10 |
| validation-results.md | SC-002/SC-004 pending |