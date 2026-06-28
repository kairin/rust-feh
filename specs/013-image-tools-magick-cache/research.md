# Research: Image Tools with Magick Cache

**Feature**: 013-image-tools-magick-cache
**Date**: 2026-06-24
**Purpose**: Resolve technical unknowns, document decisions, and provide rationale for implementation approach. All findings feed directly into data-model, contracts, quickstart, and tasks.

## Decision: Magick Cache Invocation Strategy
**Decision**: Use `std::process::Command` exclusively to invoke the external `magick-cache` binary (and `magick` for materialization/export). Never link or use Rust bindings.

**Rationale**:
- Directly satisfies constitution II (Pure Rust, Minimal Dependencies) and the original feature request ("Prefer calling magick and magick-cache via std::process::Command").
- Matches the existing, proven pattern in `tool_caps.rs` and `main.rs` for feh + magick (detection + runtime + graceful degradation).
- `magick-cache` is a separate CLI tool in modern ImageMagick 7+ distributions; binding would add heavy, less-auditable dependency surface.
- Easy to parse stdout/stderr for success/failure/identifiers; timeouts and sandboxing via Command are straightforward.
- Temp file materialization for "Prepare Fast" (to give feh real paths) is natural with Command + `magick convert 'cache:...' out.jpg` or equivalent get/export flow.

**Alternatives considered**:
- `magick-rust` or similar bindings: Rejected — violates minimal deps, adds maintenance burden, harder to audit, and unnecessary since CLI is stable and documented.
- Pure in-process via `image` crate only: Insufficient for "Prepare Fast feh" quality (auto-orient, strip, exotic format handling, cache persistence) and the explicit requirement for deep magick-cache integration.
- Shelling out to a wrapper script: Rejected — adds another moving part; direct Command is simpler and more auditable.

## Post-Adversarial Review Decisions (2026-06-24 plan revision)
**Context**: This plan was revised in response to the adversarial red-team review of the prior /speckit-analyze report, which correctly identified that critical issues (especially C1 constitution module violation and I1 fit/performance gaps) had reached late stages, indicating process weakness in early enforcement.

**Decision on C1 (module structure)**: To enforce constitution integrity (explicit core modules list and test mandate), we commit to *extending `image_proc`* for all new image ops + cache logic (pure, Command-based). No new top-level core `image_tools` module. Refactor any prior scaffolding. Documented in plan.md Constitution Check (revised to "PASS with explicit alignment"), Complexity Tracking (added entry for the addressed potential violation), and Structure Decision. This directly fixes the "intellectual dishonesty" and "process failure" critique.

**Decision on I1 (fit modes and filter)**: Explicitly support fit modes (contain/cover/stretch) and filter choice (Lanczos + high-quality) in v1 for resize, per contracts/data-model/research and to deliver on the original feature request's "professional set of image tools". Updated plan Technical Context with note recommending spec FR-015 clarification. Tasks will implement accordingly (no longer "basic only").

**Decision on performance SCs**: Added explicit requirement in plan Performance Goals for dedicated benchmark test tasks (using existing project scripts + fixtures) for SC-001/002/004/007. Addresses "vaporware" and "only quickstart/manual" risk. No "subjective" fallback without numbers.

**Gate re-evaluation**: With these choices, Constitution Check re-passed with no violations. Research now includes this section for traceability. This plan iteration demonstrates the Speckit process responding to adversarial input by making crisp, documented decisions rather than hedging.

All other research decisions (Command-only, mpsc reuse, IRI strategy, materialization for feh per clarify Q1 Option A, pure parser for rename, etc.) remain unchanged and aligned with constitution.

**Research notes**:
- `magick-cache` subcommands (from IM7 docs + user prompt): `create`, `put <input> <iri>`, `get <iri> <output>`, `identify`, `delete`, `expire`.
- Flags: `-passkey <file>`, `-ttl <duration|never>`, `-passphrase` (for some flows).
- IRI strategy (adopted from feature request, to be implemented in core): `rust-feh/originals/{sanitized-abs-path-hash}` and `rust-feh/processed/{operation}/{params-hash}`. Stable, hierarchical, easy to reason about collisions/expiry.
- To materialize for feh (per clarify Q1 Option A): After put, use `magick convert "cache:rust-feh/processed/..." /tmp/fast-xxx.jpg` (or equivalent) to produce a real file that feh can mmap/load quickly. The temp file (or a managed cache-export dir) is what goes into the feh filelist and the rust-feh inventory.
- Detection: Extend `which::which("magick-cache")` + a lightweight "is initialized?" probe (e.g. a test put/get or `magick-cache identify` on a known IRI). Failures are treated as "not ready" with guidance.

