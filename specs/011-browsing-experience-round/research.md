# Research: Browsing Experience Round

## RQ1: feh multi-directory navigation

**Decision**: Use `--filelist` with one absolute path per line, `--start-at` for selection.

**Rationale**: feh only indexes files in arguments; single parent dir was the dogfood bug. Filelist is documented in `man feh` and supports cross-folder order.

**Alternatives rejected**: Multiple dir args (order undefined); symlinking to temp dir (heavy, SMB write risk).

## RQ2: Non-blocking scan without tokio

**Decision**: `thread::spawn` + `mpsc::channel`; `try_recv` in `App::update`.

**Rationale**: Constitution defers tokio; thread pool unnecessary for one scan at a time.

## RQ3: Copyable log without tmux

**Decision**: `egui::TextEdit::multiline` read-only + `ctx.copy_text` buttons.

**Rationale**: Native clipboard; no terminal embedding; matches existing Tools panel copy pattern.