# Contract: feh Filelist Launch

## Command shape

```text
feh --geometry 1280x960 --scale-down --zoom max \
    --filelist <tempfile> --start-at <selected_abs_path>
```

## Filelist format

- One absolute path per line, UTF-8
- Order = `list_indices()` output (filter + sort)
- Temp path: `{temp_dir}/rust-feh-filelist-{pid}.txt` (overwrite each launch)

## Log line

```text
Spawning feh with filelist (N images): ...
```