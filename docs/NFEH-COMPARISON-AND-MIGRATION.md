# nfeh → rust-feh: Tool Comparison & Migration

This document compares the **archived original nfeh** (`archive/original-nfeh/`) with the **current rust-feh** implementation. It was assembled from parallel analysis of both codebases, specs, and the live `tool_caps` module.

**Audience:** Former nfeh users, contributors evaluating a switch, and maintainers tracking parity.

**Positioning summary:** [POSITIONING.md](POSITIONING.md)

---

## Summary

| | **nfeh** (archived v1.1.0) | **rust-feh** (current) |
|---|---------------------------|------------------------|
| **Relationship** | Original Electron hobby project | From-scratch Rust rewrite (not a line-by-line port) |
| **Stated purpose** | "GUI for feh" | GUI frontend for **feh** + lightweight image tools |
| **Actual viewer** | In-app thumbnails only | **feh** subprocess (explicit "Open in feh") |
| **Wallpaper backend** | `@fa7ad/wallpaper` (Node/npm) | `feh --bg-fill` |
| **feh CLI invoked?** | **No** — listed as Linux package dep only | **Yes** — view + wallpaper |
| **ImageMagick** | Not used | Detected on PATH; routing documented in UI; convert pipeline **not yet wired** |
| **Best for** | Small wallpaper pick from thumbnails | Large-folder browse (10k+), filter/sort, feh-centric workflow |

rust-feh is the **spiritual successor** to nfeh: same product idea (pick images from a folder, set wallpaper, lean on native Linux tools), but a different architecture aimed at scale and feh integration rather than an in-app thumbnail picker.

---

## Should I switch?

| You want… | Stay on archived nfeh | Use rust-feh |
|-----------|----------------------|--------------|
| Visual thumbnail grid in a tiny fixed window | ✓ (only jpg/png/jpeg) | ✗ (list view today; grid not implemented) |
| Browse 1,000+ images with filter/sort | ✗ (sync glob + per-file sizing) | ✓ (virtualized metadata list) |
| Real **feh** viewer with keyboard nav | ✗ | ✓ |
| Wallpaper without installing feh | ✓ (Node wallpaper API) | ✗ (requires `feh` on PATH) |
| Single small binary, no Electron/Node | ✗ | ✓ |
| Image resize / convert in GUI | ✗ | Partial (50% resize demo; full tools deferred) |
| Know which tool handles each format | ✗ | ✓ (Tools & capabilities panel) |

---

## Stack comparison

| Layer | nfeh (`archive/original-nfeh/`) | rust-feh |
|-------|-----------------------------------|----------|
| **Runtime** | Electron ^1.3.5 | Native Rust binary (eframe/glow) |
| **UI** | React 15 + react-desktop (macOS chrome) | egui 0.30 |
| **State** | MobX | Plain struct (`RustFehApp`) |
| **Build** | yarn + webpack + electron-builder | `cargo build --release` |
| **Window** | Fixed 480×480, frameless | Presets + resizable (640×480 floor) |
| **Default folder** | `$HOME/Pictures` | None (user picks each session) |
| **License** | MIT/X11 | MIT (new copyright) |

---

## Tool & dependency comparison

### External tools — who does what

| Capability | nfeh | rust-feh | Notes |
|------------|------|----------|-------|
| **Folder picker** | Electron `dialog.showOpenDialog` | `rfd` native dialog | Both native OS dialogs |
| **Discover images** | `glob` sync `**/*.{jpg,png,jpeg}` | `walkdir` + extension filter | rust-feh: jpg, jpeg, png, webp, gif, bmp |
| **Display previews** | `<img>` + `legit-image` + `react-lazyload` | No thumbnails — filename list | rust-feh (no grid; list UX by design) |
| **Read dimensions** | `image-size` per file (for sort) | Not yet (lazy metadata deferred) | nfeh sorts by aspect ratio |
| **View full image** | Click thumbnail (in-app only) | Spawn **feh** (`--geometry 1280x960 --scale-down --zoom max`) | feh is the real viewer in rust-feh |
| **Slideshow / navigate** | N/A | feh (folder passed as filelist context) | |
| **Set wallpaper** | `@fa7ad/wallpaper`.set(path) | `feh --bg-fill` | **Different backends** — scaling/fit may differ |
| **Resize / convert** | Not implemented | `image` crate demo (50% → JPG) | ImageMagick for exotic (not invoked yet) |
| **feh** | Package dep only (`package.json`) | Required for view + wallpaper | rust-feh degrades gracefully if missing |
| **ImageMagick** | Not referenced | Optional; shown in capabilities panel | `magick` or `convert` on PATH |
| **bash** | Linux package dep | Not required by app | nfeh packaging artifact |

