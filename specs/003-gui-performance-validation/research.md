# Research: GUI Performance Validation

**Feature**: 003-gui-performance-validation  
**Date**: 2026-06-21

## RQ1: What can be automated vs manual for SC-002 / SC-004?

**Decision**: Keep **two-tier** validation — CI runs `validate-feature-001.sh` + `cargo test`; SC-002 (scroll feel) and SC-004 (RSS) require a human GUI session with documented protocol.

**Rationale**: egui/glow has no built-in FPS export in rust-feh v1. Adding tracing dependencies violates constitution §II for a validation-only feature. Feature 001 adversarial review accepted human judgment if documented.

**Alternatives considered**:
- Embed FPS overlay in app — rejected (scope creep, new UI).
- `perf` / GPU counters — rejected (complex, environment-specific).
- Skip GUI validation — rejected (gap-audit overclaim risk).

---

## RQ2: How to sample RSS reliably?

**Decision**: Sample RSS **after** 10k load + 10s idle + during 5s rapid scroll using `ps -o rss= -p $(pgrep -n rust-feh)` (KB → MB). Take **max of 3 samples** during scroll.

**Rationale**: Single sample can miss peak during allocation. `pgrep -n` picks newest rust-feh if multiple instances.

**Alternatives considered**:
- `/proc/self/status` from inside app — requires code change.
- `smem` — not always installed.

---

## RQ3: Scroll smoothness criteria without FPS counter?

**Decision**: **Subjective protocol** — reviewer drags scrollbar thumb rapidly for 5 continuous seconds; **PASS** if no freeze lasting &gt;500ms and no sustained visible stutter; **FAIL** otherwise. Note environment (bare metal / VM / Wayland).

**Rationale**: Matches feature 001 US2 acceptance language ("perceptible stutter").

**Alternatives considered**:
- Frame time histogram — needs instrumentation.
- Video recording review — too heavy for v1.

---

## RQ4: 10k fixture generation?

**Decision**: Bash script creates temp dir with `photo_NNNNN.jpg` minimal files (1 byte), same pattern as `tests/feature_001_validation.rs` `scan_10k` test.

**Rationale**: Proven in CI; fast (~seconds); metadata-only list path.

**Alternatives considered**:
- Real JPEG headers — unnecessary for list virtualization test.
- Commit 10k files to repo — rejected.

---

## RQ5: Where to store ValidationRun results?

**Decision**: Primary artifact: `specs/003-gui-performance-validation/validation-results.md`. Cross-link from `specs/001-persistent-ui-virtual-browsing/gap-audit.md` validated column.

**Rationale**: FR-004 traceability; 001 keeps automated results separate.

**Alternatives considered**:
- Single merged file only in 001 — rejected (loses feature 003 boundary).

---

## RQ6: Relationship to `validate-feature-001.sh`?

**Decision**: New `validate-gui-performance.sh` **calls** `validate-feature-001.sh` first (FR-005), then prints manual checklist steps; does not fake GUI metrics in CI.

**Rationale**: CI stays green without display server; manual section explicit.