# Contract: ValidationRun Artifact

**Feature**: 003-gui-performance-validation  
**Version**: 1.0  
**Consumers**: Maintainers, CI (automated tier only), gap-audit updater

## Purpose

Machine- and human-readable record that feature 001 GUI performance claims were validated or rejected with evidence.

## File location

- **Primary**: `specs/003-gui-performance-validation/validation-results.md`
- **Cross-reference**: `specs/001-persistent-ui-virtual-browsing/gap-audit.md` (SC-002, SC-004 `validated` column)

## Required markdown structure

```markdown
# GUI Performance Validation Results

**Run ID**: {run_id}
**Date**: {ISO-8601}
**Tester**: {name}
**Environment**: {details}

## Automated tier (FR-005)

| Check | Result |
|-------|--------|
| validate-feature-001.sh | pass/fail |
| cargo test sc003_filter_10k_under_200ms | pass/fail |

## Manual tier (SC-002, SC-004)

| Metric | Value | Threshold | Verdict |
|--------|-------|-----------|---------|
| Scroll smoothness (5s rapid drag) | notes | no freeze >500ms | pass/fail/inconclusive |
| RSS peak (MB) | {n} | <150 | pass/fail/waived |
| Images loaded | {count} | ≥10000 | pass/fail |

## Fixture

- Path: `{fixture_path}`
- Generator: `scripts/generate-perf-fixture.sh`

## Gap audit update

- [ ] Updated 001 gap-audit SC-002 validated: {pass|fail|pending}
- [ ] Updated 001 gap-audit SC-004 validated: {pass|fail|pending}

## Notes

{free text — VM waiver, advisory consultation per 007 FR-006}
```

## Field constraints

| Field | Constraint |
|-------|------------|
| `rss_peak_mb` | Positive number, 1 decimal max |
| `scroll_verdict` | Exactly one of: pass, fail, inconclusive |
| `rss_verdict` | pass requires value &lt; 150 unless `waived` with reason |
| `image_count` | Integer ≥ 10000 for full V2 protocol |

## Scripts interface

```bash
# Generate fixture → prints path to stdout
./scripts/generate-perf-fixture.sh [count]

# Sample RSS of newest rust-feh process (KB on stdout)
./scripts/sample-rss.sh

# Run automated tier + print manual checklist
./scripts/validate-gui-performance.sh
```

## Versioning

Increment contract version if mandatory fields added. Backward-compatible additions only in minor versions.