### Speed / timing model (rust-feh only)

rust-feh documents operation speed in the **Tools & capabilities** side panel (`src/tool_caps.rs`, rendered from `src/main.rs`):

| Operation | Handler | Typical speed |
|-----------|---------|---------------|
| Browse / filter / sort list | rust-feh | Instant (10k filter &lt;200ms in CI) |
| Open / slideshow / navigate | feh | Fast (subprocess + GPU) |
| Set wallpaper | feh | Fast |
| Quick resize (jpg/png/webp) | `image` crate | Medium |
| Exotic format view (svg/heic/raw…) | ImageMagick → feh | Slower (not implemented; detection only today) |

nfeh has **no** documented performance model. Architecturally it runs synchronous `glob.sync` on each MobX-driven render and reads dimensions for every file — workable for small folders, not for 10k trees.

---

## Feature matrix

| Feature | nfeh | rust-feh | Parity |
|---------|------|----------|--------|
| Choose folder | ✓ | ✓ | ✓ |
| Recursive subfolders | ✓ (glob `**`) | ✓ (toggle) | ✓ |
| Thumbnail gallery | ✓ (120px max, lazy) | ✗ (virtual list) | **Regression** (intentional; grid not implemented) |
| Sort by aspect ratio | ✓ | ✗ (Path / Name / Folder) | Different |
| Filter / search | ✗ | ✓ (path + filename) | **New** |
| Folder column in list | ✗ | ✓ | **New** |
| Select image | Click thumbnail | Click row | Different UX |
| Open in feh | ✗ | ✓ | **New** (core value) |
| Set wallpaper | ✓ Apply button | ✓ toolbar + menu | ✓ (different backend) |
| Fill mode / scaling options | UI stub only (unused dropdown) | ✗ (only `--bg-fill`) | Neither complete |
| Quick resize | ✗ | ✓ demo | **New** |
| Format convert | ✗ | Partial (demo output JPG) | **New** (minimal) |
| Debug / status log | ✗ | ✓ collapsible panel | **New** |
| Dependency install hints | ✗ | ✓ capabilities panel + copy | **New** |
| feh missing handling | N/A (feh unused) | ✓ disabled buttons + status | **New** |
| Config persistence | ✗ | Partial (window presets; full persistence P2) | Planned |
| Multi-select | ✗ | ✗ | — |
| Keyboard navigation | ✗ | ✗ | Planned |

---

## Format support

| Format group | nfeh scan | rust-feh scan (`scanner.rs`) | rust-feh view (today) | rust-feh resize (today) |
|--------------|-----------|------------------------------|----------------------|-------------------------|
| jpg, jpeg, png | ✓ | ✓ | feh | `image` crate |
| webp, gif, bmp | ✗ | ✓ | feh | `image` crate (webp) |
| tiff, pnm | ✗ | ✗ | feh (if opened elsewhere) | Not scanned |
| svg, heic, raw, psd | ✗ | ✗ | Documented in `tool_caps` only | Not implemented |

The capabilities panel describes routing for exotic formats (where supported). Only the first row is fully wired end-to-end today.

---

## Workflow mapping

### Pick a wallpaper (the original nfeh happy path)

