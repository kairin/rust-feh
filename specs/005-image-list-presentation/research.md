# Research: Image List Presentation (005)

**Date**: 2026-06-22

## R1: Scanner return type extension

**Decision**: Replace `(Vec<ImageEntry>, Vec<String>)` with `ScanResult { entries, warnings, inventory }`. Magick-detected files are `ImageEntry` rows with `FileStatus::MagickDetected` in the unified `entries` vec (no separate `magick_entries` field).

**Rationale**: Single walk produces native list + non-image counts + magick candidates; GUI reads one snapshot.

**Alternatives considered**:
- Second pass for magick — rejected (doubles I/O)
- GUI-only inventory from flat list — rejected (misses non-image + unlisted magick files)

## R2: ImageMagick detection during scan

**Decision**: When `magick_available` (from `ToolCapabilities`), run ImageMagick identify (`magick <path> -format '%m'` or equivalent) only on files that fail native extension check. Resolve `magick`/`convert` binary **once per scan**. Cap at **500 identify calls per scan**; set `magick_identify_truncated` and count post-cap files as `non_image_skipped`.

**Rationale**: Constitution §IV optional enhance; unbounded identify @10k non-images is too slow.

**Alternatives considered**:
- Extension-only heuristics (heic, svg) — rejected (incomplete vs identify)
- Always skip magick scan — rejected (breaks US5)

## R3: Converted file detection

**Decision**: For each native or magick entry, check same directory for `{stem}_processed.{jpg|png|webp}` glob; if exists, `FileStatus::Converted`. Rows that are themselves `*_processed.*` count as `Converted`.

**Rationale**: Matches existing `process_image` output naming in `image_proc.rs`.

**Alternatives considered**:
- Sidecar metadata file — rejected (new persistence scope)

## R4: Folder tree UI strategy

**Decision**: Build `FolderTreeNode` tree in `ui_logic` from flat entries post-scan. Render with **lazy expand**: only expanded branches' files shown as flat indented rows inside `ScrollArea::show_rows` (virtualized by visible row count, not full tree depth).

**Rationale**: egui has no built-in virtualized tree; collapsed folders = O(1) rows; expanded tree at 10k stays usable if default collapsed.

**Alternatives considered**:
- Full tree widget non-virtualized — rejected for 10k
- Replace flat list entirely — rejected (FR-008 toggle)

## R5: Default view mode

**Decision**: **Flat list** default after scan (current behavior).

**Rationale**: Clarification 2026-06-22; best perf for 10k.

## R6: Inventory bar placement

**Decision**: Horizontal summary strip directly above list/tree headers in `CentralPanel`, below filter row.

**Rationale**: Clarification 2026-06-22 UX Reference.