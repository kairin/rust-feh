# Next-Round Consolidated Outstanding Work

**Created**: 2026-06-28
**Source of truth**: verified live against repo (git, `cargo check`, `cargo test`, grep of every `tasks.md`), not prior notes.
**Companion**: master index [OUTSTANDING-ISSUES-ROADMAP.md](./OUTSTANDING-ISSUES-ROADMAP.md); active pointer `.specify/feature.json`.

This file consolidates **everything still outstanding** so the next round can complete it in one pass. Items are grouped by category because they need different handling: real code, Spec Kit artifacts, manual-only validation, and stale-task reconciliation. **Do not** convert a stale/superseded box into implementation work — current code evidence already closes several of them.

## Where `/speckit-converge` fits

`/speckit-converge` is useful, but it is **feature-scoped**, not a repo-wide consolidation command. It requires the active feature to already have `spec.md`, `plan.md`, and `tasks.md`; it then compares the present code to those artifacts and appends any remaining work to that feature's `tasks.md`.

Use it in the next round like this:

1. Use this file for repo-wide consolidation and prioritization.
2. Repoint `.specify/feature.json` to `specs/006-window-viewer-stability`.
3. Run `/speckit-plan` and `/speckit-tasks` for 006, because 006 currently lacks plan/tasks.
4. Implement 006 FR-008 / SC-005 window-preference persistence.
5. Run `/speckit-converge` on 006 to append any remaining feature-scoped gaps.
6. Run `/speckit-implement` again only if converge appends new tasks.

Do **not** use converge as the first step for 006: it will stop because `plan.md` and `tasks.md` are missing. Do **not** use it to blindly reopen stale boxes in 001/003; those need manual reconciliation based on current code evidence.

---

## Verified baseline (2026-06-28)

- `cargo check` clean; `cargo test` = 109 passed / 0 failed (2 ignored: manual venice scan, needs-ImageMagick HEIC).
- Branch `main` ahead of `origin/main` by 3 commits (direnv ×2 + JPEG fallback) — unpushed.
- `.specify/feature.json` → `specs/014-multi-feh-clipboard` (**stale**; 014 is shipped & merged).
- Task-box tally per feature (`[x]` done / `[ ]` open):

  | Feature | done/total | open | Status |
  |---------|-----------|------|--------|
  | 001 persistent-ui | 59/69 | 10 | all open = manual GUI or stale |
  | 003 gui-perf-validation | 33/40 | 7 | open = manual GUI session + 1 stale |
  | 005 image-list | 56/56 | 0 | complete |
  | 008 tool-capabilities | 10/10 | 0 | complete |
  | 009 external-tool-runtime | 22/22 | 0 | complete |
  | 011 browsing-round | 26/26 | 0 | complete |
  | 012 ui-feedback-polish | 32/32 | 0 | complete |
  | 013 image-tools-cache | 75/75 | 0 | complete |
  | 014 multi-feh-clipboard | 39/39 | 0 | complete |
  | 006 window-viewer | n/a | n/a | spec-only — no plan/tasks |
  | 002 feh-runtime-detect | n/a | n/a | spec-only — superseded by 009 |
  | 004 scanner-resilience | n/a | n/a | spec-only — absorbed by 011 |

**Verdict**: all *code* that was meant to ship has shipped (7/9 feature task-lists fully checked; build+tests green). What remains is one genuine code gap (006 persistence), manual-only GUI validation (003/001/011/012), and bookkeeping.

---

## Category A — Real code gaps (the only un-built requirement)

### A1. Feature 006 — Window-preference persistence (FR-008 / SC-005)
- **Evidence it's a real gap**: no save/load of `window_size` or `window_resizable` anywhere in `src/`; only launch-list + feh-filelist persistence exist (`ui_logic.rs`: `launch_list_path`, `save_launch_list`, `load_launch_list`). Preset + resizable reset on every launch.
- **Already shipped (unspecced) for 006**: presets (Compact/Default/Large), resizable toggle, 640×480 floor (`clamp_window_size`), feh fixed geometry + zoom-max — all present in code and unit-tested. So 006 is ~90% built; only persistence is missing.
- **Plan**: mirror the launch-list pattern. Pure IO in `src/ui_logic.rs` (`window_prefs_path`, `save_window_prefs`, `load_window_prefs` over a small JSON struct in XDG config dir), wiring in `src/main.rs` (load in `RustFehApp` init; persist on preset/resizable change). Respects constitution core/GUI split.
- **Spec Kit work this requires first**: 006 has only `spec.md` — needs `/speckit-plan` then `/speckit-tasks` (see C1).

No other un-built code requirements found across any feature.

---

## Category B — Manual-only validation (needs a GUI/human session; not code)

> Blocked in this environment: the `computer_use` desktop driver fails on every capture here, so these cannot be auto-driven. They need a real display session or a documented "inconclusive (VM/software-GL)" waiver.

