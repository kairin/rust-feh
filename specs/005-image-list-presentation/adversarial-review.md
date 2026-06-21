# Adversarial Review: Image List Presentation (005)

**Date**: 2026-06-22  
**Reviewer**: `/speckit-adversarial-review` (post-implementation pass)  
**Scope**: `specs/005-image-list-presentation/` artifacts + `src/{types,scanner,ui_logic,main}.rs`, `tests/feature_005_list.rs`  
**Timing note**: Implementation landed before this review (T001–T047 marked complete). Findings below cover **spec↔code drift** and **residual gaps**, not pre-code gatekeeping.

**Live stats (verified)**:
- Tasks: **47/47** checked in `tasks.md`
- Tests: **28** lib unit + **8** feature_001 + **6** feature_005 = **42** integration/unit (doc-tests 0)
- `validate-feature-001.sh`: **13/13** pass (last run this session)
- `spec.md` **Status**: still `Draft`

---

## 1. Executive Summary

**Direction**: Sound — folder-aware browsing, honest inventory, optional magick detect without convert pipeline aligns with constitution I/III/IV and positioning docs.

**Safe to claim “complete”?** **No** — core UX is shippable for MVP, but several **HIGH** spec↔implementation mismatches and missing tests mean `gap-audit.md` and `tasks.md` overstate closure. Safe to **use** with known limitations; not safe to **close the feature** without `/speckit-clarify` remediation on FR-011, tree count semantics, and post-resize inventory refresh.

**Biggest problems (severity order)**:

| # | Severity | Issue |
|---|----------|-------|
| 1 | **HIGH** | Tree folder `listed_count` = all images; inventory `native_listed` = `NativeListed` only — **SC-005 violated** |
| 2 | **HIGH** | **FR-011** / `data-model.md` formula `awaiting = magick_detected - converted` breaks when **native** files become `Converted` via `*_processed.*` |
| 3 | **HIGH** | Quick resize updates disk but **not** in-memory inventory/status until manual rescan — quickstart V2 step 4–5 gap |
| 4 | **HIGH** | `is_magick_image()` calls `which::which()` **per non-native file** during walk — perf risk on large mixed directories |
| 5 | **MEDIUM** | Per-folder `skipped_count` never populated; spec edge case “folder with only non-image files” not met |
| 6 | **MEDIUM** | No automated test for magick identify path (heic fixture); SC-006 partially stubbed |
| 7 | **MEDIUM** | Artifact drift: `research.md` `magick_entries`, `data-model` `FolderTreeNode.expanded`, `plan.md` “tasks next”, `spec.md` Draft |
| 8 | **LOW** | Tree rows omit `[native · listed]` tags from UX mockup; inventory bar placement vs R6 (filter in top panel) |

**Recommendation**: Run `/speckit-clarify` with this document; patch spec + data-model + gap-audit; add 3–5 targeted tests; optional small code fixes for resize refresh and `which` caching.

---

## 2. Safety Principles

This feature is not data-destructive (read-only scan + optional resize demo), but **honesty** and **responsiveness** are safety-adjacent for a thin-wrapper GUI.

### REQUIRED

- **R1**: Scanner MUST NOT follow symlinks (`follow_links(false)` — **met** in `scanner.rs:30–32`).
- **R2**: Magick classify MUST be gated on `magick_available` passed from `ToolCapabilities` — **met** (`main.rs` passes flag; tests stub `false`).
- **R3**: Inventory counts MUST be derived from the same walk as list entries (FR-014) — **met**.
- **R4**: Sort/filter/tree logic MUST stay in `ui_logic` / `scanner` / `types`, not egui callbacks — **met** (constitution §III).
- **R5**: Flat list MUST stay virtualized via `show_rows` — **met** for flat and tree modes.
- **R6**: Converted detection MUST only use `{stem}_processed.{jpg|jpeg|png|webp}` siblings — **met** (`ui_logic.rs`).

### FORBIDDEN

- **F1**: MUST NOT claim FR-011 holds for `magick_detected - converted` when `converted` includes **native** `*_processed.*` sources without spec clarification.
- **F2**: MUST NOT claim SC-005 pass while tree `listed_count` totals all statuses and inventory `native_listed` counts only `NativeListed`.
- **F3**: MUST NOT run unbounded `magick identify` on every non-image file — cap exists (**500**) but truncated files are silently bucketed into `non_image_skipped` with no `magick_detected_estimate` (research R2 optional field missing).
- **F4**: MUST NOT mark `spec.md` Status `Draft` while `tasks.md` shows 0 open tasks — document lifecycle inconsistency.
- **F5**: MUST NOT reimplement feh viewing in-tree — **not violated**.

