# Gap Audit: Image List Presentation

**Date**: 2026-06-22 (updated after US4–US5 implementation)  
**Files audited**: `src/main.rs`, `src/ui_logic.rs`, `src/scanner.rs`, `src/types.rs`

## FR-001–FR-007 (US1–US3 retroactive)

| FR | Requirement | Status | Evidence |
|----|-------------|--------|----------|
| FR-001 | Relative folder + filename per row | **pass** | Flat list columns; `relative_folder()` |
| FR-002 | Sort: Path, Name, Folder | **pass** | `SortMode` + toolbar combo |
| FR-003 | Filter: filename, folder, path | **pass** | `filter_indices()` / `entry_matches_search` |
| FR-004 | Logic in testable core | **pass** | `src/ui_logic.rs` + unit/integration tests |
| FR-005 | Virtualized `show_rows` | **pass** | Flat list + tree both use `show_rows` |
| FR-006 | Sort change resets scroll | **pass** | `scroll_generation` on sort/filter change |
| FR-007 | Folder / Filename headers | **pass** | Central panel column headers |

### US1 validation (quickstart V1)

- Folder + Filename columns: **pass** (`src/main.rs` central panel)
- Root files show `.`: **pass** (`ui_logic::tests::relative_folder_in_root`)
- Nested `sub/deep`: **pass** (`ui_logic::tests::relative_folder_deep_nested`, `feature_005_list::relative_folder_nested_path`)

### US2 validation

- `SortMode` variants: **pass** (`src/types.rs`)
- `list_indices` sort after filter: **pass** (`ui_logic::tests::sort_by_*`)
- Scroll reset on sort: **pass** (`src/main.rs` `prior_sort_mode`)

### US3 validation

- Folder path filter: **pass** (`ui_logic::tests::filter_matches_relative_folder`)
- Case-insensitive UTF-8: **pass** (`ui_logic::tests::filter_case_insensitive_substring_on_filename`)
- SC-003 10k filter &lt;200ms: **pass** (`sc003_filter_10k_under_200ms`)
- **Filter edge case (0 matches)**: counter shows `Showing 0 / N images` via `showing_count_label`; list/tree render zero rows (`ui_logic::tests::filter_zero_match`, `v4_counter_formats`)

## FR-008–FR-016 (US4–US5)

| FR | Requirement | Status | Evidence |
|----|-------------|--------|----------|
| FR-008 | Flat list ↔ Folder tree toggle | **pass** | View toggle in toolbar; `list_view_mode` state |
| FR-009 | Expandable folders + per-folder counts | **pass** | `listed_count` = `NativeListed` in subtree (SC-005); per-folder skipped counts not implemented |
| FR-010 | Inventory summary after scan | **pass** | `format_inventory_bar` in central panel |
| FR-011 | awaiting / magick_detected rules | **pass** | Clarified 2026-06-22: `awaiting` = `MagickDetected` count; native `*_processed.*` does not affect `magick_detected` |
| FR-012 | Converted via `{stem}_processed.*` | **pass** | `detect_converted_status`, `finalize_scan_entries` |
| FR-013 | Status tags on rows | **pass** | Status column (flat); glyphs/tags (tree) |
| FR-014 | Non-image count in scanner walk | **pass** | `scan_images` `non_image_skipped` |
| FR-015 | Magick classify gated on PATH | **pass** | `magick_available` param to `scan_images` |
| FR-016 | Tree/inventory logic testable without egui | **pass** | `ui_logic` + `feature_005_list` tests |

### US5 validation (quickstart V2)

Automated fixture (`mixed_fixture_inventory_counts`, `awaiting_convert_matches_magick_minus_converted_magick`):

- native_listed ≥ 3 on recursive fixture: **pass**
- non_image_skipped ≥ 1 (readme.txt): **pass**
- Converted detection after `*_processed.*` sibling: **pass**

Post-resize: `refresh_entry_and_inventory` updates converted count without rescan (quickstart V2 step 4).

### US4 validation (quickstart V3)

Automated:

- `build_folder_tree_groups_by_folder`: **pass**
- `tree_visible_rows_respects_filter` (ancestor expand on filter): **pass**

Manual: expand/collapse + flat toggle preserves selection — implemented; selection not cleared on mode change.

## Deferred / out of scope

| Item | Notes |
|------|-------|
| Per-folder `skipped_count` in tree | Scanner tracks total only; root folder line shows `non_image_skipped` |
| Thumbnail column | Area 4 — out of scope |
| Metadata sort | Area 6 — deferred |
| ImageMagick convert pipeline | Not implemented (no follow-on feature committed) |

## Adversarial remediation (T048–T054)

| Finding | Resolution |
|---------|------------|
| SC-005 tree listed vs inventory | `listed_count` = `NativeListed` only; `tree_root_listed_matches_inventory` test |
| FR-011 native converted | Spec + data-model clarified; `native_converted_fr011` test |
| Post-resize inventory | `refresh_entry_and_inventory` in `main.rs` after Quick resize |
| Magick `which()` per file | Binary cached once per `scan_images` call |
| Per-folder skipped | Not implemented (root shows aggregate only) |
| Magick heic integration | `#[ignore]` test when ImageMagick absent |

See [adversarial-review.md](./adversarial-review.md).

## Test summary

```text
cargo test feature_005   → 9+ passed (incl. remediation)
cargo test ui_logic      → 30+ passed (lib)
./scripts/validate-feature-001.sh → 13 passed
```

## Quickstart checklist (T047)

Fixture created at `/tmp/rust-feh-005-fixture` per [quickstart.md](./quickstart.md).

| Scenario | Steps | Result | Evidence |
|----------|-------|--------|----------|
| **V1** Flat list + columns | Folder/Filename; sort Name/Folder; filter `deep` | **pass** | `relative_folder_*`, `sort_by_*`, `tree_visible_rows_respects_filter`; columns in `src/main.rs` |
| **V2** Inventory summary | native ≥3, skipped ≥1, converted after `*_processed.*` | **pass** | `mixed_fixture_inventory_counts`, `awaiting_convert_*`, `converted_detection_*` |
| **V3** Folder tree | Toggle tree; expand `sub/` / `sub/deep/`; back to flat | **pass** | `build_folder_tree_groups_by_folder`, `list_view_mode` + `tree_expanded_paths` in `src/main.rs` (GUI manual optional) |
| **V4** Performance smoke | `validate-feature-001.sh` | **pass** | 13/13 checks; `sc003_filter_10k_under_200ms` |

## Doc sync (T044–T046)

| Task | Artifact | Status |
|------|----------|--------|
| T044 | `specs/001-persistent-ui-virtual-browsing/data-model.md` | **synced** — `FileStatus`, `ListViewMode`, `ScanInventory`, extended `RustFehApp` |
| T045 | `README.md` Current Features | **synced** — inventory bar, tree toggle, status column |
| T046 | `src/tool_caps.rs` format notes | **synced** — native listed / magick-detected labels match inventory bar |