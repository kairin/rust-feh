# Data Model: GUI Performance Validation

**Feature**: 003-gui-performance-validation  
**Date**: 2026-06-21

## Entities

### ValidationRun

One completed GUI performance validation session. Persisted as markdown table rows in `validation-results.md` per [contracts/validation-run.md](./contracts/validation-run.md).

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `run_id` | string | yes | ISO date + optional suffix, e.g. `2026-06-21-a` |
| `date` | ISO-8601 | yes | When run completed |
| `tester` | string | yes | Human or agent identifier |
| `environment` | string | yes | OS, DE (Wayland/X11), GPU/VM notes |
| `fixture_path` | path | yes | Temp dir used for 10k images |
| `image_count` | u32 | yes | Loaded count (expect ≥10000) |
| `scroll_verdict` | enum | yes | `pass` \| `fail` \| `inconclusive` |
| `scroll_notes` | string | no | Stutter description if fail |
| `rss_peak_mb` | f32 | yes | Max RSS MB during protocol |
| `rss_verdict` | enum | yes | `pass` (&lt;150) \| `fail` \| `waived` |
| `filter_sc003` | enum | yes | `pass` (CI) \| `fail` — reference cargo test |
| `automated_tier` | enum | yes | `pass` \| `fail` — validate-feature-001.sh |
| `gap_audit_updated` | bool | yes | Whether 001 gap-audit validated column updated |

**State transitions**:

```
PLANNED → (automated script pass) → AUTO_OK
AUTO_OK → (GUI session run) → RECORDED
RECORDED → (gap-audit updated) → CLOSED
```

### PerfFixture

Ephemeral directory for validation; not stored in repo.

| Field | Type | Description |
|-------|------|-------------|
| `path` | PathBuf | Temp directory root |
| `file_count` | usize | Default 10000 |
| `pattern` | string | `photo_{i:05}.jpg` |
| `created_by` | string | `scripts/generate-perf-fixture.sh` |

### GuiPerfThresholds (constants)

| Constant | Value | Source |
|----------|-------|--------|
| `RSS_MAX_MB` | 150 | Feature 001 SC-004 |
| `FILTER_MAX_MS` | 200 | Feature 001 SC-003 (CI) |
| `SCROLL_FREEZE_MAX_MS` | 500 | Feature 003 FR-003 |
| `SCROLL_TEST_DURATION_S` | 5 | Feature 003 FR-003 |
| `MIN_IMAGE_COUNT` | 10000 | Feature 001 US2 |

## Relationships

```text
PerfFixture ──generates──▶ rust-feh load ──observed──▶ ValidationRun
ValidationRun ──updates──▶ 001 gap-audit (SC-002, SC-004 validated column)
```

## Validation rules

- `rss_peak_mb` MUST be recorded with collapsed debug log (default layout).
- `scroll_verdict` = `inconclusive` only with environment waiver documented in `scroll_notes`.
- Cannot set `gap_audit_updated=true` without both scroll and RSS verdicts ≠ pending.