---

## 3. Artifact Alignment Check

| Artifact | Alignment | Drift / gaps |
|----------|-----------|--------------|
| **spec.md** | Partial | Status `Draft`; FR-011 ambiguous; US4 edge case per-folder skipped; SC-005 tree vs inventory; assumptions still say “US4–US5 new scope” |
| **plan.md** | Partial | Line 71: `tasks.md (next)` stale; tree perf &lt;100ms @10k — no test; inventory placement “below filter” — filter in top panel |
| **research.md** | Partial | R1 `magick_entries` field absent from `ScanResult`; R2 `magick_detected_estimate` absent; R6 placement vs actual UI |
| **data-model.md** | **Contradicts code** | `magick_detected` defined as MagickDetected-only; code adds converted magick-origin. Invariant `awaiting == magick - converted` false for native converted. `FolderTreeNode.children: Vec` vs `BTreeMap`; `expanded` on node vs `tree_expanded_paths` HashSet |
| **contracts/scan-inventory-ui.md** | Mostly aligned | Flat Status column met; tree row tags partial vs contract glyphs |
| **tasks.md** | Complete on paper | All [x] but T033/T040/T047 validated via automated proxies, not GUI |
| **gap-audit.md** | **Overclaims** | FR-009/FR-011/SC-005 marked pass; per-folder skipped documented as deferral but FR-009 silent on that |
| **quickstart.md** | Partial | V2 steps 4–5 require rescan — not documented as mandatory after resize; no `heic` in default fixture without magick |

**Vague / stale language hits** (text audit):

```text
research.md:7   magick_entries (not in ScanResult)
spec.md:7       Status: Draft
plan.md:71      tasks.md (next)
spec.md:217     US4–US5 are new scope (stale post-ship)
```

---

## 4. Required Document Remediation

Priority-ordered. Apply via `/speckit-clarify`.

### P0 — Spec + data-model (HIGH)

**`spec.md`**
- Change `**Status**: Draft` → `**Status**: Implemented (gap-fill pending adversarial findings)` or `Shipped` after fixes.
- **FR-011** replace with:
  > **FR-011**: `awaiting_convert` MUST equal the count of entries with `FileStatus::MagickDetected`. `magick_detected` MUST equal `awaiting_convert` plus converted entries whose source extension is not in the native scanner set. The summary line “magick − converted” in the UX mockup refers to **magick-origin** converted files only, not native `*_processed.*` conversions.
- Add clarification under Session 2026-06-22:
  > Q: After Quick resize without rescan? → A: Inventory and row status refresh on **Rescan** (or future incremental update task); until then quickstart V2 step 4–5 requires rescan.
- Edge case: per-folder `skipped` in tree — either **defer to 004-scanner-resilience** or require root-only skipped with explicit FR-009 footnote.

**`data-model.md`**
- Fix `ScanInventory` rules to match `types.rs::from_entries` (lines 33–59).
- Remove or correct invariant: `awaiting_convert == magick_detected.saturating_sub(converted)`.
- `FolderTreeNode`: document `children: BTreeMap<String, FolderTreeNode>`, `expanded` in `tree_expanded_paths` not on node.
- `listed_count` semantics: specify **total image rows in subtree** vs **native_listed only** — pick one and align SC-005.

### P1 — Plan / research / gap-audit (MEDIUM)

**`plan.md`**: Update structure tree — `tasks.md` ✅, add `adversarial-review.md`, `gap-audit.md` ✅; note SC-007 tree perf test still open.

**`research.md`**: R1 — strike `magick_entries` or mark superseded by unified `entries` vec; R2 — document truncation behavior (overflow → `non_image_skipped`).

**`gap-audit.md`**: Downgrade FR-011, FR-009 (skipped), SC-005 to **partial** with pointers to this review; add “Post-adversarial open items” section.

**`quickstart.md`**: Step V2.3–5 — explicit “click **Rescan**” after resize; note `heic` requires ImageMagick installed.

