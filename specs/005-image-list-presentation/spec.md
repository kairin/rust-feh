# Feature Specification: Image List Presentation

**Feature Branch**: `005-image-list-presentation`

**Created**: 2026-06-21

**Status**: Implemented (adversarial remediation 2026-06-22)

**Input**: Dogfood gap from feature 001 — list showed **filenames only**; users expect **folder locations** and **sorting**. Code was added post-001 without a spec; this feature formalizes requirements and closes any remaining gaps.

**Parent**: [001-persistent-ui-virtual-browsing](../001-persistent-ui-virtual-browsing/spec.md) (US2, FR-005, FR-006)  
**Related**: [008-tool-capabilities-panel](../008-tool-capabilities-panel/spec.md) (ImageMagick optional for detect)

## Clarifications

### Session 2026-06-22

- Q: How should folder location be shown beyond the flat Folder column? → A: **Toggle Flat list / Folder tree** — flat view keeps current virtualized columns; tree view shows expandable hierarchy with per-folder counts.
- Q: What does "files not converted to images" mean? → A: **Non-image files** encountered during directory walk (e.g. txt, pdf, mp4) — counted separately, not listed as selectable rows.
- Q: What does ImageMagick "detected vs converted" mean? → A: **Magick-detected (unlisted)** = files ImageMagick identifies as images but not in native scanner extensions; **Converted** = row has `FileStatus::Converted` (in-app `{stem}_processed.*` sibling or artifact row); **Awaiting convert** = count of `MagickDetected` rows still without processed output — see **FR-011** for `magick_detected` vs magick-origin converted.
- Q: Where does the inventory summary appear? → A: **Above the list/tree** in the central browsing region after each scan completes.

### Session 2026-06-22 (adversarial remediation)

- Q: What does FR-011 “awaiting = magick − converted” mean when native files become Converted? → A: **`awaiting_convert`** = count of `FileStatus::MagickDetected` entries. **`magick_detected`** = `awaiting_convert` plus `Converted` entries whose extension is **not** in the native scanner set. The UX mockup line “magick − converted” applies to **magick-origin** converted files only, not native `*_processed.*` conversions.
- Q: What does tree folder “N listed” mean vs inventory `native_listed` (SC-005)? → A: **Same metric** — per-folder **listed** = count of `NativeListed` entries in that subtree. Magick and converted rows appear as file lines but do not increment **listed**. Root folder **skipped** shows scan-wide `non_image_skipped`; per-subfolder skipped counts deferred to feature **004**.
- Q: After Quick resize without full rescan? → A: App **incrementally updates** the selected entry’s `FileStatus` and rebuilds `ScanInventory` when `{stem}_processed.*` is detected. Full rescan still refreshes the entire list (e.g. new `*_processed.*` artifact rows).
- Q: Per-folder non-image skipped on tree lines? → A: **Deferred** — only root folder line shows total `non_image_skipped` until **004-scanner-resilience** adds per-folder walk stats.

## UX Reference: Folder Tree & Inventory (target layout)

Text mockup of the **Folder tree** view after scanning `/home/user/Photos` (recursive):

```text
┌─ Scan inventory ─────────────────────────────────────────────────────────┐
│ Root: /home/user/Photos                                                │
│   Images listed (native) .............. 1,247   jpg png webp gif bmp   │
│   Magick-detected (unlisted) ............. 18   heic svg tiff …         │
│   Converted (processed output exists) ..... 6   *_processed.*          │
│   Awaiting convert ....................... 12   magick − converted       │
│   Non-image files skipped ............... 342   txt pdf mp4 …            │
└────────────────────────────────────────────────────────────────────────┘

View: (•) Folder tree   ( ) Flat list          Filter: [ vacation      ]

▼ Photos/  ─────────────────────────── 1,247 listed │ 18 magick │ 342 skipped
  ▼ 2024/  ─────────────────────────────────────── 420 listed │ 4 magick
    ▼ vacation/  ───────────────────────────────── 312 │ 4 magick │ 0 skipped
      ● IMG_001.jpg                          [native · listed]
      ● IMG_002.jpg                          [native · listed]
      ○ DSC_0402.heic                        [magick · awaiting convert]
      ● sunset_processed.jpg                 [converted]
    ▶ work/  ──────────────────────────────────── 108 listed
  ▶ archive/  ───────────────────────────────────── 827 listed

Status: Showing 312 images in `2024/vacation/` │ Selected: IMG_001.jpg
```

**Legend**

| Glyph / tag | Meaning |
|-------------|---------|
| `▼` / `▶` | Folder expanded / collapsed |
| `●` | Native listed image (in scanner set; selectable) |
| `○` | Magick-detected image, not yet converted (selectable when magick present; view may need convert) |
| `[converted]` | Processed output file exists for source or row is a `*_processed.*` artifact |
| `[native · listed]` | In scanner extension set |
| `[magick · awaiting convert]` | ImageMagick identifies as image; not in native scanner list; no processed output yet |
| Counts on folder lines | Listed images in subtree; magick-detected; non-image skipped (when scan collects them) |

