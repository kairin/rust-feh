# Contract: Network Scan Policy

**Feature**: 012-ui-feedback-polish | **FR**: FR-001, FR-002, FR-005

## Detection

`is_network_mount_path(path)` returns true when path string contains:

- `/gvfs/`
- `smb-share:`
- `/nfs/`
- starts with `//` (UNC)

## Scan behavior

When `is_network_mount_path(scan_root)`:

1. `magick_available` passed to `scan_images` MUST be false even if ImageMagick is on PATH.
2. Activity log MUST record network-optimized scan message on scan start.
3. Native extension listing and walkdir behavior unchanged.

## Non-goals

- Does not change feh launch or filelist behavior.
- Does not disable ImageMagick for local paths or for resize demo on selected file.