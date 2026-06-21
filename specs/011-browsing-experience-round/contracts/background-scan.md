# Contract: Background Scan

## Start

1. `scanning = true`, clear list state
2. `scan_generation += 1`
3. Spawn thread with `(generation, dir, recursive, magick_available)`
4. Store `Receiver` in `scan_rx`

## Poll (each `update`)

```text
if let Ok(ScanComplete { generation, result }) = rx.try_recv():
  if generation == self.scan_generation:
    apply_scan_result(result)
  scanning = false
```

## Stale results

If `generation != scan_generation`, drop message (newer scan superseded).