# .agents/BACKLOG.md — Complete outstanding-work inventory (audited 2026-07-05)

Class key: **A** = speckit-covered (source of truth: its specs/<feature>/ folder);
**B** = not covered (yardstick: .agents/SPEC.md); **C** = should-be-covered (needs
speckit-specify, human checkpoint). See ROUTING.md for the item → path map.

| ID | Item | Class | Risk | Size | Depends on | Status |
|----|------|-------|------|------|-----------|--------|
| B-1 | .specify 0.12.4 upgrade commit+PR (#145) | B | low | XS | — | in-flight |
| A-001 | 001 residuals: T026/T038/T050/T051/T058 (manual GUI), T055 (stale) | A | low | S | GUI session | todo |
| A-003 | 003 residuals: T037 (stale, shipped via 011), T026 (conditional) | A | low | XS | GUI session outcome | todo |
| A-006 | 006 residual: T009 feh-geometry screenshot check (env-blocked → scripted capture) | A | low | XS | GUI session | todo |
| A-002/004 | Record 002 Superseded-by-009, 004 Absorbed-by-011 (already marked in specs; maintainer confirmed 2026-07-05) | A | low | XS | — | todo |
| B-2 | Branch hygiene: delete 5 audited stale branches (local+origin) | B | low | XS | B-1 merged | todo |
| B-3 | Doc hygiene: roadmap reconciliation + closure recording | B | low | S | A-closures | todo |
| B-4 | Fresh verification baseline (test/clippy/fmt/release) | B | low | XS | B-1 merged | todo |
| B-5 | validation-results.md parity 005/008/009/013/014 | B | low | S | — | OPTIONAL, not committed |
| C-1 | caption (TagForge) integration — **specified as 015, then DEFERRED** (maintainer 2026-07-05; restart point = specs/015-caption-tool-integration/spec.md; prerequisites: upstream local batch mode, LICENSE/tag) | C | med-high | M | upstream caption work | deferred |
| C-2 | image-tools (imtools) integration — likely feature 017 | C | high | L | speckit-specify by maintainer | brief in .agents/briefs/ |
| C-5 | in-viewer image actions (feh) — **specified as 016** (maintainer priority 2026-07-05: manual-use pain; jumps queue ahead of C-2) | C | med | M | 016 plan/tasks | specified |
| C-3 | Keyboard navigation (docs roadmap: Planned) | C | low | M | after C-1/C-2 | DEFERRED (maintainer, 2026-07-05) |
| C-4 | Full config persistence (docs roadmap: Partial; window presets shipped in 006) | C | low | M | after C-1/C-2 | DEFERRED (maintainer, 2026-07-05) |

## Verified facts that reshape C-1/C-2 (do not soften)
- caption `batch_caption.py` = REMOTE inference (HF Space `fancyfeast/joy-caption-beta-one`
  via gradio_client; HF_TOKEN + network). Local GPU inference only in `app.py` (Gradio UI).
  caption repo has NO license and no tags/releases.
- image-tools = GPL-2.0-only, egui/eframe 0.31 + wgpu 24.0.5 (Vulkan-mandatory at startup),
  libheif via runtime dlopen (libloading 0.8, needs libheif ≥ 1.19). rust-feh = MIT,
  egui/eframe 0.30 glow (deliberate). Maintainer ruling 2026-07-05: licenses stay as-is →
  no GPL code imported into rust-feh.

## Explicit non-goals (from NEXT-ROUND-CONSOLIDATED, unchanged)
Async scanning, thumbnails, metadata sort, background/non-blocking clipboard (014 v1
deferred), feh in-viewer shortcuts, multi-monitor memory.

## Dropped/closed during audit (evidence in plan + BOARD)
- NEXT-ROUND A1/C1/C2 (006 persistence, plan+tasks, feature.json repoint): CLOSED by PR #144.
- Stale-branch investigation: resolved (see B-2 list).
