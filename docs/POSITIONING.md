# rust-feh Positioning

This document states how rust-feh is positioned in the market, relative to the archived **nfeh** project, **feh**, and optional tools like **ImageMagick**. It is the canonical source for product messaging; keep it aligned with [NFEH-COMPARISON-AND-MIGRATION.md](NFEH-COMPARISON-AND-MIGRATION.md) and the in-app **Tools & capabilities** panel (`src/tool_caps.rs`).

**Audience:** Maintainers, contributors, and anyone writing README copy, release notes, or specs.

---

## What rust-feh is

rust-feh is a **Linux-first feh orchestrator**: a lightweight GUI for browsing and selecting images at scale, then delegating viewing, navigation, and wallpaper to native tools.

It is **not** a port of nfeh. It is a from-scratch Rust rewrite that shares the same broad idea — pick images from a folder and set wallpaper — but changes the architecture to prioritize feh integration, large directories, and a small native binary.

---

## What rust-feh is not

- **Not a feh replacement** — viewing and wallpaper stay in feh (constitution §I).
- **Not an image editor** — resize/convert are lightweight helpers; full tools dialog is deferred (Area 7).
- **Not an Electron thumbnail picker** — no in-app preview grid today (planned separately as Area 4).
- **Not ImageMagick-dependent** — magick/convert is optional; the `image` crate covers common formats in-process.

---

## Competitive framing

### vs archived nfeh

| Dimension | nfeh | rust-feh |
|-----------|------|----------|
| **Identity** | "GUI for feh" (marketing) | Actually invokes feh for view + wallpaper |
| **Primary UX** | 480×480 thumbnail grid | Virtualized list with filter, sort, folder column |
| **Scale** | Small folders (sync glob per render) | 10k+ metadata-only list |
| **Wallpaper** | Node `@fa7ad/wallpaper` | `feh --bg-fill` |
| **Runtime** | Electron + Node | Single Rust binary |

nfeh is the **spiritual predecessor**; rust-feh is the **feh-centric successor** aimed at real libraries, not a tiny wallpaper picker.

### vs bare feh

feh excels at viewing and navigating within a directory it already knows. rust-feh owns what feh does not:

- Recursive folder scan with extension filtering
- Case-insensitive filter across path and filename
- Sort by path, name, or folder
- Virtualized list that stays responsive at 10k+ images
- Explicit launch with stable geometry and folder context for slideshow navigation

rust-feh does not compete with feh's keyboard shortcuts, zoom, or slideshow — it **feeds** feh.

### vs ImageMagick

ImageMagick is an **optional format bridge**, not a core dependency:

- **Today:** detected on PATH; routing and install hints shown in the capabilities panel; convert subprocess **not yet wired** in app code.
- **Planned:** exotic formats (svg, heic, raw, etc.) convert or identify via magick, then view in feh.
- **Always available:** jpg/png/webp resize via the pure-Rust `image` crate.

Position honestly: document what is live vs planned. The capabilities panel and this doc must agree.

---

## Division of labor (tool stack)

| Job | Owner | Speed tier |
|-----|-------|------------|
| Browse, filter, sort, select | **rust-feh** | Instant |
| View, slideshow, navigate | **feh** | Fast |
| Set wallpaper | **feh** (`--bg-fill`) | Fast |
| Quick resize (common formats) | **`image` crate** | Medium |
| Exotic format view/convert | **ImageMagick** (optional) | Slower |

The GUI orchestrates; it does not reimplement feh or ImageMagick (constitution §I, §IV).

---

## Target users

**Primary:**

- Linux users who already use or want **feh** for viewing
- People with large image trees (photos, assets, archives) who need find-and-open, not a tiny fixed picker
- Users who want a small native binary instead of Electron/Node

**Secondary:**

- Former nfeh users who need wallpaper + browse — with the understanding that UX is list-based today and feh is required for wallpaper
- Maintainers who want transparent dependency and format routing (capabilities panel)

**Not primary (today):**

- Users who want in-app thumbnails only and will not install feh
- Users who need fill-mode wallpaper UI (nfeh had a stub; neither app completes this)
- macOS/Windows as first-class targets (Linux-first per constitution §IV)

---

## Messaging emphasis

| Emphasize | Why |
|-----------|-----|
| feh integration (view + wallpaper) | Core differentiator vs nfeh and generic file managers |
| 10k+ virtualized browse, filter, sort | Proven automated perf tier; real user pain point |
| Single fast Rust binary | vs Electron abandonware |
| Tools & capabilities panel | Honest deps, install commands, format routing |
| Explicit "Open in feh" workflow | Sets expectation: GUI browses, feh views |

| De-emphasize (until shipped) | Why |
|------------------------------|-----|
| Thumbnail grid | Not implemented; Area 4 / README roadmap |
| "Full image editor" or rich convert UI | Area 7 deferred |
| ImageMagick as fully integrated | Detection + docs only today |
| Parity with nfeh in-app previews | Different UX by design |
| Auto-launch feh on select | Removed intentionally (FR-007) |

---

## Claims we can make today

Backed by code and tests:

- Virtualized list for large collections (`show_rows`, feature 001)
- Filter at 10k under 200ms (automated test, SC-003)
- feh spawn with geometry + folder context for navigation
- Wallpaper via `feh --bg-fill`
- Graceful degradation when feh is missing
- Six formats discovered at scan time: jpg, jpeg, png, webp, gif, bmp
- Quick resize demo for supported formats via `image` crate
- Dependency detection with apt install hints in UI

## Claims to avoid until validated or implemented

- "Smooth 60fps scroll at 10k" — SC-002 manual validation pending (spec 003)
- "Under 150MB RSS at 10k" — **validated** SC-004 pass (2026-06-22): ~126 MB peak @10k, ~124 MB @1k — see [README](../README.md) Resource usage and [003 validation-results](../specs/003-gui-performance-validation/validation-results.md)
- "ImageMagick powers convert/view for exotic formats" — routing documented; subprocess not wired
- "Drop-in nfeh replacement" — different UX, requires feh, no thumbnail grid

---

## Evolution rules

1. **Ship feature → update three places:** this doc (if positioning changes), comparison doc (parity table), capabilities panel / `tool_caps.rs` (if routing changes).
2. **New external tool → constitution check first** (§I thin-wrapper, §II minimal deps).
3. **README Status** stays a short public summary; detail lives here and in the comparison doc.

---

## Related documents

| Document | Role |
|----------|------|
| [NFEH-COMPARISON-AND-MIGRATION.md](NFEH-COMPARISON-AND-MIGRATION.md) | Factual parity and migration |
| [../.specify/memory/constitution.md](../.specify/memory/constitution.md) | Architectural gates |
| [../specs/OUTSTANDING-ISSUES-ROADMAP.md](../specs/OUTSTANDING-ISSUES-ROADMAP.md) | Feature backlog |
| [../specs/008-tool-capabilities-panel/spec.md](../specs/008-tool-capabilities-panel/spec.md) | Capabilities panel requirements |
| [../specs/009-external-tool-runtime/spec.md](../specs/009-external-tool-runtime/spec.md) | Tool detect + recheck requirements |
| [../README.md](../README.md) | Public entry point |