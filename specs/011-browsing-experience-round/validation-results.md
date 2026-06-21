# Browsing Experience Round — Validation Results

**Run ID**: 2026-06-22-implement  
**Date**: 2026-06-22  
**Tester**: agent (automated); manual SMB/GVFS dogfood pending human  
**Environment**: Linux; DISPLAY for manual tiers

## Automated tier

| Check | Result |
|-------|--------|
| `cargo test` | pass |
| `cargo clippy -- -D warnings` | pass |
| `t068_permission_denied_warning` | pass |
| `t069_scan_skip_non_permission` | pass |
| `feh_filelist_order_matches_list_indices_sorts` | pass |
| `format_walk_warning_*` unit tests | pass |

## Manual tier (quickstart V1–V4)

| Scenario | SC | Verdict | Notes |
|----------|-----|---------|-------|
| V1 Cross-folder feh (`n` across subfolders) | SC-001 | **pending** | Re-test on SMB tree after filelist change |
| V2 Responsive scan during SMB load | SC-002 | **pending** | Resize window + menus while Scanning… |
| V3 Scan warnings in Activity log | SC-003 | partial | Automated t068/t069; manual mixed fixture optional |
| V4 Copy log / Copy status | SC-005 | **pending** | Expand Activity log → Copy log → paste |

### Manual handoff

```fish
cd /home/kkk/Apps/rust-feh
cargo build --release && ./rust-feh
# V1: load recursive SMB or local multi-subfolder tree → Open in feh → press n across folder boundary
# V2: during scan, open View menu and resize window
# V4: Copy log after scan+feh open
# Update verdict columns above when done
```