### B1. Feature 003 — GUI performance session (the one real open validation)
- `T013` load 10k fixture, confirm 10000/10000 counter.
- `T014` SC-002 scroll smoothness (5s rapid drag) → record pass/fail/inconclusive.
- `T015` sample RSS 3× during scroll (SC-004 already **pass** ~126 MB; re-confirm under scroll).
- `T016` fill ValidationRun mandatory fields in `003/validation-results.md`.
- `T017` update `001/gap-audit.md` SC-002 column from pending → verdict.
- `T026` (conditional) if inconclusive, add Clarifications waiver to `003/spec.md`.
- Runbook: `./scripts/run-003-gui-session.sh` (or `FIXTURE=$(./scripts/generate-perf-fixture.sh 10000); ./rust-feh`).

### B2. Feature 001 — quickstart manual scenarios (duplicate/coverage of B1)
- `T026, T036, T037, T038, T039, T050, T051, T055, T058` — quickstart V1–V10 manual runs (smooth scroll, filter counter accuracy, recursive rescan, filter <200ms, no-auto-feh, feh-missing, permission-denied warning, debug-log UX). Most are already covered by automated tests (`feature_001_validation.rs`, `sc003_filter_10k_under_200ms`); the only genuinely manual residue is subjective scroll (SC-002, = B1/T014).

### B3. Feature 011 — manual SMB GUI (V1/V2/V4) — automated tier passed.
### B4. Feature 012 — manual SMB GUI (V1–V5) — automated tier passed.

---

## Category C — Spec Kit artifact / lifecycle work

### C1. Feature 006 — create plan + tasks
- Repoint `.specify/feature.json` → `specs/006-window-viewer-stability` (prerequisite; Spec Kit resolves dir from here).
- `/speckit-plan` → retro-spec the already-shipped window/viewer work + define persistence config path/format.
- `/speckit-tasks` → decompose (fine-grained, per user preference): code-audit tasks for FR-001–FR-007 (already built → verify), implement tasks for FR-008/SC-005 persistence, manual SC tasks.
- `/speckit-implement` the persistence tasks (= A1).
- Add `006/gap-audit.md`.

### C2. Repoint active feature pointer
- `.specify/feature.json` currently points at finished 014 → set to 006 for the next round.

### C3. Doc hygiene
- `OUTSTANDING-ISSUES-ROADMAP.md`: refresh implement-order (013/014 shipped; 006 next; 003 manual-pending). It still implies 006 "not started" without noting code already exists.
- Mark `002-feh-runtime-detection/spec.md` **Superseded by 009**.
- Mark `004-scanner-resilience/spec.md` **Absorbed by 011**.

---

## Category D — Stale / superseded tasks to close (NOT implementation)

> These boxes are unchecked but the work is done or moot. Close them; do not build.

- **001 `T056`** "remove dead code `Selection`/`SortMode`" — **stale**: `Selection` no longer exists as a type; `SortMode` is actively used (`src/types.rs`, `src/main.rs`, `src/ui_logic.rs`). Mark done/obsolete.
- **003 `T037`** "implement 004 scan-skip warnings in scanner.rs" — **stale**: shipped via 011 (`scanner.rs` warning tests pass: `format_walk_warning_*`, `t069_scan_skip_non_permission`). Mark done-via-011.

---

## Next-round execution order (single pass)

1. **C2** repoint `.specify/feature.json` → 006.
2. **C1** `/speckit-plan` 006 → `/speckit-tasks` 006.
3. **A1** implement window-pref persistence (FR-008/SC-005); `cargo check` + `cargo test`; add `006/gap-audit.md`.
4. **D** close stale `001/T056` and `003/T037` with notes.
5. **C3** doc hygiene: roadmap order + mark 002 superseded / 004 absorbed.
6. **B1** run the 003 GUI session for SC-002 (or file inconclusive waiver via `T026`); cascade verdict to `001/gap-audit.md`. *(needs a working display — not possible in current headless env.)*
7. (Optional) **B3/B4** SMB GUI passes for 011/012 if hardware available.

**Non-goals next round**: async scanning (deferred Area 6), thumbnails, metadata sort, background/non-blocking clipboard (014 v1 explicitly deferred), feh in-viewer shortcuts, multi-monitor window-position memory.

**Done when**: 006 plan+tasks exist and persistence ships green; stale tasks closed; 002/004 marked; pointer + roadmap current. Only B (manual GUI) remains, gated on a display session.

---

## Cross-check (independent verifier, 2026-06-28)

A second read-only verifier independently checked the repo and agreed with the load-bearing conclusions: active/recent code implementations are done, `cargo build`, `cargo build --release`, and `cargo test` pass, while the Spec Kit lifecycle still has manual-validation and documentation gaps. It confirmed the same task tallies, the 006 spec-only gap, and the stale status of 001/T056 and 003/T037 (`SortMode` is used, no `Selection` type remains, and scanner skip warnings are already shipped/tested).

Additional observations to carry into the next round:

- **E1 — clippy gate unverifiable here**: `cargo clippy` is not installed in this environment, so clippy-completion task claims cannot be independently re-confirmed here. This is a tooling limitation, not a code gap; use `cargo check` + `cargo test` as the local gate unless clippy is installed.
- **E2 — uneven validation artifacts**: features 005, 008, 009, 013, and 014 have no `validation-results.md`. Their task lists are fully checked and tests pass, so this is optional documentation-lifecycle parity work, not blocking implementation work.
- **E3 — 007 is intentionally roadmap/process-only**: `007-outstanding-roadmap` lacks plan/tasks by design and should not be treated as an unimplemented feature.
