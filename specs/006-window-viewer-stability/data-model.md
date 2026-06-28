# Phase 1 Data Model: Window & Viewer Stability (006)

## Entity: WindowPreferences (NEW — FR-008 / SC-005)

Persisted user choice for window sizing. Lives in `src/types.rs` (domain types, no GUI deps).

| Field | Type | Notes |
|-------|------|-------|
| `version` | `u32` | Schema version; `1` initially. Enables future migration like `FehLaunchList.version`. |
| `preset` | `WindowSizePreset` | Selected size preset (see existing enum). Serialized as variant name. |
| `resizable` | `bool` | Whether the window can be drag-resized (false = locked min=max at current size). |

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct WindowPreferences {
    pub version: u32,
    pub preset: WindowSizePreset,
    pub resizable: bool,
}

impl Default for WindowPreferences {
    fn default() -> Self {
        Self { version: 1, preset: WindowSizePreset::Default, resizable: true }
    }
}
```

**Validation / invariants**:
- Unknown/missing file → `Default` (first-run = Default preset + resizable true; satisfies US4 scenario 2).
- Corrupt JSON → `Default` + `eprintln!` warning (never panics).
- `preset` is constrained by the enum; no out-of-range value possible once deserialized.
- Effective window size is always passed through `clamp_window_size` (floor 640×480) at apply time, so even a future hand-edited file cannot shrink the window below the floor.

## Existing entity: WindowSizePreset (REUSED — FR-004)

Defined in `src/types.rs`. Change: add `Serialize, Deserialize` derives.

| Variant | Dimensions (`window_preset_dimensions`) |
|---------|------------------------------------------|
| `Compact` | 720 × 540 |
| `Default` (enum default) | 960 × 720 |
| `Large` | 1280 × 960 |

## Conceptual entities (already realized in code, no new type)

- **WindowPolicy** (spec Key Entity): represented at runtime by `window_size` + `window_resizable` fields on the app plus `apply_window_resize_policy` / `clamp_window_size`. Not a persisted struct.
- **FehLaunchProfile** (spec Key Entity): represented by the constants `FEH_VIEWER_GEOMETRY` (`"1280x960"`) and `FEH_VIEWER_ZOOM` (`"max"`) plus `--scale-down` in `spawn_feh_viewer`. Not persisted (compile-time contract).

## State transitions (WindowPreferences lifecycle)

```text
app start ── load_window_prefs() ──> in-memory window_size + window_resizable
   │                                        │
   │                          user picks preset / toggles resizable (View menu)
   │                                        ▼
   │                          sync_frame_input_state detects change
   │                                        ▼
   │                          apply_window_preset / apply_window_resize_policy   (in-session effect)
   │                                        ▼
   └───────────────────────── save_window_prefs(WindowPreferences{..})  ──> ~/.config/rust-feh/window-prefs.json
```

Restart re-enters at `load_window_prefs()`, restoring the last saved preset + lock state (SC-005).
