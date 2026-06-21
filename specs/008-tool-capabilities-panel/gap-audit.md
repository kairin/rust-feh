# Gap Audit: Tool Capabilities Panel

**Date**: 2026-06-22  
**Files audited**: `src/tool_caps.rs`, `src/main.rs`  
**Type**: Retroactive gap-audit (code largely shipped before spec)

## FR-001–FR-005 (US1–US2: Dependencies + install copy)

| FR | Requirement | Status | Evidence |
|----|-------------|--------|----------|
| FR-001 | Persistent Tools & capabilities panel | **pass** | `SidePanel::right("tool_caps")` L584–594; always shown in `update()` |
| FR-002 | feh required with state + role | **pass** | `dependencies()` feh row; `render_tool_caps_panel` L216–252 |
| FR-003 | ImageMagick optional with state + role | **pass** | `dependencies()` ImageMagick row; role mentions magick-detect + convert (010) |
| FR-004 | apt install commands when missing | **pass** | `sudo apt install feh` / `imagemagick` in `DependencyStatus` |
| FR-005 | Copy install command to clipboard | **pass** | `ctx.copy_text` + `Copied install command for {name}` status L242–245 |

### US1 validation (quickstart V1)

- feh row required + installed/missing: **pass**
- ImageMagick optional + installed/missing: **pass**
- Resolved binary on PATH when installed: **pass** (`On PATH: {bin}`)
- Install cmd when missing: **pass**

### US2 validation (quickstart V1 step 5)

- Copy button beside install cmd: **pass** (code audit)
- Status confirms dependency name: **pass**
- Missing-required reminder + Recheck: **pass** L300–305
- Clipboard paste: **manual** (requires egui runtime; code path verified)

## FR-006–FR-008 (US3: Speed + format routing)

| FR | Requirement | Status | Evidence |
|----|-------------|--------|----------|
| FR-006 | Speed / timing section with 5 operations | **pass** | `operation_timings()` + grid L259–277 |
| FR-007 | Format discovery with Scan/View/Resize | **pass** | `format_routes()` + groups L280–297 |
| FR-008 | Dynamic content vs feh/magick detection | **pass** | Handlers/notes vary in `operation_timings` / `format_routes`; tests `operation_timings_reflect_missing_feh`, `format_routes_reflect_*` |

### US3 validation (quickstart V2)

- Five operation rows with tiers: **pass**
- Format legend mentions inventory bar (005): **pass** L281–282
- jpg native listed note: **pass** `format_routes[0].note`
- heic note changes with magick: **pass** (unit tests)
- feh absent view note: **pass** `operation_timings_reflect_missing_feh`
- Wallpaper feh-missing note: **pass** (gap-fill T007 — was static `feh --bg-fill` only)

## FR-009–FR-013 (US4 + cross-cutting)

| FR | Requirement | Status | Evidence |
|----|-------------|--------|----------|
| FR-009 | Install-and-recheck reminder when feh missing | **pass** | Red label L300–305; Recheck button L254–256 |
| FR-010 | Logic testable without GUI | **pass** | `tool_caps` module; 9 unit tests |
| FR-011 | Status region short pointer only | **pass** | L777 `Tools panel → dependencies, speed tiers, format routing` |
| FR-012 | Scrollable panel | **pass** | `ScrollArea::vertical()` L589–593 |
| FR-013 | Horizontally resizable min width | **pass** | `resizable(true)`, `min_width(240.0)` L585–587 |

### US4 validation (quickstart V3)

- Side region separate from list: **pass**
- Scroll when content exceeds height: **pass** (ScrollArea)
- Resize panel edge: **pass** (SidePanel resizable) — **manual** layout confirm

## Success criteria

| SC | Outcome | Status | Notes |
|----|---------|--------|-------|
| SC-001 | feh install state in 5s | **pass** | Required row at top of panel |
| SC-002 | One-click copy install cmd | **pass** | Copy button per missing dep |
| SC-003 | Unit tests without GUI | **pass** | 9 tests in `tool_caps::tests` |
| SC-004 | Recheck updates routing same session | **pass** | `refresh_tool_caps()` re-detects; panel reads `self.tool_caps` |
| SC-005 | POSITIONING claims accurate | **pass** | See cross-check below |

## POSITIONING.md cross-check (T009)

| Claim | Panel alignment | Status |
|-------|-----------------|--------|
| Tools & capabilities panel (messaging emphasis) | Panel title + right SidePanel | **pass** |
| Division of labor table (browse/feh/wallpaper/resize/exotic) | `operation_timings()` rows | **pass** |
| ImageMagick optional; convert not wired | Notes reference 010 / awaiting convert | **pass** |
| Dependency detection + apt hints | Dependencies section + Copy | **pass** |
| Honest live vs planned | No false convert-running claims | **pass** |

## 005 inventory alignment (T010)

| Inventory term | Panel usage | Status |
|----------------|-------------|--------|
| `native_listed` | jpg group note "Native listed in inventory" | **pass** |
| `magick_detected` | heic/tiff notes "Magick-detected" | **pass** |
| Inventory bar legend | Format discovery legend L281–282 | **pass** |
| `awaiting convert` | heic note "awaiting convert until processed" | **pass** |

## Gaps fixed (T007)

| Gap | Fix |
|-----|-----|
| Wallpaper timing static when feh missing | `operation_timings()` now uses feh-missing note (parity with view/slideshow) |
| FR-008 dynamic branches undertested | Added 4 unit tests for feh/magick absent/present |

## Deferred / out of scope

| Item | Owner |
|------|-------|
| Tools menu Recheck | **009** T008 |
| Spawn failure → `feh_available` sync | **009** T010 |
| ImageMagick convert subprocess | **010** |
| Non-apt install strings | Out of scope per spec |

## Verdict

**7/7 FR groups pass** after T007 wallpaper note fix. Feature ready to mark **Implemented**.