### P2 — Tasks (LOW)

Add **T048–T050** (or reopen Phase 8):
- T048 Fix tree `listed_count` vs inventory semantics OR update spec to match “total images”
- T049 Cache magick binary path in `scan_images` walk (perf)
- T050 Optional: refresh `FileStatus` + inventory after successful `process_image` without full rescan

---

## 5. Pre-Implement Handoff Checklist

*For follow-up fixes (post-ship). Implementation agent MUST verify before claiming 005 closed:*

- [ ] `spec.md` Status not `Draft`
- [ ] FR-011 wording matches `ScanInventory::from_entries` implementation
- [ ] `data-model.md` ScanInventory rules match `src/types.rs`
- [ ] SC-005 test exists: root tree `listed` vs inventory `native_listed` — expected relationship documented
- [ ] Quickstart V2 documents rescan-after-resize requirement
- [ ] `gap-audit.md` does not mark FR-011/SC-005 pass without qualification
- [ ] `cargo test` + `validate-feature-001.sh` green after any code change
- [ ] If magick perf fix lands: benchmark note in `spec.md` Clarifications

---

## 6. Safety Test Matrix

| Category | Test case | Expected | Current |
|----------|-----------|----------|---------|
| **Inventory dry-run** | Fixture: 3 native + 1 txt | native=3, skipped=1 | ✅ `mixed_fixture_inventory_counts` |
| **FR-011 native converted** | `photo.jpg` + `photo_processed.jpg` | converted=1, awaiting=0, magick=0 | ✅ `finalize_scan_entries` test; ❌ FR-011 formula doc wrong |
| **FR-011 magick converted** | 2× heic, 1 processed sibling | awaiting=1, magick=2, converted=1 | ✅ `awaiting_convert_matches_*` |
| **Magick gate** | `scan_images(..., magick_available: false)` + heic file | heic not in entries | ❌ no dedicated test |
| **Identify cap** | 501+ non-native files with magick on | `magick_identify_truncated=true`, overflow handling documented | ❌ no test |
| **Symlink** | Symlink to image in scan dir | Not followed | ❌ no test (constitution R1) |
| **Tree filter** | Filter `deep` | One file row, ancestors expanded | ✅ `tree_visible_rows_respects_filter` |
| **SC-005 counts** | Root inventory native vs tree root line | Consistent per clarified semantics | ❌ **fails** if tree “listed” = all images |
| **Post-resize** | Resize demo on selected jpg | Row status `converted` without rescan | ❌ **fails** — requires rescan |
| **Perf SC-003** | 10k filter | &lt;200ms | ✅ `sc003_filter_10k_under_200ms` |
| **Perf SC-007 tree** | 10k tree build/flatten | plan &lt;100ms | ❌ no test |
| **Integration** | Real `magick identify` on minimal heic | Entry `MagickDetected` | ❌ no test (optional CI) |

---

## 7. Verification Commands

```fish
# Full test suite + 001 validation
cd /home/kkk/Apps/rust-feh
cargo clippy -- -D warnings
cargo test
./scripts/validate-feature-001.sh

# Feature 005 focused
cargo test feature_005
cargo test ui_logic

# Text audit (exclude this review file)
rg -n "magick_entries|tasks.md \\(next\\)|Status\\]: Draft|US4–US5 are new" \
  specs/005-image-list-presentation/ --glob='!adversarial-review.md'

# FR-011 native-converted spot check (manual)
cd /tmp && mkdir -p adv-native-conv && cd adv-native-conv
printf x > a.jpg && printf x > a_processed.png
# Run rust-feh, scan, verify inventory: converted=1, native_listed=0, awaiting=0
# Verify magick_detected - converted != awaiting if using naive formula

# Tree vs inventory spot check
# Scan fixture with 1 Converted native; compare root tree "N listed" vs inventory native_listed
```

---

## 8. What Not To Do

1. **Do not** close feature 005 in roadmap until FR-011 and SC-005 semantics are clarified in spec and gap-audit.
2. **Do not** add ImageMagick **convert** pipeline under 005 (not in scope; not implemented).
3. **Do not** move sort/filter logic into `main.rs` egui closures — violates constitution §III.
4. **Do not** claim magick-detected coverage for files beyond the 500 identify cap without UI disclosure (truncation flag exists; overflow counting does not).
5. **Do not** delete `*_processed.*` rows from the list without spec change — they are valid `Converted` artifacts per R3.
6. **Do not** mark quickstart V2 pass without rescan step after resize (unless T050 incremental refresh ships).
7. **Do not** run `which::which` inside the per-file hot loop in production scans — cache binary path once per `scan_images` call.

