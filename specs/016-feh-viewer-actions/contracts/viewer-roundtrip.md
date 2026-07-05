# Contract — Viewer Round-Trip (spawn / handoff protocol)

## Spawn (rust-feh → feh)

Round-trip viewers are spawned with exactly:

```
program: feh
args:    --geometry 1280x960 --scale-down --zoom max        # existing policy (006)
         --info  echo %F > <handoff_path>                   # single arg, feh-escaped %F
         --filelist <filelist_path>
         --start-at <selected_image>
envs:    XDG_CONFIG_HOME=<config_dir>/rust-feh/viewer-profile
```

Rules:
- Args are passed as an **argument vector** (`std::process::Command::arg`),
  never through a shell constructed by rust-feh. The only shell involved is
  feh's own `/bin/sh -c` for the `--info` command, where `%F` is escaped by
  feh itself.
- `<handoff_path>` = `runtime_cache_dir()/handoff-<rustfeh_pid>-<viewer_id>`;
  it is rust-feh-generated (no user-controlled content) and unique per viewer.
- `<config_dir>/rust-feh/viewer-profile/` is created (empty) before first
  spawn. The user's `~/.config/feh/` is never read, written, or copied.
- Non-round-trip feh launches (wallpaper, Launch All entries) are UNCHANGED by
  this contract.

## Exit handling (feh → rust-feh)

- rust-feh retains the `Child` and polls `try_wait()` every frame.
- On exit (any status, success or signal):
  1. Read `<handoff_path>` if it exists; treat content as UNTRUSTED.
  2. Trim to first line; canonicalize; accept only if the result equals a
     canonicalized entry of the exact filelist this viewer launched with.
  3. Accepted → select that image in the list (scroll to it) and stage it;
     if it no longer passes the active filter, surface that (annotate/clear
     filter) rather than dropping the handoff (spec US2-3).
  4. Rejected/missing → keep current selection; log a warning with the raw
     content length (never echo untrusted content verbatim into the UI).
  5. Delete the handoff file; log the round-trip (launched_with → landed_on).
- If the handoff equals `launched_with`, selection is explicitly unchanged
  (US2-2: "no spurious change").
- Multiple live round-trips: each exit is processed independently in close
  order; the most recent close wins the stage (spec US2-4).

## Failure modes

| Condition | Behavior |
|---|---|
| handoff file absent (feh crashed before first image) | keep state; log |
| handoff content not in filelist (tampered/corrupt) | ignore; warn in log |
| handoff image deleted meanwhile | select nearest surviving neighbor in filelist; log |
| rust-feh exits with viewers open | viewers keep running (assumption: launched tools outlive rust-feh); stale handoff files under runtime cache are cleaned at next startup |
