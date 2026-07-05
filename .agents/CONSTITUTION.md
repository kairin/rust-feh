# .agents/CONSTITUTION.md — Binding constitutional rules (pointer + summary)

**Source of truth: `.specify/memory/constitution.md` (v1.0.1, amended 2026-06-24).**
This file is a summary for agents; when in doubt, read the source. These rules bind
class-A AND class-B work alike. Every agent reads this file plus its task file before acting.

## The five MUST principles

1. **Thin-Wrapper Architecture** — rust-feh is a GUI *frontend* for feh, not a replacement.
   NEVER reimplement feh features in Rust; delegate via subprocess spawn. New features need
   a concrete user need feh does not address.
2. **Pure Rust, Minimal Dependencies** — crates.io only, actively maintained. No Qt/Electron/
   GTK. egui/eframe is the approved GUI toolkit. Release profile keeps LTO, single codegen
   unit, panic=abort.
3. **Clean Module Separation** — core modules `scanner`, `image_proc`, `types`, `ui_logic`,
   `tool_caps` stay egui-independent (re-exported by lib.rs). GUI never leaks into core;
   core never depends on GUI types. Async/threading extracted from GUI when introduced.
4. **Linux-First, feh-Centric** — feh detected or installable; graceful degradation with a
   clear status message when missing. Wallpaper = `feh --bg-fill` only. ImageMagick optional,
   enhancement-only; the `image` crate is the always-available processor.
5. **Performance Awareness** — walkdir with symlink-follow disabled; metadata lazy; never read
   full image data during listing; UI responsive during scans.

## Technical standards that constrain integration work
- GUI stack pinned: **egui 0.30 / eframe 0.30, glow backend** ("fewer graphics driver
  requirements than wgpu" — deliberate). Changing backend or major-bumping the stack is a
  Technical Standards change → constitution amendment territory.
- License: **MIT** with SPDX headers. Importing GPL code (e.g. from image-tools) is a
  license event → HUMAN decision, currently ruled OUT (user decision 2026-07-05: keep
  licenses as-is; no GPL code into rust-feh).
- No network access from rust-feh itself. External tools are subprocesses detected via the
  capability matrix (`tool_caps.rs`).

## Amendment procedure (humans only)
Documented proposal (PR/issue) → consistency review against all specs/plans → semver bump
(MAJOR: principle removal/redefinition; MINOR: new principle/section; PATCH: wording) →
propagate to templates/active plans. **Agents never amend the constitution.** If a task
conflicts with a principle, STOP and surface to the maintainer.
