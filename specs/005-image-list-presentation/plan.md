# Implementation Plan: Image List Presentation

**Branch**: `005-image-list-presentation` | **Date**: 2026-06-22 (updated post-ship) | **Spec**: [spec.md](./spec.md)  
**Status**: **Complete** — T001–T054; adversarial remediation integrated

**Input**: Feature specification from `specs/005-image-list-presentation/spec.md`

**Parent**: [001-persistent-ui-virtual-browsing](../001-persistent-ui-virtual-browsing/spec.md)  
**Session index**: [SESSION-2026-06-22-TRACEABILITY.md](../SESSION-2026-06-22-TRACEABILITY.md)

## Summary

Image list presentation is **shipped** in three waves:

1. **US1–US3 (retroactive)**: Flat list with Folder + Filename columns, Path/Name/Folder sort, path-aware filter — verified in `gap-audit.md`.
2. **US4–US5 (new)**: Flat list / Folder tree toggle, scan inventory bar, Status column, optional magick `identify` during walk.
3. **Adversarial remediation (Phase 9)**: FR-011 + SC-005 spec alignment, tree `listed_count` = `native_listed`, post-resize inventory refresh, magick binary cached per scan — see [adversarial-review.md](./adversarial-review.md).

Scanner returns `ScanResult { entries, warnings, inventory }`; GUI calls `finalize_scan_entries` then `refresh_entry_and_inventory` after Quick resize. Full ImageMagick **convert** pipeline remains **010** (out of scope).

## Technical Context

**Language/Version**: Rust stable, edition 2021  
**Primary Dependencies**: `walkdir`, `which`, `egui` 0.30 / `eframe`; optional ImageMagick subprocess (`magick`/`convert` resolved once per scan)  
**Storage**: In-memory `ScanInventory` + `Vec<ImageEntry>`; `tree_expanded_paths: HashSet<String>` session UI state  
**Testing**: `cargo test` — 30 lib + 8 feature_001 + 8 feature_005 (+1 `#[ignore]` heic); `./scripts/validate-feature-001.sh` (13 checks)  
**Target Platform**: Linux desktop (glow)  
**Project Type**: desktop-app feature (core modules + GUI)  
**Performance Goals**:
- Flat filter &lt;200ms @10k — **met** (`sc003_filter_10k_under_200ms`)
- Scan walk &lt;10s @10k native — **met** (`scan_10k_supported_files`)
- Tree build/flatten @10k — lazy expand; formal &lt;100ms benchmark **deferred** (SC-007 partial)
- Magick identify — cap 500/scan; binary cached once (T049)

**Constraints**: `show_rows` virtualization (flat + tree); no thumbnails; constitution §III — logic in `scanner` / `ui_logic` / `types`  
**Scale/Scope**: All US1–US5 + Phase 9 remediation complete

## Constitution Check

*Post-implementation re-check (2026-06-22).*

| Principle | Status | Evidence |
|-----------|--------|----------|
| I. Thin-Wrapper Architecture | ✅ PASS | Browse/inventory only; feh for view/wallpaper |
| II. Pure Rust, Minimal Dependencies | ✅ PASS | No new crates |
| III. Clean Module Separation | ✅ PASS | `refresh_entry_and_inventory`, `build_folder_tree` in `ui_logic`; scan in `scanner` |
| IV. Linux-First, feh-Centric | ✅ PASS | Inventory + `tool_caps` honest routing |
| V. Performance Awareness | ✅ PASS | Virtualized lists; magick cap + cached binary |

**Gate result**: ALL PASS.

## Dinner session coverage (005 scope)

| Session topic | Status | Notes |
|---------------|--------|-------|
| Folder + filename flat list | ✅ Shipped | US1–US3 |
| Folder tree hierarchy | ✅ Shipped | US4 |
| Inventory + magick vs converted | ✅ Shipped | US5; FR-011 clarified |
| Tools capabilities panel | ↔ **008** | Labels aligned (T046) |
| Tool recheck / PATH detect | ↔ **009** | `magick_available` gates scan |
| feh filelist / conversion-timeout | ❌ **011** | Deferred |
| magick convert pipeline | ❌ **010** | Deferred |
| Per-folder skipped on tree | ❌ **004** | Root shows total only |

## Project Structure

### Documentation (this feature)

```text
specs/005-image-list-presentation/
├── spec.md                 # Implemented; clarifications through adversarial remediation
├── plan.md                 # This file
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   └── scan-inventory-ui.md
├── gap-audit.md            # FR pass/partial + quickstart checklist
├── adversarial-review.md   # Post-ship review; DoD 7/7
└── tasks.md                # T001–T054 complete
```

### Source Code (shipped)

