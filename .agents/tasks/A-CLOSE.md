# A-CLOSE — Class-A residual closure (001 / 003 / 006 + 002/004 recording)

Class-A: the system of record is each feature's tasks.md (single writer = this task's
orchestrator). This file only sequences the work. Maintainer decisions of 2026-07-05:
GUI validations via scripted local session; 002/004 closed as superseded/absorbed.

## Work items
1. **Stale closures (no build):**
   - 001 T055 → check with note "superseded by T068 automation (see NEXT-ROUND D-notes)".
   - 003 T037 → check with note "shipped via 011 (format_walk_warning*,
     t069_scan_skip_non_permission green); closure per NEXT-ROUND D2, maintainer 2026-07-05".
2. **002/004 recording:** Clarifications entry in each spec.md (already marked
   Superseded-by-009 / Absorbed-by-011 since 2026-06-22; record maintainer confirmation
   2026-07-05). Coordinate with B-3 (same files) — B-3 is blocked on this item, not parallel.
3. **Scripted GUI session (xdotool + grim/import; app via cargo run --release):**
   - 001 T026 (V1), T038 (V5 recursive toggle), T050 (V3 no auto-feh), T051 (V8 feh
     missing → disabled buttons; use PATH mask), T058 (V6 debug log).
   - 003 T013–T017 runbook via scripts/run-003-gui-session.sh if present: 10k fixture,
     SC-002 scroll, RSS sampling; fill validation-results.md mandatory fields; update
     001/gap-audit.md SC-002 verdict. T026 only if inconclusive.
   - 006 T009: launch feh via app with fixed geometry, capture screenshot, verify
     1280x960 --scale-down --zoom max policy visually; attach evidence path in tasks.md note.
   - Subjective residue (scroll "feel"): waiver note citing maintainer decision 2026-07-05.
4. Green checkpoint + commit per GUARDRAILS (datetime branch, PR).

## Acceptance
- Zero silent skips: every open box in 001/003/006 ends checked-with-note,
  validated-with-evidence, or waived-with-note.
- speckit-analyze (read-only) run on 001/003/006 afterwards reports no NEW gaps
  introduced by the edits.