**Flat list view** (current shipped UX, retained as toggle option):

```text
┌─ Scan inventory ─ (same summary bar as above) ───────────────────────────┐

Folder          │ Filename              │ Status
────────────────┼───────────────────────┼─────────────────────
2024/vacation   │ IMG_001.jpg           │ native
2024/vacation   │ IMG_002.jpg           │ native
2024/vacation   │ DSC_0402.heic         │ magick · awaiting
2024/vacation   │ sunset_processed.jpg  │ converted
```

## User Scenarios & Testing *(mandatory)*

### User Story 1 - See Where Each Image Lives (Priority: P1)

A user scans a folder with subdirectories. The list shows each image's folder path relative to the scan root alongside its filename, so they can distinguish `vacation/img.jpg` from `work/img.jpg`.

**Why this priority**: Filename-only lists are ambiguous in recursive scans — core browsing UX.

**Independent Test**: Load `root/a/x.jpg` and `root/b/x.jpg`; both rows distinguish folders.

**Acceptance Scenarios**:

1. **Given** a loaded scan root, **When** viewing the list, **Then** each row shows **Folder** and **Filename** columns.
2. **Given** a file directly in the scan root, **When** displayed, **Then** folder column shows `.` (or equivalent "root" indicator documented in quickstart).
3. **Given** a nested path `root/sub/deep/img.png`, **When** displayed, **Then** folder column shows `sub/deep` relative to root.

---

### User Story 2 - Sort the List (Priority: P1)

A user wants to group images by subdirectory or sort by filename without leaving the app.

**Why this priority**: Scanner default path sort is insufficient for browsing by name or folder.

**Independent Test**: Change Sort dropdown; order changes predictably; scroll resets.

**Acceptance Scenarios**:

1. **Given** a loaded folder, **When** user selects Sort **Name**, **Then** rows order by filename case-insensitively.
2. **Given** a loaded folder, **When** user selects Sort **Folder**, **Then** rows group by relative folder then filename.
3. **Given** a loaded folder, **When** user selects Sort **Path**, **Then** rows order by full path (legacy scanner order).
4. **Given** sort mode changes, **When** list updates, **Then** scroll position resets to top (same policy as filter change).

---

### User Story 3 - Filter Matches Paths (Priority: P2)

A user types a subdirectory name in the filter to narrow the list without scrolling.

**Why this priority**: FR-005/FR-006 in 001 mentioned filenames only; folder-aware filter matches user mental model.

**Independent Test**: Filter `"vacation"` returns only images under matching folders.

**Acceptance Scenarios**:

1. **Given** images in folders `vacation` and `work`, **When** filter is `vacation`, **Then** only vacation paths appear.
2. **Given** filter matches filename, **When** applied, **Then** filename substring match still works (case-insensitive).
3. **Given** 10k images, **When** filter applied, **Then** response remains under 200ms (inherits SC-003 from 001).

---

### User Story 4 - Browse by Folder Tree (Priority: P1)

A user scans a recursive tree with many subfolders. They want to expand/collapse folders, see image counts per folder, and select files in context — not only a flat Folder column.

**Why this priority**: Folder column helps but does not show hierarchy, subtree counts, or magick/non-image breakdown per directory.

**Independent Test**: Load nested fixture; switch to Folder tree view; expand `2024/vacation`; counts and files match scan inventory.

**Acceptance Scenarios**:

1. **Given** a loaded scan, **When** user selects **Folder tree** view, **Then** folders render as an expandable text hierarchy relative to scan root with per-folder listed-image counts.
2. **Given** a folder row in the tree, **When** expanded, **Then** child folders and image files appear indented beneath it per UX Reference mockup.
3. **Given** user toggles back to **Flat list**, **When** view changes, **Then** existing Folder + Filename columns and virtualization behavior are preserved.
4. **Given** filter is active, **When** in tree view, **Then** tree and counts reflect filtered subset only.

---

### User Story 5 - Scan Inventory Summary (Priority: P1)

After scanning, a user wants totals: how many native images were listed, how many non-image files were skipped, how many ImageMagick-detected images are awaiting convert, and how many already have processed output.

**Why this priority**: Supports positioning honesty — separates native list, magick bridge, and skipped files.

**Independent Test**: Scan fixture with jpg + heic + readme.txt; summary shows native=1, magick-detected=1, non-image=1 (when magick present).

**Acceptance Scenarios**:

1. **Given** scan completes, **When** browsing region renders, **Then** inventory summary shows: native listed count, magick-detected (unlisted) count, converted count, awaiting convert count, non-image skipped count.
2. **Given** ImageMagick is not on PATH, **When** summary renders, **Then** magick-detected and awaiting convert show zero with note that install enables detection (or section hidden per plan).
3. **Given** a file has `{stem}_processed.{ext}` sibling from in-app resize, **When** inventory computes converted, **Then** source file counts toward **converted** not **awaiting convert**.
4. **Given** rescan, **When** scan completes, **Then** all inventory counts refresh from new walk.

---

### Edge Cases

