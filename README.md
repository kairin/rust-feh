# rust-feh

Linux-first **feh orchestrator**: browse and select images at scale in a lightweight GUI; view, navigate, and set wallpaper via **feh**. Lightweight resize for common formats uses the in-process `image` crate; optional ImageMagick extends format coverage when installed.

**Not a feh replacement** — the GUI owns folder scan, filter, sort, and launch; feh owns the viewer and desktop background.

## Status

From-scratch Rust successor to archived **nfeh** — same broad idea (pick from a folder, set wallpaper), different architecture: feh is actually invoked, lists scale to 10k+ images, and the runtime is a single native binary (egui/eframe, no Electron).

- Original nfeh code lives in `archive/original-nfeh/` until rust-feh is fully verified.
- **Positioning:** [docs/POSITIONING.md](docs/POSITIONING.md)
- **nfeh comparison & migration:** [docs/NFEH-COMPARISON-AND-MIGRATION.md](docs/NFEH-COMPARISON-AND-MIGRATION.md)

## Requirements

- Rust (stable) — **install via rustup** (see below)
- `feh` (for viewing and wallpaper features): `sudo apt install feh`
- Optional: `magick` or `convert` (ImageMagick) for more format options in image tools

### Install Rust (do this first)

Since you saw "cargo not found", install the official toolchain with **rustup** (much better than apt's old cargo):

```fish
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Follow the prompts (default is fine).

Then either open a **new terminal** or run:

```fish
source ~/.cargo/env.fish
```

Verify:

```fish
cargo --version
rustc --version
```

### Optional: system libraries for GUI on Ubuntu/Debian

```fish
sudo apt update
sudo apt install -y build-essential pkg-config libssl-dev \
    libxcb1 libxcb-render0 libxcb-shape0 libxcb-xfixes0
```

## Build & Run

```fish
cargo run --release
```

The binary ends up at `target/release/rust-feh`.

To place a copy at the project root (as mentioned in the plan):

```fish
./build-and-place.sh
```

Then you can run `./rust-feh` from the root.

## Current Features (MVP)

- **Persistent layout**: top toolbar + menu bar and bottom status bar stay visible while scrolling large lists
- **Virtualized browsing**: `show_rows` for flat list and folder tree — smooth on 10k+ images (metadata only)
- **Flat list**: Folder + Filename + **Status** columns (`native`, `magick · awaiting convert`, `converted`)
- **Folder tree**: toggle Flat list / Folder tree; expand/collapse folders with per-folder counts
- **Scan inventory bar**: after each scan — native listed, magick-detected, converted, awaiting convert, non-image skipped
- **Filter & sort**: case-insensitive filter (filename, folder, path); Path / Name / Folder sort with scroll reset
- Choose folder (toolbar or File menu); native formats (jpg, png, webp, gif, bmp) plus optional ImageMagick identify for unlisted types
- Select image in list; first image auto-selected on load — **feh does not auto-launch**
- "Open in feh" (filelist across filtered list — cross-subfolder navigation in feh)
- "Set as wallpaper" using `feh --bg-fill`
- Graceful degradation when `feh` is missing (disabled buttons, clear status message)
- Quick resize demo (50%, powered by the `image` crate; `*_processed.*` tracked in inventory)
- **Tools & capabilities panel** (right side): dependency status with ✅ collapsed header when OK, install commands, format routing aligned with scan inventory labels
- **Activity log** (bordered panel, detachable window): selectable text, Copy log / Copy status / Clear logs
- **Background scanning**: UI stays responsive on large or network paths; NAS/GVFS scans skip ImageMagick identify
- **Live status bar**: pulsing scan indicator, count highlight, rotating speed/timing tips (4s) in bottom bar

More coming: thumbnail grid, full image tools dialog, multi-select, keyboard navigation, config persistence.

## Architecture (high level)

See the approved plan for full details. Core modules (`scanner`, `image_proc`, `tool_caps`, `ui_logic`, `types`) are independent of the egui GUI; `main.rs` handles rendering and feh subprocess spawn.

## Documentation

| Doc | Purpose |
|-----|---------|
| [docs/POSITIONING.md](docs/POSITIONING.md) | Product positioning, messaging, claims |
| [docs/NFEH-COMPARISON-AND-MIGRATION.md](docs/NFEH-COMPARISON-AND-MIGRATION.md) | nfeh vs rust-feh tools, formats, migration |
| [specs/OUTSTANDING-ISSUES-ROADMAP.md](specs/OUTSTANDING-ISSUES-ROADMAP.md) | Feature backlog (002–007) |
| [specs/001-persistent-ui-virtual-browsing/](specs/001-persistent-ui-virtual-browsing/) | Primary shipped feature spec |
| [specs/005-image-list-presentation/](specs/005-image-list-presentation/) | Folder column, tree, inventory, status tags |
| [specs/008-tool-capabilities-panel/spec.md](specs/008-tool-capabilities-panel/spec.md) | Tools panel (retroactive) |
| [specs/009-external-tool-runtime/spec.md](specs/009-external-tool-runtime/spec.md) | PATH detect + recheck (supersedes 002) |
| [specs/012-ui-feedback-polish/](specs/012-ui-feedback-polish/) | Status feedback, NAS scan policy, detach log |

## Verification

```fish
./scripts/validate-feature-001.sh
```

Runs build, clippy, tests (10k scan/filter perf, permission-denied, FR static checks). Feature 005: `cargo test feature_005`. Tool runtime: `cargo test tool_caps`.

**GUI performance (feature 003)** — automated tier plus manual scroll/RSS protocol:

```fish
./scripts/validate-gui-performance.sh
```

Manual steps: [specs/003-gui-performance-validation/quickstart.md](specs/003-gui-performance-validation/quickstart.md). Results: [specs/003-gui-performance-validation/validation-results.md](specs/003-gui-performance-validation/validation-results.md).

See also `specs/001-persistent-ui-virtual-browsing/validation-results.md` and `specs/005-image-list-presentation/gap-audit.md`.

## License

MIT — fresh copyright for the new project.
