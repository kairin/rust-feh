# Magick Cache setup for rust-feh

**Purpose**: Install `magick-cache`, create a cache repository with a passkey, configure rust-feh, and verify cache hits.

`magick-cache` is **not** bundled with the standard Ubuntu `imagemagick` package. You build it from the [MagickCache](https://github.com/ImageMagick/MagickCache) project. rust-feh calls it via `std::process::Command` only — the app does not run `create` for you (one-time external setup).

## Prerequisites

| Requirement | Check |
|-------------|-------|
| ImageMagick 7+ (`magick` on PATH) | `magick -version` |
| feh (viewing) | `which feh` |
| Build tools (compile MagickCache) | `gcc`, `make`, `git` |
| ImageMagick **dev** headers | `pkg-config --modversion MagickCore` |
| autotools (MagickCache `make`) | `aclocal-1.18` or `automake` |

Install on Ubuntu/Debian:

```fish
sudo apt install -y build-essential git pkg-config automake autoconf libtool \
    libmagickcore-7.q16-dev libmagickwand-7.q16-dev
```

Verify headers:

```fish
pkg-config --modversion MagickCore
# Expected: 7.x (e.g. 7.1.2)
```

If `pkg-config` reports *Package 'MagickCore' not found*, install `libmagickcore-7.q16-dev` and retry.

## Build and install magick-cache

```fish
cd ~/Apps   # or any build directory
git clone -b main https://github.com/ImageMagick/MagickCache.git
cd MagickCache
./configure
make -j(nproc)
sudo make install
sudo ldconfig
```

`configure` must show `checking for MagickCore >= 7.1.0... yes`. If it says **no**, install the dev packages above and run `./configure` again.

`make` requires **automake** (`aclocal-1.18`). If you see:

```text
aclocal-1.18: command not found
```

install `automake autoconf libtool` and rerun `make`.

After install, verify the binary and shared library:

```fish
which magick-cache          # often /usr/local/bin/magick-cache
magick-cache                  # prints version + usage
```

If you get `error while loading shared libraries: libMagickCache.so.0`, run `sudo ldconfig` or add `/usr/local/lib` to your linker path.

If `/usr/local/bin` is not on PATH (common in fish), either add it to config or symlink:

```fish
sudo ln -sf /usr/local/bin/magick-cache /usr/bin/magick-cache
```

## One-time cache repository + passkey

Choose a cache root (SSD recommended) and a passkey file. **Keep the passkey safe** — without it you cannot get, identify, or expire your cached entries.

```fish
mkdir -p ~/.cache/rust-feh-magick
echo -n "your-secret-passkey" > ~/.magick-cache-passkey
chmod 600 ~/.magick-cache-passkey

magick-cache -passkey ~/.magick-cache-passkey create ~/.cache/rust-feh-magick
```

Sanity check (should list 0 resources on a new cache):

```fish
magick-cache -passkey ~/.magick-cache-passkey identify ~/.cache/rust-feh-magick /
```

### CLI syntax note

Upstream `magick-cache` uses **single-dash** flags and puts the **cache root** before the IRI:

```fish
magick-cache -passkey <file> -ttl "90 days" put <cache-root> <iri> <input-file>
magick-cache -passkey <file> get <cache-root> <iri> <output-file>
```

IRIs use the form `project/type/resource-path` where `type` is `image`, `blob`, or `meta`. rust-feh generates IRIs like `rustfeh/image/processed/<hash>`.

## Configure rust-feh (in-app)

1. Build and run: `cargo run --release` or `./target/release/rust-feh`
2. Choose a folder with images
3. **Inspector → Image actions → Image Tools → Cache** tab
4. Check **Enable cache**
5. **Pick root…** → `~/.cache/rust-feh-magick` (or your cache path)
6. **Pick passkey…** → `~/.magick-cache-passkey`
7. **TTL** → `90 days` (default; in-memory for this session)
8. Click **Apply cache settings**

The panel should show `magick-cache: ready` when the binary is on PATH and the passkey identifies the cache. Settings are **in-memory only** for v1 (not persisted across app restarts).

### What cache enables

- **Automatic put/get** on single and batch image operations (cache hit logged as `[cache hit]` in Activity Log)
- **Pre-cache folder** — background job puts originals into the cache
- **Prepare Fast feh** — materializes optimized JPEGs; **Launch feh on optimized** uses a temp filelist

Basic resize/convert still works via the `image` crate when cache is disabled or unavailable.

## Verify cache hits

### In the GUI

1. Enable cache and apply settings (above)
2. Resize the same image twice with **identical** parameters
3. First run: full compute + cache put
4. Second run: fast path; Activity Log should show `[cache hit]`

### Automated test

```fish
cd /path/to/rust-feh
cargo test --test integration_image_tools cache_repeat_resize_faster_when_cache_available -- --nocapture
```

Requires `magick-cache` on PATH and passkey at `~/.magick-cache-passkey` with cache at `~/.cache/rust-feh-magick`. If not configured, the test passes but prints a skip/hint message.

### Manual magick-cache put/get

```fish
magick -size 32x32 xc:red /tmp/test-cache.png
magick-cache -passkey ~/.magick-cache-passkey -ttl "90 days" \
  put ~/.cache/rust-feh-magick rustfeh/image/test/v1 /tmp/test-cache.png
magick-cache -passkey ~/.magick-cache-passkey identify ~/.cache/rust-feh-magick rustfeh/image/test
magick-cache -passkey ~/.magick-cache-passkey \
  get ~/.cache/rust-feh-magick rustfeh/image/test/v1 /tmp/out.png
```

## Troubleshooting

| Symptom | Fix |
|---------|-----|
| `MagickCore >= 7.1.0 ... no` at `./configure` | `sudo apt install libmagickcore-7.q16-dev libmagickwand-7.q16-dev` |
| `aclocal-1.18: command not found` during `make` | `sudo apt install automake autoconf libtool` |
| `magick-cache: command not found` | `sudo make install`; ensure `/usr/local/bin` on PATH or symlink to `/usr/bin` |
| `libMagickCache.so.0: cannot open shared object file` | `sudo ldconfig` |
| `magick-cache: ready` never shows | Recheck passkey path; run `identify` manually (above) |
| No cache hit on repeat op | Apply cache settings; confirm enable + passkey + root; check Activity Log |
| `--passkey` fails (official CLI) | Use `-passkey` (single dash), not `--passkey` |

## Related docs

- Feature spec: [specs/013-image-tools-magick-cache/spec.md](../specs/013-image-tools-magick-cache/spec.md)
- Quickstart scenarios: [specs/013-image-tools-magick-cache/quickstart.md](../specs/013-image-tools-magick-cache/quickstart.md)
- External tool contract: [specs/013-image-tools-magick-cache/contracts/magick-cache-tool.md](../specs/013-image-tools-magick-cache/contracts/magick-cache-tool.md)