# GUI Performance Validation Results

**Run ID**: 2026-06-22-automated  
**Date**: 2026-06-22T01:47:15+08:00  
**Tester**: agent (automated tier); manual GUI pending human  
**Environment**: Linux; display required for manual tier (not run in this session)

## Automated tier (FR-005)

| Check | Result |
|-------|--------|
| validate-feature-001.sh | pass (13/13) |
| cargo test sc003_filter_10k_under_200ms | pass |
| cargo clippy -- -D warnings | pass |
| validate-gui-performance.sh | pass |

**Automated tier duration**: ~45s (excludes manual GUI protocol).

## Manual tier (SC-002, SC-004)

| Metric | Value | Threshold | Verdict |
|--------|-------|-----------|---------|
| Scroll smoothness (5s rapid drag) | pending | no freeze >500ms | **pending — needs GUI** |
| RSS peak (MB) | pending | <150 | **pending — needs GUI** |
| Images loaded | pending | ≥10000 | **pending — needs GUI** |
| Layout spot check (V1) | pending | controls visible | **pending — needs GUI** |

### Manual handoff (no sudo — ~30 min)

```fish
cd /home/kkk/Apps/rust-feh
./scripts/run-003-gui-session.sh
# or manually:
set FIXTURE (./scripts/generate-perf-fixture.sh 10000)
./rust-feh
# 1. Choose folder → $FIXTURE → wait for scan
# 2. Confirm Showing 10000 / 10000
# 3. Rapid scrollbar drag 5s → record scroll verdict
# 4. In another terminal: ./scripts/sample-rss.sh (3×) → peak MB
# 5. Update this file + 001 gap-audit SC-002/SC-004 validated columns
```

## Fixture

- Path: `/tmp/rust-feh-perf-PEWpwP` (regenerate with `./scripts/generate-perf-fixture.sh 10000`)
- Count verified: 10000 files (T005)
- Generator: `scripts/generate-perf-fixture.sh`

## Gap audit update

- [ ] Updated 001 gap-audit SC-002 validated: pending (awaiting manual scroll)
- [ ] Updated 001 gap-audit SC-004 validated: pending (awaiting RSS samples)

## Notes

- Automated prerequisites complete 2026-06-22; features 005/008/009 shipped since plan authored.
- If scroll/RSS inconclusive (VM/software GL), document in `spec.md` Clarifications before marking pass in gap-audit (007 FR-006).
- Estimated manual protocol: ~30–45 min per quickstart Steps 2–8.