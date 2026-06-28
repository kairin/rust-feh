# Phase 0 Research: Window & Viewer Stability (006)

All decisions verified against the existing codebase so the implementation mirrors a proven, constitution-compliant pattern.

## D1 — Persistence format: JSON via serde

- **Decision**: Serialize `WindowPreferences` to JSON with `serde` / `serde_json`.
- **Rationale**: The project already persists `FehLaunchList` as pretty JSON (`save_launch_list` in `ui_logic.rs`) using `serde`/`serde_json`, both already in `Cargo.toml` (added for feature 014). Reusing the same format keeps one config convention and adds zero dependencies (constitution §II).
- **Alternatives considered**: TOML (`toml` crate — new dependency, rejected); a hand-rolled key=value file (more code, no validation, rejected); `eframe` persistence/`egui` storage (pulls window state into the GUI layer, violates constitution §III separation, rejected).

## D2 — File location: XDG config dir

- **Decision**: `~/.config/rust-feh/window-prefs.json`, resolved by a `window_prefs_path()` helper.
- **Rationale**: Sibling of the existing `~/.config/rust-feh/launch-entries.json` (`launch_list_path()`). Same `HOME`-based resolution keeps tests simple (set `HOME` to a temp dir, exactly as the launch-list unit tests do).
- **Alternatives considered**: `dirs`/`directories` crate for true XDG resolution (new dependency; existing code already uses the simpler `HOME`-based approach, so match it for consistency — rejected); runtime `~/.cache` dir (cache is for ephemeral filelists, not user prefs — rejected).

## D3 — Save timing: on-change, not on-exit

- **Decision**: Persist whenever the user changes the preset or toggles resizable, inside the existing `sync_frame_input_state` change-detection branches.
- **Rationale**: `main.rs` already detects `window_size != prior_window_size` and `window_resizable != prior_window_resizable` each frame and applies the change. Hooking save there reuses proven plumbing and avoids relying on `eframe::App::on_exit`, which may not fire on crash/SIGKILL. Mirrors how `persist_launch_entries` is called right after a mutation.
- **Alternatives considered**: Save in `on_exit` (missed on abnormal exit; `panic = "abort"` in release profile makes clean teardown less certain — rejected); debounced timer (unnecessary complexity for a setting changed rarely — rejected).

## D4 — Load timing: at app construction

- **Decision**: Call `load_window_prefs()` in `create_rust_feh_app` and seed `window_size`, `prior_window_size`, `window_resizable`, `prior_window_resizable` from it.
- **Rationale**: The struct fields are currently initialized to `WindowSizePreset::default()` / `true` at construction. Seeding both the live and `prior_*` fields from loaded prefs prevents a spurious "changed" event on the first frame (which would otherwise re-save and re-apply). The initial `NativeOptions` inner size can keep the Default preset; the first frame's `apply_window_preset` reconciles to the loaded preset via the existing on-change path — or, optionally, `try_run_gui` can load prefs to size the initial viewport (documented as a follow-up, not required for SC-005).
- **Alternatives considered**: Loading inside `try_run_gui` to size `NativeOptions` directly (cleaner first-paint, but touches the pre-App bootstrap; acceptable as an enhancement, not required to satisfy "preferences survive restart").

## D5 — Corruption / first-run handling

- **Decision**: Missing or unparseable file → `WindowPreferences::default()` with an `eprintln!` warning.
- **Rationale**: Identical to `load_launch_list`, which recovers to `FehLaunchList::default()` and warns on corrupt JSON. Satisfies spec US4 scenario 2 (first run → Default preset, resizable=true) and the edge-case requirement that a bad file never crashes the app.
- **Alternatives considered**: Hard error on corrupt file (poor UX, rejected); silent overwrite without warning (loses diagnosability, rejected).

## D6 — `WindowSizePreset` serde derives

- **Decision**: Add `Serialize, Deserialize` to `WindowSizePreset` (currently `Debug, Clone, Copy, PartialEq, Eq, Default`).
- **Rationale**: Required to embed the enum in `WindowPreferences`. The default serde representation (variant name string: `"Compact"`/`"Default"`/`"Large"`) is human-readable and stable.
- **Alternatives considered**: Persist raw width/height instead of the enum (loses the "which preset is selected" UI state and complicates restoring the menu selection — rejected); custom int discriminant (less readable, no benefit — rejected).

## Open clarifications

None. All Technical Context items are resolved; no `NEEDS CLARIFICATION` markers remain.
