# Research: External Tool Runtime (009)

**Date**: 2026-06-22 (updated post-008)

## R1: Single detect function

**Decision**: `ToolCapabilities::detect()` remains the sole PATH lookup entry; `refresh_tool_caps()` assigns snapshot + mirrors `feh_available`.

**Rationale**: Avoid duplicate `which` calls; 008 established `tool_caps.rs` as source of truth.

**Alternatives considered**:
- Separate `detect_feh()` / `detect_magick()` — rejected (two lookups OK inside one struct method)
- Re-detect feh only on recheck — rejected (spec FR-005 requires both tools in one operation)

## R2: Tools menu vs panel recheck

**Decision**: Same `refresh_tool_caps()` handler; menu item label **Recheck tools on PATH** (matches panel button and 008 contract).

**Rationale**: Satisfies 002 FR-002 and 009 FR-004 with one code path; idempotent per user click.

**Alternatives considered**:
- Menu label "Re-check feh" only — rejected (009 unifies feh + magick)
- Separate menu items for feh and magick — rejected (unnecessary; one snapshot)

## R3: Spawn failure classification

**Decision**: Mark feh unavailable only when:
1. `ErrorKind::NotFound`, **or**
2. Error message contains `"No such file"` (platform variance), **and**
3. Confirming `which::which("feh").is_err()` (re-lookup guard)

Do **not** flip on display/X11 permission errors when feh still exists on PATH.

**Rationale**: 009 US3 acceptance scenario 2 — distinguish missing binary from runtime failures.

**Alternatives considered**:
- Flip on any spawn error — rejected (false unavailable on transient errors)
- Always re-run full `refresh_tool_caps()` on spawn failure — acceptable alternative; plan uses targeted flip + optional full refresh

## R4: State sync on spawn failure

**Decision**: Update **both** `self.feh_available` and `self.tool_caps.feh_available` so panel `dependencies()` and `operation_timings()` reflect unavailable on next frame without calling full `detect()` for magick.

**Rationale**: Panel reads `self.tool_caps`; mirroring field alone is insufficient.

**Alternatives considered**:
- Call `refresh_tool_caps()` on spawn failure — works but re-runs magick lookup unnecessarily; targeted sync is sufficient when only feh failed

## R5: Magick recheck vs scan inventory

**Decision**: Recheck updates `tool_caps.magick_available` only; **does not** auto-rescan loaded folder.

**Rationale**: Spec edge case "recheck during scan must not block scan"; rescan is explicit user action (Rescan button). Panel format notes update immediately; inventory counts update on next scan.

**Alternatives considered**:
- Auto-rescan on magick detect change — rejected (scope creep; could surprise user mid-browse)

## R6: Test strategy

**Decision**:
- Keep `tool_caps::detect` smoke tests (existing 9 tests from 008)
- Add pure helper test for `is_feh_not_found(err)` if extracted to `tool_caps.rs` or `ui_logic.rs`
- Manual quickstart for menu discoverability and spawn failure (US2/US4)

**Rationale**: Spawn uses `Command`; unit test the classification logic without subprocess mocks.

## R7: Converge → plan → implement workflow

**Decision**: After `/speckit-converge`, re-run `/speckit-plan` to add a **Convergence consolidation** section to `plan.md` mapping findings (F1–Fn) → tasks (Txxx) → code paths. Do **not** rewrite `tasks.md` during re-plan; implement reads both `plan.md` consolidated order and `tasks.md` checkboxes.

**Rationale**: Converge is append-only on tasks; re-plan absorbs partial/outstanding outcomes so the next `/speckit-implement` has one ordered backlog without duplicate task IDs.

**Alternatives considered**:
- Re-run `/speckit-tasks` to merge converge items — rejected (would renumber/reorder existing tasks)
- Implement converge tasks immediately without re-plan — rejected (implement order P2 classifier before P3 spawn is not obvious from tasks alone)