- No folder loaded — list empty; sort/filter disabled; inventory hidden.
- Filter zero matches — counter shows 0 / N; tree shows empty or "no matches" state.
- Very long paths — truncate display with ellipsis in plan phase if needed; full path still in selection/status.
- ImageMagick absent — magick-detected counts stay 0; heic/svg files may be invisible to list until feature 010 scanner extension.
- 10k+ tree nodes — tree view MUST remain responsive (lazy expand or virtualize in plan); flat list remains default for huge sets if perf requires.
- Folder with only non-image files — shows in tree with skipped count; no selectable image rows.
- Duplicate filenames in same folder — each row remains unique by full path.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: List MUST display relative folder path and filename for each entry when a scan root is loaded.
- **FR-002**: User MUST be able to choose sort mode: Path, Name, Folder.
- **FR-003**: Filter MUST match filename, relative folder, or full path substring (case-insensitive UTF-8).
- **FR-004**: Sort and filter logic MUST live in testable core module (constitution §III), not only in egui callbacks.
- **FR-005**: Virtualized list (`show_rows`) MUST be preserved — no per-row widget explosion.
- **FR-006**: Changing sort MUST bump scroll generation / reset scroll like filter (FR-005 from 001).
- **FR-007**: Column headers **Folder** and **Filename** MUST be visible above the list in **Flat list** view.
- **FR-008**: User MUST be able to toggle **Flat list** and **Folder tree** presentation modes without losing scan data.
- **FR-009**: **Folder tree** view MUST show expandable folders relative to scan root with per-folder **native listed** counts on folder lines (same definition as inventory `native_listed` in subtree).
- **FR-010**: After each scan, an **inventory summary** MUST display native listed, magick-detected (unlisted), converted, awaiting convert, and non-image skipped counts.
- **FR-011**: **`awaiting_convert`** MUST equal the count of `FileStatus::MagickDetected` entries. **`magick_detected`** MUST equal `awaiting_convert` plus `Converted` entries whose source extension is not in the native scanner set. UX “magick − converted” refers to magick-origin converted files only.
- **FR-012**: **Converted** MUST include files with matching `{stem}_processed.*` output from in-app processing in the same directory.
- **FR-013**: Image rows in tree or flat view MUST show a **status tag**: `native`, `magick · awaiting convert`, or `converted`.
- **FR-014**: Non-image file counting MUST occur during the same directory walk as image discovery (scanner module); GUI displays counts only.
- **FR-015**: Magick-detected classification MUST only run when ImageMagick is on PATH; logic lives in testable core module.
- **FR-016**: Tree/filter/sort/inventory logic MUST remain in core modules testable without egui (constitution §III).

### Key Entities

- **SortMode**: Path | Name | Folder.
- **ListViewMode**: FlatList | FolderTree.
- **ListRow**: (relative_folder, filename, absolute_path, file_status) — display vs selection path.
- **FileStatus**: NativeListed | MagickDetected | Converted.
- **list_indices**: filter + sort output indices for virtualization.
- **ScanInventory**: native_listed, magick_detected, converted, awaiting_convert, non_image_skipped — snapshot after scan.
- **FolderTreeNode**: relative_path, listed_count, magick_count, skipped_count, children, file entries.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: User can distinguish same-named files in different folders without opening feh.
- **SC-002**: Sort modes produce deterministic order verified by unit tests on fixed fixtures.
- **SC-003**: Folder filter on 10k dataset completes in &lt;200ms (reuse feature 001 perf test pattern).
- **SC-004**: `cargo test` includes tests for `relative_folder`, sort, and path-aware filter.
- **SC-005**: User can switch Flat list ↔ Folder tree and see **native listed** counts on the root tree folder line match inventory `native_listed` (per-folder **listed** uses the same definition).
- **SC-006**: Inventory summary on a mixed fixture (native + magick + non-image) matches expected counts in unit tests.
- **SC-007**: Flat list mode at 10k images retains filter &lt;200ms (SC-003); tree mode perf target defined in plan phase.

## Assumptions

- US1–US5 shipped; adversarial remediation (2026-06-22) aligned tree counts, FR-011, and post-resize inventory refresh.
- Thumbnail column out of scope (Area 4).
- Metadata sort (size, date) deferred to Area 6.
- Depends on feature 001 virtualization and scan root (`current_dir`).
- Magick-detected scan requires ImageMagick on PATH (feature 009); full convert pipeline deferred to feature 010+.
- Non-image counting walks all files when enabled — may increase scan time; plan phase sets perf budget.
- Default view mode after load: **Flat list** (current behavior) unless user switches to tree.

## Traceability

| Source | ID |
|--------|-----|
| lessons-learned.md | Shadow work §5 |
| feature 001 data-model | Sort deferred Area 6 → now in scope |
| user dogfood | folder + sort requests |
| clarify 2026-06-22 | folder tree, inventory, magick vs converted |
| adversarial-review 2026-06-22 | FR-011, SC-005, resize refresh, magick perf — remediation T048–T054 |
| docs/POSITIONING.md | honest format routing |