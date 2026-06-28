# External Tool Contract: magick + magick-cache

**Purpose**: Defines the expected interface between rust-feh and the external ImageMagick tools for all processing and cache operations. This is the "integration contract" that must hold for the feature to deliver its performance and correctness guarantees.

## Detection (extended in tool_caps)
- `which magick` → basic processing capability (already present).
- `which magick-cache` → cache capability.
- Additional probe (on enable or recheck): attempt a lightweight `magick-cache identify` or a guarded put/get. Success = "ready"; specific errors ("not initialized", permission) produce user guidance.

## Core Operations (via Command)
All invocations use `std::process::Command::new("magick")` or `new("magick-cache")` with explicit args. Never shell.

### Cache Put (after successful op or for pre-cache)
```
magick-cache put <input-file> <iri> \
  --passkey <configured-passkey-path> \
  --ttl <from CacheConfig or "90 days">
```
- `<iri>` examples (per research + spec): `rust-feh/originals/<hash-of-abs-path>` or `rust-feh/processed/resize/<param-hash>`
- On success: the iri is recorded with the ProcessedResult.

### Materialize / Export for feh or list (Prepare Fast + any "use from cache")
```
magick convert "cache:<iri>" [ -auto-orient -strip -quality 92 ] <real-output-path.jpg>
```
- Produces a real filesystem file that feh can open directly and that can be added to rust-feh inventory.
- Used for every "Prepare Fast" output and optionally for cache-hit "get" when we need a real file (e.g. to offer to user or for further processing).

### Get (for cache-hit fast path without full re-compute)
```
magick-cache get <iri> <temp-or-dest> --passkey ...
```
or the convert "cache:" form above.

### Other
- `magick-cache create ...` and passkey setup: documented in README / quickstart; app only guides, never runs privileged create.
- Identify / probe commands for capability.

## Error Handling Expectations
- Non-zero exit + stderr containing recognizable phrases ("cache not found", "passkey", "permission", "format not supported") → mapped to friendly messages + Activity Log.
- Timeout: long operations get a reasonable wall-time limit (e.g. 5-10 min for huge images) with cancellation.
- Missing binary: tool_caps reports absent; UI disables cache features but keeps basic image tools via `image` crate fallback.

## IRI & Naming Invariants
- IRIs are stable for a given (source + operation + params).
- Hierarchical: `rust-feh/` prefix, then `originals/` vs `processed/<op>/`.
- TTL respected on put; app may also do client-side expiry checks in future.

## Feh Handoff Contract (for "Launch on optimized")
- After prepare-fast, a temp text file is written: one absolute path per line (to the materialized optimized .jpg/.png files).
- Launch: reuse/extend existing `launch_in_feh` with `--filelist /tmp/rust-feh-fast-xxx/list.txt` (or equivalent `-f`).
- The same materialized paths are also inserted into the rust-feh inventory as ImageAssets (status "optimized").

This contract is intentionally narrow and CLI-stable so that future ImageMagick updates or magick-cache improvements do not require rust-feh changes beyond Command arg tweaks.

See `research.md` for exact Command construction, timeout, and parsing details. Implementation lives in `image_proc` (`MagickCacheManager`, `ImageToolsService`; not in `main.rs`).