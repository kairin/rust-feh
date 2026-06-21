# Quickstart: GUI Performance Validation

**Feature**: 003-gui-performance-validation  
**Date**: 2026-06-21  
**Parent scenarios**: [001 quickstart V1/V2/V4/V5](../001-persistent-ui-virtual-browsing/quickstart.md)

## Automated vs manual tier

| Metric | Tier | Command / doc |
|--------|------|----------------|
| SC-003 filter &lt;200ms @10k | **Automated** | `cargo test sc003_filter_10k_under_200ms` |
| FR-001–FR-011 static checks | **Automated** | `./scripts/validate-feature-001.sh` |
| SC-002 smooth scroll @10k | **Manual** | Step 4 below (human judgment) |
| SC-004 RSS &lt;150MB @10k | **Manual** | Step 5 + `./scripts/sample-rss.sh` |
| ValidationRun artifact | **Manual** | Step 7 → `validation-results.md` |

CI and `validate-gui-performance.sh` cover the automated row only — never claim SC-002/SC-004 in CI.

## Prerequisites

- Feature 001 automated tier green: `./scripts/validate-feature-001.sh`
- Linux with display (X11 or Wayland), OpenGL for glow
- ~500MB disk for temp fixture; ~45 minutes first run
- Built binary: `./build-and-place.sh` or `cargo build --release`

## One-command entry

```bash
./scripts/validate-gui-performance.sh   # automated tier only
./scripts/run-003-gui-session.sh        # fixture + launch GUI (no sudo)
```

`run-003-gui-session.sh` runs preflight, generates 10k fixture, prints path, starts `./rust-feh`. No sudo — unlike 009 spawn-failure test (V3).

---

## Step 0: Generate 10k fixture

```bash
FIXTURE=$(./scripts/generate-perf-fixture.sh 10000)
echo "Fixture: $FIXTURE"
```

Expected: path under `/tmp/rust-feh-perf-*` with 10,000 `.jpg` files.

---

## Step 1: Automated tier (must pass before GUI)

```bash
./scripts/validate-feature-001.sh
cargo test sc003_filter_10k_under_200ms -- --nocapture
```

Record: both pass → automated tier **pass**.

---

## Step 2: Launch GUI

```bash
./rust-feh
# or: cargo run --release
```

**Environment**: Note Wayland vs X11, VM vs bare metal, GPU model in validation results.

---

## Step 3: V1 spot check (layout, 30s)

Cross-ref [001 V1](../001-persistent-ui-virtual-browsing/quickstart.md#v1-persistent-controls-us1-fr-001-fr-002-fr-003):

1. Choose folder → select `$FIXTURE`
2. Wait for scan to complete (status no longer "Scanning…")
3. Verify top controls + bottom counter visible
4. Scroll to bottom — controls still visible

**Verdict**: pass / fail (layout regression)

---

## Step 4: V2 scroll protocol — SC-002 (5 min)

1. Confirm counter shows **Showing 10000 / 10000** (or filtered subset after load)
2. **Debug log collapsed** (default)
3. Rapidly drag scrollbar thumb **5 seconds** continuously
4. **PASS** if no freeze &gt;500ms and no sustained stutter
5. **FAIL** if obvious lockups; **inconclusive** if VM/software GL — document in notes

---

## Step 5: V2 RSS protocol — SC-004 (2 min)

**Preferred (full audit):** `./scripts/measure-resources.sh 10000 60` — samples RSS each second, reports peak/VmHWM, PASS/FAIL vs 150 MB. Use `RUST_FEH_START_FOLDER` with a fixture path to auto-load on startup.

**Quick spot-check** while app holds 10k load:

```bash
# Sample 3 times during scroll
./scripts/sample-rss.sh   # idle after load
./scripts/sample-rss.sh   # during scroll
./scripts/sample-rss.sh   # after scroll
```

Convert KB → MB: divide by 1024. Record **peak** value.

**Recorded audit (2026-06-22):** ~124 MB @1k images, ~126 MB @10k — see [validation-results.md](./validation-results.md).

| Peak RSS (MB) | Verdict |
|---------------|---------|
| &lt; 150 | pass |
| ≥ 150 | fail (investigate before waiving) |

---

## Step 6: V4/V5 spot checks (optional, 5 min)

- **V4**: Filter `photo_050` → counter updates; clear filter restores
- **V5**: N/A for flat fixture (no subdirs) — skip or use nested test dir

---

## Step 7: Record results

Copy template from [contracts/validation-run.md](./contracts/validation-run.md) into:

`specs/003-gui-performance-validation/validation-results.md`

Fill all mandatory fields.

---

## Step 8: Update feature 001 gap audit

Edit `specs/001-persistent-ui-virtual-browsing/gap-audit.md`:

- SC-002 `validated`: pass | fail | inconclusive (+ date)
- SC-004 `validated`: pass | fail | waived (+ date)

Link to 003 `validation-results.md`.

---

## Expected outcomes

| ID | Threshold | Tier |
|----|-----------|------|
| SC-002 (001) | Smooth 5s scroll | manual |
| SC-003 (001) | Filter &lt;200ms @10k | automated (cargo test) |
| SC-004 (001) | RSS &lt;150MB @10k | manual |
| SC-001 (003) | ValidationRun complete | manual |
| SC-004 (003) | Automated tier green | CI |

---

## When unsure

Per [007 FR-006](../007-outstanding-roadmap/spec.md): consult Codex, Grok, Hermes, or DeepSeek 4 Pro if scroll/RSS verdict is ambiguous; add Clarifications to `spec.md` before closing gap-audit.