## Decision: Background Job & Responsiveness Pattern
**Decision**: Reuse/extend the existing `std::sync::mpsc` + thread + "status pulse" pattern already used for background scanning (features 001/011/012). Long-running ops (batch processing, pre-cache, prepare-fast) run on dedicated threads; results are sent back to the egui app for inventory merge + Activity Log append. Progress is reported via the existing "Session status" + bottom bar animation mechanisms.

**Rationale**:
- Constitution V + existing code already solve "UI must remain responsive during scans" and "use channels and never block the egui render loop".
- Avoids introducing tokio/flume in v1 (explicitly allowed to stay in future per constitution III; current mpsc is sufficient and zero-dep).
- Keeps changes localized; main.rs already knows how to handle async results without freezing.
- Interruptibility: the job thread can check a cancellation flag (Arc<AtomicBool>) on reasonable boundaries (per-image).

**Alternatives considered**:
- Full async runtime (tokio): Deferred (per constitution). Adds dependency + complexity for something the current architecture already handles well.
- Blocking in egui frame: Forbidden by constitution and would regress existing scan behavior.

## Decision: Crop Preview Implementation
**Decision**: For the numeric crop (WxH+X+Y) preview in the Single tool panel: use the `image` crate (already a dependency) to open the source (or a low-res proxy), perform an in-memory crop + resize for preview size, then upload as an egui texture (via `egui::ColorImage` + `ctx.load_texture`). Update live as the user edits the geometry fields. No full-resolution decode needed for preview.

**Rationale**:
- Keeps core image processing in the `image` crate (constitution III + existing `image_proc`).
- Zero new deps.
- Matches the "live preview image rendered via egui (decode with `image` crate → egui texture)" requirement from the original feature request.
- For very large images, we can add a "use lower res proxy for preview" heuristic if needed (future-proof).

**Alternatives**: Using magick for preview: Possible but unnecessary (adds a subprocess roundtrip for something the pure-Rust `image` crate already does well and quickly for preview purposes). Rejected for simplicity.

## Decision: Rename Pattern Syntax & Parsing
**Decision**: Support (at minimum) the tokens explicitly called out in the spec + original request:
- `{prefix}`, `{suffix}`
- `{counter:03}` (width specifier)
- Date components e.g. `{date:YYYYMMDD}`, `{date:YYYY}`, etc.
- `{original}` (basename without ext), `{ext}`
- Literal text.

Implementation: Small parser in `ui_logic` (pure functions, no filesystem side effects). Use `regex` crate *only if* pure string/iterator parsing becomes fragile for the width specifiers (document justification in Cargo.toml). Expansion produces a `Vec<(old_path, new_name)>` preview table. Collisions detected by building a set of proposed names.

**Rationale**:
- Directly satisfies FR-007 and User Story 3 (live preview table, explicit confirmation).
- "at minimum" language in spec allows us to start with the requested set and document the grammar in quickstart.md / contracts.
- Keeps rename logic pure and testable (no filesystem side effects until Apply + confirmation).

## Decision: Temp Materialization Location & Lifecycle for Optimized Versions
**Decision** (per clarify Q1 Option A):
- On "Prepare Fast feh Viewing Cache": for each selected asset, if not already in cache or needs refresh, compute optimized (magick auto-orient + strip + convert to high-quality JPEG or PNG as appropriate), `magick-cache put`, then `magick convert "cache:..." /tmp/rust-feh-fast/<sanitized-name>-optimized.jpg` (or a `std::env::temp_dir()/rust-feh-fast-<pid>/` subdir for the session).
- The real materialized paths go into:
  1. A temp `feh-filelist.txt` (one absolute path per line) passed to the existing feh launch.
  2. The in-memory inventory as new `ImageEntry` items (with status "optimized" or similar) so they appear in the list immediately.
- Lifecycle: Clean up the temp dir on app exit or explicit "Clear Fast Cache" action. Do not pollute user's image folders.

