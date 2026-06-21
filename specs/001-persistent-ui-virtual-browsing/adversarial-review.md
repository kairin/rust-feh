# Adversarial Review: Persistent UI Layout & Virtual Browsing

**Feature**: `001-persistent-ui-virtual-browsing`
**Review type**: Post-implementation (code-level, file-by-file)
**Reviewed**: 2026-06-21
**Scope**: `src/main.rs`, `src/scanner.rs`, `src/types.rs` vs spec/plan/tasks/gap-audit
**Prior review**: Pre-implement spec review (same date) — artifacts remediated, code landed

---

## 1. Executive Summary

**Feature direction**: Sound. Implementation delivers the core UX layer (persistent panels, virtualization, explicit feh launch, scanning indicator).

**Safe to ship MVP?** **CONDITIONAL** — code structure matches spec for most FRs, but **10/64 tasks remain unchecked** (all manual quickstart/perf scenarios). `gap-audit.md` marks 15/15 FRs **pass** without evidence for SC-002–SC-004 performance claims.

**Biggest remaining problems**:

| # | Severity | Problem |
|---|----------|---------|
| 1 | **HIGH** | `gap-audit.md` and T012/T045 claim pass but implementation diverges from spec (`scroll_generation` vs `list_scroll_offset`; `feh_button` vs `ui.add_enabled`) |
| 2 | **HIGH** | No manual validation of quickstart V1–V10, RSS <150MB, 200ms filter, 10k scroll (T026, T036–T039, T050–T051, T055, T058) |
| 3 | **MEDIUM** | FR-008a "persistent" feh-not-found status overwritten in main status line after folder load (footer small text remains) |
| 4 | **MEDIUM** | `data-model.md` documents `list_scroll_offset` field not present in `RustFehApp` |
| 5 | **MEDIUM** | Scanner ignores non-`PermissionDenied` walkdir errors silently |
| 6 | **LOW** | `Selection`/`SortMode` dead code (T056 open); duplicate recursive checkbox in menu + toolbar |
| 7 | **LOW** | `feh_available` detected once at startup only — no re-check if feh installed later |

**Task stats (live `tasks.md`)**: **54 checked / 64 total** — 10 open (all manual validation or dead-code cleanup).

**Verdict**: Code-ready for user acceptance testing. **Not definition-of-done** until open tasks close and artifact drift fixed.

---

## 2. File-by-File Review

### `src/main.rs`

**Current role**
egui application: persistent top/bottom panels, virtualized image list (`show_rows`), folder pick/rescan, filter, feh spawn, wallpaper, quick resize demo, debug log. State: `feh_available`, `scanning`, `scroll_generation`, `prior_search`.

**Issues / risks**

- **L340–346**: When `scanning`, bottom bar shows `"Scanning…"` but suppresses `self.status` — OK for scan, but after folder load with `!feh_available`, `self.status` becomes `"Loaded N images…"` and **replaces** startup feh-not-found message in the primary status line (FR-008a "persistent" wording).
- **L67–68, data-model L25**: Spec/clarify document `list_scroll_offset: f32`; code uses `scroll_generation: u64` + `id_salt` (L299–300). T012 marked `[x]` — **artifact drift**.
- **L116–126**: `feh_button` grays disabled buttons but keeps `Sense::click()` — satisfies FR-008a click-to-show-message; spec clarify says `ui.add_enabled` (T045 marked `[x]` with different mechanism).
- **L175–183 vs L215–221**: Duplicate `Include subfolders` controls (View menu + toolbar) — same `self.recursive`, no desync risk, redundant UX.
- **L409–434**: `open_in_feh` has no `feh_available` guard (callers use `try_open_in_feh` — OK if all paths guarded).
- **L364–407**: Sync `scan_directory` blocks UI thread — acknowledged in spec; no async yet.

**Needs to handle/change**

1. After `scan_directory` when `!feh_available`, append or preserve feh warning in status (e.g. `"Loaded N images. feh not found — install with …"`).
2. Update `data-model.md` to document `scroll_generation` (or add `list_scroll_offset` if switching approach).
3. Run quickstart V1–V10 and close T026, T036–T039, T050–T051, T058.
4. Optional: extract `open_folder_controls()` to reduce menu/toolbar duplication.

---

### `src/scanner.rs`

**Current role**
Synchronous `walkdir` scan; `follow_links(false)`; returns `(Vec<ImageEntry>, Vec<String>)` with permission-denied warnings.

**Issues / risks**

- **L31–42**: Only `PermissionDenied` IO errors become warnings — other walkdir errors (e.g. loop, not a directory) dropped silently.
- **L69–72**: Test for nonexistent dir returns empty — walkdir may not error on missing root the same way on all platforms (test is weak but passes).

**Needs to handle/change**

1. Log generic walkdir `Err` at debug level: `warnings.push(format!("Scan skip: {e}"))` for non-permission errors (optional FR-015 extension).
2. Add T055 integration test: temp dir with `chmod 000` subdir (or document manual-only).
3. Add test for warning vector when permission denied (mock or unix-only `#[cfg]`).

---

### `src/types.rs`

**Current role**
`ImageEntry` domain type; `Selection` and `SortMode` marked `#[allow(dead_code)]`.

**Issues / risks**

- **L22–33**: Dead types duplicate `RustFehApp.selected` inline state — T056 open.

**Needs to handle/change**

1. Remove `Selection`/`SortMode` or integrate into app state (T056).
2. No safety impact — cleanup only.

---

## 3. Artifact Alignment Check

