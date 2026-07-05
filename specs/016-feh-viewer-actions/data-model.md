# Data Model — 016 Viewer Round-Trip & Staged-Image Actions

All types live in `src/types.rs` (state) with pure helpers in `src/ui_logic.rs`
— egui-independent per Constitution III.

## StagedImage

State of the stage pane. Exactly one at a time; always mirrors the selection.

| Field | Type | Notes |
|---|---|---|
| path | PathBuf | selected image |
| state | enum StageState | `Loading` \| `Ready { width, height }` \| `Failed { reason }` |
| generation | u64 | decode-job tag; only the latest generation's result is applied |

Texture handle itself is GUI-side (main.rs cache keyed by (path, generation));
the model stays egui-free.

## ViewerRoundTrip

One launched cycling viewer. Vec<ViewerRoundTrip> on the app.

| Field | Type | Notes |
|---|---|---|
| child | std::process::Child | retained; polled with try_wait() per frame |
| handoff_path | PathBuf | `runtime_cache_dir()/handoff-<pid>-<viewer_id>` |
| launched_with | PathBuf | image at spawn (scenario US2-2: unchanged if user never navigated) |
| filelist | Vec<PathBuf> | exact list the viewer was launched with — the ONLY acceptable handoff values (R4) |
| viewer_id | u64 | monotonic per session |

Lifecycle: spawn → poll → exit → read handoff → validate (canonicalized ∈
filelist) → select+stage → delete handoff → drop entry → log round-trip
(launched_with, landed_on, outcome).

## ContextAction

| Variant | Input | Output |
|---|---|---|
| SaveCopyTo | staged path + chosen dir | collision-safe copy path |
| MoveTo | staged path + chosen dir | loss-proof move; stage advances |
| ResizeCopy | staged path | derived file per existing Image Tools rules |
| ConvertFormat | staged path + target format | derived file per existing routing |
| CopyPath | staged path | clipboard text |
| CopyImage | staged path | clipboard image (RGBA); fallback = path + message |

Dialog-bearing actions (SaveCopyTo, MoveTo) are refused while one is pending
(edge case: no queued duplicate dialogs).

## ActionOutcome

Activity-log record for every action and round-trip.

| Field | Type |
|---|---|
| action | enum (the ContextAction variant or `RoundTrip`) |
| image | PathBuf |
| destination | Option<PathBuf> |
| result | Ok { produced: Option<PathBuf> } \| Err { reason: String } |

## ActionPrefs (persisted)

`~/.config/rust-feh/action-prefs.json`, version-tagged like WindowPreferences.

| Field | Type | Notes |
|---|---|---|
| version | u32 | 1 |
| last_destination | Option<PathBuf> | starting point for the folder chooser (FR-003) |

## Pure helpers (ui_logic.rs, unit-tested)

- `collision_suffixed_path(dest_dir, file_name) -> PathBuf` (R7)
- `plan_loss_proof_move(src, dest_dir) -> MovePlan` (copy-verify-rename-remove steps)
- `viewer_spawn_command(filelist_path, start_at, handoff_path, profile_dir) -> (program, args, envs)` (R1/R2)
- `validate_handoff(content, filelist) -> Option<PathBuf>` (R4)
- `stage_decode_bounds(w, h, max_edge) -> (u32, u32)` (R5)
