# GUI Performance Validation Results

**Run ID**: 2026-06-28-scroll-rss-audit
**Date**: 2026-06-28T20:02:29+08:00
**Tester**: agent + maintainer (automated RSS sampling; manual scroll observation by user)
**Environment**: Linux desktop session; metadata-only list, no thumbnails; `feh` not open during RSS samples

## Automated tier (FR-005)

| Check | Result |
|-------|--------|
| cargo check | pass |
| cargo test | pass |
| cargo build --release | pass |
| cargo test sc003_filter_10k_under_200ms | pass |
| validate-feature-001.sh | blocked unchanged: `cargo clippy` unavailable in this environment; no-clippy temporary run passed build/tests but hit stale bottom-status static check |
| cargo clippy -- -D warnings | not run: clippy unavailable |
| validate-gui-performance.sh | blocked unchanged: delegates to clippy-gated `validate-feature-001.sh` |

**Automated tier notes**: Canonical Rust gates pass locally. `cargo fmt`/`rustfmt` and `cargo clippy` are unavailable in this environment.

## Manual tier (SC-002, SC-004)

| Metric | Value | Threshold | Verdict |
|--------|-------|-----------|---------|
| Scroll smoothness (5s rapid drag) | 10k fixture; user confirmed smooth rapid full-range scrollbar drag, no freeze | no freeze >500ms | **pass** |
| RSS peak @ 10,000 images (MB) | 142.8 | <150 | **pass** |
| Images loaded (SC-004 protocol) | 10,000 / 10,000 shown in counter | ≥10,000 | **pass** |
| Layout spot check (V1) | controls remained visible while 10k list loaded | controls visible | **pass** |

### RSS audit protocol (2026-06-28)

Command:

```bash
./scripts/measure-resources.sh 10000 15
```

Summary:

| Images in fixture | RSS min | RSS avg | RSS peak | VmHWM peak | SC-004 goal | Verdict |
|-------------------|---------|---------|----------|------------|-------------|---------|
| 10,000 | 95.7 MB | 139.7 MB | 142.8 MB | 142.8 MB | < 150 MB | pass |

Fixture: `/tmp/rust-feh-perf-OCjDyZ`

Log tail from measurement:

```text
[rust-feh] Inventory: native=10000 magick=0 converted=0 awaiting=0 skipped=0
[rust-feh] Scanned '/tmp/rust-feh-perf-OCjDyZ' (recursive=true), found 10000 supported images
[rust-feh] Sample files: ["photo_00000.jpg", "photo_00001.jpg", "photo_00002.jpg"]
[rust-feh] Auto-selected first image: /tmp/rust-feh-perf-OCjDyZ/photo_00000.jpg
[rust-feh] Converted-status metadata updated (background)
```

## Fixture

- Path: `/tmp/rust-feh-perf-OCjDyZ`
- Generator: `scripts/generate-perf-fixture.sh`
- Count verified: 10,000 files

## Gap audit update

- [x] Updated 001 gap-audit SC-002 validated: **pass** (2026-06-28 — 10k scroll smoothness manual pass)
- [x] Updated 001 gap-audit SC-004 validated: **pass** (2026-06-28 — peak RSS 142.8 MB @10k)

## Notes

- SC-002 closed for feature 001/003 with user-observed 10k rapid scrollbar drag evidence.
- SC-004 closed for feature 001/003 with scripted RSS evidence under the 150 MB threshold.
- `validate-feature-001.sh` still contains a stale static check for `TopBottomPanel::bottom("status")`; current UI exposes count/status via the right inspector session-status panel. Decide separately whether to update the validator/spec to match current UI or restore a bottom status panel.
- If scroll/RSS are re-run on other hardware (VM/software GL), record local results here rather than assuming this machine's result generalizes.