```text
src/
├── types.rs              # FileStatus, ListViewMode, ScanInventory, ImageEntry.status
├── scanner.rs              # ScanResult, magick identify (cap 500, cached binary)
├── ui_logic.rs             # list_indices, build_folder_tree, finalize/refresh inventory
├── tool_caps.rs            # Format notes aligned with inventory labels
└── main.rs                 # Inventory bar, view toggle, flat/tree render, resize refresh

tests/
├── feature_001_validation.rs
└── feature_005_list.rs     # Inventory, SC-005, post-resize, #[ignore] heic
```

**Structure Decision**: Extend existing modules; no new crate. Inventory helpers live in `ui_logic.rs`; scan aggregation in `scanner.rs`.

## Complexity Tracking

> No violations.

## Phase 0: Research

Consolidated in [research.md](./research.md). Updated: unified `entries` vec (no `magick_entries`); magick binary once per scan; cap overflow → `non_image_skipped`.

## Phase 1: Design Artifacts

| Artifact | Path | Status |
|----------|------|--------|
| Data model | [data-model.md](./data-model.md) | Synced post-remediation |
| UI contract | [contracts/scan-inventory-ui.md](./contracts/scan-inventory-ui.md) | Synced post-remediation |
| Validation | [quickstart.md](./quickstart.md) | V1–V4 checklist in gap-audit |

## Implementation Phases (completed)

| Phase | Deliverable | FRs / SC | Status |
|-------|-------------|----------|--------|
| P0 | Gap-audit US1–US3 → `gap-audit.md` | FR-001–FR-007 | ✅ |
| P1 | `ScanResult` + `ScanInventory` | FR-010, FR-014–FR-015 | ✅ |
| P2 | `FileStatus` + `detect_converted_status` | FR-012–FR-013 | ✅ |
| P3 | Inventory summary UI bar | FR-010, FR-011 | ✅ |
| P4 | `ListViewMode` + Status column | FR-008, FR-013 | ✅ |
| P5 | Folder tree + lazy expand | FR-008–FR-009 | ✅ |
| P6 | Tests + quickstart | SC-001–SC-007 | ✅ (SC-007 tree perf partial) |
| P7 | Doc sync | T044–T047 | ✅ |
| P8 | Adversarial remediation | T048–T054 | ✅ |

## Key implementation decisions (post-clarify)

| Topic | Decision | Code / doc |
|-------|----------|------------|
| FR-011 | `awaiting_convert` = `MagickDetected` count; `magick_detected` includes magick-origin converted | `types.rs::from_entries`, `spec.md` |
| SC-005 | Tree folder **listed** = `NativeListed` in subtree (= inventory `native_listed` policy) | `ui_logic::bump_folder_counts` |
| Post-resize | Incremental `refresh_entry_and_inventory` without full rescan | `main.rs` after `process_image` |
| Magick perf | Resolve `magick`/`convert` once per `scan_images` | `scanner.rs` |
| Per-folder skipped | Deferred to **004**; root tree line shows scan-wide total | `spec.md` Clarifications |

## Dependencies on other features

| Feature | Relationship | Status |
|---------|--------------|--------|
| **001** | Virtualized list, scan root, filter perf | ✅ Consuming |
| **009** | `ToolCapabilities.magick_available` gates identify | Partial — panel recheck shipped; menu T008 open |
| **008** | Format routing ↔ inventory labels | ✅ Aligned (T046) |
| **004** (future) | Per-folder `non_image_skipped` in tree | Deferred |
| **010** (future) | Convert pipeline for awaiting rows | Deferred |
| **003** | Re-run GUI perf validation | Optional |

## Deferred / follow-up (not blocking 005 closure)

| Item | Target | Notes |
|------|--------|-------|
| Tree perf benchmark @10k | SC-007 / **003** | Lazy expand shipped; no formal tree timing test |
| Per-folder skipped counts | **004** | Spec edge case documented |
| Tree row `[native · listed]` tags | Polish | Flat Status column complete |
| `#[ignore]` heic magick test | CI optional | Run with `- ignored` when magick installed |

## Advisory (007 FR-006)

Magick identify capped at 500; overflow counted as `non_image_skipped` with `magick_identify_truncated` flag. Re-benchmark if scan perf regresses on mixed non-image dirs.

## Verification

```fish
cargo clippy -- -D warnings
cargo test
cargo test feature_005
./scripts/validate-feature-001.sh
```

**Last verified**: 2026-06-22 — 46 tests pass (1 ignored), validation 13/13.

## Next features (repo implement order)

1. **008** — gap-audit shipped Tools panel  
2. **009** — Tools menu recheck + feh spawn-failure sync  
3. **003** — GUI perf validation when display available