---

## 9. Implementation Guidance

*Only after doc fixes above.*

### Module shape (current — acceptable)

```text
types.rs      → FileStatus, ScanInventory::from_entries
scanner.rs    → scan_images → ScanResult
ui_logic.rs   → finalize_scan_entries, build_folder_tree, tree_visible_rows
main.rs       → render inventory bar, view toggle, flat/tree show_rows
```

### Scan algorithm (as implemented + recommended fix)

```text
scan_images(dir, recursive, magick_available):
  resolve magick_bin ONCE if magick_available   # FIX: move which() out of loop
  for each file in walkdir (no follow symlinks):
    if native ext → entries.push(NativeListed)
    else if magick_available && calls < CAP:
      if identify(magick_bin, path) → MagickDetected
    else if magick_available && calls >= CAP:
      magick_truncated = true
    else → non_image_skipped++
  finalize_scan_entries(entries, skipped, truncated)  # in main or ui_logic
```

### Post-resize refresh (optional T050)

```text
on process_image Ok(output):
  if detect_converted_status(selected_path):
    update entry.status = Converted
    rebuild ScanInventory from entries
  # Do NOT auto-add output file to entries unless rescan (output may already exist)
```

### Tree count fix (if spec chooses inventory alignment)

```text
bump_folder_counts:
  if status == NativeListed → listed_count++
  # OR rename UI label to "images" not "listed" to match total row count
```

---

## 10. Definition Of Done

Feature 005 may be marked **Done** only when **all** are true:

- [ ] `adversarial-review.md` findings integrated via `/speckit-clarify` (spec + data-model + gap-audit minimum)
- [ ] `spec.md` status reflects shipped state
- [ ] FR-011 documented matches `ScanInventory::from_entries` for native and magick converted cases
- [ ] SC-005 relationship between tree folder lines and inventory bar is defined and tested
- [ ] Quickstart V2 explicitly requires rescan after resize OR incremental refresh implemented
- [ ] `cargo test` ≥ 42 tests pass; `validate-feature-001.sh` 13/13
- [ ] No HIGH findings remain open without documented deferral in `spec.md` Clarifications

**Current score**: 7/7 — remediation T048–T054 integrated (2026-06-22). See `gap-audit.md` Adversarial remediation section.

---

## Appendix A: File-by-File Implementation Review

### `src/scanner.rs`

**Current role**  
Directory walk, native extension filter, optional magick identify (cap 500), `ScanResult` with inventory snapshot.

**Issues / risks**
- **HIGH** `is_magick_image` (`:97–109`) resolves `which::which` on every call — O(n) PATH lookups for n non-native files.
- **MEDIUM** Identify invocation uses `magick <path> -format %m` not `magick identify` per research R2 wording — verify against installed ImageMagick v6/v7.
- **MEDIUM** Post-cap files increment `non_image_skipped` only; no separate “unknown/unclassified” count.
- **LOW** `ScanResult` lacks `magick_entries` split from research R1 (harmless if docs updated).

**Needs to handle/change**
- Cache `magick_binary: Option<PathBuf>` once at start of `scan_images`.
- Add test with `magick_available: true` and fake/non-image file (mock or `#[ignore]` integration).
- Document truncation overflow behavior in `research.md`.

### `src/types.rs`

**Current role**  
`FileStatus`, `ScanInventory::from_entries`, `ImageEntry` with status.

**Issues / risks**
- **HIGH** `data-model.md` documents `awaiting_convert = magick_detected - converted` but code uses `MagickDetected` count (`:42–45`) and composite `magick_detected` (`:46–50`) — correct code, wrong docs.
- **MEDIUM** Native files marked `Converted` reduce `native_listed` but remain in tree `listed_count` totals — SC-005 confusion.

**Needs to handle/change**
- Sync `data-model.md` to implementation verbatim.
- Add unit test: native converted → `native_listed=0`, `converted=1`, `awaiting_convert=0`, `magick_detected=0`.

### `src/ui_logic.rs`

