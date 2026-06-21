# Quickstart: Browsing Experience Round

## V1 — Cross-folder feh

1. Load recursive folder with images in ≥2 subfolders.
2. Select image in subfolder A → Open in feh.
3. In feh, press `n` until you enter subfolder B.
4. **Pass**: No relaunch; order matches rust-feh list.

## V2 — Responsive scan

1. Choose folder on SMB/GVFS path.
2. While "Scanning…", resize window and open View menu.
3. **Pass**: UI responds within 1s.

## V3 — Scan warnings

1. On Unix: folder with permission-denied subdir + scan.
2. Expand Activity log.
3. **Pass**: "Permission denied" line; other errors show `Scan skip:`.

## V4 — Copy log

1. After scan, expand Activity log.
2. Click **Copy log** → paste in terminal.
3. **Pass**: Full log text pasted.