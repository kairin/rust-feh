# Data Model: Image Tools with Magick Cache

**Feature**: 013-image-tools-magick-cache
**Date**: 2026-06-24 (revised post-converge /speckit-plan)
**Source**: Derived from `spec.md` (User Stories, Functional Requirements, Key Entities, Clarifications) + `research.md` (including post-adversarial review decisions on module structure and fit modes).

**Note on architecture (from plan revision)**: Per constitution alignment (C1 fix), image processing + cache logic is implemented by extending `image_proc` (not new top-level core module). Data model entities remain the same.

## Core Entities

### ImageAsset (existing, extended)
- Represents a discoverable image file in the current folder tree.
- Key fields (existing + relevant):
  - `path: PathBuf` (absolute, canonical)
  - `filename: String`
  - `folder: String` (relative for display/tree)
  - `size_bytes: u64`
  - `dimensions: Option<(u32, u32)>` (lazy)
  - `format_status: FormatStatus` (Native | MagickAwaiting | Converted | etc.)
  - `status: AssetStatus` (new for this feature: Regular | Optimized("fast") | Processed)
- Relationships: Belongs to current scan inventory. Can be selected. Can be input to a ProcessingOperation.
- Lifecycle: Discovered by scanner (or added directly for materialized optimized files). Updated in place for renames. New instances created for processing outputs and optimized materializations.
- Validation: Path must exist and be readable at time of operation (checked before dispatch).

### ProcessingOperation
- A user-initiated transformation request.
- Variants / fields:
  - `Resize { width: Option<u32>, height: Option<u32>, percent: Option<f32>, fit: Option<FitMode>, filter: Option<Filter>, quality: Option<u8> }`
  - `Crop { geometry: String }` (e.g. "800x600+100+50"; validated as WxH+X+Y +repage semantics)
  - `Convert { target_format: String, quality: Option<u8> }`
  - `Rename { pattern: RenamePattern }` (batch only)
- Common: `targets: Vec<ImageAsset>`, `output_policy: OutputPolicy`, `use_cache: bool`
- Validation rules (from FRs + spec):
  - At least one dimension/percent for Resize.
  - Geometry must parse for Crop.
  - Target format supported (jpeg/png/webp + magick-detected others).
  - For batch: all targets must be valid for the op.
- State transitions: Created from UI → validated → dispatched to ImageToolsService → produces ProcessedResult(s).

### OutputPolicy
- Enum: `NewSubfolder { name: String }` | `SuffixedSibling { suffix: String }` | `InPlaceWithBackup { backup_suffix: String }`
- Rules:
  - Default for all ops: NewSubfolder or SuffixedSibling (safe, originals untouched).
  - InPlaceWithBackup: requires explicit user confirmation (twice per spec FR-018) + backup creation before any mutation.
- Used by the service to compute destination paths before any write.

### RenamePattern
- Structured representation of a user pattern (e.g. "trip-{date:YYYYMMDD}-{counter:03}").
- Tokens supported (minimum per spec + clarify): prefix/suffix literals, `{counter:N}`, `{date:fmt}`, `{original}`, `{ext}`.
- Fields: `raw: String`, `parsed_tokens: Vec<Token>`
- Validation: Must produce unique names for the selection (checked in preview + apply).
- Expansion: Given selection + starting counter → `Vec<(original_path, proposed_name)>` for live preview.

### CacheConfig (in-memory for v1)
- `enabled: bool`
- `root: Option<PathBuf>`
- `passkey_path: Option<PathBuf>`
- `default_ttl: String` ("90 days" | "never" | custom)
- Used by MagickCacheManager. Serializable for future persistence (but not in v1).

### CachedAsset / MaterializedAsset
- `iri: String` (the rust-feh/... identifier used with magick-cache)
- `source_asset: ImageAsset` (or hash)
- `operation: Option<ProcessingOperation>` (for processed results)
- `materialized_path: Option<PathBuf>` (for "Prepare Fast" outputs that must be real files for feh + list)
- `created_at: Instant` (for TTL/expiry decisions in future)
- Relationship: 1:1 with a put/get in the external cache. When materialized, also becomes an ImageAsset in the inventory.

### PreparationJob
- Represents a background "Pre-cache entire folder" or "Prepare Fast feh Viewing Cache" run.
- Fields: `id: JobId`, `folder: PathBuf`, `kind: PreCache | PrepareFast`, `progress: Progress (current/total)`, `cancel_flag: Arc<AtomicBool>`, `results: Vec<ProcessedResult or Materialized>`
- State transitions: Queued → Running (background thread) → Completed (with materialized paths) | Cancelled | Error.
- Communicates results back via mpsc channel to the App for inventory merge + log.

### ProcessedResult
- Output of a single operation on one asset.
- `source: ImageAsset`
- `dest_path: PathBuf` (the new/suffixed/in-place file)
- `cache_iri: Option<String>`
- `materialized_for_fast: Option<PathBuf>`
- `success: bool`, `error: Option<String>`
- Used for batch summary, inventory update, Activity Log entry.

## State Transitions (High Level)
1. User selects assets → creates ProcessingOperation + OutputPolicy (UI layer).
2. Confirmation (for batch or in-place) → dispatch to service.
3. Service:
   - For each target: compute dest, check cache (get if hit), run op (Command or image crate), put result to cache, materialize if needed for fast/preview, write output file.
   - Send ProcessedResult(s) back.
4. App receives results → append to Activity Log, insert new ImageAssets into inventory (for materialized/processed), request repaint, update selection if appropriate.
5. For prepare-fast: similar but bulk, with progress updates.

## Validation Rules (Cross-Entity)
- All file writes default to non-destructive (FR-002).
- Cache put/get only attempted when `CacheConfig.enabled && tool present`.
- Rename collision check happens before any filesystem mutation.
- Numeric crop geometry must be parsable and produce a valid (non-empty) region.
- Optimized materialized files must be real, readable files on disk before being offered to feh or added to inventory.

This model keeps core logic (service, cache manager, operation impl) pure and testable. GUI only observes and drives via the existing patterns. New types are additive and do not change existing ImageEntry/ScanInventory contracts beyond optional status extension.