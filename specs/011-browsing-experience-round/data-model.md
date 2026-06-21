# Data Model: Browsing Experience Round

## ScanJob (main.rs)

| Field | Type | Notes |
|-------|------|-------|
| `scan_generation` | `u64` | Incremented each scan start |
| `scan_rx` | `Option<Receiver<ScanComplete>>` | Latest job receiver |

## ScanComplete

| Field | Type |
|-------|------|
| `generation` | `u64` |
| `result` | `ScanResult` |

## FehFilelist (ephemeral)

| Field | Type |
|-------|------|
| `path` | `PathBuf` | Under `temp_dir()` |
| `entries` | `usize` | Line count logged |

## ActivityLog (unchanged buffer + UI)

| Field | Type |
|-------|------|
| `debug_logs` | `Vec<String>` | Ring cap 100 |
| `log_display` | derived join for TextEdit |