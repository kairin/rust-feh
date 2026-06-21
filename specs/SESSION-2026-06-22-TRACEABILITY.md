# Session Traceability: 2026-06-22 (post-dinner)

Master index of topics discussed in the evening session and where they live in Spec Kit / docs / code.

## Topic → Artifact map

| # | Topic discussed | Status | Spec / doc / code |
|---|-----------------|--------|-------------------|
| 1 | ImageMagick: detection only, no convert pipeline | Documented | `008` spec, `tool_caps.rs`, `docs/POSITIONING.md` claims |
| 2 | Fast tools matrix (feh / magick / image crate tiers) | Documented | `008` panel, `docs/POSITIONING.md`, dinner tool analysis |
| 3 | Tools & capabilities side panel | **Shipped** | `src/main.rs`, `src/tool_caps.rs` → formalize **`008`** |
| 4 | nfeh vs rust-feh comparison | **Docs** | `docs/NFEH-COMPARISON-AND-MIGRATION.md` |
| 5 | Product positioning | **Docs** | `docs/POSITIONING.md`, README |
| 6 | Spec **008** capabilities panel | Specified | `specs/008-tool-capabilities-panel/` |
| 7 | Spec **009** external tool runtime (supersedes 002) | Specified | `specs/009-external-tool-runtime/` |
| 8 | Folder tree + scan inventory + magick vs converted | **Shipped** | `005` US4–US5, `src/main.rs`, `src/ui_logic.rs` |
| 9 | Flat list Folder + Filename (pre-dinner shipped) | **Shipped** | `005` US1–US3, status column, `gap-audit.md` |
| 12 | Adversarial review + remediation (FR-011, SC-005, resize) | **Done** | `005/adversarial-review.md`, T048–T054, `plan.md` updated |
| 10 | feh filelist / `--conversion-timeout` / wallpaper modes | **Not specced** | Proposed **`011-feh-launch-orchestration`** |
| 11 | ImageMagick convert fallback + scanner extensions | **Not specced** | Proposed **`010-imagemagick-format-bridge`** |

## Implement order (session-adjusted)

1. ~~**005** — gap-audit US1–US3; implement US4–US5 (tree + inventory)~~ **Done**
2. **008** — gap-audit shipped panel
3. **009** — Tools menu recheck + spawn-failure sync
4. **003** — GUI perf validation (parallel / when display available)
5. **010** — magick identify in scan + convert fallback (from dinner tier 2)
6. **011** — feh launch enhancements (from dinner tier 1)

## Plans generated this session

| Feature | plan.md | research | data-model | contracts | quickstart |
|---------|---------|----------|------------|-----------|------------|
| 005 | ✅ complete | ✅ | ✅ | ✅ | ✅ | adversarial-review, gap-audit |
| 008 | ✅ | ✅ | ✅ | ✅ | ✅ |
| 009 | ✅ | ✅ | ✅ | — | ✅ |

## Cross-links

- [OUTSTANDING-ISSUES-ROADMAP.md](./OUTSTANDING-ISSUES-ROADMAP.md)
- [005 plan](./005-image-list-presentation/plan.md) — **complete** (T001–T054); next: **008** / **009**
- [008 plan](./008-tool-capabilities-panel/plan.md)
- [009 plan](./009-external-tool-runtime/plan.md)