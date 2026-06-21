# Outstanding Issues — Consolidated (post-implementation)

**Feature**: `001-persistent-ui-virtual-browsing`
**Created**: 2026-06-21 via `/speckit-clarify` + `/speckit-converge`
**Source**: Post-implementation `adversarial-review.md`

This document classifies every flagged issue into one of four buckets so nothing is chased twice.

---

## Bucket A — Not bugs (artifact drift / wording)

These looked like failures but are **equivalent implementations** or **documentation lag**.

| Issue | What reviewers said | Resolution |
|-------|---------------------|------------|
| **A1** `list_scroll_offset` vs `scroll_generation` | data-model/spec mention `list_scroll_offset`; code uses `scroll_generation` + `id_salt` | **Same FR-005 behavior.** Clarify: `scroll_generation` is the canonical field. Update data-model (T066). T012 satisfied. |
| **A2** `ui.add_enabled` vs `feh_button` | Spec clarify says `add_enabled`; code uses `feh_button` helper | **Same FR-008a behavior** (grayed + click shows message). Clarify: `feh_button` is acceptable. T045 satisfied. |
| **A3** gap-audit 15/15 pass | Overclaim without manual perf tests | **Two-tier audit**: `code` pass ≠ `validated` pass. Update gap-audit columns (T066). SC-002–004 stay `validated: pending`. |
| **A4** Duplicate recursive checkbox | View menu + toolbar both have toggle | **Intentional** — FR-011 requires menu parity with toolbar. Not a defect. |
| **A5** `feh_available` startup-only | No re-detect if feh installed while app runs | **Deferred Area 6** — not in FR scope. |

**Action**: Sync artifacts only (T066). No code change required for A1–A5.

---

## Bucket B — Real code gaps (fix before ship)

| Issue | FR | Evidence | Task |
|-------|-----|----------|------|
| **B1** feh-not-found not in main status after folder load | FR-008a | `scan_directory` sets `"Loaded N images…"` and drops startup warning from primary status line | **T065** |
| **B2** Dead `Selection` / `SortMode` in `types.rs` | polish | `#[allow(dead_code)]` | **T069** |

**Action**: `/speckit-implement` T065, T069.

---

## Bucket C — Manual validation (not automatable here)

Eight separate tasks (T026, T036–T039, T050, T051, T058) are the **same work**: one quickstart session.

| Scenario | Covers | Old tasks |
|----------|--------|-----------|
| V1 | Persistent controls @ 5k+ | T026 |
| V2, V4, V5, V9, V10 | Virtual scroll, filter, recursive, scan state | T036–T039 |
| V3, V8 | No auto-feh, feh missing | T050, T051 |
| V6 | Debug log empty state | T058 |

**Action**: **T067** — single consolidated validation task. Old tasks remain in file for traceability but are **superseded** by T067 (see tasks.md supersession table).

---

## Bucket D — Optional / deferred

| Issue | Severity | Decision |
|-------|----------|----------|
| Scanner ignores non-`PermissionDenied` walkdir errors | MEDIUM | Log as debug in Area 6; not blocking MVP |
| T055 permission-denied subdir test | MEDIUM | Folded into **T068** (manual or unit test) |
| Performance SC-002/003/004 unmeasured | HIGH until run | Closed when **T067** completes with notes |

---

## Summary

| Bucket | Count | Status |
|--------|-------|--------|
| A — Not bugs | 5 | Resolved in clarify (docs) |
| B — Code gaps | 2 | T065, T069 |
| C — Manual validation | 1 consolidated task | T067 |
| D — Optional | 2 | T068 + Area 6 deferral |

**Before**: 10 open tasks, unclear overlap.
**After**: **5 convergence tasks** (T065–T069).

---

## Promoted to new features (2026-06-21)

Remaining items are **not** more work on 001 — each has a dedicated spec:

| Issue | Feature |
|-------|---------|
| A5 feh re-detect | [002-feh-runtime-detection](../002-feh-runtime-detection/spec.md) |
| C SC-002 / SC-004 / manual V gaps | [003-gui-performance-validation](../003-gui-performance-validation/spec.md) |
| D scanner non-permission errors | [004-scanner-resilience](../004-scanner-resilience/spec.md) |
| Dogfood list/folder/sort | [005-image-list-presentation](../005-image-list-presentation/spec.md) |
| Dogfood window/feh stability | [006-window-viewer-stability](../006-window-viewer-stability/spec.md) |

Master index: [OUTSTANDING-ISSUES-ROADMAP.md](../OUTSTANDING-ISSUES-ROADMAP.md) (spec: [007-outstanding-roadmap](../007-outstanding-roadmap/spec.md))

**When unsure** about bucket, scope, or order: consult Codex, Grok, Hermes, or DeepSeek 4 Pro per FR-006 in 007; record decision in feature Clarifications.

## Next command

```
/speckit-plan   # pick feature 003 or 005 first — see roadmap
```

Feature 001 Phase 7 (T065–T069) is complete; close 001 validated tier via **003**.