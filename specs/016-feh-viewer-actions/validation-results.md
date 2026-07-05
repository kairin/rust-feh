# Viewer Round-Trip & Staged-Image Actions — Validation Results (T026)

**Run ID**: 2026-07-05-016-scripted-validation
**Date**: 2026-07-05
**Tester**: Sonnet orchestrator (scripted), no human-driven click session performed
**Environment**: Linux desktop (GNOME/Wayland), screenshots forced through XWayland per
`.agents/ENV.md` (`env -u WAYLAND_DISPLAY` + ImageMagick `import`/`xwininfo`), `rust-feh`
built in release mode (`cargo build --release`).

## Environment constraint (read first)

Per `.agents/ENV.md`: synthetic input (XTEST) is not delivered under this
XWayland/libei-portal setup. This session additionally confirmed **no synthetic-input
tool is even installed** (`xdotool`, `ydotool`, `wtype` all absent) — so click-driven
and keystroke-driven interaction with the rust-feh window (selecting a different list
row, right-clicking the stage, clicking "Open in feh") could not be scripted at all in
this environment. Where a scenario's key step is a mouse click, this is stated
explicitly below as **requires a short manual check**, with everything reachable
without a click evidenced automatically. `RUST_FEH_START_FOLDER` (auto-load + auto-select
first image, no click needed) and real `feh` invocations (driven directly by shell,
reproducing exactly what `viewer_spawn_command`'s unit-tested output constructs) covered
the rest.

## V1 — Stage displays selection

**Scenario**: load a folder; each selected image appears in the stage pane within
~0.5 s; list scrolling stays smooth; no text drawn over the image.

- Launched `rust-feh` with `RUST_FEH_START_FOLDER=/tmp/rust-feh-016-v1` (a fixture
  folder containing one 64×48 solid-green JPEG, `first-image.jpg`), forcing
  auto-select of the only/first image with no click needed.
- Screenshot (`import -window rust-feh`) taken ~3 s after launch shows: the stage pane
  ("☐ Stage · 64×48") rendering the exact green rectangle, positioned below the image
  list, collapsible toggle visible, dimension label ("64×48") drawn **outside** the
  image rectangle (never over it).
- **Verdict: PASS** for the single-selection case (auto-select, no click needed).
  Cycling through *several* different images via list clicks was not independently
  click-driven in this session (see environment constraint) — the underlying
  selection→stage wiring (`kick_stage_decode_if_selection_changed`/`poll_stage_decode`,
  generation-tagged, discards stale decodes) is exercised by 3 unit tests in
  `src/image_proc.rs` (`decode_stage_rgba_*`) and 5 in `src/ui_logic.rs`
  (`stage_decode_bounds_*`), and by code review of the frame-loop wiring in `main.rs`.
  **Multi-image click-through: requires a short (~1 min) manual check.**

## V2 — Round-trip: cycle in feh, close, land in rust-feh

**Scenario**: open the viewer from rust-feh, navigate, close feh; rust-feh selects and
displays the landed-on image within 1 s, list scrolls to it; closing without
navigating leaves the selection unchanged.

The GUI-side trigger ("Open in feh" click) could not be scripted (no synthetic click).
Instead, the **actual round-trip protocol** was exercised live against a real `feh`
process, using the exact spawn shape `viewer_spawn_command` builds and unit-tests
(`--geometry 1280x960 --scale-down --zoom max --info "echo %F > <handoff>"
--filelist <list> --start-at <path>`, `XDG_CONFIG_HOME=<viewer_profile_dir>`):

1. Spawned real `feh` on an 11-image fixture (`/tmp/rust-feh-016-fixture`, including
   the adversarial `vacation café day.jpg` filename), `--start-at img-01.jpg`, with
   the real handoff path under `~/.cache/rust-feh/`.
   - Handoff file appeared immediately with content `img-01.jpg` — confirms R1's
     `--info` hook fires on the initial displayed image with **no on-screen overlay**
     (screenshot: solid-color frame, only the WM titlebar shows the path — see V3).
   - Sent `SIGTERM` (simulating "closed via kill signal", the abnormal-termination
     edge case) without any navigation: **handoff still read `img-01.jpg`**
     — the exact "no spurious change" property (US2-2), proven live.
2. Repeated with `--slideshow-delay 1` (feh's own auto-advance, used as a scripted
   substitute for "user pressed next several times" since no keypress could be sent):
   handoff content was observed to advance through the filelist (`img-01` → … →
   `img-05` mid-session) before being killed, and held the last-displayed image
   (`img-02`) at the moment of close — confirms the handoff always reflects whichever
   image was on screen at close time, which is exactly what `poll_round_trip_viewers`
   /`handle_round_trip_exit` read once `try_wait()` reports the process gone.
3. `validate_handoff`/`handoff_path` against these exact real files are additionally
   covered by 6 automated integration tests
   (`tests/integration/feature_016_roundtrip.rs`): correct resolution, the unchanged
   case, tampered/garbage content rejected, empty content rejected, and two
   independent round-trips resolving without cross-contamination.
- **Verdict: PASS** for the handoff protocol end-to-end against real feh (spawn shape,
  no-overlay `--info` hook, correct-image resolution, no-spurious-change,
  abnormal-termination handling). The GUI-visible effect (rust-feh's own window
  re-selecting/scrolling/staging after a *real click-driven* "Open in feh" → browse →
  close) **requires a short (~1 min) manual check** — the click to open the viewer
  cannot be scripted here. "Most-recent-close-wins" ordering across two simultaneously
  open round-trip viewers is a GUI-polling-order property in `main.rs`'s
  `poll_round_trip_viewers` (each exit handled independently, most recent overwrites
  `self.selected` last) — the integration test proves its necessary precondition
  (independent, non-cross-contaminating resolution); the ordering itself is not
  independently GUI-driven here.

## V3 — Clean isolated viewer

**Scenario**: launched viewer shows stock feh behaviour with no overlay text; the
personal feh config is untouched; manually-started feh is unaffected.

- Snapshotted `~/.config/feh/themes` (the real personal theme file on this machine,
  confirmed to exist) via `md5sum` before any round-trip spawn.
- Spawned real `feh` with `XDG_CONFIG_HOME=~/.config/rust-feh/viewer-profile` (the
  real `viewer_profile_dir()` path) exactly as above; screenshot confirms a plain
  solid-color frame with **no filename/action-list overlay text drawn on the image**
  (only the window manager's own titlebar shows the path, which is WM chrome, not
  feh's own draw).
- After **two** separate real round-trip sessions (static + slideshow), re-ran
  `md5sum -c` against the pre-session snapshot: `~/.config/feh/themes: OK` — **byte
  identical**, confirming SC-006 (0 modifications to the personal feh configuration).
- **Verdict: PASS** (isolation + no-overlay + personal-config-untouched, all verified
  live against the real, pre-existing personal feh config on this machine).

## V4 — Context actions

**Scenario**: right-click the staged image; exactly six items; save/move/resize/
convert/copy-path/copy-image all work correctly, including collision-safety and
adversarial filenames.

The right-click itself could not be scripted (no synthetic click) — **requires a
short (~1 min) manual check** to confirm the menu's exact six items render and the
disabled/enabled state is visually correct for a decodable vs. undecodable stage (the
`decodable` flag wiring was code-reviewed: `render_stage_context_menu` disables
Resize/Convert/Copy-image, not Save-copy/Move/Copy-path, exactly matching FR-012).

Everything reachable without a click was already covered automatically:

- `tests/integration/feature_016_actions.rs` (6 tests, real tempdir fixtures): basic
  save-copy + collision-suffixing, loss-proof move + collision-suffixing, resize-copy
  (original untouched, correct output dimensions), convert-format (PNG→JPEG), and both
  save-copy and move against a filename with spaces + non-ASCII (`vacation café
  day.png`) — SC-003 (0 overwrites) and SC-004 (adversarial filenames) both green.
- `src/ui_logic.rs` unit tests for `collision_suffixed_path` (6), `save_copy_to` (4),
  `plan_loss_proof_move`/`execute_move_plan` (4), `format_action_outcome` (5).
- Native folder chooser (`rfd::FileDialog`) start-at/persist-on-use wiring
  (`pick_action_destination`) is code-reviewed against `ActionPrefs` round-trip tests
  (3 unit tests, `action_prefs_*`).
- **Verdict: file-operation logic PASS (fully automated); the six-item menu rendering
  and right-click interaction itself requires the short manual check noted above.**

## V5 — Undecodable file

**Scenario**: select an undecodable file; the stage shows a placeholder + reason;
Copy image/Resize disabled; Copy path/Move/Save copy still available; the feh
round-trip still works for such files.

- Launched `rust-feh` with `RUST_FEH_START_FOLDER=/tmp/rust-feh-016-v5` (a fixture
  folder containing one file, `broken.jpg`, with valid extension but garbage bytes —
  passes the scanner's extension filter, fails `image::open`), auto-selected with no
  click needed.
- Screenshot confirms the stage status line reads exactly:
  `Cannot preview broken.jpg: Failed to decode /tmp/rust-feh-016-v5/broken...` — a
  clear placeholder + reason, no crash, no stale/broken frame.
- Disabled-action wiring (Copy image/Resize/Convert disabled when
  `StageState != Ready`, Copy path/Move/Save copy always enabled) is code-reviewed in
  `render_stage_context_menu`; `decode_stage_rgba_undecodable_file_returns_error`
  (unit test) proves the underlying decode error path never panics.
- The feh round-trip does not depend on in-process decodability at all (feh has its
  own format support, unrelated to the `image` crate) — no additional check needed
  beyond V2's already-passing protocol tests.
- **Verdict: PASS.**

## V6 — Performance guard

**Scenario**: 10k-image fixture; scan/filter timings within 10% of the pre-feature
baseline; stage updates lazily; RSS stays within budget.

Built the pre-feature commit (`186139c`, the tip immediately before this feature's
first commit) in a scratch `git worktree` for a true side-by-side comparison, using
the existing `scripts/generate-perf-fixture.sh` (10,000 tiny fixture files) and
`scripts/measure-resources.sh`.

**RSS** (`scripts/measure-resources.sh 10000 30`, this feature's build):

| Metric | Value | Goal | Verdict |
|---|---|---|---|
| RSS min | 95.7 MB | — | — |
| RSS avg | 139.0 MB | — | — |
| RSS peak | 140.5 MB | < 150 MB (SC-004) | **PASS** |

(`.agents/ENV.md`'s historical baseline for the same protocol was ~126 MB; the small
increase is consistent with the new bounded stage-pane texture cache holding one
decoded ≤2048px-edge image, not a per-item cost — RSS still comfortably under the
150 MB ceiling.)

**Scan + background-pass wall time** (auto-load via `RUST_FEH_START_FOLDER`, timed
from process start to the `"Converted-status metadata updated (background)"` log
line, 3 runs each, fresh 10k fixture per run):

| Build | Run 1 | Run 2 | Run 3 | Avg |
|---|---|---|---|---|
| Baseline (`186139c`, pre-feature) | 231.0 ms | 221.5 ms | 225.1 ms | 225.9 ms |
| Feature 016 (this branch, HEAD) | 234.1 ms | 233.4 ms | 233.4 ms | 233.6 ms |

Delta: **+3.4%**, well within the 10% SC-005 budget. This is also expected by
construction, not just by measurement: this feature never modifies `src/scanner.rs`,
or `list_indices`/`filter_indices`/`build_folder_tree` in `src/ui_logic.rs` — the scan
and filter pipeline is untouched code; the only new per-frame cost on the hot path is
one `Option<PathBuf>` equality check (`self.selected == self.stage_requested_path`) to
detect a new selection, which is why the two builds land within measurement noise of
each other.

- **Verdict: PASS** (RSS and scan/filter timing both within budget; stage decode is
  off-thread and only triggered on selection change, so it does not run during
  scan/filter at all).

## Summary

| Scenario | Automated/scripted result | Manual check still needed |
|---|---|---|
| V1 Stage displays selection | PASS (single auto-selected image, screenshot) | Multi-image click-through (~1 min) |
| V2 Round-trip cycle/close | PASS (protocol proven live with real feh + 6 integration tests) | GUI click-driven open→browse→close (~1 min) |
| V3 Clean isolated viewer | PASS (screenshot + real personal-config md5 unchanged) | none |
| V4 Context actions | PASS (file-op logic, 6 integration + 18 unit tests) | Right-click menu rendering/interaction (~1 min) |
| V5 Undecodable file | PASS (screenshot: placeholder + reason) | none |
| V6 Performance guard | PASS (RSS 140.5 MB < 150 MB; scan/filter +3.4%, within 10%) | none |

No regressions found. Three short (~1 minute each) manual click-driven checks remain
because no synthetic-input tool is installed in this environment and none could be
installed without `sudo`/explicit approval — everything reachable without a mouse
click or keypress was evidenced above, either via screenshot, a real (non-mocked)
`feh` process, or the feature's existing automated test suite (cross-referenced by
name, not re-derived here).

## Fixtures used (scratch, not committed)

- `/tmp/rust-feh-016-fixture/` — 11 images (10 solid-color JPEGs + one
  `vacation café day.jpg`), plus `dest/`/`dest2/` collision-test subfolders.
- `/tmp/rust-feh-016-v1/first-image.jpg` — single-image auto-select fixture (V1).
- `/tmp/rust-feh-016-v5/broken.jpg` — single undecodable-file fixture (V5).
- `scripts/generate-perf-fixture.sh 10000` — V6 RSS/timing fixtures (regenerated per
  run, auto-cleaned).

Screenshots captured during this session (not committed to the repo, consistent with
prior features' `validation-results.md` files, which are text/log-based): stage pane
with auto-selected image, undecodable-file placeholder, and the isolated real-feh
window with no overlay — described in full above.
