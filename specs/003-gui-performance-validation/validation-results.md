# GUI Performance Validation Results

**Run ID**: 2026-06-22-rss-audit  
**Date**: 2026-06-22  
**Tester**: agent + maintainer (automated sampling); scroll protocol not run this session  
**Environment**: Linux (X11), metadata-only list, no thumbnails; `feh` not open during RSS samples

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
| Scroll smoothness (5s rapid drag) | not run | no freeze >500ms | **pending — needs GUI** |
| RSS peak @ 1,000 images (MB) | ~124 | <150 | **pass** |
| RSS peak @ 10,000 images (MB) | ~126 | <150 | **pass** |
| Images loaded (SC-004 protocol) | 10,000 | ≥10,000 | **pass** |
| Layout spot check (V1) | not run | controls visible | **pending** |

### RSS audit protocol (2026-06-22)

Tools (documented in [README.md](../../README.md) Resource usage):

- `./scripts/measure-resources.sh [count] [seconds]` — timed RSS/VSZ/%CPU samples + VmHWM peak + PASS/FAIL vs SC-004
- `./scripts/sample-rss.sh` — one-shot RSS (KB)
- `RUST_FEH_START_FOLDER=/path ./rust-feh` — auto-load folder on startup (test hook in `src/main.rs`)

**Conditions:**

- Release binary `./rust-feh` or `target/release/rust-feh`
- Synthetic fixtures via `./scripts/generate-perf-fixture.sh`
- Metadata-only inventory (no thumbnail decode)
- `feh` runs as a **separate process** when viewing — its RSS is **not** included in `rust-feh` samples

**Recorded peaks (this machine):**

| Images in fixture | RSS peak | SC-004 goal | Verdict |
|-------------------|----------|-------------|---------|
| 1,000 | ~124 MB | < 150 MB | pass |
| 10,000 | ~126 MB | < 150 MB | pass |

Evidence captures: `docs/assets/readme-resource-measure.png`, `docs/assets/readme-resource-ps.png`.

## Fixture

- Generator: `scripts/generate-perf-fixture.sh`
- 10k path: regenerate with `./scripts/generate-perf-fixture.sh 10000` (ephemeral under `/tmp`)

## Gap audit update

- [ ] Updated 001 gap-audit SC-002 validated: **pending** (scroll protocol not run)
- [x] Updated 001 gap-audit SC-004 validated: **pass** (2026-06-22 — RSS audit above)

## Notes

- SC-004 closed for feature 001 with scripted RSS evidence; subjective SC-002 scroll still open.
- If scroll/RSS inconclusive on other hardware (VM/software GL), document in `spec.md` Clarifications before waiving (007 FR-006).
- Remaining manual handoff for SC-002 only:

```fish
cd /home/kkk/Apps/rust-feh
./scripts/run-003-gui-session.sh
# or: set FIXTURE (./scripts/generate-perf-fixture.sh 10000); ./rust-feh
# Step 4 quickstart: 5s rapid scrollbar drag → record pass/fail/inconclusive
```