**Rationale**:
- Satisfies clarified spec exactly: real files, usable in browser list, feh via existing launch + filelist.
- Temp dir is safe, automatically cleaned by OS in many cases, and scoped.
- Allows the user to further process the optimized versions with the new tools (resize them again, etc.) — consistent behavior.
- Performance win for feh comes from the stripped + standard format + orientation being baked in (feh doesn't have to do the work on open).

**Risks / Mitigations**:
- Disk usage for materialized files on 10k-image prepare: User-triggered action only; we can show estimated size in the prepare dialog. Future work could add size limits or LRU eviction via cache TTL.
- Temp dir not surviving app restart: Acceptable for v1 (re-run prepare is cheap with cache hits).

## Decision: Inventory Update for New/Optimized Assets
**Decision**: After any successful create (processing result or materialized optimized file), the core "inventory" (the `Vec<ImageEntry>` or equivalent held in the App state) is updated by inserting the new entry (or replacing if rename) and triggering a light re-sort/filter + UI refresh. Reuse whatever mechanism the existing "Quick resize" demo already uses (likely a channel message or direct mutation + request_repaint).

Do **not** trigger a full `scan_images` background walk for these synthetic assets — they are known by path and can be added directly (with metadata extracted on the fly or lazily).

**Rationale**:
- Matches clarified spec ("update the in-memory inventory so new files appear immediately").
- Preserves low-memory contract (no full re-scan, no extra metadata load for the whole tree).
- Consistent with how processed files from the current demo appear.

## Other Research Notes / Best Practices
- **Command safety**: Always use `Command::new("magick-cache").args([...])` (never shell). Escape paths properly (Rust Command handles this). Capture stdout/stderr. Use `.output()` or `.spawn()` + wait with timeout for long ops.
- **Error handling**: Parse common magick/magick-cache error patterns for user-friendly messages ("cache not initialized — run `magick-cache create ...` first"). Surface via Activity Log + status.
- **Passkey**: Treat as a file path the user configures (in-memory `CacheConfig`). Pass via `-passkey /path/to/key` on every relevant command. Never read the key content ourselves.
- **feh filelist format**: Plain text, one absolute path per line. `feh --filelist /tmp/list.txt` (or `feh -f /tmp/list.txt`). Existing launch code can be extended with an optional "filelist" parameter.
- **Large image handling**: For prepare-fast on huge files, stream where possible; use magick's `-limit` if needed for resource control. Cache TTL helps avoid repeated work.
- **Testing the external tools**: In integration tests, use `which` + `Command` to skip or mock when not present (consistent with existing tool tests).

All research above was derived from:
- Existing codebase patterns (tool_caps, scanner background, ui_logic, image_proc demo, main.rs feh launch).
- Constitution v1.0.1 (esp. I–V).
- Clarified spec (especially Q1 on materialization + list visibility).
- Original feature request details (IRI examples, "Prepare Fast" description, Command preference).
- ImageMagick 7 behavior (magick CLI, cache concepts, -repage, auto-orient, etc.).

No blocking unknowns remain for Phase 1 design.

## Post-Convergence Assessment (2026-06-24)

**Context**: `/speckit-converge` compared the codebase to spec/plan/tasks and found T001–T092 marked complete while US1–US5 remain largely unimplemented. Phase 9 tasks (T093–T106) were appended.

**Decision: Test harness wiring**
- Register `tests/unit/image_proc.rs` and `tests/unit/ui_logic.rs` as `[[test]]` targets in `Cargo.toml`.
- Delete stale `tests/unit/image_tools/operations_test.rs` (references removed module).
- Create `tests/integration/image_tools.rs` and `tests/perf/` per plan Performance Goals.

**Decision: Crop preview module boundary**
- `ui_logic` returns raw RGBA pixel buffer + dimensions for crop preview.
- `main.rs` converts to `egui::ColorImage` and loads texture. Zero egui types in `ui_logic` (constitution III).

**Decision: Edge cases (spec.md Edge Cases section)**
- **Invalid crop geometry / out-of-bounds**: Parse and validate before apply; clamp or reject with friendly error; preview shows validation state.
- **Rename collisions**: Block apply in preview; highlight rows; offer skip/auto-number at apply time.
- **Permission / disk-full errors**: Per-item skip in batch; log + summary; no partial writes on single item.
- **Very large images (100s of MB)**: Optional size guard with user message; magick path preferred; preview uses downscaled proxy.
- **App exit / folder switch during job**: Cancel flag on background thread; best-effort temp dir cleanup; job state cleared on folder change.
- **Prepare-fast idempotent**: Skip re-materialize when cache IRI + dest already exist and are fresh.
- **magick-cache not initialized**: `magick_cache_ready` false → guidance message; core tools unaffected.
- **Network/slow storage**: Reuse existing `is_network_mount_path` policy where applicable; warn on cache put for GVFS/NFS paths; no special blocking in v1.
- **Re-scan/filter during job**: Job continues on snapshot selection; results merge into inventory; filter does not cancel job.

**Decision: SC-005 verification**
- Dedicated integration test: hash original file before/after non-in-place ops; assert unchanged.