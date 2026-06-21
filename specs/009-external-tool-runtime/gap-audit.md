# Gap Audit: External Tool Runtime

**Date**: 2026-06-22  
**Files audited**: `src/tool_caps.rs`, `src/main.rs`, `src/ui_logic.rs`  
**Supersedes**: [002-feh-runtime-detection](../002-feh-runtime-detection/spec.md) for implementation

## FR-001–FR-011

| FR | Requirement | Status | Evidence |
|----|-------------|--------|----------|
| FR-001 | Startup detect feh + magick snapshot | **pass** | `main.rs` L38–39 `ToolCapabilities::detect()` |
| FR-002 | `feh_available` derives from snapshot | **pass** | L39, L203; `refresh_tool_caps` sync |
| FR-003 | Panel on-demand recheck | **pass** | `render_tool_caps_panel` L254–256 |
| FR-004 | Tools menu recheck | **pass** | Tools menu `Recheck tools on PATH` → `refresh_tool_caps` |
| FR-005 | Unified refresh updates panel + buttons | **pass** | Menu + panel share `refresh_tool_caps` |
| FR-006 | Debug log per recheck | **pass** | `Rechecked tools: feh=…, magick=…` |
| FR-007 | No restart required | **pass** | In-session `detect()` |
| FR-008 | Spawn NotFound → unavailable + messaging | **pass** | `feh_spawn_unavailable` + `mark_feh_unavailable` in `open_in_feh` and `set_wallpaper` |
| FR-009 | Core logic testable without GUI | **pass** | `is_feh_not_found`, `feh_confirmed_missing`, `feh_spawn_unavailable` + 13 unit tests |
| FR-010 | No background polling | **pass** | On-demand only |
| FR-011 | 002 FR-001–FR-006 satisfied here | **pass** | See supersession note below |

## User stories

### US1 — Install feh mid-session (quickstart V1)

- Panel recheck enables feh buttons: **pass** (`refresh_tool_caps` → `feh_available`)
- Panel shows feh installed after recheck: **pass** (`dependencies()`)
- Degraded state when still absent: **pass**

### US2 — ImageMagick mid-session (quickstart V4)

- Recheck detects magick/convert: **pass** (`ToolCapabilities::detect`)
- Format routes update on recheck: **pass** (panel reads `self.tool_caps`)
- Rescan not auto-triggered: **by design** (research R5)

### US3 — Spawn failure recovery (quickstart V3)

- `feh_available` flips false on NotFound + PATH confirm: **pass** (`mark_feh_unavailable`)
- Non-NotFound errors keep availability if PATH ok: **pass** (`feh_spawn_unavailable` guard)
- Panel reflects not installed: **pass** (`tool_caps.feh_available = false`)

### US4 — Menu recheck (quickstart V2)

- Tools menu entry present: **pass**
- Same handler as panel: **pass** (`refresh_tool_caps`)
- One log line per user click: **pass**

## Success criteria

| SC | Outcome | Status |
|----|---------|--------|
| SC-001 | feh install → launch without restart | **pass** (recheck path) |
| SC-002 | Recheck &lt;500ms | **pass** (two `which` lookups) |
| SC-003 | UI reflects unavailable after spawn failure | **pass** |
| SC-004 | Magick recheck updates panel routing | **pass** |
| SC-005 | Unit tests without GUI | **pass** (13 `tool_caps` tests) |

## Converge remediation (F1–F7)

| ID | Fix |
|----|-----|
| F1 | Tools menu Recheck added |
| F2–F5 | `mark_feh_unavailable` + `feh_spawn_unavailable` in both spawn paths |
| F6 | Classifier helpers + tests |
| F7 | This gap-audit |
| F8 | Non-executable feh — **deferred** (edge case note) |

## Feature 002 supersession

Feature **002** (`Re-check feh` menu, startup-only detect, spawn stale state) is **fully superseded** by **009**:

| 002 requirement | 009 implementation |
|-----------------|-------------------|
| FR-001 startup detect | FR-001 (unchanged) |
| FR-002 Tools menu recheck | FR-004 unified recheck (feh + magick) |
| FR-003 PATH re-check | `ToolCapabilities::detect()` |
| FR-004 no restart | FR-007 |
| FR-005 spawn failure sync | FR-008 + classifier |
| FR-006 idempotent recheck | `refresh_tool_caps` |

Do **not** implement 002 separately.

## Verdict

**11/11 FRs pass.** Feature **009** ready — Status: **Implemented**.