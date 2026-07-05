# Quickstart — 016 Viewer Round-Trip & Staged-Image Actions

Manual/scripted validation scenarios. Automated proxies exist for the logic
underneath each (see tasks.md); these confirm the real UX.

Fixture: a folder with ~20 mixed images including at least one with spaces +
non-ASCII in the name, one undecodable-in-process file (e.g. HEIC without
ImageMagick), and duplicate-named files in a second destination folder.

### V1 — Stage displays selection

1. Load the fixture; click several images in the list.
2. **Verify**: each selected image appears in the stage pane within ~0.5 s;
   list scrolling stays smooth; no text is drawn over the image.

### V2 — Round-trip: cycle in feh, close, land in rust-feh

1. Select image A; open the viewer (Open in feh).
2. In feh, navigate several images; note the image you stop on (B); press `q`.
3. **Verify**: within 1 s rust-feh selects B, scrolls the list to it, stages
   it; activity log records the round-trip (A → B).
4. Repeat but close via the window's close button, and once via `kill <pid>`.
5. Repeat without navigating: selection stays A ("no spurious change").

### V3 — Clean isolated viewer

1. In the round-trip viewer, confirm stock feh behavior (scroll = prev/next)
   and NO filename/actions overlay text (personal feh theme must not apply).
2. Run `feh <some image>` manually in a terminal: personal theme/overlays
   appear as before. `diff -r` personal `~/.config/feh` before/after session:
   identical.

### V4 — Context actions

1. Right-click the staged image: exactly six items (Save a copy…, Move to…,
   Resize copy, Convert format ▸, Copy path, Copy image).
2. Save a copy… into the destination folder that already has a file with the
   same name → **verify** suffixed copy (`name-1.ext`), original untouched.
3. Move to… another folder → **verify** file moved, stage advances, list
   updates; repeat onto a read-only folder → native error dialog, source
   intact.
4. Copy path → paste in a terminal; Copy image → paste into an image editor.
5. Run 2–4 on the spaces+non-ASCII filename.

### V5 — Undecodable file

1. Select the undecodable file.
2. **Verify**: placeholder + reason in the stage; Copy image / Resize disabled;
   Copy path / Move / Save copy still work; round-trip through feh still
   works if feh can show it.

### V6 — Performance guard

1. Load the 10k perf fixture (`scripts/generate-perf-fixture.sh`).
2. **Verify**: scan + filter timings within 10% of the 2026-07-05 baseline
   (see `.agents/ENV.md`), stage updates lazily while scrolling selections
   rapidly, RSS stays within the established budget.