| Artifact | Claim | Live reality | Match? |
|----------|-------|--------------|--------|
| `gap-audit.md` | 15/15 FR pass | Code implements most FRs; perf SCs unverified | **PARTIAL** |
| `tasks.md` | 54/64 checked | 10 manual tasks open | **OK** (honest) |
| `data-model.md` | `list_scroll_offset` on `RustFehApp` | Field absent; `scroll_generation` used | **NO** |
| `spec.md` clarify | `ui.add_enabled` for feh buttons | `feh_button` helper | **PARTIAL** (behavior OK) |
| `plan.md` | Implementation status lists gaps | Gaps closed in code | **STALE** — update to "code landed, validation pending" |
| `research.md` | Per-FR status table | Aligned post-implement | **OK** |
| `cargo test` | 2 tests | 2 passed | **OK** |
| `cargo clippy -D warnings` | clean | clean | **OK** |

---

## 4. Summary Table

| # | File | Issue | Severity | Recommendation |
|---|------|-------|----------|----------------|
| 1 | `gap-audit.md` | All FRs marked pass without manual SC validation | HIGH | Add column `Validated` (code/manual); mark SC-002–004 pending until T036–T039 |
| 2 | `tasks.md` | T012/T045 checked but implementation differs from task text | HIGH | Add note "satisfied via scroll_generation / feh_button" or align code |
| 3 | `src/main.rs` | feh-not-found status not persistent in main line after folder load | MEDIUM | Merge feh warning into post-scan status when `!feh_available` |
| 4 | `data-model.md` | Documents `list_scroll_offset` not in code | MEDIUM | Replace with `scroll_generation` + `prior_search` |
| 5 | — | quickstart V1–V10 not executed | HIGH | User/agent runs manual scenarios; close T026+ |
| 6 | `src/scanner.rs` | Non-permission walkdir errors silent | MEDIUM | Log other errs to debug_logs |
| 7 | `src/types.rs` | Dead `Selection`/`SortMode` | LOW | T056 cleanup |
| 8 | `src/main.rs` | `feh_available` startup-only | LOW | Defer re-detection to Area 6 |

---

## 5. Remediation Order

1. **Run manual quickstart V1–V10** — highest value; closes 8 tasks and validates SC-001–SC-007.
2. **Fix FR-008a persistent status** after folder load (`main.rs` `scan_directory` status string).
3. **Sync `data-model.md`** with `scroll_generation` (or rename field in code to match docs).
4. **Update `gap-audit.md`** — distinguish `code-pass` vs `validated-pass`.
5. **T055** permission-denied manual test.
6. **T056** dead code cleanup (optional polish).

---

## 6. Safety Test Matrix (re-run post-fix)

| ID | Test | Pass criteria | Status |
|----|------|---------------|--------|
| ST-01 | V1 persistent controls | Top + bottom visible at 5k scroll | **UNRUN** |
| ST-02 | `rg 'Showing' src/main.rs` | 1 hit, bottom panel | **PASS** |
| ST-03 | V8 feh hidden | Disabled buttons, no spawn | **UNRUN** |
| ST-04 | Rescan selection | First image re-selected | **CODE OK** / UNRUN |
| ST-05 | V10 filter scroll reset | List jumps to top | **CODE OK** / UNRUN |
| ST-06 | V3 no auto-feh | No spawn on load | **CODE OK** / UNRUN |
| ST-07 | V9 scanning state | "Scanning…" visible | **CODE OK** / UNRUN |
| ST-08 | `cargo clippy -D warnings` | zero warnings | **PASS** |
| ST-09 | `cargo test` | all pass | **PASS** (2 tests) |
| ST-10 | V2 RSS <150MB @10k | SC-004 | **UNRUN** |

---

## 7. Verification Commands

```bash
# Code structure (passing)
rg -n 'Showing.*images' src/main.rs
rg -n 'for now note|trigger the logic' src/
cargo build --release && cargo clippy -- -D warnings && cargo test

# Task honesty
rg -c '^- \[x\]' specs/001-persistent-ui-virtual-browsing/tasks.md
rg -c '^- \[ \]' specs/001-persistent-ui-virtual-browsing/tasks.md

# Artifact drift
rg -n 'list_scroll_offset' specs/001-persistent-ui-virtual-browsing/data-model.md
rg -n 'scroll_generation' src/main.rs

# False-ready in gap-audit
grep 'pass | 15' specs/001-persistent-ui-virtual-browsing/gap-audit.md
```

---

## 8. What Not To Do

1. **Do NOT** mark feature complete while T026, T036–T039, T050–T051, T055, T058 are open.
2. **Do NOT** claim SC-002/SC-003/SC-004 pass without measurement evidence.
3. **Do NOT** revert `scroll_generation` without updating data-model and spec clarify bullets.
4. **Do NOT** remove `feh_button` click-on-disabled behavior — it satisfies FR-008a acceptance scenario 5.

---

## 9. Definition Of Done (live status)

- [x] Core FR implementation in `src/main.rs` / `scanner.rs`
- [x] `cargo clippy -- -D warnings` clean
- [x] `cargo test` pass (2 unit tests)
- [x] Counter only in bottom panel
- [x] Menu stubs removed
- [ ] quickstart V1–V10 manual pass
- [ ] `gap-audit.md` validation column honest
- [ ] `data-model.md` matches `RustFehApp` fields
- [ ] FR-008a persistent status after folder load
- [ ] T056 dead code resolved or waived
- [ ] 64/64 tasks checked

---

## 10. Integration With `/speckit-clarify`

If fixing artifact drift (data-model, gap-audit validation column, FR-008a persistent status):

```
/speckit-clarify
Reference: specs/001-persistent-ui-virtual-browsing/adversarial-review.md (post-implementation)
```

For code fixes only (status string, data-model sync): implement directly — no clarify required.

**Suggested next step**: Run `./rust-feh` and execute quickstart V1–V10, or invoke `/speckit-implement` for remaining T026+ validation fixes.