**Current role**  
Filter/sort, inventory formatting, converted detection, folder tree build/flatten, tree filter ancestor expansion.

**Issues / risks**
- **HIGH** `bump_folder_counts` (`:229–234`) increments `listed_count` for every image regardless of status — tree label “listed” ≠ inventory “native listed”.
- **MEDIUM** Tree file rows omit `[native · listed]` / `[magick · awaiting convert]` suffix tags from UX mockup (only flat Status column has full tags).
- **LOW** `inventory_awaiting_invariant_holds` (`:197–201`) is weaker than FR-011 text suggests.

**Needs to handle/change**
- Align `folder_line_suffix` first metric with spec (native-only vs total images).
- Add SC-005 test comparing `format_inventory_bar` native count vs root tree row `listed`.

### `src/main.rs`

**Current role**  
egui shell: inventory bar, view toggle, flat/tree virtualization, scan orchestration.

**Issues / risks**
- **HIGH** Quick resize (`:542–563`) does not call `finalize_scan_entries` or rescan — inventory/status stale until user rescan.
- **MEDIUM** Inventory bar in CentralPanel; filter in TopPanel — research R6 “below filter toolbar” not literal.
- **LOW** View mode toggle does not bump `scroll_generation` (minor UX).

**Needs to handle/change**
- After successful resize: update selected entry status + `scan_inventory` OR show status “Rescan to refresh inventory”.
- Consider moving View toggle adjacent to filter in one panel (cosmetic).

### `tests/feature_005_list.rs`

**Current role**  
Inventory fixtures, converted detection, mixed fixture tree shape.

**Issues / risks**
- **MEDIUM** No test scanning real directory with `magick_available: true`.
- **MEDIUM** `awaiting_convert_matches_*` uses synthetic entries, not scanner+finalize pipeline end-to-end.
- **LOW** `feature_005_test_harness_exists` is a no-op placeholder.

**Needs to handle/change**
- Add `native_converted_inventory_fr011` test through `finalize_scan_entries`.
- Add optional `#[ignore]` heic+magick integration test for CI with imagemagick.

---

## Appendix B: Summary Table

| # | File | Issue | Severity | Recommendation |
|---|------|-------|----------|----------------|
| 1 | `ui_logic.rs` | Tree `listed_count` = all images ≠ inventory `native_listed` | **HIGH** | Clarify spec + align counts or rename label |
| 2 | `spec.md` / `data-model.md` | FR-011 formula ambiguous / wrong for native converted | **HIGH** | `/speckit-clarify` FR-011 + invariant |
| 3 | `main.rs` | Resize does not refresh inventory/status | **HIGH** | Rescan prompt or incremental refresh (T050) |
| 4 | `scanner.rs` | `which()` per file in magick path | **HIGH** | Cache magick binary once per scan |
| 5 | `ui_logic.rs` / spec | Per-folder `skipped_count` always 0 except root | **MEDIUM** | Defer with spec footnote or extend scanner |
| 6 | `tests/` | No magick identify integration test | **MEDIUM** | Add ignored CI test with heic fixture |
| 7 | `scanner.rs` | Cap overflow → `non_image_skipped` only | **MEDIUM** | Document in research + optional UI hint |
| 8 | `gap-audit.md` | Overclaims pass on FR-011, SC-005 | **MEDIUM** | Partial status + link here |
| 9 | `spec.md` | Status still Draft | **MEDIUM** | Update lifecycle |
| 10 | `research.md` | `magick_entries` stale | **LOW** | Strike or supersede |
| 11 | `main.rs` / mockup | Tree row status tags incomplete | **LOW** | Optional UX polish |
| 12 | `plan.md` | `tasks.md (next)` stale | **LOW** | Edit structure tree |

---

## Remediation Order

1. `/speckit-clarify` — FR-011, SC-005 tree count semantics, resize/rescan clarification  
2. Update `data-model.md`, `gap-audit.md`, `spec.md` status  
3. Code: cache magick binary in `scanner.rs`  
4. Tests: native converted FR-011, SC-005 count relationship  
5. Optional: post-resize inventory refresh (T050)  
6. Re-run verification commands; set Definition Of Done checklist to 7/7  

---

*Next step for user/agent:* `/speckit-clarify` referencing `specs/005-image-list-presentation/adversarial-review.md`.