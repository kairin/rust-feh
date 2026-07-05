# Integration brief — image-tools (imtools) × rust-feh
_Lead-architect brief for the maintainer's `speckit-specify` session. Facts verified
2026-07-05 against github.com/kairin/image-tools (private repo). Likely feature number: 016._

## Context
imtools: native Rust desktop photo browser/EDITOR. 19 `.rs` files (~318 KB), v0.1.2,
edition 2024, last commit 2026-04-18, 0 open issues/PRs, binaries `imtools` + `perf_probe`.
Features verified: RAW via rawler 0.7.1 (RAF/DNG/NEF/CR2/ARW); HEIC/AVIF via **runtime
dlopen** of system libheif (libloading 0.8; needs libheif ≥ 1.19; deliberately NOT a
build-time C dep); non-destructive edits as sidecar JSON (README self-contradicts on the
path — `<image>.json` vs `.edits/<image>.json`; check `src/state.rs` during specify);
export JPG/PNG/WebP.

## The two verified hard constraints
1. **Stack divergence (load-bearing):** rust-feh = egui/eframe **0.30 on glow (OpenGL)** —
   Cargo.toml comment: chosen for "fewer graphics driver requirements than wgpu".
   imtools = egui/eframe **0.31 + wgpu 24.0.5, Vulkan-mandatory at startup** (exits without
   an adapter). This is a backend divergence, not a version bump: rust-feh explicitly
   avoided the exact dependency imtools mandates.
2. **License divergence (bigger than the backend):** imtools is **GPL-2.0-only**; rust-feh
   is **MIT**. Any imtools code imported into rust-feh (option b's module absorption,
   option c, or shared crates flowing FROM imtools) makes the combined work GPL.
   **Maintainer ruling 2026-07-05: licenses stay as-is → no GPL code enters rust-feh.**
   (MIT rust-feh code may flow INTO GPL imtools; not the reverse.)

## Options weighed
- **(a) Cargo workspace, sibling crates, shared extracted commons** — viable only with
  commons extracted FROM rust-feh (MIT) and consumed BY imtools; nothing flows back.
  Two release cadences, one repo; egui versions may stay split per crate but workspace
  dependency unification will fight the 0.30/0.31 split. Cost: repo surgery + CI rework now,
  for benefits (shared scanner/types) imtools may not even want. **Defer.**
- **(b) Absorb selected modules behind flags + launch imtools for heavy editing** — the
  absorption half is DEAD under the license ruling (HEIC dlopen loader, EXIF, sidecar
  pattern are GPL code). The launch half survives and is exactly the existing pattern.
- **(c) Full absorption, editor as a mode** — requires BOTH a constitution amendment
  (Technical Standards pin egui 0.30/glow; Principles I/II tension: "not an image editor",
  lightweight, minimal deps; wgpu/Vulkan reverses a deliberate decision) AND a license
  decision, and grafts an editor onto a repo whose biggest liability is already a
  3510-line main.rs. **Recommend against, independent of licensing.**

## RECOMMENDATION: "(b)-lite" — imtools as a detected external tool
imtools becomes a first-class entry in the `tool_caps.rs` capability matrix (like feh /
magick / magick-cache): detect binary on PATH or configured path; from the browser,
"Edit in imtools" on selection/folder → spawn `imtools <paths>`; graceful degradation when
absent. Zero license contact, zero backend contact, constitution-clean, and it makes
imtools the SECOND external-tool consumer — the right moment to generalize `tool_caps.rs`
into whatever "plugin" shape both it and caption (brief C-1) actually need.

Cheap upstream additions to imtools (GPL side, fine): a `--select <path>` / filelist arg
form mirroring feh's, so rust-feh can hand over multi-selections; document sidecar path.

What rust-feh does NOT gain under this shape: in-process HEIC/RAW decoding for its own
browser list. If that's ever wanted, the licensed-clean path is implementing a dlopen
libheif loader FRESH in rust-feh (the technique isn't copyrightable; the code is) or a
crates.io MIT alternative — new feature, own spec, own security review of the dlopen
surface. Not part of 016.

## Predicted failure modes (likelihood × damage, high→low)
1. **License contamination via "just this one module"** (med × very high): future
   contributor copies the HEIC loader. Mitigation: this brief + GUARDRAILS rule; review
   gate: any PR adding code "inspired by" imtools cites provenance.
2. **Scope creep toward (c)** (med × high): "while we're at it, embed the editor."
   Mitigation: spec states non-goals explicitly; constitution amendment required = hard gate.
3. **Selection-handoff mismatch** (high × low): imtools may lack a filelist/multi-path CLI
   contract today. Mitigation: verify `src/main.rs` arg parsing during specify; upstream
   the small CLI addition first.
4. **Sidecar collision** (low × med): if rust-feh later renames/moves files it could orphan
   imtools sidecars. Mitigation: document sidecar path; rust-feh treats `.edits/` and
   `<stem>.json` as untouchable companions during any file ops.
5. **Version skew** (med × low): imtools evolves independently; capability probe should be
   tolerant (`imtools --version` parse, feature-gate on major).

## Constitution check
(b)-lite: I ✔ (delegates editing exactly as it delegates viewing), II ✔ (no new deps),
III ✔ (detection in tool_caps, launch logic testable), IV ✔ (graceful absence), V ✔.
Options (a)(c): blocked/deferred as above; (c) additionally requires human-approved
constitution amendment before it is even eligible.

## Open questions for the maintainer
1. Confirm (b)-lite as the 016 shape; (a) revisitable later without wasted work.
2. Should the `tool_caps.rs` generalization ("external tools registry": detect + launch +
   probe + capability flags) be part of 016 or its own small feature that 015 and 016 both
   consume? (My lean: own small feature, spec'd first — it's the real architecture here.)
3. Any appetite for the fresh MIT dlopen-libheif loader in rust-feh's own browser later?
   (Separate feature; security-review-heavy.)
4. Upstream imtools CLI additions (filelist/multi-path) — do these before or during 016?