| Step | nfeh | rust-feh |
|------|------|----------|
| 1. Open app | Window opens on `~/Pictures` | Empty state — click **Choose folder** |
| 2. Pick folder | **Choose folder** (if needed) | **Choose folder** or File → Choose folder… |
| 3. Browse | Scroll thumbnail grid | Scroll **Folder / Filename** list; optional **Filter** |
| 4. Select | Click thumbnail (highlight) | Click row (first image auto-selected on load) |
| 5. Apply | **Apply** → `@fa7ad/wallpaper` | **Set as wallpaper (feh --bg-fill)** |
| 6. View full size | Not available | **Open in feh** |

### View images (new in rust-feh)

nfeh never launched feh. rust-feh treats viewing as a **deliberate** action:

1. Select row in list  
2. Click **Open in feh**  
3. feh opens at 1280×960, scales small images up, passes parent dir for navigation  

---

## Install & run (migration)

### nfeh (archived)

```bash
cd archive/original-nfeh
yarn install
yarn start   # or electron .
```

Requires Node, Electron, and Linux packages `feh` + `bash` per `package.json` (feh still unused at runtime).

### rust-feh

```fish
# System deps
sudo apt install feh
sudo apt install imagemagick   # optional

# Build
cargo build --release
./build-and-place.sh
./rust-feh
```

See [README.md](../README.md) for Rust/rustup setup.

**Coexistence:** Both can be installed. They do not share config (nfeh had none; rust-feh persistence is not fully implemented).

---

## Architecture philosophy

| Principle | nfeh | rust-feh |
|-----------|------|----------|
| Thin wrapper over feh | Named only | Constitution §I — delegate view/wallpaper to feh |
| GUI vs core separation | React components + MobX store | `scanner`, `image_proc`, `tool_caps`, `ui_logic` vs `main.rs` |
| Performance at scale | Not a design goal | Virtualized list, 10k targets; SC-004 RSS &lt;150MB **pass** (~126 MB @10k, 2026-06-22 audit) |
| Optional ImageMagick | N/A | Enhance formats; never required |

---

## Known intentional differences

1. **No in-app thumbnails** — rust-feh uses metadata-only list for memory and speed; thumbnail grid is not implemented.
2. **feh is required** for wallpaper in rust-feh; nfeh used a Node wallpaper module.
3. **No default `~/Pictures`** — rust-feh starts with no folder until user picks one.
4. **Larger, resizable window** — nfeh was a fixed 480×480 picker; rust-feh targets browsing workflows.
5. **ImageMagick is informational today** — detected and documented in the UI; convert subprocess not yet called from app code.

---

## Roadmap pointers

| Topic | Where to track |
|-------|----------------|
| Thumbnail grid | Not implemented |
| Full image tools | Not implemented (basic quick resize + feh delegation only) |
| ImageMagick integration | `src/image_proc.rs` comment; capabilities panel |
| Window / feh policy | `specs/006-window-viewer-stability/` |
| List UX formalization | `specs/005-image-list-presentation/` |
| Performance validation | `specs/003-gui-performance-validation/` |
| Master backlog | `specs/OUTSTANDING-ISSUES-ROADMAP.md` |

---

## Archive policy

- Original nfeh sources live in `archive/original-nfeh/` for reference and credit.
- README states the archive is removed only after rust-feh is fully verified.
- This comparison doc should be updated if/when additional features such as thumbnails or expanded ImageMagick support are implemented.

---

## Credits & lineage

- **nfeh** — [fa7ad/nfeh](https://github.com/fa7ad/nfeh), MIT/X11, Electron abandonware (see archived README warning).
- **rust-feh** — New MIT implementation; no Electron/React code carried over.
- **feh** — [feh](https://feh.finalrewind.org/) image viewer (required external tool in rust-feh).
- **ImageMagick** — Optional format detection (full pipelines not implemented).

---

## Live reference in the app

After building rust-feh, open the **Tools & capabilities** panel on the right side of the window. It mirrors much of this document at runtime:

- Installed vs missing dependencies with `sudo apt install …` copy buttons  
- Speed tiers per operation  
- Format → handler routing (scan / view / resize)  
- **Recheck tools on PATH** after installing feh or ImageMagick  

Source: `src/tool_caps.rs` (logic), `src/main.rs` (`render_tool_caps_panel`).