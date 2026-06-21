# Outstanding Issues → Spec Kit Features (Master Index)

**Created**: 2026-06-21  
**Spec**: [007-outstanding-roadmap/spec.md](./007-outstanding-roadmap/spec.md)  
**Source**: [001-persistent-ui-virtual-browsing/outstanding-issues.md](./001-persistent-ui-virtual-browsing/outstanding-issues.md), [adversarial-review.md](./001-persistent-ui-virtual-browsing/adversarial-review.md), dogfood feedback after feature 001

Feature 001 convergence (T065–T069) closed **Bucket B** and automated most of **Bucket C**. Remaining work is promoted to **features 002–006** below. This file is the **master index** (feature 007).

---

## Multi-agent advisory (when unsure)

**Requirement (FR-006)**: Before changing scope, bucket classification, implement order, or creating/patching features, **seek advice** when uncertain. Valid advisors:

| Advisor | Typical use |
|---------|-------------|
| **Codex** | Implementation approach, Rust/egui patterns, test design |
| **Grok** | Spec Kit workflow, repo context, Cursor session |
| **Hermes** | Orchestration, kanban workers, multi-step pipelines |
| **DeepSeek 4 Pro** | Second opinion on architecture or requirement ambiguity |
| **Human maintainer** | Final arbitration on priority and ship decisions |

**Record outcomes** in the target feature `spec.md` **Clarifications** section (date, question, decision).

### Triggers — consult before proceeding

- Is this Bucket A (docs) or Bucket B (code)?
- Patch feature 001 vs new `00N-*` spec?
- Can SC-002 / SC-004 be automated or must they stay manual?
- Retroactive spec (005/006) vs new implement work?
- Change implement order below?

---

## Promoted features

| Feature | Outstanding source | What it covers |
|---------|-------------------|----------------|
| [002-feh-runtime-detection](./002-feh-runtime-detection/spec.md) | A5 | Re-check feh after install without restart |
| [003-gui-performance-validation](./003-gui-performance-validation/spec.md) | Bucket C | Automated tier **pass**; SC-004 RSS **pass** (2026-06-22); SC-002 scroll **pending** |
| [004-scanner-resilience](./004-scanner-resilience/spec.md) | Bucket D | **Absorbed by [011](./011-browsing-experience-round/spec.md)** (scan warnings + t069) |
| [011-browsing-experience-round](./011-browsing-experience-round/spec.md) | Dogfood SMB | feh filelist, background scan, Activity log — **shipped** (automated pass; manual V1/V2/V4 pending) |
| [012-ui-feedback-polish](./012-ui-feedback-polish/spec.md) | Dogfood UI screenshots | Status animation, NAS scan policy, deps collapse, bottom tips, detach log — **shipped** (automated pass; manual V1–V5 pending) |
| [005-image-list-presentation](./005-image-list-presentation/spec.md) | Dogfood | Folder column, sort, path-aware filter (retroactive spec) |
| [006-window-viewer-stability](./006-window-viewer-stability/spec.md) | Dogfood | Window presets, resizable lock, list fill, feh 5px / fixed geometry |
| [008-tool-capabilities-panel](./008-tool-capabilities-panel/spec.md) | Positioning / shipped | Tools side panel: deps, install copy, speed tiers, format routing |
| [009-external-tool-runtime](./009-external-tool-runtime/spec.md) | A5 + 002 merge | Unified PATH detect + recheck for feh and ImageMagick (**supersedes 002**) |

(No named future features or conversion/wallpaper-mode extensions were promised.)

---

## Bucket → Feature map

| Bucket | Issue ID | Status after 001 | New feature | Priority |
|--------|----------|------------------|-------------|----------|
| **A** | A1–A4 | Docs synced (T066) | *(none — maintenance on 001 artifacts)* | — |
| **A** | A5 | Specified | [009](./009-external-tool-runtime/spec.md) (supersedes [002](./002-feh-runtime-detection/spec.md)) | P2 |
| **B** | B1, B2 | Fixed (T065, T069) | *(none)* | — |
| **C** | SC-002, SC-004 | SC-004 RSS **pass**; SC-002 scroll pending | [003](./003-gui-performance-validation/spec.md) | P1 |
| **C** | V1–V10 manual gaps | Partially automated (T067) | **003** + `validate-feature-001.sh` | P1 |
| **D** | Scanner non-permission errors | **Shipped in 011** | [011](./011-browsing-experience-round/spec.md) (supersedes 004 scope) | — |
| **D** | T055 / FR-015 extension | T068 + T069 done | **011** | — |
| **Dogfood** | List filename only | Shipped unspecced | [005](./005-image-list-presentation/spec.md) | P1 |
| **Dogfood** | Window / feh stability | Shipped unspecced | [006](./006-window-viewer-stability/spec.md) | P1 |
| **Positioning** | Tools & capabilities panel | Shipped unspecced | [008](./008-tool-capabilities-panel/spec.md) | P1 |
| **Positioning** | External tool runtime / recheck | **Implemented** (009) | [009](./009-external-tool-runtime/spec.md) | — |

---

## Not promoted (already closed on 001)

| Issue | Why no new feature |
|-------|-------------------|
| A1–A4 | Doc drift / intentional design — sync 001 artifacts only |
| B1, B2 | Fixed in T065, T069 |
| T067/T068 automation | Stays in 001; **003** finishes what automation cannot |

---

## Recommended implement order

1. **003** — SC-004 RSS **done**; finish SC-002 scroll manual session
2. **012** — Manual SMB GUI (V1–V5 in [quickstart](./012-ui-feedback-polish/quickstart.md)); automated tier **pass**  
3. **006** — Window/viewer stability (retroactive; not started)  
4. ~~005, 008, 009, 011, 012~~ — **complete** (2026-06-22)  
5. **004** — Absorbed by **011** (scanner warnings shipped)

**Session index**: [SESSION-2026-06-22-TRACEABILITY.md](./SESSION-2026-06-22-TRACEABILITY.md) — all post-dinner topics mapped.

Change order only after **advisory consultation** and update this file + `.specify/feature.json`.

---

## Pipeline per feature

```bash
/speckit-plan    # → plan.md, research.md, data-model.md, quickstart.md
/speckit-tasks   # → tasks.md
/speckit-implement
```

**Active feature** (for plan/tasks/implement): see `.specify/feature.json` → `feature_directory` (default **003** after **012** manual validation).

---

## Feature 001 artifact hygiene (no new feature)

| Artifact | Action |
|----------|--------|
| `plan.md` | Status → "code landed; validation via 003" |
| `gap-audit.md` | SC-004 **pass** (2026-06-22); SC-002 pending until scroll session |
| `data-model.md` | Extend when **005** / **006** land |
| `outstanding-issues.md` | Points here; do not reopen B1/B2 |

---

## Links

- [007 spec](./007-outstanding-roadmap/spec.md) — normative process + advisory FRs  
- [001 lessons-learned](./001-persistent-ui-virtual-browsing/lessons-learned.md) — why this index exists  
- `.specify/feature.json` — active feature pointer + full `features[]` list