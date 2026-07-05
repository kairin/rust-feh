# Research — 016 Viewer Round-Trip & Staged-Image Actions

All findings verified live on the target machine (feh 3.x, GNOME Wayland,
XWayland feh windows) on 2026-07-05 unless noted.

## R1. Per-image trail hook (the close-handoff mechanism)

**Decision**: spawn feh with `--info 'echo %F > <handoff-file>'`.

**Verified**: `--info` executes the command each time an image is displayed;
`%F` is the shell-escaped path of the current image. With the command's stdout
redirected to a file, **nothing is drawn on screen** (feh draws the command's
stdout; empty stdout = no overlay). The handoff file then always contains the
currently displayed image's path; at viewer exit it holds the landed-on image.

**Rejected — `--info ';command'` (silent flag)**: verified that with the `;`
flag feh defers running the command until the user presses toggle_info, so the
handoff file is never written during normal browsing. Not usable.

**Rejected — window-title polling**: feh titles do contain the current path,
but polling requires X11 client machinery or an `xprop` subprocess (new
external dependency) and breaks under pure-Wayland assumptions. Unnecessary
given R1.

**Rejected — `--action`-on-quit**: would demand a dedicated "select" key
instead of the natural close (q / Escape / window button), contradicting the
maintainer's flow ("close the feh, that last image will be loaded").

## R2. Viewer config isolation

**Decision**: spawn feh with `XDG_CONFIG_HOME=~/.config/rust-feh/viewer-profile`
(directory created empty on first use).

**Verified**: feh reads `keys`/`buttons`/`themes` from `$XDG_CONFIG_HOME/feh/`,
falling back to `~/.config/feh/` only when the variable is unset. Pointing it
at a rust-feh-owned (empty) directory yields stock defaults: standard
navigation, standard quit keys, and — importantly — none of the personal
theme's on-image overlays (verified: the "defined actions" numbering the
maintainer disliked came from their personal feh theme and disappears under
isolation). The personal config is never read or written; manual feh sessions
are untouched.

## R3. Viewer exit detection

**Decision**: retain the `std::process::Child` for round-trip spawns and call
`try_wait()` once per egui frame (with `ctx.request_repaint_after` keeping a
low-frequency tick when idle). On `Some(status)` — any status, including
abnormal — read + validate the handoff.

**Note**: existing feh spawns drop the `Child` (fire-and-forget); those code
paths remain for non-round-trip launches (wallpaper, Launch All), but
round-trip spawns must keep the handle. Side benefit: proper reaping (no
zombies) for these children.

## R4. Handoff validation (security)

**Decision**: treat the handoff file as untrusted input. Accept its content
only if it canonicalizes to a path that was in the exact filelist this viewer
was launched with (rust-feh already writes that filelist); otherwise ignore
with an activity-log warning. File lives under `runtime_cache_dir()` with a
`handoff-<pid>-<viewer-id>` name; deleted after read.

## R5. Stage decode

**Decision**: decode off-thread (existing background-thread + channel pattern
used by the scanner), downscale so the longest edge ≤ 2048 px before creating
the egui texture, and tag each job with a generation counter so only the
latest selection's result is applied. Formats: whatever `image` 0.25 decodes
in-process (jpeg/png/webp/gif/bmp per current scan set); undecodable files get
the FR-012 placeholder (the feh round-trip still handles viewing them when feh
supports them).

## R6. Context menu & dialogs

**Decision**: egui's built-in `Response::context_menu` on the stage image
widget provides right-click menus natively — no new UI framework surface.
Folder chooser and error dialogs use `rfd` (already a dependency; already
used for folder picking). Clipboard text (path) and image (RGBA) use
`arboard` (already a dependency).

## R7. Collision-safe naming & loss-proof move

**Decision**: `name.ext` → `name-1.ext`, `name-2.ext`, … first free suffix
(pure function, unit-tested). Move = copy to temp name at destination →
verify byte length (and same-filesystem `rename` fast path when possible) →
atomic rename into final name → remove source only after verification.
Cross-filesystem covered by the copy path; interrupted operations leave the
source intact plus at most one orphaned temp file that is reported.

## R8. Deferred/rejected scope

- feh-side action shortcuts (previous 016 revision): superseded by the
  rust-side context menu (maintainer rework decision).
- Zoom/pan/slideshow in the stage: out of scope — feh is the engine
  (Constitution I; spec Assumptions).
- Set-as-wallpaper menu item: not selected for v1.
