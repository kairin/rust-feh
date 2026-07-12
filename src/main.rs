// SPDX-License-Identifier: MIT
// rust-feh — Fast lightweight GUI for feh + simple image tools
// All original nfeh / old maintainer code and traces have been archived (see archive/original-nfeh/).

use eframe::{egui, App, Frame};
use rust_feh::image_proc::{decode_stage_rgba, process_image, ImageToolsService, ProcessOptions};
use rust_feh::scanner::{scan_images_streaming, ScanResult};
use rust_feh::tool_caps::{feh_spawn_unavailable, DepKind, FormatRoute, ToolCapabilities};
use rust_feh::types::{
    ActionKind, ActionOutcome, ActionPrefs, ActionResult, AssetStatus, CacheConfig, ContextAction,
    FehLaunchEntry, FehLaunchList, Filter, FitMode, ImageEntry, ImageOperation, ListViewMode,
    OutputPolicy, PreparedFastSet, ProcessedResult, ScanInventory, SortMode, StageState,
    WindowPreferences, WindowSizePreset,
};
use rust_feh::ui_logic::{
    add_or_update_asset_in_inventory, apply_converted_detection_cancellable, apply_rename_pairs,
    build_entry_filelist, clamp_window_size, cleanup_stale_handoffs, collision_suffixed_path,
    compute_output_path, copy_image_to_clipboard, crop_preview_pixels, default_tree_expanded,
    entry_is_launchable, execute_move_plan, expand_rename_pattern, feh_entry_filelist_path,
    feh_filelist_temp_path, feh_missing_status, feh_not_installed_launch_status, file_name_display,
    file_status_decodable, file_status_label, finalize_scan_entries_fast, folder_line_suffix,
    folder_tree_display_name, format_action_outcome, format_image_tools_log, format_inventory_bar,
    handoff_path, initial_open_sections, inventory_magick_hint, is_network_mount_path,
    join_activity_log, list_indices, list_subfolders, list_view_mode_label, load_action_prefs,
    load_launch_list, load_window_prefs, merge_converted_statuses, plan_loss_proof_move,
    post_scan_status, prepare_fast_work_dir, relative_folder, save_action_prefs, save_copy_to,
    save_launch_list, save_window_prefs, scan_magick_enabled, sections_drawer_body_height,
    sections_drawer_reserved_height, showing_count_label, sort_mode_label, spawn_job,
    tree_file_glyph, tree_visible_rows, validate_handoff, viewer_profile_dir, viewer_spawn_command,
    window_preset_dimensions, window_preset_label, write_feh_filelist, write_feh_filelist_to,
    AutoExpandState, DetachedWindow, EntryLaunchState, InspectorSection, JobMsg, PanelContext,
    PanelPin, TreeRow, TreeRowKind, FEH_VIEWER_GEOMETRY, FEH_VIEWER_ZOOM, WINDOW_MAX_RESIZABLE,
    WINDOW_MIN_RESIZABLE,
};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver};
use std::sync::{Arc, Once};
use std::thread;

static STARTUP_NOTICE: Once = Once::new();

fn create_rust_feh_app(
    status: String,
    feh_available: bool,
    tool_caps: ToolCapabilities,
    deps_section_open: bool,
    tools_panel_ok: bool,
) -> Box<dyn App> {
    let window_prefs = load_window_prefs();
    // Reap any handoff files orphaned by a prior session's round-trip
    // viewers that outlived rust-feh (contract: "stale handoff files under
    // runtime cache are cleaned at next startup").
    let _ = cleanup_stale_handoffs();
    let initial_open = initial_open_sections(deps_section_open, tools_panel_ok);
    // Seed the auto-expand machine's ownership from the startup fold set so
    // startup auto-opens retract through the edge machine (018 FIX-1).
    let auto_expand = AutoExpandState::seeded(&initial_open);
    Box::new(RustFehApp {
        current_dir: None,
        images: vec![],
        selected: None,
        status,
        debug_logs: vec![],
        search: String::new(),
        prior_search: String::new(),
        recursive: false,
        deep_scan_magick: false,
        feh_available,
        tool_caps,
        scanning: false,
        scroll_generation: 0,
        sort_mode: SortMode::default(),
        prior_sort_mode: SortMode::default(),
        window_size: window_prefs.preset,
        prior_window_size: window_prefs.preset,
        window_resizable: window_prefs.resizable,
        prior_window_resizable: window_prefs.resizable,
        window_prefs_applied: false,
        scan_inventory: None,
        list_view_mode: ListViewMode::default(),
        tree_expanded_paths: default_tree_expanded(),
        scan_generation: 0,
        scan_rx: None,
        scan_cancel: Arc::new(AtomicBool::new(false)),
        subfolders: Vec::new(),
        subfolders_dir: None,
        subfolder_generation: 0,
        subfolder_rx: None,
        subfolders_pending: false,
        pending_select_path: None,
        detached: HashMap::new(),
        inspector_open: initial_open,
        inspector_drawer_collapsed: true,
        auto_expand,
        prior_folder_present: true,
        format_route_open: HashSet::new(),
        start_folder_loaded: false,
        image_tools: ImageToolsService::new(None),
        cache_config: CacheConfig {
            default_ttl: "90 days".into(),
            ..CacheConfig::default()
        },
        tools_panel: ImageToolsPanelState::default(),
        tools_job: None,
        prepared_fast: None,
        prepare_fast_temp: None,
        launch_entries: load_launch_list(),
        selected_tree_folder: None,
        stage_generation: 0,
        stage_requested_path: None,
        stage_state: StageState::Loading,
        stage_rx: None,
        stage_texture: None,
        stage_pane_collapsed: false,
        action_prefs: load_action_prefs(),
        round_trips: Vec::new(),
        next_viewer_id: 0,
        pending_scroll_path: None,
        images_revision: 0,
        list_index_cache: std::cell::RefCell::new(None),
        tree_rows_cache: Vec::new(),
        tree_rows_cache_key: None,
        inspector_width_cache: None,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum ToolsSingleOp {
    #[default]
    Resize,
    Crop,
    Convert,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum ToolsPolicyUi {
    #[default]
    Subfolder,
    Suffixed,
    InPlace,
}

#[derive(Clone)]
struct ImageToolsPanelState {
    section: ToolsSection,
    single_op: ToolsSingleOp,
    resize_width: String,
    resize_height: String,
    resize_percent: String,
    use_percent: bool,
    fit: FitMode,
    filter: Filter,
    quality: u8,
    crop_geometry: String,
    convert_format: String,
    policy_ui: ToolsPolicyUi,
    subfolder_name: String,
    suffix: String,
    inplace_confirm: u8,
    batch_inplace_confirm: u8,
    batch_summary: Option<String>,
    rename_pattern: String,
    rename_preview: Vec<(PathBuf, String)>,
    rename_error: Option<String>,
    batch_confirm_open: bool,
    rename_confirm_open: bool,
    crop_texture: Option<egui::TextureHandle>,
    last_crop_key: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum ToolsSection {
    #[default]
    Single,
    Batch,
    Rename,
    Cache,
}

enum ToolsJobKind {
    Batch,
    PreCache,
    PrepareFast,
}

struct ActiveToolsJob {
    kind: ToolsJobKind,
    rx_batch: Option<mpsc::Receiver<JobMsg<ProcessedResult>>>,
    rx_bool: Option<mpsc::Receiver<JobMsg<bool>>>,
    rx_paths: Option<mpsc::Receiver<JobMsg<PathBuf>>>,
    cancel: Arc<AtomicBool>,
    current: usize,
    total: usize,
    message: String,
    batch_ok: usize,
    batch_fail: usize,
    precache_ok: usize,
    prepare_paths: Vec<PathBuf>,
    prepare_temp: PathBuf,
}

fn filter_label(f: Filter) -> &'static str {
    match f {
        Filter::Lanczos3 => "Lanczos",
        Filter::Nearest => "Nearest",
        Filter::Triangle => "Triangle",
        Filter::CatmullRom => "CatmullRom",
        Filter::Gaussian => "Gaussian",
    }
}

impl Default for ImageToolsPanelState {
    fn default() -> Self {
        Self {
            section: ToolsSection::default(),
            single_op: ToolsSingleOp::default(),
            resize_width: String::new(),
            resize_height: String::new(),
            resize_percent: "50".into(),
            use_percent: false,
            fit: FitMode::default(),
            filter: Filter::default(),
            quality: 85,
            crop_geometry: "800x600+0+0".into(),
            convert_format: "jpg".into(),
            policy_ui: ToolsPolicyUi::default(),
            subfolder_name: "processed".into(),
            suffix: "_edited".into(),
            inplace_confirm: 0,
            batch_inplace_confirm: 0,
            batch_summary: None,
            rename_pattern: "img-{counter:03}".into(),
            rename_preview: Vec::new(),
            rename_error: None,
            batch_confirm_open: false,
            rename_confirm_open: false,
            crop_texture: None,
            last_crop_key: String::new(),
        }
    }
}

fn main() {
    if let Err(err) = try_run_gui() {
        eprintln!("[rust-feh] {}", err);
        std::process::exit(1);
    }
}

fn try_run_gui() -> Result<(), String> {
    let (w, h) = window_preset_dimensions(WindowSizePreset::default());
    let (min_w, min_h) = WINDOW_MIN_RESIZABLE;

    let (status, feh_available, tool_caps, deps_section_open, tools_panel_ok) = detect_app_state();

    let options = build_native_options(w, h, min_w, min_h);

    let status_first = status.clone();
    let tool_caps_first = tool_caps.clone();

    let result = eframe::run_native(
        "rust-feh",
        options.clone(),
        Box::new(move |_cc| {
            Ok(create_rust_feh_app(
                status_first,
                feh_available,
                tool_caps_first,
                deps_section_open,
                tools_panel_ok,
            ) as Box<dyn App>)
        }),
    );

    if let Err(err) = result {
        return handle_gui_failure(
            err,
            status,
            feh_available,
            tool_caps,
            deps_section_open,
            tools_panel_ok,
        );
    }
    Ok(())
}

fn detect_app_state() -> (String, bool, ToolCapabilities, bool, bool) {
    let tool_caps = ToolCapabilities::detect();
    let feh_available = tool_caps.feh_available;
    let deps_section_open = tool_caps.has_missing_required();
    let tools_panel_ok = !tool_caps.has_missing_required();
    let status = if feh_available {
        String::new()
    } else {
        "feh not found — install with `sudo apt install feh`".to_string()
    };
    (
        status,
        feh_available,
        tool_caps,
        deps_section_open,
        tools_panel_ok,
    )
}

fn build_native_options(w: f32, h: f32, min_w: f32, min_h: f32) -> eframe::NativeOptions {
    eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([w, h])
            .with_min_inner_size([min_w, min_h])
            .with_resizable(true)
            .with_title("rust-feh"),
        ..Default::default()
    }
}

fn handle_gui_failure(
    err: eframe::Error,
    status: String,
    feh_available: bool,
    tool_caps: ToolCapabilities,
    deps_section_open: bool,
    tools_panel_ok: bool,
) -> Result<(), String> {
    eprintln!("[rust-feh] Failed to initialize GUI window: {}", err);

    let on_wayland = std::env::var_os("WAYLAND_DISPLAY").is_some();
    if on_wayland {
        eprintln!("[rust-feh] Wayland environment detected (WAYLAND_DISPLAY set).");
        eprintln!("[rust-feh] Native Wayland backend failed to connect (this happens when no compositor is available,");
        eprintln!(
            "[rust-feh] e.g. some SSH sessions, broken sockets, or misconfigured Wayland setups)."
        );
        eprintln!("[rust-feh] Most real Wayland desktops (GNOME, KDE Plasma, Sway, Hyprland, etc.) work great with");
        eprintln!("[rust-feh] native Wayland when a compositor is running.");
        eprintln!("[rust-feh] Retrying with X11 backend (XWayland) as fallback...");

        std::env::set_var("WINIT_UNIX_BACKEND", "x11");

        let (w, h) = window_preset_dimensions(WindowSizePreset::default());
        let (min_w, min_h) = WINDOW_MIN_RESIZABLE;
        let x11_options = build_native_options(w, h, min_w, min_h);

        if let Err(err2) = eframe::run_native(
            "rust-feh",
            x11_options,
            Box::new(move |_cc| {
                Ok(create_rust_feh_app(
                    status,
                    feh_available,
                    tool_caps,
                    deps_section_open,
                    tools_panel_ok,
                ) as Box<dyn App>)
            }),
        ) {
            return Err(format!("X11 fallback also failed: {}. No usable display server found. For normal Wayland use: make sure a compositor is active (e.g. start from a login manager). You can also force X11 manually: WINIT_UNIX_BACKEND=x11 ./rust-feh", err2));
        }
        eprintln!("[rust-feh] Running via X11 fallback this time. Native Wayland works on standard desktop compositors.");
    } else {
        return Err("No Wayland detected. Ensure a display (X11 or Wayland compositor) is available. Check DISPLAY and/or WAYLAND_DISPLAY environment variables.".to_string());
    }
    Ok(())
}

#[derive(PartialEq)]
struct ListIndexKey {
    revision: u64,
    current_dir: Option<PathBuf>,
    search: String,
    sort_mode: SortMode,
}

/// `(total_count, filtered_indices)` — the result of `compute_list_indices`,
/// cached alongside the `ListIndexKey` it was computed for (018 B1.5: named to
/// resolve `clippy::type_complexity` on the `list_index_cache` field).
type ListIndexCacheValue = (usize, Vec<usize>);
/// Cross-frame cache cell for `compute_list_indices`: `None` until first
/// computed, then `Some((key, value))`.
type ListIndexCache = std::cell::RefCell<Option<(ListIndexKey, ListIndexCacheValue)>>;

#[derive(PartialEq)]
struct TreeRowsKey {
    revision: u64,
    current_dir: Option<PathBuf>,
    search: String,
    sort_mode: SortMode,
    root_skipped: usize,
    expanded: std::collections::HashSet<String>,
}

struct RustFehApp {
    current_dir: Option<PathBuf>,
    images: Vec<ImageEntry>,
    selected: Option<PathBuf>,
    status: String,
    debug_logs: Vec<String>,
    search: String,
    prior_search: String,
    recursive: bool,
    /// Slow per-file ImageMagick identify for exotic formats (off by default for feh speed).
    deep_scan_magick: bool,
    feh_available: bool,
    tool_caps: ToolCapabilities,
    scanning: bool,
    /// Incremented when filter/sort changes to reset ScrollArea position (FR-005).
    scroll_generation: u64,
    sort_mode: SortMode,
    prior_sort_mode: SortMode,
    window_size: WindowSizePreset,
    prior_window_size: WindowSizePreset,
    window_resizable: bool,
    prior_window_resizable: bool,
    /// False until the loaded window preferences are applied to the viewport on the first frame.
    window_prefs_applied: bool,
    scan_inventory: Option<ScanInventory>,
    list_view_mode: ListViewMode,
    tree_expanded_paths: HashSet<String>,
    scan_generation: u64,
    scan_rx: Option<Receiver<ScanMsg>>,
    scan_cancel: Arc<AtomicBool>,
    /// Immediate subdirectories of `current_dir` (feature 017 drill-down nav).
    subfolders: Vec<PathBuf>,
    /// Directory `subfolders` was computed for (None until first request).
    subfolders_dir: Option<PathBuf>,
    /// Bumped on every request_subfolders call; stale off-thread results are discarded.
    subfolder_generation: u64,
    subfolder_rx: Option<Receiver<SubfolderMsg>>,
    /// True while an off-thread list_subfolders request is in flight.
    subfolders_pending: bool,
    /// Set by a cross-folder round-trip landing (Phase 4); consumed by
    /// apply_scan_result so the auto-select-first-image doesn't clobber it.
    pending_select_path: Option<PathBuf>,
    /// Detached (floating-window) inspector sections (018 Batch 4); replaces
    /// 7 discrete `*_detached` bools. Absence = docked; presence = detached.
    /// Carries per-window pin state (018 Batch 5 target pinning).
    detached: HashMap<InspectorSection, DetachedWindow>,
    /// Per-section fold state (018 Batch 1); replaces 7 discrete open-bools.
    inspector_open: HashSet<InspectorSection>,
    /// Zone D meta-drawer fold state (018 Batch 2): true = the 7 detail
    /// sections are hidden behind the "Details" toggle. Collapsed by default so
    /// the file list (Zone C) is the primary content on launch.
    inspector_drawer_collapsed: bool,
    /// Edge-triggered auto-expand policy state (018 FIX-1, FR-003): tracks which
    /// sections/drawer the machine opened so it can retract them and honor a
    /// user-close latch. Drives `inspector_open`/`inspector_drawer_collapsed` at
    /// scan/no-folder/tool-missing edges instead of the old per-frame inserts.
    auto_expand: AutoExpandState,
    /// Previous frame's `current_dir.is_some()`, for edge-detecting the
    /// no-folder ↔ folder-loaded transitions that drive Browse auto-expand
    /// (018 FIX-1). Seeded `true` so a first frame with no folder fires the
    /// no-folder rising edge (Browse opens — SC-001).
    prior_folder_present: bool,
    image_tools: ImageToolsService,
    cache_config: CacheConfig,
    tools_panel: ImageToolsPanelState,
    tools_job: Option<ActiveToolsJob>,
    prepared_fast: Option<PreparedFastSet>,
    prepare_fast_temp: Option<PathBuf>,
    launch_entries: FehLaunchList,
    selected_tree_folder: Option<PathBuf>,
    format_route_open: HashSet<String>,
    /// Dev/test: auto-load `RUST_FEH_START_FOLDER` once on first frame.
    start_folder_loaded: bool,
    /// Stage pane (feature 016): current decode job's generation; only the
    /// latest generation's result is ever applied (stale decodes discarded).
    stage_generation: u64,
    /// Path the current/most-recent decode job was kicked off for; compared
    /// against `selected` each frame to detect a new selection needing decode.
    stage_requested_path: Option<PathBuf>,
    stage_state: StageState,
    stage_rx: Option<Receiver<StageDecodeMsg>>,
    stage_texture: Option<egui::TextureHandle>,
    stage_pane_collapsed: bool,
    action_prefs: ActionPrefs,
    /// Live round-trip viewers, polled every frame (feature 016, US2).
    round_trips: Vec<ViewerRoundTrip>,
    next_viewer_id: u64,
    /// Set when a round-trip handoff lands on an image; consumed by the flat
    /// list's render pass to force-scroll to it once (US2 AS1: "list scrolls
    /// to it").
    pending_scroll_path: Option<PathBuf>,
    /// Bumped on EVERY mutation of `self.images` (content, order, length, or
    /// per-entry FileStatus/path). Cache-invalidation key for `compute_list_indices`.
    images_revision: u64,
    /// Frame/cross-frame cache for `compute_list_indices`: (key, (total, indices)).
    list_index_cache: ListIndexCache,
    /// Cross-frame cache for the folder-tree rows (reused when `TreeRowsKey` unchanged).
    tree_rows_cache: Vec<TreeRow>,
    tree_rows_cache_key: Option<TreeRowsKey>,
    /// Cached measured static-label text width for the Inspector (feature 017
    /// Phase 5; re-clamp semantics fixed in 018 F3):
    /// (pixels_per_point the text was measured at, measured max text width in px
    /// — NOT the final clamped panel width, which is recomputed every call from
    /// the live half-viewport so it never goes stale on resize).
    /// Recomputed only when `pixels_per_point` changes — the sole input to
    /// static-label text measurement, since this app never mutates fonts, text
    /// styles, theme, or zoom. Prevents per-frame re-measurement while keeping
    /// the final width live.
    inspector_width_cache: Option<(f32, f32)>,
}

enum FehEntryAction {
    Remove(String),
    Launch(String),
    SetFolder(String, Option<PathBuf>),
    SetLabel(String, Option<String>),
}

#[derive(Clone, Copy)]
struct ImageListMetrics {
    list_height: f32,
    folder_col_w: f32,
    /// Fixed width of the filename column (018 FIX-10): lets a long filename
    /// truncate instead of clipping/jittering the Status column at 440px.
    name_col_w: f32,
    status_col_w: f32,
    row_h: f32,
}

enum ScanMsg {
    Partial {
        generation: u64,
        entries: Vec<ImageEntry>,
        skipped: usize,
    },
    Complete {
        generation: u64,
        dir_label: String,
        result: ScanResult,
    },
    Converted {
        generation: u64,
        entries: Vec<ImageEntry>,
        non_image_skipped: usize,
        magick_truncated: bool,
    },
}

/// Off-thread `list_subfolders` result; stale generations are discarded on receipt.
struct SubfolderMsg {
    generation: u64,
    dir: PathBuf,
    folders: Vec<PathBuf>,
}

/// Longest edge (px) for stage-pane decodes (feature 016, R5).
const STAGE_MAX_EDGE: u32 = 2048;

/// Off-thread stage decode result; stale generations are discarded on receipt.
enum StageDecodeMsg {
    Ready {
        generation: u64,
        width: u32,
        height: u32,
        rgba: Vec<u8>,
    },
    Failed {
        generation: u64,
        reason: String,
    },
}

/// A launched cycling viewer tied to its originating rust-feh state (feature
/// 016, contracts/viewer-roundtrip.md). Polled once per frame via `try_wait`;
/// on exit the handoff file is validated and, if accepted, staged. Holds a
/// live `Child`, which is why this lives in main.rs rather than types.rs
/// (same precedent as `ActiveToolsJob`, which holds a live `Receiver`).
struct ViewerRoundTrip {
    child: std::process::Child,
    handoff_path: PathBuf,
    launched_with: PathBuf,
    filelist: Vec<PathBuf>,
    viewer_id: u64,
}

impl Drop for RustFehApp {
    fn drop(&mut self) {
        if let Some(job) = self.tools_job.take() {
            job.cancel.store(true, Ordering::Relaxed);
        }
        if let Some(dir) = self.prepare_fast_temp.take() {
            let _ = std::fs::remove_dir_all(&dir);
        }
    }
}

impl RustFehApp {
    fn log(&mut self, msg: impl Into<String>) {
        let msg = msg.into();
        eprintln!("[rust-feh] {}", msg);
        self.debug_logs.push(msg);
        if self.debug_logs.len() > 100 {
            self.debug_logs.remove(0);
        }
    }

    fn compute_list_indices(&self) -> (usize, Vec<usize>) {
        {
            let cache = self.list_index_cache.borrow();
            if let Some((key, value)) = cache.as_ref() {
                if key.revision == self.images_revision
                    && key.sort_mode == self.sort_mode
                    && key.search == self.search
                    && key.current_dir.as_deref() == self.current_dir.as_deref()
                {
                    return value.clone();
                }
            }
        } // immutable borrow dropped before the mutable borrow below

        let total = self.images.len();
        let indices = list_indices(
            &self.images,
            self.current_dir.as_deref(),
            &self.search,
            self.sort_mode,
        );
        let value = (total, indices);
        let key = ListIndexKey {
            revision: self.images_revision,
            current_dir: self.current_dir.clone(),
            search: self.search.clone(),
            sort_mode: self.sort_mode,
        };
        *self.list_index_cache.borrow_mut() = Some((key, value.clone()));
        value
    }

    /// Single entry point for changing the active folder (feature 017
    /// drill-down nav): updates current_dir, clears any stale search filter,
    /// kicks a fresh scan, and requests the new folder's immediate subfolder
    /// listing. Used by folder-picker, the start-folder env hook, subfolder
    /// row clicks, Up/breadcrumb nav, and cross-folder round-trip landing.
    fn navigate_to_folder(&mut self, dir: &Path) {
        self.current_dir = Some(dir.to_path_buf());
        self.search.clear();
        self.scan_directory(dir);
        self.request_subfolders(dir);
    }

    /// Edge-detect the no-folder ↔ folder-loaded transition and drive Browse
    /// auto-expand through the same machine as the scan/tool triggers (018
    /// FIX-1). `current_dir` has many mutation sites, so this compares against a
    /// stored flag rather than hooking each one. Seeded `prior_folder_present =
    /// true`, so a first frame with no folder fires the no-folder rising edge
    /// (Browse opens — SC-001), while a start-folder that loads during frame 1
    /// (before this runs) leaves `present == prior` and never opens Browse
    /// (maintainer clarification: "folder already set at launch → Browse stays
    /// folded").
    fn sync_auto_expand_folder_edge(&mut self) {
        let present = self.current_dir.is_some();
        if present == self.prior_folder_present {
            return;
        }
        self.prior_folder_present = present;
        if present {
            // Folder loaded → falling edge for Browse: retract it.
            self.auto_expand.retract(
                InspectorSection::Browse,
                &mut self.inspector_open,
                &mut self.inspector_drawer_collapsed,
            );
        } else {
            // No folder → rising edge for Browse (a fresh no-folder scope).
            self.auto_expand.begin_scope(InspectorSection::Browse);
            self.auto_expand.request_open(
                InspectorSection::Browse,
                &mut self.inspector_open,
                &mut self.inspector_drawer_collapsed,
            );
        }
    }

    fn pick_folder(&mut self) {
        if let Some(dir) = rfd::FileDialog::new().pick_folder() {
            self.log(format!("User chose folder: {}", dir.display()));
            self.navigate_to_folder(&dir);
        }
    }

    /// Test hook: `RUST_FEH_START_FOLDER=/path` auto-loads on first frame (resource/perf scripts).
    fn maybe_load_start_folder(&mut self) {
        if self.start_folder_loaded {
            return;
        }
        self.start_folder_loaded = true;
        let Ok(dir) = std::env::var("RUST_FEH_START_FOLDER") else {
            return;
        };
        let path = PathBuf::from(&dir);
        if !path.is_dir() {
            self.log(format!(
                "RUST_FEH_START_FOLDER ignored (not a directory): {}",
                path.display()
            ));
            return;
        }
        self.log(format!(
            "Auto-loading RUST_FEH_START_FOLDER: {}",
            path.display()
        ));
        self.navigate_to_folder(&path);
    }

    fn feh_button(
        ui: &mut egui::Ui,
        label: &str,
        available: bool,
        enabled: bool,
    ) -> egui::Response {
        if available && enabled {
            ui.add(egui::Button::new(label))
        } else {
            ui.add(
                egui::Button::new(label)
                    .sense(egui::Sense::click())
                    .fill(ui.visuals().widgets.inactive.bg_fill),
            )
        }
    }

    fn feh_open_ready(&self) -> bool {
        self.feh_available && !self.compute_list_indices().1.is_empty()
    }

    /// Whether `path` is a member of the current filtered/sorted list (018
    /// FIX-3/FIX-4). This in-memory membership check is the pin-staleness signal
    /// for the detached Image-actions window — it replaces the per-frame
    /// blocking `Path::exists()` stat (a 017-class UI-freeze risk on SMB mounts)
    /// AND gates the pinned "Open in feh" action, which needs the pinned path to
    /// be in the live filelist it builds. Absent ⇒ the pin was moved/deleted or
    /// the user navigated to a different folder ⇒ disable pinned actions.
    fn pinned_path_in_filtered_list(&self, path: &Path) -> bool {
        let (_, indices) = self.compute_list_indices();
        indices
            .iter()
            .any(|&i| self.images[i].path.as_path() == path)
    }

    /// Keep selection aligned with the filtered list (FR-002 filelist / --start-at).
    fn sync_selection_to_filter(&mut self) {
        let (_, indices) = self.compute_list_indices();
        if indices.is_empty() {
            self.selected = None;
            return;
        }
        if let Some(sel) = &self.selected {
            if indices
                .iter()
                .any(|&i| self.images[i].path.as_path() == sel.as_path())
            {
                return;
            }
        }
        self.selected = Some(self.images[indices[0]].path.clone());
    }

    fn resolve_feh_start_path(&mut self) -> Option<PathBuf> {
        let (_, indices) = self.compute_list_indices();
        if indices.is_empty() {
            return None;
        }
        if let Some(sel) = &self.selected {
            if indices
                .iter()
                .any(|&i| self.images[i].path.as_path() == sel.as_path())
            {
                return Some(sel.clone());
            }
            let path = self.images[indices[0]].path.clone();
            self.log(format!(
                "Selection not in filtered list; using {} for feh",
                path.display()
            ));
            self.selected = Some(path.clone());
            self.status = format!(
                "Selected: {}. Use Image actions to open in feh, or right-click for Resize/Convert.",
                path.display()
            );
            return Some(path);
        }
        let path = self.images[indices[0]].path.clone();
        self.selected = Some(path.clone());
        Some(path)
    }

    fn try_open_in_feh(&mut self) {
        if !self.feh_available {
            self.status = feh_missing_status();
            return;
        }
        let Some(path) = self.resolve_feh_start_path() else {
            self.status = "No images in filtered list".to_owned();
            return;
        };
        self.open_in_feh(&path);
    }

    fn panel_context(&self, pin: Option<&PanelPin>) -> PanelContext {
        PanelContext::resolve(pin, self.selected.as_deref(), self.current_dir.as_deref())
    }

    fn clamp_viewport_size(&self, size: egui::Vec2) -> egui::Vec2 {
        let (w, h) = clamp_window_size(size.x, size.y);
        egui::vec2(w, h)
    }

    fn current_viewport_size(&self, ctx: &egui::Context) -> egui::Vec2 {
        let raw = ctx.input(|i| {
            i.viewport()
                .inner_rect
                .map(|r| r.size())
                .filter(|s| s.x > 0.0 && s.y > 0.0)
                .unwrap_or_else(|| {
                    let (w, h) = window_preset_dimensions(self.window_size);
                    egui::vec2(w, h)
                })
        });
        self.clamp_viewport_size(raw)
    }

    fn apply_window_resize_policy(&self, ctx: &egui::Context, lock_size: egui::Vec2) {
        let lock_size = self.clamp_viewport_size(lock_size);
        ctx.send_viewport_cmd(egui::ViewportCommand::Resizable(self.window_resizable));
        let (min_w, min_h) = WINDOW_MIN_RESIZABLE;
        ctx.send_viewport_cmd(egui::ViewportCommand::MinInnerSize(egui::vec2(
            min_w, min_h,
        )));
        if self.window_resizable {
            let (max_w, max_h) = WINDOW_MAX_RESIZABLE;
            ctx.send_viewport_cmd(egui::ViewportCommand::MaxInnerSize(egui::vec2(
                max_w, max_h,
            )));
        } else {
            ctx.send_viewport_cmd(egui::ViewportCommand::MaxInnerSize(lock_size));
        }
    }

    fn apply_window_preset(&self, ctx: &egui::Context) {
        let (w, h) = window_preset_dimensions(self.window_size);
        let size = self.clamp_viewport_size(egui::vec2(w, h));
        ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(size));
        self.apply_window_resize_policy(ctx, size);
    }

    /// Apply persisted window preferences to the viewport once, on the first frame.
    /// Startup seeds `prior_*` equal to the loaded values, so the change-detection
    /// branches in `sync_frame_input_state` never fire for restored prefs; without
    /// this the saved preset/resizable state stays in memory but is never pushed to
    /// the OS window (FR-008 / SC-005).
    fn apply_startup_window_prefs(&mut self, ctx: &egui::Context) {
        if self.window_prefs_applied {
            return;
        }
        self.window_prefs_applied = true;
        self.apply_window_preset(ctx);
        self.log(format!(
            "Restored window preference: {} ({})",
            window_preset_label(self.window_size),
            if self.window_resizable {
                "resizable"
            } else {
                "locked"
            }
        ));
    }

    fn refresh_tool_caps(&mut self) {
        self.tool_caps = ToolCapabilities::detect();
        self.feh_available = self.tool_caps.feh_available;
        if self.tool_caps.has_missing_required() {
            // Recheck still finds a missing tool → rising edge (fresh detection
            // scope) for the tool sections (018 FIX-1/FIX-7).
            for section in [
                InspectorSection::Dependencies,
                InspectorSection::FormatDiscovery,
            ] {
                self.auto_expand.begin_scope(section);
                self.auto_expand.request_open(
                    section,
                    &mut self.inspector_open,
                    &mut self.inspector_drawer_collapsed,
                );
            }
        } else {
            // All tools OK → symmetric falling edge: retract BOTH auto-opened
            // tool sections (018 FIX-7 — previously FormatDiscovery was never
            // removed on recovery).
            for section in [
                InspectorSection::Dependencies,
                InspectorSection::FormatDiscovery,
            ] {
                self.auto_expand.retract(
                    section,
                    &mut self.inspector_open,
                    &mut self.inspector_drawer_collapsed,
                );
            }
        }
        self.log(format!(
            "Rechecked tools: feh={}, magick={}",
            self.tool_caps.feh_available, self.tool_caps.magick_available
        ));
    }

    fn mark_feh_unavailable(&mut self) {
        self.feh_available = false;
        self.tool_caps.feh_available = false;
        self.status = feh_missing_status();
        // Mid-session tool loss is a rising edge (an event) → surface
        // Dependencies + expand the drawer via the same machine (018 FIX-7).
        self.auto_expand.begin_scope(InspectorSection::Dependencies);
        self.auto_expand.request_open(
            InspectorSection::Dependencies,
            &mut self.inspector_open,
            &mut self.inspector_drawer_collapsed,
        );
        self.log("feh marked unavailable after spawn failure".to_owned());
    }

    fn rotating_operation_tip(&self, time: f64) -> (char, String) {
        let ops: Vec<_> = self.tool_caps.operation_timings();
        if ops.is_empty() {
            return (' ', String::new());
        }
        let spinner = ['|', '/', '-', '\\'][(time * 4.0).floor() as usize % 4];
        let idx = (time / 4.0).floor() as usize % ops.len();
        let op = &ops[idx];
        (
            spinner,
            format!(
                "{} → {} · {} ({})",
                op.operation,
                op.handler.label(),
                op.speed.label(),
                op.speed.detail()
            ),
        )
    }

    fn is_activity_busy(&self) -> bool {
        self.scanning
    }

    fn activity_pulse_color(time: f64, busy: bool) -> egui::Color32 {
        if !busy {
            return egui::Color32::TRANSPARENT;
        }
        let pulse = ((time * 5.0).sin() * 0.5 + 0.5) as f32;
        egui::Color32::from_rgba_unmultiplied(
            (40.0 + 30.0 * pulse) as u8,
            (90.0 + 50.0 * pulse) as u8,
            (200.0 + 40.0 * pulse) as u8,
            48,
        )
    }

    fn render_activity_log_body(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.vertical(|ui| {
            egui::Frame::none()
                .inner_margin(egui::Margin::symmetric(4.0, 2.0))
                .stroke(egui::Stroke::new(
                    1.0,
                    ui.style().visuals.widgets.noninteractive.bg_stroke.color,
                ))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        if ui.button("Copy log").clicked() {
                            ctx.copy_text(join_activity_log(&self.debug_logs));
                        }
                        if ui.button("Clear logs").clicked() {
                            self.debug_logs.clear();
                        }
                    });
                });

            ui.add_space(6.0);

            let log_text = join_activity_log(&self.debug_logs);
            let log_height = ui.available_height().clamp(100.0, 220.0);
            egui::Frame::group(ui.style())
                .inner_margin(6.0)
                .show(ui, |ui| {
                    ui.set_min_height(log_height);
                    egui::ScrollArea::vertical()
                        .auto_shrink([false; 2])
                        .max_height(log_height)
                        .id_salt("activity_log_scroll")
                        .show(ui, |ui| {
                            ui.add(
                                egui::Label::new(egui::RichText::new(log_text).monospace())
                                    .selectable(true)
                                    .wrap_mode(egui::TextWrapMode::Wrap),
                            );
                        });
                });
        });
    }

    fn render_dependency_row(
        &mut self,
        ui: &mut egui::Ui,
        ctx: &egui::Context,
        dep: &rust_feh::tool_caps::DependencyStatus,
    ) {
        let (icon, color) = if dep.installed {
            ("✅", egui::Color32::from_rgb(80, 200, 120))
        } else if dep.kind == DepKind::Required {
            ("❌", egui::Color32::from_rgb(220, 80, 80))
        } else {
            ("○", egui::Color32::from_rgb(200, 160, 60))
        };

        ui.horizontal(|ui| {
            ui.label(icon);
            ui.vertical(|ui| {
                let kind = match dep.kind {
                    DepKind::Required => "required",
                    DepKind::Optional => "optional",
                };
                ui.colored_label(color, format!("{} ({})", dep.name, kind));
                ui.small(dep.role);
                if dep.installed {
                    if let Some(bin) = &dep.resolved_binary {
                        ui.small(format!("On PATH: {bin}"));
                    }
                } else {
                    ui.colored_label(color, "Not installed");
                    ui.horizontal(|ui| {
                        ui.monospace(dep.install_cmd);
                        if ui.small_button(format!("Copy##{}", dep.name)).clicked() {
                            ctx.copy_text(dep.install_cmd.to_string());
                            self.status = format!("Copied install command for {}", dep.name);
                        }
                    });
                }
            });
        });
        ui.add_space(4.0);
    }

    fn toggle_inspector_section(&mut self, section: InspectorSection) {
        if self.inspector_open.contains(&section) {
            self.inspector_open.remove(&section);
            // Manual close latches suppression so the machine will not re-open it
            // this scope (018 FIX-1 / FR-003).
            self.auto_expand.note_user_close(section);
        } else {
            self.inspector_open.insert(section);
            // Manual open makes the section user-owned (survives retraction).
            self.auto_expand.note_user_open(section);
        }
    }

    fn toggle_format_route(&mut self, route_id: &str) {
        if self.format_route_open.contains(route_id) {
            self.format_route_open.remove(route_id);
        } else {
            self.format_route_open.insert(route_id.to_string());
        }
    }

    fn render_format_route_body(ui: &mut egui::Ui, route: &FormatRoute) {
        ui.horizontal(|ui| {
            ui.label(format!("Scan: {}", route.scan.label()));
            ui.label(format!("View: {}", route.view.label()));
            ui.label(format!("Resize: {}", route.resize.label()));
        });
        ui.small(format!(
            "View speed: {} — {}",
            route.view_speed.label(),
            route.note
        ));
    }

    fn header_with_detach_suffix(label: String, detached: bool) -> String {
        if detached {
            format!("{label} — detached")
        } else {
            label
        }
    }

    fn render_segment_detach_toolbar(ui: &mut egui::Ui, caption: &str, detach_label: &str) -> bool {
        let mut detach = false;
        egui::Frame::none()
            .inner_margin(egui::Margin::symmetric(4.0, 2.0))
            .stroke(egui::Stroke::new(
                1.0,
                ui.style().visuals.widgets.noninteractive.bg_stroke.color,
            ))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.small(caption);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.small_button(detach_label).clicked() {
                            detach = true;
                        }
                    });
                });
            });
        ui.add_space(6.0);
        detach
    }

    fn render_detached_placeholder(ui: &mut egui::Ui, segment: &str) {
        ui.small(format!(
            "{segment} is in a separate window. Close it with X to return here."
        ));
    }

    fn activity_log_header_label(&self) -> String {
        let base = {
            let n = self.debug_logs.len();
            if n == 0 {
                "Activity log".to_string()
            } else {
                format!("Activity log — {n} events")
            }
        };
        Self::header_with_detach_suffix(
            base,
            self.detached.contains_key(&InspectorSection::ActivityLog),
        )
    }

    fn deps_header_label(&self) -> String {
        let base = if !self.tool_caps.has_missing_required() {
            "✅ Dependencies — all required tools OK".to_string()
        } else {
            "⚠ Dependencies — action needed".to_string()
        };
        Self::header_with_detach_suffix(
            base,
            self.detached.contains_key(&InspectorSection::Dependencies),
        )
    }

    fn format_discovery_header_label(&self) -> String {
        let routes = self.tool_caps.format_routes();
        let base = format!("Format discovery — {} groups", routes.len());
        Self::header_with_detach_suffix(
            base,
            self.detached
                .contains_key(&InspectorSection::FormatDiscovery),
        )
    }

    fn render_inspector_activity_log(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        if self.detached.contains_key(&InspectorSection::ActivityLog) {
            Self::render_detached_placeholder(ui, "Activity log");
            return;
        }

        if Self::render_segment_detach_toolbar(
            ui,
            "Scan events, feh commands, warnings",
            "Detach window",
        ) {
            self.detached
                .insert(InspectorSection::ActivityLog, DetachedWindow::default());
        }
        self.render_activity_log_body(ui, ctx);
    }

    fn render_deps_section_body(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let deps = self.tool_caps.dependencies();
        for dep in &deps {
            self.render_dependency_row(ui, ctx, dep);
        }

        if ui.button("Recheck tools on PATH").clicked() {
            self.refresh_tool_caps();
        }
    }

    fn render_inspector_dependencies(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        if self.detached.contains_key(&InspectorSection::Dependencies) {
            Self::render_detached_placeholder(ui, "Dependencies");
            return;
        }

        if Self::render_segment_detach_toolbar(
            ui,
            "feh, ImageMagick, and other PATH tools",
            "Detach window",
        ) {
            self.detached
                .insert(InspectorSection::Dependencies, DetachedWindow::default());
        }
        self.render_deps_section_body(ui, ctx);
    }

    fn render_format_discovery_body(&mut self, ui: &mut egui::Ui) {
        ui.small("Scan = native listed or magick-detected; View = feh.");
        let routes = self.tool_caps.format_routes();
        for route in &routes {
            let route_id = route.extensions.to_string();
            let route_open = self.format_route_open.contains(&route_id);
            let route_response = egui::CollapsingHeader::new(route.summary_line())
                .id_salt(format!("tool_route_{route_id}"))
                .open(Some(route_open))
                .show(ui, |ui| {
                    Self::render_format_route_body(ui, route);
                });
            if route_response.header_response.clicked() {
                self.toggle_format_route(&route_id);
            }
        }
    }

    fn render_inspector_format_discovery(&mut self, ui: &mut egui::Ui) {
        if self
            .detached
            .contains_key(&InspectorSection::FormatDiscovery)
        {
            Self::render_detached_placeholder(ui, "Format discovery");
            return;
        }

        if Self::render_segment_detach_toolbar(
            ui,
            "Per-format scan, view, and resize routing",
            "Detach window",
        ) {
            self.detached
                .insert(InspectorSection::FormatDiscovery, DetachedWindow::default());
        }
        self.render_format_discovery_body(ui);
    }

    fn browse_header_label(&self) -> String {
        let base = match &self.current_dir {
            None => "Browse — No folder loaded".to_string(),
            Some(dir) => {
                let name = dir
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| dir.display().to_string());
                if self.search.is_empty() {
                    format!("Browse — {name}")
                } else {
                    format!("Browse — {name} · filtered")
                }
            }
        };
        Self::header_with_detach_suffix(base, self.detached.contains_key(&InspectorSection::Browse))
    }

    fn render_inspector_browse(&mut self, ui: &mut egui::Ui) {
        if self.detached.contains_key(&InspectorSection::Browse) {
            Self::render_detached_placeholder(ui, "Browse");
            return;
        }

        if Self::render_segment_detach_toolbar(
            ui,
            "Folder, filter, and sort controls",
            "Detach window",
        ) {
            self.detached
                .insert(InspectorSection::Browse, DetachedWindow::default());
        }
        self.render_browse_controls_body(ui);
    }

    fn image_actions_header_label(&self) -> String {
        let base = match &self.selected {
            None => "Image actions — no selection".to_string(),
            Some(path) => format!(
                "Image actions — {}",
                path.file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| path.display().to_string())
            ),
        };
        Self::header_with_detach_suffix(
            base,
            self.detached.contains_key(&InspectorSection::ImageActions),
        )
    }

    fn render_image_actions_body(&mut self, ui: &mut egui::Ui, pctx: &PanelContext) {
        if pctx.pinned {
            let enabled = self.feh_available && pctx.image.is_some();
            if Self::feh_button(ui, "Open in feh", self.feh_available, enabled).clicked() {
                self.log("User clicked 'Open in feh' (pinned)");
                if let Some(p) = pctx.image.as_deref() {
                    self.open_in_feh_pinned(p);
                }
            }
            return;
        }
        let feh_ready = self.feh_open_ready();
        if Self::feh_button(ui, "Open in feh", self.feh_available, feh_ready).clicked() {
            self.log("User clicked 'Open in feh' (inspector)");
            self.try_open_in_feh();
        }
    }

    /// Pin-to-current-image / Unpin toggle for the detached Image-actions
    /// window (018 Batch 5, decision 4): pinning captures `self.selected` at
    /// the moment of the click, so the window keeps acting on that file even
    /// as the live selection moves elsewhere. `advance_stage_after_move` only
    /// fixes up the central stage's path on a move — it does NOT follow or
    /// clear pins, so a pinned file that gets moved/deleted goes stale until
    /// the user unpins (handled by the stale-pin hint in the caller).
    fn render_image_actions_pin_toggle(&mut self, ui: &mut egui::Ui) {
        let pinned = self
            .detached
            .get(&InspectorSection::ImageActions)
            .and_then(|w| w.pin.as_ref())
            .is_some();
        ui.horizontal(|ui| {
            if pinned {
                if ui.small_button("Unpin (follow selection)").clicked() {
                    if let Some(w) = self.detached.get_mut(&InspectorSection::ImageActions) {
                        w.pin = None;
                    }
                }
            } else {
                let can_pin = self.selected.is_some();
                if ui
                    .add_enabled(can_pin, egui::Button::new("Pin to current image").small())
                    .clicked()
                {
                    if let Some(sel) = self.selected.clone() {
                        if let Some(w) = self.detached.get_mut(&InspectorSection::ImageActions) {
                            w.pin = Some(PanelPin::Image(sel));
                        }
                    }
                }
            }
        });
        ui.separator();
    }

    /// Image-actions body + Image Tools, as a single unit (018 Batch 4 parity
    /// fix): the detached Image-actions window previously showed only the
    /// body (missing Image Tools) while the docked inspector showed both.
    /// Both call sites now go through this one function so they can't drift
    /// apart again.
    fn render_image_actions_full(
        &mut self,
        ui: &mut egui::Ui,
        ctx: &egui::Context,
        pctx: &PanelContext,
    ) {
        self.render_image_actions_body(ui, pctx);
        ui.separator();
        self.render_inspector_image_tools(ui, ctx, pctx);
    }

    fn render_inspector_image_actions(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        if self.detached.contains_key(&InspectorSection::ImageActions) {
            Self::render_detached_placeholder(ui, "Image actions");
            return;
        }

        if Self::render_segment_detach_toolbar(ui, "Open in feh and image tools", "Detach window") {
            self.detached
                .insert(InspectorSection::ImageActions, DetachedWindow::default());
        }
        let pctx = self.panel_context(None);
        self.render_image_actions_full(ui, ctx, &pctx);
    }

    /// FR-002 default folder resolution for a new launch entry.
    fn resolve_default_entry_folder(&self) -> Option<PathBuf> {
        if let Some(folder) = &self.selected_tree_folder {
            if folder.is_dir() {
                return Some(folder.clone());
            }
        }
        if let Some(sel) = &self.selected {
            if let Some(parent) = sel.parent() {
                return Some(parent.to_path_buf());
            }
        }
        self.current_dir.clone()
    }

    /// Unique folders from the current scan, for the per-entry folder ComboBox.
    fn folder_candidates(&self) -> Vec<PathBuf> {
        let mut seen = HashSet::new();
        let mut folders = Vec::new();
        if let Some(dir) = &self.current_dir {
            if seen.insert(dir.clone()) {
                folders.push(dir.clone());
            }
        }
        for folder in &self.subfolders {
            if seen.insert(folder.clone()) {
                folders.push(folder.clone());
            }
        }
        for entry in &self.launch_entries.entries {
            if let Some(folder) = &entry.folder_path {
                if seen.insert(folder.clone()) {
                    folders.push(folder.clone());
                }
            }
        }
        folders.sort();
        folders
    }

    fn feh_instances_header_label(&self) -> String {
        let n = self.launch_entries.entries.len();
        let base = if n == 0 {
            "Feh instances".to_string()
        } else {
            format!("Feh instances — {n}")
        };
        Self::header_with_detach_suffix(
            base,
            self.detached.contains_key(&InspectorSection::FehInstances),
        )
    }

    fn persist_launch_entries(&mut self) {
        if let Err(e) = save_launch_list(&self.launch_entries) {
            self.log(format!("Failed to save launch entries: {e}"));
            self.status = e;
        }
    }

    fn persist_window_prefs(&mut self) {
        let prefs = WindowPreferences {
            version: 1,
            preset: self.window_size,
            resizable: self.window_resizable,
        };
        if let Err(e) = save_window_prefs(&prefs) {
            self.log(format!("Failed to save window preferences: {e}"));
            self.status = e;
        }
    }

    fn add_launch_entry(&mut self) {
        let folder = self.resolve_default_entry_folder();
        let created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let id = format!("{created_at:x}-{}", self.launch_entries.entries.len());
        if self.launch_entries.version == 0 {
            self.launch_entries.version = 1;
        }
        self.launch_entries.entries.push(FehLaunchEntry {
            id,
            label: None,
            folder_path: folder,
            created_at,
        });
        self.persist_launch_entries();
        self.log("Added feh launch entry");
    }

    fn remove_launch_entry(&mut self, id: &str) {
        let before = self.launch_entries.entries.len();
        self.launch_entries.entries.retain(|e| e.id != id);
        if self.launch_entries.entries.len() != before {
            self.persist_launch_entries();
            self.log("Removed feh launch entry");
        }
    }

    fn update_entry_folder(&mut self, id: &str, folder: Option<PathBuf>) {
        if let Some(entry) = self.launch_entries.entries.iter_mut().find(|e| e.id == id) {
            entry.folder_path = folder;
            self.persist_launch_entries();
        }
    }

    fn update_entry_label(&mut self, id: &str, label: &str) {
        let trimmed = label.trim();
        let label = if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        };
        if let Some(entry) = self.launch_entries.entries.iter_mut().find(|e| e.id == id) {
            entry.label = label;
            self.persist_launch_entries();
        }
    }

    fn launch_entry(&mut self, id: &str) {
        if !self.feh_available {
            self.status = feh_missing_status();
            return;
        }
        let Some(entry) = self
            .launch_entries
            .entries
            .iter()
            .find(|e| e.id == id)
            .cloned()
        else {
            return;
        };
        self.launch_entry_feh(&entry);
    }

    fn spawn_feh_viewer(
        &mut self,
        list_path: &Path,
        start_at: &Path,
        log_msg: String,
        success_status: String,
    ) {
        let mut cmd = Command::new("feh");
        cmd.arg("--geometry")
            .arg(FEH_VIEWER_GEOMETRY)
            .arg("--scale-down")
            .arg("--zoom")
            .arg(FEH_VIEWER_ZOOM)
            .arg("--filelist")
            .arg(list_path)
            .arg("--start-at")
            .arg(start_at);
        self.log(log_msg);
        match cmd.spawn() {
            Ok(child) => {
                self.log(format!("feh launched (pid {:?})", child.id()));
                self.status = success_status;
            }
            Err(e) => {
                self.log(format!("Failed to spawn feh: {e}"));
                if feh_spawn_unavailable(&e) {
                    self.mark_feh_unavailable();
                } else {
                    self.status = format!("Failed to launch feh (is it installed?): {e}");
                }
            }
        }
    }

    fn launch_entry_feh(&mut self, entry: &FehLaunchEntry) {
        let state = entry_is_launchable(entry, self.feh_available);
        if !state.launchable {
            self.status = format!("Cannot launch: {}", state.status);
            return;
        }
        let paths = build_entry_filelist(entry);
        if paths.is_empty() {
            self.status = format!(
                "No images in {}",
                entry
                    .folder_path
                    .as_deref()
                    .map(|p| p.display().to_string())
                    .unwrap_or_default()
            );
            return;
        }
        let start = paths[0].clone();
        let list_path = feh_entry_filelist_path(&entry.id);
        let count = match write_feh_filelist_to(&list_path, &paths) {
            Ok(n) => n,
            Err(e) => {
                self.log(format!("Failed to write feh filelist: {e}"));
                self.status = format!("Failed to prepare feh filelist: {e}");
                return;
            }
        };
        self.spawn_feh_viewer(
            &list_path,
            &start,
            format!("Spawning feh for entry {} ({count} images)", entry.id),
            format!("Launched feh ({count} images)"),
        );
    }

    fn launch_all_entries(&mut self) {
        if !self.feh_available {
            self.status = feh_missing_status();
            return;
        }
        let ids: Vec<String> = self
            .launch_entries
            .entries
            .iter()
            .filter(|e| entry_is_launchable(e, self.feh_available).launchable)
            .map(|e| e.id.clone())
            .collect();
        let count = ids.len();
        for id in ids {
            self.launch_entry(&id);
        }
        self.status = format!("Launched {count} feh instances");
        self.log(format!("Launch All spawned {count} feh instances"));
    }

    fn apply_feh_entry_action(&mut self, action: FehEntryAction) {
        match action {
            FehEntryAction::Remove(id) => self.remove_launch_entry(&id),
            FehEntryAction::Launch(id) => self.launch_entry(&id),
            FehEntryAction::SetFolder(id, folder) => self.update_entry_folder(&id, folder),
            FehEntryAction::SetLabel(id, label) => {
                self.update_entry_label(&id, label.as_deref().unwrap_or(""));
            }
        }
    }

    fn feh_entry_display_label(entry: &FehLaunchEntry) -> String {
        entry.label.clone().unwrap_or_else(|| {
            entry
                .folder_path
                .as_ref()
                .and_then(|p| p.file_name())
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "(new)".to_string())
        })
    }

    fn render_feh_entry_label_editor(
        ui: &mut egui::Ui,
        entry: &FehLaunchEntry,
        action: &mut Option<FehEntryAction>,
    ) {
        let mut label_buf = entry.label.clone().unwrap_or_default();
        let placeholder = entry
            .folder_path
            .as_ref()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "Label".to_string());
        ui.horizontal(|ui| {
            ui.small("Label:");
            if ui
                .add(egui::TextEdit::singleline(&mut label_buf).hint_text(placeholder))
                .changed()
            {
                let trimmed = label_buf.trim();
                let new_label = if trimmed.is_empty() {
                    None
                } else {
                    Some(trimmed.to_string())
                };
                if new_label != entry.label {
                    *action = Some(FehEntryAction::SetLabel(entry.id.clone(), new_label));
                }
            }
        });
    }

    fn render_feh_entry_folder_picker(
        ui: &mut egui::Ui,
        entry: &FehLaunchEntry,
        candidates: &[PathBuf],
        action: &mut Option<FehEntryAction>,
    ) {
        let current = entry
            .folder_path
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "None selected".to_string());
        egui::ComboBox::from_id_salt(format!("feh_entry_folder_{}", entry.id))
            .selected_text(current)
            .show_ui(ui, |ui| {
                if ui
                    .selectable_label(entry.folder_path.is_none(), "None selected")
                    .clicked()
                {
                    *action = Some(FehEntryAction::SetFolder(entry.id.clone(), None));
                }
                for cand in candidates {
                    let selected = entry.folder_path.as_deref() == Some(cand.as_path());
                    if ui
                        .selectable_label(selected, cand.display().to_string())
                        .clicked()
                    {
                        *action = Some(FehEntryAction::SetFolder(
                            entry.id.clone(),
                            Some(cand.clone()),
                        ));
                    }
                }
            });
    }

    fn render_feh_entry_launch_button(
        ui: &mut egui::Ui,
        entry: &FehLaunchEntry,
        state: &EntryLaunchState,
        action: &mut Option<FehEntryAction>,
    ) {
        if state.launchable {
            if ui
                .add_sized(
                    [ui.available_width(), 0.0],
                    egui::Button::new(format!("Launch ({})", state.status)),
                )
                .clicked()
            {
                *action = Some(FehEntryAction::Launch(entry.id.clone()));
            }
        } else {
            ui.add_enabled(
                false,
                egui::Button::new(format!("Launch — {}", state.status)),
            );
        }
    }

    fn render_feh_entry_card(
        ui: &mut egui::Ui,
        idx: usize,
        entry: &FehLaunchEntry,
        state: &EntryLaunchState,
        candidates: &[PathBuf],
        action: &mut Option<FehEntryAction>,
    ) {
        let display_label = Self::feh_entry_display_label(entry);
        egui::Frame::group(ui.style())
            .inner_margin(6.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.strong(format!("#{}: {display_label}", idx + 1));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.small_button("×").clicked() {
                            *action = Some(FehEntryAction::Remove(entry.id.clone()));
                        }
                    });
                });
                Self::render_feh_entry_label_editor(ui, entry, action);
                Self::render_feh_entry_folder_picker(ui, entry, candidates, action);
                Self::render_feh_entry_launch_button(ui, entry, state, action);
            });
    }

    fn render_feh_instances_body(&mut self, ui: &mut egui::Ui) {
        let feh_available = self.feh_available;
        let candidates = self.folder_candidates();
        ui.horizontal(|ui| {
            if ui.button("+ Add").clicked() {
                self.add_launch_entry();
            }
            let any_launchable = self
                .launch_entries
                .entries
                .iter()
                .any(|e| entry_is_launchable(e, feh_available).launchable);
            if ui
                .add_enabled(any_launchable, egui::Button::new("Launch All"))
                .clicked()
            {
                self.launch_all_entries();
            }
        });
        if self.launch_entries.entries.is_empty() {
            ui.small("Add folders to launch in feh.");
            return;
        }
        if !feh_available {
            ui.small(feh_not_installed_launch_status());
        }
        let mut action = None;
        egui::ScrollArea::vertical()
            .id_salt("feh_instances_list")
            .max_height(320.0)
            .auto_shrink([false, true])
            .show(ui, |ui| {
                let entries = self.launch_entries.entries.clone();
                for (idx, entry) in entries.iter().enumerate() {
                    let state = entry_is_launchable(entry, feh_available);
                    Self::render_feh_entry_card(ui, idx, entry, &state, &candidates, &mut action);
                }
            });
        if let Some(action) = action {
            self.apply_feh_entry_action(action);
        }
    }

    fn render_inspector_feh_instances(&mut self, ui: &mut egui::Ui) {
        if self.detached.contains_key(&InspectorSection::FehInstances) {
            Self::render_detached_placeholder(ui, "Feh instances");
            return;
        }
        if Self::render_segment_detach_toolbar(
            ui,
            "Manage multiple feh launch configurations",
            "Detach window",
        ) {
            self.detached
                .insert(InspectorSection::FehInstances, DetachedWindow::default());
        }
        self.render_feh_instances_body(ui);
    }

    fn tools_output_policy(&self) -> OutputPolicy {
        match self.tools_panel.policy_ui {
            ToolsPolicyUi::Subfolder => OutputPolicy::NewSubfolder {
                name: self.tools_panel.subfolder_name.clone(),
            },
            ToolsPolicyUi::Suffixed => OutputPolicy::SuffixedSibling {
                suffix: self.tools_panel.suffix.clone(),
            },
            ToolsPolicyUi::InPlace => OutputPolicy::InPlaceWithBackup {
                backup_suffix: ".bak".into(),
            },
        }
    }

    fn tools_build_resize_op(&self) -> Result<ImageOperation, String> {
        let (width, height, percent) = if self.tools_panel.use_percent {
            let p: f32 = self
                .tools_panel
                .resize_percent
                .parse()
                .map_err(|_| "invalid percent".to_string())?;
            (None, None, Some(p))
        } else {
            let w = if self.tools_panel.resize_width.is_empty() {
                None
            } else {
                Some(
                    self.tools_panel
                        .resize_width
                        .parse()
                        .map_err(|_| "invalid width".to_string())?,
                )
            };
            let h = if self.tools_panel.resize_height.is_empty() {
                None
            } else {
                Some(
                    self.tools_panel
                        .resize_height
                        .parse()
                        .map_err(|_| "invalid height".to_string())?,
                )
            };
            (w, h, None)
        };
        Ok(ImageOperation::Resize {
            width,
            height,
            percent,
            fit: Some(self.tools_panel.fit),
            filter: Some(self.tools_panel.filter),
            quality: Some(self.tools_panel.quality),
        })
    }

    fn tools_build_single_op(&self) -> Result<ImageOperation, String> {
        match self.tools_panel.single_op {
            ToolsSingleOp::Resize => self.tools_build_resize_op(),
            ToolsSingleOp::Crop => Ok(ImageOperation::Crop {
                geometry: self.tools_panel.crop_geometry.clone(),
            }),
            ToolsSingleOp::Convert => Ok(ImageOperation::Convert {
                target_format: self.tools_panel.convert_format.clone(),
                quality: Some(self.tools_panel.quality),
            }),
        }
    }

    fn tools_apply_single(&mut self, path: &Path) {
        let op = match self.tools_build_single_op() {
            Ok(o) => o,
            Err(e) => {
                self.log(format!("Image tools param error: {e}"));
                return;
            }
        };
        let policy = self.tools_output_policy();
        if matches!(policy, OutputPolicy::InPlaceWithBackup { .. }) {
            if self.tools_panel.inplace_confirm < 2 {
                self.tools_panel.inplace_confirm += 1;
                self.status = format!(
                    "In-place requires confirmation ({}/2) — click Apply again",
                    self.tools_panel.inplace_confirm
                );
                return;
            }
            self.tools_panel.inplace_confirm = 0;
        }
        match self.image_tools.process_single(path, op, policy) {
            Ok(res) => self.tools_on_processed(res),
            Err(e) => self.log(format!("Image tools error: {e}")),
        }
    }

    fn tools_on_processed(&mut self, res: ProcessedResult) {
        add_or_update_asset_in_inventory(
            &mut self.images,
            res.dest_path.clone(),
            AssetStatus::Processed,
        );
        if let Some(ref inv) = self.scan_inventory {
            let inventory = ScanInventory::from_entries(
                &self.images,
                inv.non_image_skipped,
                inv.magick_identify_truncated,
            );
            self.scan_inventory = Some(inventory);
        }
        self.log(format_image_tools_log(&res));
        self.status = format!("Created {}", res.dest_path.display());
    }

    fn tools_batch_paths(&self) -> Vec<PathBuf> {
        let (_, indices) = self.compute_list_indices();
        indices
            .into_iter()
            .map(|i| self.images[i].path.clone())
            .collect()
    }

    fn tools_job_active(&self) -> bool {
        self.tools_job.is_some()
    }

    fn cancel_tools_job(&mut self) {
        if let Some(job) = self.tools_job.take() {
            job.cancel.store(true, Ordering::Relaxed);
            self.status = "Cancelled background image-tools job".into();
            self.log("Image tools job cancelled");
        }
    }

    fn cleanup_prepare_fast_temp(&mut self) {
        if let Some(dir) = self.prepare_fast_temp.take() {
            let _ = std::fs::remove_dir_all(&dir);
        }
        self.prepared_fast = None;
    }

    fn tools_start_batch_job(&mut self) {
        if self.tools_job_active() {
            self.status = "Another image-tools job is already running".into();
            return;
        }
        let paths = self.tools_batch_paths();
        if paths.is_empty() {
            self.status = "No images in filtered list for batch".into();
            return;
        }
        let op = match self.tools_build_single_op() {
            Ok(o) => o,
            Err(e) => {
                self.log(format!("Batch param error: {e}"));
                return;
            }
        };
        let policy = self.tools_output_policy();
        let svc = self.image_tools.clone();
        let cancel = Arc::new(AtomicBool::new(false));
        let rx = spawn_job(paths.clone(), cancel.clone(), move |path| {
            svc.process_single(path, op.clone(), policy.clone())
        });
        self.tools_job = Some(ActiveToolsJob {
            kind: ToolsJobKind::Batch,
            rx_batch: Some(rx),
            rx_bool: None,
            rx_paths: None,
            cancel,
            current: 0,
            total: paths.len(),
            message: "Starting batch…".into(),
            batch_ok: 0,
            batch_fail: 0,
            precache_ok: 0,
            prepare_paths: vec![],
            prepare_temp: PathBuf::new(),
        });
        self.tools_panel.batch_confirm_open = false;
        self.tools_panel.batch_inplace_confirm = 0;
        self.status = format!("Batch started ({} images)…", paths.len());
        self.log(format!("Batch job started for {} images", paths.len()));
    }

    fn tools_start_precache_job(&mut self) {
        if self.tools_job_active() {
            self.status = "Another image-tools job is already running".into();
            return;
        }
        if !self.cache_config.enabled {
            self.status = "Enable cache before pre-caching".into();
            return;
        }
        self.image_tools.update_cache(self.cache_config.clone());
        if !self.image_tools.cache_enabled_and_ready() {
            self.status =
                "Cache not ready — create cache root (see README) and Apply cache settings".into();
            return;
        }
        let paths = self.tools_batch_paths();
        if paths.is_empty() {
            return;
        }
        let svc = self.image_tools.clone();
        let cancel = Arc::new(AtomicBool::new(false));
        let rx = spawn_job(paths.clone(), cancel.clone(), move |path| {
            if svc.pre_cache_one(path) {
                Ok(true)
            } else {
                Err("cache put skipped".into())
            }
        });
        self.tools_job = Some(ActiveToolsJob {
            kind: ToolsJobKind::PreCache,
            rx_batch: None,
            rx_bool: Some(rx),
            rx_paths: None,
            cancel,
            current: 0,
            total: paths.len(),
            message: "Pre-cache…".into(),
            batch_ok: 0,
            batch_fail: 0,
            precache_ok: 0,
            prepare_paths: vec![],
            prepare_temp: PathBuf::new(),
        });
        self.log(format!("Pre-cache job started for {} images", paths.len()));
    }

    fn tools_start_prepare_fast_job(&mut self) {
        if self.tools_job_active() {
            self.status = "Another image-tools job is already running".into();
            return;
        }
        let paths = self.tools_batch_paths();
        if paths.is_empty() {
            return;
        }
        self.cleanup_prepare_fast_temp();
        let temp = prepare_fast_work_dir();
        let svc = self.image_tools.clone();
        let temp_clone = temp.clone();
        let cancel = Arc::new(AtomicBool::new(false));
        let rx = spawn_job(paths.clone(), cancel.clone(), move |path| {
            svc.prepare_fast_one(path, &temp_clone)
        });
        self.prepare_fast_temp = Some(temp.clone());
        self.tools_job = Some(ActiveToolsJob {
            kind: ToolsJobKind::PrepareFast,
            rx_batch: None,
            rx_bool: None,
            rx_paths: Some(rx),
            cancel,
            current: 0,
            total: paths.len(),
            message: "Prepare Fast…".into(),
            batch_ok: 0,
            batch_fail: 0,
            precache_ok: 0,
            prepare_paths: vec![],
            prepare_temp: temp,
        });
        self.log(format!(
            "Prepare Fast job started for {} images",
            paths.len()
        ));
    }

    fn tools_finish_prepare_fast(&mut self, paths: Vec<PathBuf>, temp: PathBuf) {
        let source_folder = self
            .current_dir
            .clone()
            .unwrap_or_else(|| PathBuf::from("."));
        let filelist_path = feh_filelist_temp_path();
        let mut set = PreparedFastSet {
            materialized_paths: paths.clone(),
            filelist_path: None,
            source_folder,
        };
        if let Ok(n) = write_feh_filelist(&paths) {
            set.filelist_path = Some(filelist_path);
            self.log(format!(
                "Prepare Fast: {n} optimized files in {}",
                temp.display()
            ));
        }
        for p in &paths {
            add_or_update_asset_in_inventory(&mut self.images, p.clone(), AssetStatus::Optimized);
        }
        if let Some(ref inv) = self.scan_inventory {
            let inventory = ScanInventory::from_entries(
                &self.images,
                inv.non_image_skipped,
                inv.magick_identify_truncated,
            );
            self.scan_inventory = Some(inventory);
        }
        self.prepared_fast = Some(set);
        self.status = format!(
            "Prepare Fast complete — {} optimized files ready",
            paths.len()
        );
    }

    fn poll_batch_job_rx(
        &mut self,
        job: &mut ActiveToolsJob,
        rx: &mpsc::Receiver<JobMsg<ProcessedResult>>,
    ) -> (bool, bool) {
        let mut done = false;
        let mut cancelled = false;
        while let Ok(msg) = rx.try_recv() {
            match msg {
                JobMsg::Progress(p) => {
                    job.current = p.current;
                    job.total = p.total;
                    job.message = p.message.clone();
                    if p.message.starts_with("skip:") {
                        job.batch_fail += 1;
                    }
                    self.status = format!("Batch {}/{}: {}", p.current + 1, p.total, p.message);
                }
                JobMsg::Item(res) => {
                    job.batch_ok += 1;
                    self.tools_on_processed(res);
                }
                JobMsg::Cancelled => {
                    cancelled = true;
                    done = true;
                }
                JobMsg::Done => {
                    let summary = format!(
                        "Batch done: {} ok, {} failed of {}",
                        job.batch_ok, job.batch_fail, job.total
                    );
                    self.tools_panel.batch_summary = Some(summary.clone());
                    self.status = summary;
                    done = true;
                }
            }
        }
        (done, cancelled)
    }

    fn poll_precache_job_rx(
        &mut self,
        job: &mut ActiveToolsJob,
        rx: &mpsc::Receiver<JobMsg<bool>>,
    ) -> (bool, bool) {
        let mut done = false;
        let mut cancelled = false;
        while let Ok(msg) = rx.try_recv() {
            match msg {
                JobMsg::Progress(p) => {
                    job.current = p.current;
                    job.total = p.total;
                    job.message = p.message.clone();
                    self.status = format!("Pre-cache {}/{}: {}", p.current + 1, p.total, p.message);
                }
                JobMsg::Item(true) => job.precache_ok += 1,
                JobMsg::Item(false) => {}
                JobMsg::Cancelled => {
                    cancelled = true;
                    done = true;
                }
                JobMsg::Done => {
                    self.log(format!(
                        "Pre-cache put attempted for {}/{} images",
                        job.precache_ok, job.total
                    ));
                    self.status =
                        format!("Pre-cache done: {}/{} cached", job.precache_ok, job.total);
                    done = true;
                }
            }
        }
        (done, cancelled)
    }

    fn poll_prepare_fast_job_rx(
        &mut self,
        job: &mut ActiveToolsJob,
        rx: &mpsc::Receiver<JobMsg<PathBuf>>,
    ) -> (bool, bool) {
        let mut done = false;
        let mut cancelled = false;
        while let Ok(msg) = rx.try_recv() {
            match msg {
                JobMsg::Progress(p) => {
                    job.current = p.current;
                    job.total = p.total;
                    job.message = p.message.clone();
                    self.status =
                        format!("Prepare Fast {}/{}: {}", p.current + 1, p.total, p.message);
                }
                JobMsg::Item(path) => job.prepare_paths.push(path),
                JobMsg::Cancelled => {
                    cancelled = true;
                    done = true;
                }
                JobMsg::Done => {
                    let paths = job.prepare_paths.clone();
                    let temp = job.prepare_temp.clone();
                    self.tools_finish_prepare_fast(paths, temp);
                    done = true;
                }
            }
        }
        (done, cancelled)
    }

    fn poll_tools_job(&mut self, ctx: &egui::Context) {
        let Some(mut job) = self.tools_job.take() else {
            return;
        };
        let (done, cancelled) = if let Some(rx) = job.rx_batch.take() {
            let outcome = self.poll_batch_job_rx(&mut job, &rx);
            job.rx_batch = Some(rx);
            outcome
        } else if let Some(rx) = job.rx_bool.take() {
            let outcome = self.poll_precache_job_rx(&mut job, &rx);
            job.rx_bool = Some(rx);
            outcome
        } else if let Some(rx) = job.rx_paths.take() {
            let outcome = self.poll_prepare_fast_job_rx(&mut job, &rx);
            job.rx_paths = Some(rx);
            outcome
        } else {
            (true, false)
        };

        if cancelled {
            self.log("Image tools job cancelled by user");
            if matches!(job.kind, ToolsJobKind::PrepareFast) {
                self.cleanup_prepare_fast_temp();
            }
        } else if !done {
            self.tools_job = Some(job);
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
        }
    }

    fn tools_request_batch(&mut self) {
        let policy = self.tools_output_policy();
        if matches!(policy, OutputPolicy::InPlaceWithBackup { .. }) {
            if self.tools_panel.batch_inplace_confirm < 2 {
                self.tools_panel.batch_inplace_confirm += 1;
                self.status = format!(
                    "In-place batch requires confirmation ({}/2) — confirm again",
                    self.tools_panel.batch_inplace_confirm
                );
                return;
            }
            self.tools_panel.batch_inplace_confirm = 0;
            self.log("WARNING: in-place batch will create .bak backups then overwrite originals");
        }
        self.tools_start_batch_job();
    }

    fn open_feh_on_prepared_fast(&mut self) {
        let Some(set) = self.prepared_fast.clone() else {
            self.status = "Run Prepare Fast first".into();
            return;
        };
        if set.materialized_paths.is_empty() {
            self.status = "No optimized files to launch".into();
            return;
        }
        let list_path = set
            .filelist_path
            .clone()
            .unwrap_or_else(feh_filelist_temp_path);
        if set.filelist_path.is_none() {
            if let Err(e) = write_feh_filelist(&set.materialized_paths) {
                self.status = format!("Failed to write feh filelist: {e}");
                return;
            }
        }
        let count = set.materialized_paths.len();
        let start = set.materialized_paths[0].as_path();
        self.spawn_feh_viewer(
            &list_path,
            start,
            format!("Spawning feh on {count} optimized images"),
            format!("Launched feh on {count} optimized images"),
        );
    }

    fn tools_refresh_rename_preview(&mut self) {
        let paths = self.tools_batch_paths();
        match expand_rename_pattern(&self.tools_panel.rename_pattern, &paths, 1) {
            Ok(p) => {
                self.tools_panel.rename_preview = p;
                self.tools_panel.rename_error = None;
            }
            Err(e) => {
                self.tools_panel.rename_preview.clear();
                self.tools_panel.rename_error = Some(e);
            }
        }
    }

    fn tools_apply_rename(&mut self) {
        let pairs = self.tools_panel.rename_preview.clone();
        let outcome = apply_rename_pairs(&pairs);
        if let Some(e) = &outcome.error {
            self.log(format!(
                "Rename failed: {e}{}",
                if outcome.rolled_back {
                    " (rolled back)"
                } else {
                    ""
                }
            ));
            self.status = format!("Rename failed: {e}");
        } else {
            for (old, dest) in &outcome.applied {
                if let Some(e) = self.images.iter_mut().find(|e| e.path == *old) {
                    e.path = dest.clone();
                }
                self.log(format!("Renamed {} -> {}", old.display(), dest.display()));
            }
            // cache invariant: bump on every self.images mutation
            self.images_revision = self.images_revision.wrapping_add(1);
            self.status = format!("Renamed {} files", outcome.applied.len());
        }
        self.tools_panel.rename_confirm_open = false;
    }

    fn render_tools_resize_controls(&mut self, ui: &mut egui::Ui) {
        ui.checkbox(&mut self.tools_panel.use_percent, "Use percent");
        if self.tools_panel.use_percent {
            ui.text_edit_singleline(&mut self.tools_panel.resize_percent);
        } else {
            ui.horizontal(|ui| {
                ui.label("W");
                ui.text_edit_singleline(&mut self.tools_panel.resize_width);
                ui.label("H");
                ui.text_edit_singleline(&mut self.tools_panel.resize_height);
            });
        }
        egui::ComboBox::from_id_salt("fit_mode")
            .selected_text(format!("{:?}", self.tools_panel.fit))
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut self.tools_panel.fit, FitMode::Contain, "Contain");
                ui.selectable_value(&mut self.tools_panel.fit, FitMode::Cover, "Cover");
                ui.selectable_value(&mut self.tools_panel.fit, FitMode::Stretch, "Stretch");
            });
        egui::ComboBox::from_id_salt("resize_filter")
            .selected_text(filter_label(self.tools_panel.filter))
            .show_ui(ui, |ui| {
                for (filter, label) in [
                    (Filter::Lanczos3, "Lanczos"),
                    (Filter::Nearest, "Nearest"),
                    (Filter::Triangle, "Triangle"),
                    (Filter::CatmullRom, "CatmullRom"),
                    (Filter::Gaussian, "Gaussian"),
                ] {
                    ui.selectable_value(&mut self.tools_panel.filter, filter, label);
                }
            });
    }

    fn render_tools_crop_preview(
        &mut self,
        ui: &mut egui::Ui,
        ctx: &egui::Context,
        pctx: &PanelContext,
    ) {
        ui.label("Geometry WxH+X+Y:");
        if ui
            .text_edit_singleline(&mut self.tools_panel.crop_geometry)
            .changed()
        {
            self.tools_panel.last_crop_key.clear();
        }
        let Some(sel) = pctx.image.as_deref() else {
            return;
        };
        let key = format!("{}:{}", sel.display(), self.tools_panel.crop_geometry);
        if self.tools_panel.last_crop_key != key {
            if let Ok(px) = crop_preview_pixels(sel, &self.tools_panel.crop_geometry, 256) {
                let img = egui::ColorImage::from_rgba_unmultiplied(
                    [px.width as usize, px.height as usize],
                    &px.rgba,
                );
                self.tools_panel.crop_texture =
                    Some(ctx.load_texture("crop_preview", img, egui::TextureOptions::LINEAR));
                self.tools_panel.last_crop_key = key;
            }
        }
        if let Some(tex) = &self.tools_panel.crop_texture {
            ui.image(tex);
        }
    }

    fn render_tools_output_policy(&mut self, ui: &mut egui::Ui) {
        ui.add(egui::Slider::new(&mut self.tools_panel.quality, 1..=100).text("Quality"));
        ui.horizontal(|ui| {
            ui.selectable_value(
                &mut self.tools_panel.policy_ui,
                ToolsPolicyUi::Subfolder,
                "Subfolder",
            );
            ui.selectable_value(
                &mut self.tools_panel.policy_ui,
                ToolsPolicyUi::Suffixed,
                "Suffix",
            );
            ui.selectable_value(
                &mut self.tools_panel.policy_ui,
                ToolsPolicyUi::InPlace,
                "In-place",
            );
        });
        match self.tools_panel.policy_ui {
            ToolsPolicyUi::Subfolder => {
                ui.text_edit_singleline(&mut self.tools_panel.subfolder_name);
            }
            ToolsPolicyUi::Suffixed => {
                ui.text_edit_singleline(&mut self.tools_panel.suffix);
            }
            ToolsPolicyUi::InPlace => {}
        }
    }

    fn render_tools_single_batch_section(
        &mut self,
        ui: &mut egui::Ui,
        ctx: &egui::Context,
        pctx: &PanelContext,
    ) {
        ui.horizontal(|ui| {
            ui.selectable_value(
                &mut self.tools_panel.single_op,
                ToolsSingleOp::Resize,
                "Resize",
            );
            ui.selectable_value(&mut self.tools_panel.single_op, ToolsSingleOp::Crop, "Crop");
            ui.selectable_value(
                &mut self.tools_panel.single_op,
                ToolsSingleOp::Convert,
                "Convert",
            );
        });
        match self.tools_panel.single_op {
            ToolsSingleOp::Resize => self.render_tools_resize_controls(ui),
            ToolsSingleOp::Crop => self.render_tools_crop_preview(ui, ctx, pctx),
            ToolsSingleOp::Convert => {
                ui.text_edit_singleline(&mut self.tools_panel.convert_format);
            }
        }
        self.render_tools_output_policy(ui);
    }

    fn render_tools_rename_section(&mut self, ui: &mut egui::Ui) {
        if ui
            .text_edit_singleline(&mut self.tools_panel.rename_pattern)
            .changed()
        {
            self.tools_refresh_rename_preview();
        }
        if let Some(e) = &self.tools_panel.rename_error {
            ui.colored_label(egui::Color32::RED, e);
        }
        egui::ScrollArea::vertical()
            .max_height(120.0)
            .show(ui, |ui| {
                for (old, new) in &self.tools_panel.rename_preview {
                    ui.label(format!(
                        "{} → {}",
                        old.file_name().unwrap_or_default().to_string_lossy(),
                        new
                    ));
                }
            });
    }

    fn render_tools_cache_path_pickers(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Root:");
            let root_label = self
                .cache_config
                .root
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| "(default)".into());
            ui.label(egui::RichText::new(root_label).small());
            if ui.button("Pick root…").clicked() {
                if let Some(dir) = rfd::FileDialog::new().pick_folder() {
                    self.cache_config.root = Some(dir);
                }
            }
        });
        ui.horizontal(|ui| {
            ui.label("Passkey:");
            let pk_label = self
                .cache_config
                .passkey_path
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| "(none)".into());
            ui.label(egui::RichText::new(pk_label).small());
            if ui.button("Pick passkey…").clicked() {
                if let Some(file) = rfd::FileDialog::new().pick_file() {
                    self.cache_config.passkey_path = Some(file);
                }
            }
        });
    }

    fn render_tools_cache_config(&mut self, ui: &mut egui::Ui) {
        ui.checkbox(&mut self.cache_config.enabled, "Enable cache");
        self.render_tools_cache_path_pickers(ui);
        ui.horizontal(|ui| {
            ui.label("TTL:");
            ui.text_edit_singleline(&mut self.cache_config.default_ttl);
        });
        let magick_hint = if self.tool_caps.magick_cache_available {
            if self.tool_caps.magick_cache_ready {
                "magick-cache: ready"
            } else {
                "magick-cache: install/setup needed"
            }
        } else {
            "magick-cache not on PATH — see README"
        };
        ui.small(magick_hint);
        if ui.button("Apply cache settings").clicked() {
            self.image_tools.update_cache(self.cache_config.clone());
            self.log("Cache settings applied (in-memory session)");
        }
    }

    fn render_tools_cache_jobs(&mut self, ui: &mut egui::Ui) {
        let paths = self.tools_batch_paths();
        let job_busy = self.tools_job_active();
        if ui
            .add_enabled(
                !paths.is_empty() && !job_busy,
                egui::Button::new("Pre-cache folder"),
            )
            .clicked()
        {
            self.tools_start_precache_job();
        }
        if ui
            .add_enabled(
                !paths.is_empty() && !job_busy,
                egui::Button::new("Prepare Fast feh"),
            )
            .clicked()
        {
            self.tools_start_prepare_fast_job();
        }
        if let Some(set) = &self.prepared_fast {
            ui.small(format!(
                "{} optimized files ready",
                set.materialized_paths.len()
            ));
            if ui
                .add_enabled(
                    self.feh_available,
                    egui::Button::new("Launch feh on optimized"),
                )
                .clicked()
            {
                self.open_feh_on_prepared_fast();
            }
        }
    }

    fn render_tools_cache_section(&mut self, ui: &mut egui::Ui) {
        self.render_tools_cache_config(ui);
        self.render_tools_cache_jobs(ui);
    }

    fn render_tools_job_status(&mut self, ui: &mut egui::Ui) {
        if let Some(job) = &self.tools_job {
            ui.separator();
            ui.label(format!(
                "Job: {}/{} — {}",
                job.current + 1,
                job.total.max(1),
                job.message
            ));
            if ui.button("Cancel job").clicked() {
                self.cancel_tools_job();
            }
        }
        if let Some(summary) = &self.tools_panel.batch_summary {
            ui.colored_label(egui::Color32::LIGHT_GREEN, summary);
            if ui.button("Dismiss summary").clicked() {
                self.tools_panel.batch_summary = None;
            }
        }
    }

    fn render_tools_batch_actions(&mut self, ui: &mut egui::Ui) {
        let n = self.tools_batch_paths().len();
        ui.label(format!("Batch targets: {n} filtered images"));
        if self.tools_panel.policy_ui == ToolsPolicyUi::InPlace {
            ui.colored_label(
                egui::Color32::YELLOW,
                "In-place batch creates .bak backups then overwrites originals",
            );
        }
        if ui
            .add_enabled(!self.tools_job_active(), egui::Button::new("Run batch…"))
            .clicked()
        {
            self.tools_panel.batch_confirm_open = true;
        }
        if self.tools_panel.batch_confirm_open {
            ui.label(format!("Process {n} images with current settings?"));
            if ui.button("Confirm batch").clicked() {
                self.tools_request_batch();
            }
            if ui.button("Cancel").clicked() {
                self.tools_panel.batch_confirm_open = false;
                self.tools_panel.batch_inplace_confirm = 0;
            }
        }
    }

    fn render_tools_rename_actions(&mut self, ui: &mut egui::Ui) {
        if ui.button("Refresh preview").clicked() {
            self.tools_refresh_rename_preview();
        }
        let ok =
            self.tools_panel.rename_error.is_none() && !self.tools_panel.rename_preview.is_empty();
        if ui
            .add_enabled(ok, egui::Button::new("Apply rename…"))
            .clicked()
        {
            self.tools_panel.rename_confirm_open = true;
        }
        if self.tools_panel.rename_confirm_open && ok && ui.button("Confirm rename").clicked() {
            self.tools_apply_rename();
        }
    }

    fn render_tools_action_buttons(
        &mut self,
        ui: &mut egui::Ui,
        pctx: &PanelContext,
        section: ToolsSection,
    ) {
        ui.separator();
        match section {
            ToolsSection::Single => {
                if ui
                    .add_enabled(pctx.image.is_some(), egui::Button::new("Apply"))
                    .clicked()
                {
                    if let Some(sel) = pctx.image.as_deref() {
                        self.tools_apply_single(sel);
                    }
                }
            }
            ToolsSection::Batch => self.render_tools_batch_actions(ui),
            ToolsSection::Rename => self.render_tools_rename_actions(ui),
            ToolsSection::Cache => {}
        }
    }

    fn render_inspector_image_tools(
        &mut self,
        ui: &mut egui::Ui,
        ctx: &egui::Context,
        pctx: &PanelContext,
    ) {
        ui.vertical(|ui| {
            ui.label(egui::RichText::new("Image Tools").strong());
            // 018 FIX-5: Batch / Rename / Cache act on the LIVE folder's filtered
            // list (via compute_list_indices), NOT the pinned image, so a pinned
            // detached window must not expose them (SC-006: pinned actions target
            // the pinned image). Hide those tabs when pinned and force the Single
            // tab, whose ops act on `pctx.image` (the pin) directly.
            let section = if pctx.pinned {
                ui.small("Folder-scoped tools follow the main window — unpin to use here.");
                ui.separator();
                ToolsSection::Single
            } else {
                ui.horizontal(|ui| {
                    ui.selectable_value(
                        &mut self.tools_panel.section,
                        ToolsSection::Single,
                        "Single",
                    );
                    ui.selectable_value(
                        &mut self.tools_panel.section,
                        ToolsSection::Batch,
                        "Batch",
                    );
                    ui.selectable_value(
                        &mut self.tools_panel.section,
                        ToolsSection::Rename,
                        "Rename",
                    );
                    ui.selectable_value(
                        &mut self.tools_panel.section,
                        ToolsSection::Cache,
                        "Cache",
                    );
                });
                ui.separator();
                self.tools_panel.section
            };
            match section {
                ToolsSection::Single | ToolsSection::Batch => {
                    self.render_tools_single_batch_section(ui, ctx, pctx);
                }
                ToolsSection::Rename => self.render_tools_rename_section(ui),
                ToolsSection::Cache => self.render_tools_cache_section(ui),
            }
            self.render_tools_job_status(ui);
            self.render_tools_action_buttons(ui, pctx, section);
        });
    }

    fn render_browse_controls_body(&mut self, ui: &mut egui::Ui) {
        let has_folder = self.current_dir.is_some();

        ui.vertical(|ui| {
            if ui.button("Choose folder").clicked() {
                self.pick_folder();
            }

            if let Some(dir) = &self.current_dir {
                let dir_str = dir.display().to_string();
                ui.add(
                    egui::Label::new(dir_str.clone())
                        .selectable(true)
                        .wrap_mode(egui::TextWrapMode::Truncate),
                )
                .on_hover_text(dir_str);
            } else {
                ui.small("No folder loaded");
            }

            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.label("Filter:");
                ui.add_enabled_ui(has_folder, |ui| {
                    let field_w = (ui.available_width() - 4.0).clamp(100.0, 200.0);
                    ui.add(
                        egui::TextEdit::singleline(&mut self.search)
                            .hint_text("Type something to filter")
                            .desired_width(field_w),
                    );
                });
            });

            ui.add_space(4.0);
            ui.horizontal(|ui| {
                let mut recursive_changed = false;
                ui.add_enabled_ui(has_folder, |ui| {
                    recursive_changed =
                        ui.checkbox(&mut self.recursive, "Include subfolders").changed();
                });
                if recursive_changed {
                    self.rescan_current_folder_if_any();
                }
            });

            ui.horizontal(|ui| {
                let mut deep_changed = false;
                ui.add_enabled_ui(has_folder, |ui| {
                    deep_changed = ui
                        .checkbox(
                            &mut self.deep_scan_magick,
                            "Detect exotic formats (slow)",
                        )
                        .on_hover_text(
                            "Runs ImageMagick identify on unknown extensions. Off by default for fast feh launch.",
                        )
                        .changed();
                });
                if deep_changed {
                    self.rescan_current_folder_if_any();
                }

                if ui.add_enabled(has_folder, egui::Button::new("Rescan")).clicked() {
                    self.rescan_current_folder_if_any();
                }
            });

            ui.horizontal(|ui| {
                ui.label("Sort:");
                ui.add_enabled_ui(has_folder, |ui| {
                    egui::ComboBox::from_id_salt("sort_mode")
                        .selected_text(sort_mode_label(self.sort_mode))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.sort_mode, SortMode::Path, "Path");
                            ui.selectable_value(&mut self.sort_mode, SortMode::Name, "Name");
                            ui.selectable_value(&mut self.sort_mode, SortMode::Folder, "Folder");
                        });
                });
            });

        });
    }

    fn render_inspector_panel(&mut self, ui: &mut egui::Ui, ctx: &egui::Context, time: f64) {
        // Auto-expand is EDGE-TRIGGERED, not per-frame (018 FIX-1, FR-003): the
        // rising/falling edges are wired at their event sites — scan start
        // (`scan_directory`), scan complete (`apply_scan_result`), the
        // no-folder ↔ folder-loaded transition (`sync_auto_expand_folder_edge`,
        // called from `update`), and tool-missing detection
        // (`mark_feh_unavailable`/`refresh_tool_caps`). Nothing to do here.

        // Stable base for the Zone D height clamp: the full inspector content
        // height, captured once before any zone consumes vertical space.
        let full_h = ui.available_height();

        let (total, filtered) = self.compute_list_indices();
        let shown = filtered.len();
        let list_root = self.current_dir.clone();

        // Zone A: persistent nav strip (Up + Flat/Tree + spinner + breadcrumb).
        self.render_inspector_nav_strip(ui);
        // Zone B: subfolder drill-down (hidden when there are no subfolders).
        self.render_inspector_subfolder_drilldown(ui);
        // Zone C (PRIMARY): the flat/tree image list, virtualized via show_rows.
        self.render_inspector_file_list(ui, &filtered, list_root.as_deref(), full_h);

        // Zone D: collapsed-by-default meta-drawer gating the 7 detail sections.
        ui.horizontal(|ui| {
            let toggle_label = if self.inspector_drawer_collapsed {
                "▶ Details"
            } else {
                "▼ Details"
            };
            if ui.small_button(toggle_label).clicked() {
                self.inspector_drawer_collapsed = !self.inspector_drawer_collapsed;
                // The user now owns the drawer's open/closed state; a later
                // retraction must not move it (018 FIX-1 / FR-003).
                self.auto_expand.note_user_drawer_toggle();
            }
            ui.add(
                egui::Label::new(
                    egui::RichText::new("Browse · actions · session · log · deps · formats").weak(),
                )
                .wrap_mode(egui::TextWrapMode::Truncate),
            );
        });
        if self.inspector_drawer_collapsed {
            return;
        }
        let drawer_body_h = Self::sections_drawer_body_height(full_h);
        egui::ScrollArea::vertical()
            .id_salt("inspector_sections_drawer")
            .max_height(drawer_body_h)
            .show(ui, |ui| {
                self.render_inspector_sections_drawer_body(ui, ctx, shown, total, time);
            });
    }

    /// Zone D body (018 Batch 2): the 7 detail `CollapsingHeader`s, unchanged
    /// from Batch 1 except for being wrapped in the bounded drawer ScrollArea
    /// above instead of rendering inline in the (now-removed) infinite outer
    /// ScrollArea.
    fn render_inspector_sections_drawer_body(
        &mut self,
        ui: &mut egui::Ui,
        ctx: &egui::Context,
        shown: usize,
        total: usize,
        time: f64,
    ) {
        let browse_header = self.browse_header_label();
        let browse_response = egui::CollapsingHeader::new(browse_header)
            .id_salt("inspector_browse")
            .open(Some(
                self.inspector_open.contains(&InspectorSection::Browse),
            ))
            .show(ui, |ui| {
                self.render_inspector_browse(ui);
            });
        if browse_response.header_response.clicked() {
            self.toggle_inspector_section(InspectorSection::Browse);
        }

        let actions_header = self.image_actions_header_label();
        let actions_response = egui::CollapsingHeader::new(actions_header)
            .id_salt("inspector_image_actions")
            .open(Some(
                self.inspector_open
                    .contains(&InspectorSection::ImageActions),
            ))
            .show(ui, |ui| {
                self.render_inspector_image_actions(ui, ctx);
            });
        if actions_response.header_response.clicked() {
            self.toggle_inspector_section(InspectorSection::ImageActions);
        }

        let feh_instances_header = self.feh_instances_header_label();
        let feh_instances_response = egui::CollapsingHeader::new(feh_instances_header)
            .id_salt("inspector_feh_instances")
            .open(Some(
                self.inspector_open
                    .contains(&InspectorSection::FehInstances),
            ))
            .show(ui, |ui| {
                self.render_inspector_feh_instances(ui);
            });
        if feh_instances_response.header_response.clicked() {
            self.toggle_inspector_section(InspectorSection::FehInstances);
        }

        let pulse_fill = if self.scanning {
            Self::activity_pulse_color(time, true)
        } else {
            egui::Color32::TRANSPARENT
        };
        let pulse_stroke = if self.scanning {
            let pulse = ((time * 5.0).sin() * 0.5 + 0.5) as f32;
            egui::Stroke::new(
                1.5,
                egui::Color32::from_rgb(
                    (80.0 + 100.0 * pulse) as u8,
                    (160.0 + 60.0 * pulse) as u8,
                    255,
                ),
            )
        } else {
            egui::Stroke::NONE
        };
        let status_header = self.session_status_header_rich(shown, total, time);
        let status_response = egui::CollapsingHeader::new(status_header)
            .id_salt("inspector_session_status")
            .open(Some(
                self.inspector_open
                    .contains(&InspectorSection::SessionStatus),
            ))
            .show(ui, |ui| {
                if self.scanning && !self.detached.contains_key(&InspectorSection::SessionStatus) {
                    egui::Frame::none()
                        .fill(pulse_fill)
                        .stroke(pulse_stroke)
                        .inner_margin(egui::Margin::symmetric(2.0, 1.0))
                        .show(ui, |ui| {
                            self.render_inspector_session_status(ui, ctx, shown, total, time);
                        });
                } else {
                    self.render_inspector_session_status(ui, ctx, shown, total, time);
                }
            });
        if status_response.header_response.clicked() {
            self.toggle_inspector_section(InspectorSection::SessionStatus);
        }

        let log_header = self.activity_log_header_label();
        let log_response = egui::CollapsingHeader::new(log_header)
            .id_salt("inspector_activity_log")
            .open(Some(
                self.inspector_open.contains(&InspectorSection::ActivityLog),
            ))
            .show(ui, |ui| {
                self.render_inspector_activity_log(ui, ctx);
            });
        if log_response.header_response.clicked() {
            self.toggle_inspector_section(InspectorSection::ActivityLog);
        }

        let deps_header = self.deps_header_label();
        let deps_response = egui::CollapsingHeader::new(deps_header)
            .id_salt("tool_deps")
            .open(Some(
                self.inspector_open
                    .contains(&InspectorSection::Dependencies),
            ))
            .show(ui, |ui| {
                self.render_inspector_dependencies(ui, ctx);
            });
        if deps_response.header_response.clicked() {
            self.toggle_inspector_section(InspectorSection::Dependencies);
        }

        let fd_header = self.format_discovery_header_label();
        let fd_response = egui::CollapsingHeader::new(fd_header)
            .id_salt("tool_format_discovery")
            .open(Some(
                self.inspector_open
                    .contains(&InspectorSection::FormatDiscovery),
            ))
            .show(ui, |ui| {
                self.render_inspector_format_discovery(ui);
            });
        if fd_response.header_response.clicked() {
            self.toggle_inspector_section(InspectorSection::FormatDiscovery);
        }

        if self.tool_caps.has_missing_required() {
            ui.separator();
            ui.colored_label(
                egui::Color32::from_rgb(220, 80, 80),
                "Install required tools above, then click Recheck.",
            );
        }
    }

    fn status_text_for_copy(&self) -> String {
        if self.scanning {
            let network = self
                .current_dir
                .as_ref()
                .is_some_and(|p| is_network_mount_path(p));
            if network {
                "Scanning… (network folder — UI stays responsive)".to_string()
            } else {
                "Scanning…".to_string()
            }
        } else {
            self.status.clone()
        }
    }

    /// Collapsed inspector headers stay compact (count only); full status lives in the body.
    fn session_status_header_rich(&self, shown: usize, total: usize, time: f64) -> egui::RichText {
        let count = showing_count_label(shown, total);
        let mut label = if self.scanning {
            let dots = [".", "..", "..."][(time as usize / 2) % 3];
            format!("● Session status — {count} · Scanning{dots}")
        } else {
            format!("Session status — {count}")
        };

        if self.detached.contains_key(&InspectorSection::SessionStatus) {
            label = Self::header_with_detach_suffix(label, true);
        }

        let mut rich = egui::RichText::new(label);
        if self.scanning && !self.detached.contains_key(&InspectorSection::SessionStatus) {
            let pulse = ((time * 5.0).sin() * 0.5 + 0.5) as f32;
            rich = rich.color(egui::Color32::from_rgb(
                (120.0 + 80.0 * pulse) as u8,
                (170.0 + 60.0 * pulse) as u8,
                255,
            ));
        }
        rich
    }

    fn inspector_max_width(ctx: &egui::Context) -> f32 {
        let viewport_w = ctx.input(|i| i.viewport().inner_rect.map(|r| r.width()).unwrap_or(720.0));
        // Inspector must not exceed the central image-list panel (each gets at least half).
        (viewport_w * 0.5).max(260.0)
    }

    /// Auto-sized width (px) for the right-hand Inspector SidePanel
    /// (feature 017, Phase 5).
    ///
    /// The panel is non-resizable and sized to the widest *static* label so its
    /// width never jitters as dynamic text (folder names, file names, live
    /// counts, scan status, selection) changes — a long SMB path must never be
    /// able to peg it wide. Only compile-time string literals are measured (the
    /// `const STATIC_LABELS: &[&str]` type makes it impossible to add runtime
    /// state here). Only the *measured text width* is cached, keyed on
    /// `pixels_per_point` — the sole input to text measurement in this app (it
    /// never mutates fonts, text styles, theme, or zoom; egui's Ctrl +/- zoom is
    /// also covered because it flows through `pixels_per_point`). The clamp to
    /// the live half-viewport is recomputed EVERY call (018 F3) so the width
    /// tracks window resizes instead of going stale at whatever half-viewport
    /// happened to be in effect the last time `pixels_per_point` changed.
    fn inspector_width(&mut self, ctx: &egui::Context) -> f32 {
        // ONLY literal, hardcoded strings that never change at runtime. NEVER a
        // `self.*` value or `format!()` output. For headers whose live text is
        // dynamic, a static identity/fallback template is measured instead of the
        // live label; the dynamic tail (counts, names, paths) truncates, it does
        // not size the panel.
        const STATIC_LABELS: &[&str] = &[
            // Section-header identity / static fallback templates.
            "Browse — No folder loaded",
            "Image actions — no selection",
            "Feh instances",
            "Session status",
            "Activity log",
            "✅ Dependencies — all required tools OK",
            "Format discovery",
            // Button / checkbox captions. All below the 440px floor today, so
            // they never change the result; kept for completeness + robustness.
            // Being constants, they can never introduce jitter.
            "Choose folder",
            "Rescan",
            "Include subfolders",
            "Detect exotic formats (slow)",
            "Recheck tools on PATH",
            "Open in feh",
            "Copy status",
            "Detach window",
            // Zone A nav strip (018 Batch 2): Up button + Flat/Tree view toggle.
            "⬆ Up",
            "Flat list",
            "Folder tree",
            // Zone D drawer meta-toggle + its static description line (018
            // Batch 2) — the description is the widest static label in the
            // panel (measured ~252px incl. Button-style font metrics), still
            // well under the 440px floor's ~392px usable text budget.
            "▶ Details",
            "▼ Details",
            "Browse · actions · session · log · deps · formats",
            // Pin-to-current-image toggle in the detached Image-actions window
            // (018 Batch 5). Rendered in a separate egui::Window, not this
            // SidePanel, but included for completeness/robustness per the
            // doctrine above — cannot introduce jitter, being a constant.
            "Pin to current image",
            "Unpin (follow selection)",
            // Zone C flat-list column headers (018 Batch 2).
            "Folder",
            "Filename",
            "Status",
            // NOTE (018 FIX-11): the seven detach-toolbar descriptions and the
            // "Install required tools above, then click Recheck." line are
            // panel-rendered but intentionally NOT listed here. They render at
            // `ui.small()` (smaller than the Button metric measured above) and
            // their widest (~285px) sits well under the 440px floor, so they can
            // never size the panel. This list is therefore ADVISORY below the
            // floor: it does not need to enumerate every sub-floor caption.
        ];

        // Fixed horizontal chrome added around the widest label so it is never
        // clipped:
        //   SidePanel frame inner margin (8 + 8; exact_width is "incl. margins") = 16
        //   CollapsingHeader indent gutter (spacing.indent = 18)                 = 18
        //   CollapsingHeader trailing button_padding.x (= 4)                     =  4
        //   inner list/drawer ScrollArea bar allowance (scroll bar_width = 10)   = 10
        // (018 FIX-11: post-Batch-2 the infinite OUTER panel ScrollArea is gone;
        // the 10px now covers the Zone C list / Zone D drawer inner scrollbars.)
        const PANEL_CHROME: f32 = 48.0;

        let ppp = ctx.pixels_per_point();
        let max_text = match self.inspector_width_cache {
            Some((cached_ppp, cached_max_text)) if cached_ppp == ppp => cached_max_text,
            _ => {
                // CollapsingHeader labels render at TextStyle::Button; Body == Button
                // size in egui defaults, so one FontId measures headers, the intro
                // label, and button captions correctly.
                let font_id = egui::TextStyle::Button.resolve(&ctx.style());
                let measured = ctx.fonts(|f| {
                    STATIC_LABELS
                        .iter()
                        .map(|s| {
                            f.layout_no_wrap((*s).to_owned(), font_id.clone(), egui::Color32::WHITE)
                                .size()
                                .x
                        })
                        .fold(0.0_f32, f32::max)
                });
                self.inspector_width_cache = Some((ppp, measured));
                measured
            }
        };

        // Clamp to [440, half-viewport] EVERY call (not cached) so width tracks
        // live window resizes. The half-viewport cap is a HARD upper bound; when
        // the window is so narrow the cap falls below 440, the cap wins — and
        // floor == upper here avoids an f32::clamp(min > max) panic.
        let upper = Self::inspector_max_width(ctx);
        let floor = 440.0_f32.min(upper);
        (max_text + PANEL_CHROME).clamp(floor, upper)
    }

    fn render_session_status_body(
        &mut self,
        ui: &mut egui::Ui,
        ctx: &egui::Context,
        shown: usize,
        total: usize,
        time: f64,
    ) {
        ui.vertical(|ui| {
            egui::Frame::none()
                .inner_margin(egui::Margin::symmetric(4.0, 2.0))
                .stroke(egui::Stroke::new(
                    1.0,
                    ui.style().visuals.widgets.noninteractive.bg_stroke.color,
                ))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        if ui.button("Copy status").clicked() {
                            ctx.copy_text(self.status_text_for_copy());
                        }
                    });
                });

            ui.add_space(6.0);

            let body_h = 120.0;
            egui::Frame::group(ui.style())
                .inner_margin(6.0)
                .show(ui, |ui| {
                    ui.set_min_height(body_h);
                    ui.label(showing_count_label(shown, total));
                    if self.scanning {
                        ui.small(self.status_text_for_copy());
                    } else {
                        ui.add(
                            egui::Label::new(&self.status)
                                .selectable(true)
                                .wrap_mode(egui::TextWrapMode::Truncate),
                        )
                        .on_hover_text(self.status.clone());
                    }

                    ui.add_space(8.0);
                    ui.separator();
                    ui.small("Speed / timing (rotates every 4s)");
                    let (spinner, tip) = self.rotating_operation_tip(time);
                    ui.horizontal(|ui| {
                        if !tip.is_empty() {
                            ui.monospace(spinner.to_string());
                        }
                        ui.add(
                            egui::Label::new(tip.clone())
                                .selectable(true)
                                .wrap_mode(egui::TextWrapMode::Truncate),
                        )
                        .on_hover_text(tip);
                    });
                });

            self.render_scan_inventory_banner(ui, self.current_dir.as_deref());
        });
    }

    fn render_inspector_session_status(
        &mut self,
        ui: &mut egui::Ui,
        ctx: &egui::Context,
        shown: usize,
        total: usize,
        time: f64,
    ) {
        if self.detached.contains_key(&InspectorSection::SessionStatus) {
            Self::render_detached_placeholder(ui, "Session status");
            return;
        }

        if Self::render_segment_detach_toolbar(
            ui,
            "Image count, current status, operation speed tips",
            "Detach window",
        ) {
            self.detached
                .insert(InspectorSection::SessionStatus, DetachedWindow::default());
        }
        self.render_session_status_body(ui, ctx, shown, total, time);
    }

    /// Window title + default width for a detached inspector section. Pure
    /// lookup, no `self` borrow, so it can be called freely from inside the
    /// `render_detached_inspector_windows` loop without fighting the borrow
    /// checker over the closure's `&mut self` capture.
    fn detached_window_chrome(section: InspectorSection) -> (&'static str, f32) {
        match section {
            InspectorSection::Browse => ("Browse", 520.0),
            InspectorSection::ImageActions => ("Image actions", 360.0),
            InspectorSection::FehInstances => ("Feh instances", 420.0),
            InspectorSection::SessionStatus => ("Session status", 480.0),
            InspectorSection::ActivityLog => ("Activity log", 520.0),
            InspectorSection::Dependencies => ("Dependencies", 420.0),
            InspectorSection::FormatDiscovery => ("Format discovery", 480.0),
        }
    }

    /// Detached (floating) inspector windows: one per `InspectorSection`
    /// present in `self.detached`. Iterates `InspectorSection::ALL` (NEVER
    /// the `HashMap`, whose iteration order is unspecified) so window
    /// spawn/z-order is deterministic frame to frame. Body dispatch is a
    /// `match` (not fn pointers — a `[fn(&mut Self, ...); 7]` table can't
    /// paper over the differing per-section arg lists — e.g. session status
    /// needs `shown`/`total`/`time` — without a wrapper closure per entry
    /// anyway, so a direct `match` is both simpler and avoids `&mut self`
    /// fn-pointer variance pain).
    fn render_detached_inspector_windows(&mut self, ctx: &egui::Context) {
        let time = ctx.input(|i| i.time);
        let (total, filtered) = self.compute_list_indices();
        let shown = filtered.len();

        for section in InspectorSection::ALL {
            if !self.detached.contains_key(&section) {
                continue;
            }
            let (chrome_title, default_width) = Self::detached_window_chrome(section);
            let title: String = if section == InspectorSection::ImageActions {
                match self.detached.get(&section).and_then(|w| w.pin.as_ref()) {
                    Some(PanelPin::Image(path)) => {
                        format!("{chrome_title} — 📌 {}", file_name_display(path))
                    }
                    None => chrome_title.to_string(),
                }
            } else {
                chrome_title.to_string()
            };
            let mut open = true;
            egui::Window::new(title)
                .id(egui::Id::new("detached_window").with(section))
                .open(&mut open)
                .collapsible(true)
                .resizable(true)
                .default_width(default_width)
                .show(ctx, |ui| match section {
                    InspectorSection::Browse => self.render_browse_controls_body(ui),
                    InspectorSection::ImageActions => {
                        self.render_image_actions_pin_toggle(ui);
                        let pin = self.detached.get(&section).and_then(|w| w.pin.as_ref());
                        let pctx = self.panel_context(pin);
                        // 018 FIX-3/FIX-4: in-memory membership (not a per-frame
                        // Path::exists() stat) is the pin-staleness signal. A pinned
                        // path absent from the live filtered list (moved/deleted, or
                        // a different folder is loaded) disables the actions with an
                        // inline hint IN this window, so the pinned "Open in feh"
                        // can no longer silently no-op against the main window.
                        let stale_pin = match (pctx.pinned, pctx.image.as_deref()) {
                            (true, Some(p)) => !self.pinned_path_in_filtered_list(p),
                            _ => false,
                        };
                        if stale_pin {
                            ui.colored_label(
                                egui::Color32::RED,
                                format!(
                                    "Pinned image is not in the current folder/filter: {}. Unpin to follow the live selection, or load its folder.",
                                    pctx.image.as_deref().map(file_name_display).unwrap_or_default()
                                ),
                            );
                        } else {
                            self.render_image_actions_full(ui, ctx, &pctx);
                        }
                    }
                    InspectorSection::FehInstances => self.render_feh_instances_body(ui),
                    InspectorSection::SessionStatus => {
                        let pulse_fill = if self.scanning {
                            Self::activity_pulse_color(time, true)
                        } else {
                            egui::Color32::TRANSPARENT
                        };
                        egui::Frame::none().fill(pulse_fill).show(ui, |ui| {
                            self.render_session_status_body(ui, ctx, shown, total, time);
                        });
                    }
                    InspectorSection::ActivityLog => self.render_activity_log_body(ui, ctx),
                    InspectorSection::Dependencies => self.render_deps_section_body(ui, ctx),
                    InspectorSection::FormatDiscovery => self.render_format_discovery_body(ui),
                });
            if !open {
                self.detached.remove(&section);
            }
        }
    }

    fn emit_startup_notice_once() {
        STARTUP_NOTICE.call_once(|| {
            eprintln!(
                "[rust-feh] App started. Use 'Choose folder' to load images. Debug logs appear in the UI after actions."
            );
        });
    }

    fn sync_frame_input_state(&mut self, ctx: &egui::Context) {
        if self.search != self.prior_search {
            self.prior_search = self.search.clone();
            self.scroll_generation = self.scroll_generation.wrapping_add(1);
            self.sync_selection_to_filter();
        }
        if self.sort_mode != self.prior_sort_mode {
            self.prior_sort_mode = self.sort_mode;
            self.scroll_generation = self.scroll_generation.wrapping_add(1);
            self.sync_selection_to_filter();
        }
        if self.window_size != self.prior_window_size {
            self.prior_window_size = self.window_size;
            self.apply_window_preset(ctx);
            self.persist_window_prefs();
            self.log(format!(
                "Window size set to {}",
                window_preset_label(self.window_size)
            ));
        }
        if self.window_resizable != self.prior_window_resizable {
            self.prior_window_resizable = self.window_resizable;
            let lock_size = self.current_viewport_size(ctx);
            self.apply_window_resize_policy(ctx, lock_size);
            self.persist_window_prefs();
            self.log(if self.window_resizable {
                "Window resizing enabled".to_string()
            } else {
                format!(
                    "Window size locked at {} × {}",
                    lock_size.x.round() as i32,
                    lock_size.y.round() as i32
                )
            });
        }
    }

    fn rescan_current_folder_if_any(&mut self) {
        if let Some(d) = self.current_dir.clone() {
            self.scan_directory(&d);
        }
    }

    fn render_view_menu(&mut self, ui: &mut egui::Ui) {
        ui.menu_button("Window size", |ui| {
            for preset in [
                WindowSizePreset::Compact,
                WindowSizePreset::Default,
                WindowSizePreset::Large,
            ] {
                if ui
                    .selectable_value(&mut self.window_size, preset, window_preset_label(preset))
                    .clicked()
                {
                    ui.close_menu();
                }
            }
        });
        if ui
            .checkbox(&mut self.window_resizable, "Resizable window")
            .changed()
        {
            ui.close_menu();
        }
    }

    fn render_top_menu_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("controls").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("View", |ui| self.render_view_menu(ui));
            });
        });
    }

    fn render_inspector_side_panel(&mut self, ctx: &egui::Context) {
        let inspector_w = self.inspector_width(ctx);
        egui::SidePanel::right("inspector")
            .resizable(false)
            .exact_width(inspector_w)
            .show(ctx, |ui| {
                // No set_max_width here (018 FIX-9): SidePanel::exact_width already
                // fixes the content width; an extra set_max_width(inspector_w) is
                // applied to the post-margin inner Ui and pushes content ~8px past
                // the panel edge (egui 0.30 placer semantics).
                let time = ctx.input(|i| i.time);
                self.render_inspector_panel(ui, ctx, time);
            });
    }

    /// Zone A (018 Batch 2): persistent single-row nav strip — Up button,
    /// Flat/Tree view toggle, subfolder-scan spinner, and the truncated
    /// current-folder breadcrumb (hover shows the full path). The breadcrumb is
    /// placed last so it truncates within the row's remaining width rather than
    /// pushing the toggle off the edge. Collect-then-act: the Up click is
    /// recorded into `target` and `navigate_to_folder` (needs `&mut self`) runs
    /// only after the ui closure's borrows end.
    fn render_inspector_nav_strip(&mut self, ui: &mut egui::Ui) {
        let cur = self.current_dir.clone();
        let has_folder = cur.is_some();
        let up_target = cur
            .as_ref()
            .and_then(|c| c.parent())
            .map(|p| p.to_path_buf());
        let mut target: Option<PathBuf> = None;
        ui.horizontal(|ui| {
            if ui
                .add_enabled(up_target.is_some(), egui::Button::new("⬆ Up"))
                .clicked()
            {
                target = up_target.clone();
            }
            ui.add_enabled_ui(has_folder, |ui| {
                if ui
                    .selectable_label(
                        self.list_view_mode == ListViewMode::FlatList,
                        list_view_mode_label(ListViewMode::FlatList),
                    )
                    .clicked()
                {
                    self.list_view_mode = ListViewMode::FlatList;
                }
                if ui
                    .selectable_label(
                        self.list_view_mode == ListViewMode::FolderTree,
                        list_view_mode_label(ListViewMode::FolderTree),
                    )
                    .clicked()
                {
                    self.list_view_mode = ListViewMode::FolderTree;
                    if self.tree_expanded_paths.is_empty() {
                        self.tree_expanded_paths = default_tree_expanded();
                    }
                }
            });
            if self.subfolders_pending {
                ui.spinner();
            }
            if let Some(cur) = &cur {
                let dir_str = cur.display().to_string();
                ui.add(
                    egui::Label::new(dir_str.clone())
                        .selectable(true)
                        .wrap_mode(egui::TextWrapMode::Truncate),
                )
                .on_hover_text(dir_str);
            }
        });
        if let Some(dir) = target {
            self.navigate_to_folder(&dir);
        }
    }

    /// Zone B (018 Batch 2): drill-down rows for the immediate subfolders of the
    /// current folder, in a bounded 120px scroll. Hidden entirely when there are
    /// no subfolders. Collect-then-act as in Zone A.
    fn render_inspector_subfolder_drilldown(&mut self, ui: &mut egui::Ui) {
        if self.subfolders.is_empty() {
            return;
        }
        let mut target: Option<PathBuf> = None;
        egui::ScrollArea::vertical()
            .id_salt("subfolder_nav")
            .max_height(120.0)
            .show(ui, |ui| {
                for folder in &self.subfolders {
                    let name = folder
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| folder.display().to_string());
                    if ui.selectable_label(false, format!("📁 {name}")).clicked() {
                        target = Some(folder.clone());
                    }
                }
            });
        if let Some(dir) = target {
            self.navigate_to_folder(&dir);
        }
    }

    /// Zone C (018 Batch 2, PRIMARY): the flat or tree image list, virtualized
    /// via `show_rows`. Sized to the live remaining inspector height minus the
    /// flat column-header row and the space Zone D will occupy below it; floored
    /// at four rows so it never vanishes. Zones A/B were placed earlier in this
    /// same `ui`, so `ui.available_height()` here already excludes them — no
    /// fixed banner/subfolder estimate is subtracted (that was the old
    /// central-panel double-count; see 018 O-review Batch 2 notes).
    fn render_inspector_file_list(
        &mut self,
        ui: &mut egui::Ui,
        filtered: &[usize],
        list_root: Option<&Path>,
        full_h: f32,
    ) {
        let row_h = 18.0;
        let item_spacing_y = ui.spacing().item_spacing.y;
        // Only FLAT mode renders a "Folder / Filename / Status" column-header row;
        // TREE mode has no header, so subtracting it there is phantom reservation
        // that shrinks the list (018 FIX-8).
        let flat_header_h = row_h + item_spacing_y;
        let header_h = if self.list_view_mode == ListViewMode::FlatList {
            flat_header_h
        } else {
            0.0
        };
        // The list sits inside an `egui::Frame::group` with `inner_margin(4.0)`
        // (8px top+bottom) followed by one trailing `item_spacing`; reserve both
        // so tall lists never overflow the inspector (018 FIX-8).
        let group_chrome = 4.0 * 2.0 + item_spacing_y;
        let drawer_reserved = self.sections_drawer_reserved_height(full_h);
        let total_w = ui.available_width();
        let folder_col_w = total_w * 0.35;
        let status_col_w = total_w * 0.25;
        let metrics = ImageListMetrics {
            list_height: (ui.available_height() - header_h - group_chrome - drawer_reserved)
                .max(row_h * 4.0),
            folder_col_w,
            // Filename column takes the middle band; a fixed width lets the
            // filename truncate instead of clipping the Status column (018 FIX-10).
            name_col_w: (total_w - folder_col_w - status_col_w - item_spacing_y * 2.0).max(0.0),
            status_col_w,
            row_h,
        };
        egui::Frame::group(ui.style())
            .inner_margin(4.0)
            .show(ui, |ui| {
                if self.list_view_mode == ListViewMode::FlatList {
                    self.render_flat_image_list(ui, filtered, list_root, metrics);
                } else {
                    self.render_tree_image_list(ui, list_root, metrics.list_height, row_h);
                }
            });
    }

    /// Inner scroll-body height of the expanded Zone D drawer. Clamped so the
    /// drawer never starves Zone C nor grows unbounded. Base is the full
    /// inspector content height (captured once, before any zone renders) so the
    /// drawer size is stable frame-to-frame regardless of Zone A/B content.
    fn sections_drawer_body_height(full_h: f32) -> f32 {
        sections_drawer_body_height(full_h)
    }

    /// Total vertical space Zone D consumes, which Zone C reserves before it
    /// renders. Collapsed: just the toggle row. Expanded: toggle row plus the
    /// bounded scroll body. Pure math lives in `ui_logic` (unit-tested, 018 FIX-8).
    fn sections_drawer_reserved_height(&self, full_h: f32) -> f32 {
        sections_drawer_reserved_height(full_h, self.inspector_drawer_collapsed)
    }

    fn render_scan_inventory_banner(&self, ui: &mut egui::Ui, list_root: Option<&Path>) {
        let (Some(inv), Some(dir)) = (&self.scan_inventory, &self.current_dir) else {
            return;
        };
        egui::Frame::group(ui.style())
            .inner_margin(6.0)
            .show(ui, |ui| {
                ui.strong("Scan inventory");
                for line in format_inventory_bar(inv, &dir.display().to_string()) {
                    ui.label(line);
                }
                if let Some(hint) =
                    inventory_magick_hint(self.tool_caps.magick_available, list_root)
                {
                    ui.small(hint);
                }
                if inv.magick_identify_truncated {
                    ui.small("ImageMagick identify capped at 500 files this scan.");
                }
            });
        ui.add_space(4.0);
    }

    /// Primary-click selection for a file-list row. Right-click is handled
    /// natively by `render_image_context_menu`'s `Response::context_menu`, so
    /// this no longer routes a custom popup (feature 018 B3.3).
    fn handle_image_row_click(&mut self, response: &egui::Response, path: PathBuf) {
        if response.clicked() {
            self.select_image(path);
        }
    }

    fn render_flat_list_row(
        &mut self,
        ui: &mut egui::Ui,
        idx: usize,
        list_root: Option<&Path>,
        metrics: ImageListMetrics,
    ) {
        if idx >= self.images.len() {
            return;
        }
        let path = self.images[idx].path.clone();
        let folder = relative_folder(list_root, &path);
        let name = file_name_display(&path);
        let status = file_status_label(self.images[idx].status);
        let decodable = file_status_decodable(self.images[idx].status);
        let is_selected = self.selected.as_ref() == Some(&path);
        ui.horizontal(|ui| {
            ui.allocate_ui(egui::vec2(metrics.folder_col_w, metrics.row_h), |ui| {
                ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Truncate);
                ui.label(egui::RichText::new(folder).weak());
            });
            // Fixed-width, truncating filename column with a full-path hover
            // (same pattern as the breadcrumb) so long names never clip the
            // Status column (018 FIX-10).
            let response = ui
                .allocate_ui(egui::vec2(metrics.name_col_w, metrics.row_h), |ui| {
                    ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Truncate);
                    ui.selectable_label(is_selected, &name)
                })
                .inner
                .on_hover_text(path.display().to_string());
            self.render_image_context_menu(&response, &path, decodable);
            self.handle_image_row_click(&response, path);
            ui.allocate_ui(egui::vec2(metrics.status_col_w, metrics.row_h), |ui| {
                ui.small(status);
            });
        });
    }

    fn render_flat_image_list(
        &mut self,
        ui: &mut egui::Ui,
        filtered: &[usize],
        list_root: Option<&Path>,
        metrics: ImageListMetrics,
    ) {
        ui.horizontal(|ui| {
            ui.allocate_ui(egui::vec2(metrics.folder_col_w, metrics.row_h), |ui| {
                ui.strong("Folder");
            });
            ui.strong("Filename");
            ui.allocate_ui(egui::vec2(metrics.status_col_w, metrics.row_h), |ui| {
                ui.strong("Status");
            });
        });
        let mut scroll_area = egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .max_height(metrics.list_height)
            .id_salt(self.scroll_generation);
        if let Some(offset) = self.pending_flat_scroll_offset(filtered, metrics) {
            scroll_area = scroll_area.vertical_scroll_offset(offset);
        }
        scroll_area.show_rows(ui, metrics.row_h, filtered.len(), |ui, row_range| {
            for row in row_range {
                if row >= filtered.len() {
                    break;
                }
                self.render_flat_list_row(ui, filtered[row], list_root, metrics);
            }
        });
    }

    /// One-shot forced scroll offset for a pending round-trip landing (feature
    /// 016, US2 AS1: "the list scrolls to it"). Peek-then-take: `pending_scroll_path`
    /// is consumed ONLY once the target row is actually found in `filtered`, so a
    /// frame where the list has not caught up yet (e.g. mid cross-folder round-trip
    /// landing, before the rescan lands) does not lose the pending scroll. The
    /// pending path is otherwise cleared by `scan_directory`'s reset and by tree
    /// mode (which cannot scroll-to-path), bounding its lifetime (018 FIX-6).
    fn pending_flat_scroll_offset(
        &mut self,
        filtered: &[usize],
        metrics: ImageListMetrics,
    ) -> Option<f32> {
        let target = self.pending_scroll_path.clone()?;
        let row = filtered
            .iter()
            .position(|&i| self.images[i].path == target)?;
        self.pending_scroll_path = None;
        Some((row as f32 * metrics.row_h - metrics.list_height / 2.0).max(0.0))
    }

    fn toggle_tree_folder(&mut self, folder_path: &str) {
        let abs = match &self.current_dir {
            Some(root) if folder_path == "." => root.clone(),
            Some(root) => root.join(folder_path),
            None => PathBuf::from(folder_path),
        };
        self.selected_tree_folder = Some(abs);
        let path_key = folder_path.to_string();
        if self.tree_expanded_paths.contains(&path_key) {
            self.tree_expanded_paths.remove(&path_key);
        } else {
            self.tree_expanded_paths.insert(path_key);
        }
        self.scroll_generation = self.scroll_generation.wrapping_add(1);
    }

    fn render_tree_folder_row(
        &mut self,
        ui: &mut egui::Ui,
        tree_row: &TreeRow,
        list_root: Option<&Path>,
    ) {
        let prefix = if tree_row.expanded { "▼" } else { "▶" };
        let name = folder_tree_display_name(&tree_row.folder_path, list_root);
        let suffix = folder_line_suffix(tree_row.listed, tree_row.magick, tree_row.skipped);
        let label = format!("{prefix} {name}  — {suffix}");
        if ui.selectable_label(false, label).clicked() {
            self.toggle_tree_folder(&tree_row.folder_path);
        }
    }

    fn render_tree_file_row(&mut self, ui: &mut egui::Ui, tree_row: &TreeRow, indent: f32) {
        let Some(idx) = tree_row.entry_index else {
            return;
        };
        if idx >= self.images.len() {
            return;
        }
        let path = self.images[idx].path.clone();
        let name = file_name_display(&path);
        let glyph = tree_file_glyph(self.images[idx].status);
        let status = file_status_label(self.images[idx].status);
        let decodable = file_status_decodable(self.images[idx].status);
        let label = if self.images[idx].status == rust_feh::types::FileStatus::Converted {
            format!("{glyph} {name}  [{status}]")
        } else {
            format!("{glyph} {name}")
        };
        let is_selected = self.selected.as_ref() == Some(&path);
        ui.horizontal(|ui| {
            ui.add_space(indent);
            let response = ui.selectable_label(is_selected, label);
            self.render_image_context_menu(&response, &path, decodable);
            self.handle_image_row_click(&response, path);
        });
    }

    fn render_tree_image_list(
        &mut self,
        ui: &mut egui::Ui,
        list_root: Option<&Path>,
        list_height: f32,
        row_h: f32,
    ) {
        let root_skipped = self
            .scan_inventory
            .as_ref()
            .map(|i| i.non_image_skipped)
            .unwrap_or(0);

        let hit = self.tree_rows_cache_key.as_ref().is_some_and(|k| {
            k.revision == self.images_revision
                && k.sort_mode == self.sort_mode
                && k.root_skipped == root_skipped
                && k.search == self.search
                && k.current_dir.as_deref() == self.current_dir.as_deref()
                && k.expanded == self.tree_expanded_paths
        });
        if !hit {
            self.tree_rows_cache = tree_visible_rows(
                &self.images,
                list_root,
                &self.search,
                self.sort_mode,
                &self.tree_expanded_paths,
                root_skipped,
            );
            self.tree_rows_cache_key = Some(TreeRowsKey {
                revision: self.images_revision,
                current_dir: self.current_dir.clone(),
                search: self.search.clone(),
                sort_mode: self.sort_mode,
                root_skipped,
                expanded: self.tree_expanded_paths.clone(),
            });
        }

        // Tree mode does not implement scroll-to-path, so consume-or-clear any
        // pending landing scroll here (018 FIX-6): otherwise a scroll armed while
        // in tree mode would linger and fire much later when the user switches to
        // flat mode. Clearing is the "consume" — the tree simply has nowhere to
        // scroll to.
        self.pending_scroll_path = None;

        // Move rows out so the show_rows closure can borrow &mut self freely,
        // then move them back afterward. Avoids a per-frame Vec<TreeRow> clone.
        let tree_rows = std::mem::take(&mut self.tree_rows_cache);
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .max_height(list_height)
            .id_salt(self.scroll_generation)
            .show_rows(ui, row_h, tree_rows.len(), |ui, row_range| {
                for row in row_range {
                    if row >= tree_rows.len() {
                        break;
                    }
                    let tree_row = &tree_rows[row];
                    let indent = tree_row.depth as f32 * 14.0;
                    match tree_row.kind {
                        TreeRowKind::Folder => {
                            self.render_tree_folder_row(ui, tree_row, list_root);
                        }
                        TreeRowKind::File => {
                            self.render_tree_file_row(ui, tree_row, indent);
                        }
                    }
                }
            });
        self.tree_rows_cache = tree_rows; // restore
    }

    /// Central panel after 018 Batch 2 (decision 2): the file list, subfolder
    /// nav, and inventory banner all moved into the inspector (Zones A-D); the
    /// central panel now renders ONLY the stage (currently-viewed) image,
    /// filling the panel.
    fn render_central_image_panel(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::Frame::group(ui.style())
                .inner_margin(6.0)
                .show(ui, |ui| {
                    self.render_stage_pane(ui);
                });
        });
    }

    fn request_repaint_if_busy(&self, ctx: &egui::Context) {
        let busy = self.is_activity_busy();
        let tip_animating = !self.tool_caps.operation_timings().is_empty();
        if busy || tip_animating {
            ctx.request_repaint_after(std::time::Duration::from_millis(200));
        }
    }

    // --- Stage pane: off-thread decode (feature 016, FR-001/R5) ---

    /// Detect a new selection and kick an off-thread decode job for it,
    /// tagged with a fresh generation so any in-flight stale job is discarded
    /// on receipt (contract: "selection changes always win").
    fn kick_stage_decode_if_selection_changed(&mut self, ctx: &egui::Context) {
        if self.selected == self.stage_requested_path {
            return;
        }
        self.stage_requested_path = self.selected.clone();
        self.stage_generation = self.stage_generation.wrapping_add(1);
        let generation = self.stage_generation;
        let Some(path) = self.selected.clone() else {
            self.stage_state = StageState::Failed {
                reason: "No image selected".to_string(),
            };
            self.stage_texture = None;
            return;
        };
        self.stage_state = StageState::Loading;
        let (tx, rx) = mpsc::channel();
        self.stage_rx = Some(rx);
        let ctx = ctx.clone();
        thread::spawn(move || {
            let msg = match decode_stage_rgba(&path, STAGE_MAX_EDGE) {
                Ok((width, height, rgba)) => StageDecodeMsg::Ready {
                    generation,
                    width,
                    height,
                    rgba,
                },
                Err(reason) => StageDecodeMsg::Failed { generation, reason },
            };
            let _ = tx.send(msg);
            ctx.request_repaint();
        });
    }

    /// Apply the latest decode result, if any; stale-generation results are
    /// silently dropped (a later selection has already superseded them).
    fn poll_stage_decode(&mut self, ctx: &egui::Context) {
        let Some(rx) = &self.stage_rx else {
            return;
        };
        let mut latest = None;
        while let Ok(msg) = rx.try_recv() {
            latest = Some(msg);
        }
        let Some(msg) = latest else {
            return;
        };
        match msg {
            StageDecodeMsg::Ready {
                generation,
                width,
                height,
                rgba,
            } if generation == self.stage_generation => {
                let color_image = egui::ColorImage::from_rgba_unmultiplied(
                    [width as usize, height as usize],
                    &rgba,
                );
                let texture = ctx.load_texture(
                    format!("stage-{generation}"),
                    color_image,
                    egui::TextureOptions::LINEAR,
                );
                self.stage_texture = Some(texture);
                self.stage_state = StageState::Ready { width, height };
            }
            StageDecodeMsg::Failed { generation, reason }
                if generation == self.stage_generation =>
            {
                self.stage_texture = None;
                self.stage_state = StageState::Failed { reason };
            }
            _ => {}
        }
    }

    // --- Stage pane render + context menu (feature 016, contracts/stage-context-menu.md) ---

    fn render_stage_pane(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let toggle_label = if self.stage_pane_collapsed {
                "▶ Stage"
            } else {
                "▼ Stage"
            };
            if ui.small_button(toggle_label).clicked() {
                self.stage_pane_collapsed = !self.stage_pane_collapsed;
            }
            self.render_stage_status_line(ui);
        });
        if self.stage_pane_collapsed {
            return;
        }
        self.render_stage_image(ui);
    }

    fn render_stage_status_line(&self, ui: &mut egui::Ui) {
        match &self.stage_state {
            StageState::Loading => {
                ui.weak("Loading…");
            }
            StageState::Failed { reason } => {
                let name = self
                    .selected
                    .as_deref()
                    .map(file_name_display)
                    .unwrap_or_default();
                ui.weak(format!("Cannot preview {name}: {reason}"));
            }
            StageState::Ready { width, height } => {
                ui.weak(format!("{width}×{height}"));
            }
        }
    }

    fn render_stage_image(&mut self, ui: &mut egui::Ui) {
        let Some(texture) = self.stage_texture.clone() else {
            return;
        };
        let Some(path) = self.selected.clone() else {
            return;
        };
        let avail = ui.available_size();
        let tex_size = texture.size_vec2();
        if avail.x <= 0.0 || avail.y <= 0.0 || tex_size.x <= 0.0 || tex_size.y <= 0.0 {
            return;
        }
        // Never upscale beyond 1:1 (contract: "no crop, no upscale beyond 1:1").
        let scale = (avail.x / tex_size.x).min(avail.y / tex_size.y).min(1.0);
        let draw_size = tex_size * scale.max(0.01);
        let response = ui.add(
            egui::Image::new(&texture)
                .fit_to_exact_size(draw_size)
                .sense(egui::Sense::click()),
        );
        let decodable = matches!(self.stage_state, StageState::Ready { .. });
        self.render_image_context_menu(&response, &path, decodable);
    }

    /// Shared right-click context menu for a decodable image, used by BOTH the
    /// central stage and file-list rows (feature 018 B3.3). `decodable` gates the
    /// process-only actions (Resize/Convert/Copy image): the stage derives it from
    /// `StageState::Ready`, list rows from `file_status_decodable(status)`.
    fn render_image_context_menu(
        &mut self,
        response: &egui::Response,
        path: &Path,
        decodable: bool,
    ) {
        response.context_menu(|ui| {
            let ctx = ui.ctx().clone();
            if ui.button("Save a copy…").clicked() {
                self.action_save_copy(path);
                ui.close_menu();
            }
            if ui.button("Move to…").clicked() {
                self.action_move_to(path);
                ui.close_menu();
            }
            if ui
                .add_enabled(decodable, egui::Button::new("Resize copy"))
                .clicked()
            {
                self.action_resize_copy(path);
                ui.close_menu();
            }
            ui.add_enabled_ui(decodable, |ui| {
                ui.menu_button("Convert format", |ui| {
                    for fmt in ["jpg", "png", "webp"] {
                        if ui.button(fmt).clicked() {
                            self.action_convert_format(path, fmt);
                            ui.close_menu();
                        }
                    }
                });
            });
            if ui.button("Copy path").clicked() {
                ctx.copy_text(path.display().to_string());
                self.record_action_outcome(ContextAction::CopyPath, path, None, Ok(None));
                ui.close_menu();
            }
            if ui
                .add_enabled(decodable, egui::Button::new("Copy image"))
                .clicked()
            {
                self.action_copy_image(path);
                ui.close_menu();
            }
        });
    }

    // --- Context actions (feature 016, FR-002..FR-006/FR-010) ---

    fn action_save_copy(&mut self, path: &Path) {
        let Some(dest_dir) = self.pick_action_destination() else {
            return;
        };
        match save_copy_to(path, &dest_dir) {
            Ok(produced) => self.record_action_outcome(
                ContextAction::SaveCopyTo,
                path,
                Some(dest_dir),
                Ok(Some(produced)),
            ),
            Err(reason) => self.record_action_outcome(
                ContextAction::SaveCopyTo,
                path,
                Some(dest_dir),
                Err(reason),
            ),
        }
    }

    fn action_move_to(&mut self, path: &Path) {
        let Some(dest_dir) = self.pick_action_destination() else {
            return;
        };
        let plan = match plan_loss_proof_move(path, &dest_dir) {
            Ok(p) => p,
            Err(reason) => {
                self.record_action_outcome(
                    ContextAction::MoveTo,
                    path,
                    Some(dest_dir),
                    Err(reason),
                );
                return;
            }
        };
        match execute_move_plan(&plan) {
            Ok(produced) => {
                self.record_action_outcome(
                    ContextAction::MoveTo,
                    path,
                    Some(dest_dir),
                    Ok(Some(produced)),
                );
                self.advance_stage_after_move(path);
            }
            Err(reason) => {
                self.record_action_outcome(ContextAction::MoveTo, path, Some(dest_dir), Err(reason))
            }
        }
    }

    /// Native folder chooser starting at the persisted last destination
    /// (FR-003); `None` on cancel (no side effects, per edge case).
    fn pick_action_destination(&mut self) -> Option<PathBuf> {
        let mut dialog = rfd::FileDialog::new();
        if let Some(dir) = &self.action_prefs.last_destination {
            dialog = dialog.set_directory(dir);
        }
        let dest_dir = dialog.pick_folder()?;
        self.action_prefs.last_destination = Some(dest_dir.clone());
        if let Err(e) = save_action_prefs(&self.action_prefs) {
            self.log(format!("Failed to persist action prefs: {e}"));
        }
        Some(dest_dir)
    }

    fn action_resize_copy(&mut self, path: &Path) {
        let dest = match Self::derived_action_output_path(path, "_resized", "jpg") {
            Ok(d) => d,
            Err(reason) => {
                self.record_action_outcome(ContextAction::ResizeCopy, path, None, Err(reason));
                return;
            }
        };
        let opts = ProcessOptions {
            width: None,
            height: None,
            percent: Some(50.0),
            fit: None,
            filter: None,
            target_format: Some("jpg".into()),
            quality: Some(80),
            output_path: Some(dest),
        };
        self.run_derived_action(ContextAction::ResizeCopy, path, &opts);
    }

    fn action_convert_format(&mut self, path: &Path, target_format: &str) {
        let dest = match Self::derived_action_output_path(path, "", target_format) {
            Ok(d) => d,
            Err(reason) => {
                self.record_action_outcome(ContextAction::ConvertFormat, path, None, Err(reason));
                return;
            }
        };
        let opts = ProcessOptions {
            width: None,
            height: None,
            percent: None,
            fit: None,
            filter: None,
            target_format: Some(target_format.to_string()),
            quality: Some(85),
            output_path: Some(dest),
        };
        self.run_derived_action(ContextAction::ConvertFormat, path, &opts);
    }

    /// Collision-safe destination for a derived (resize/convert) output,
    /// reusing the existing Image Tools "processed" subfolder convention
    /// (FR-006) plus `collision_suffixed_path` (FR-004).
    fn derived_action_output_path(
        source: &Path,
        stem_suffix: &str,
        ext: &str,
    ) -> Result<PathBuf, String> {
        let policy = OutputPolicy::NewSubfolder {
            name: "processed".into(),
        };
        let base = compute_output_path(source, stem_suffix, ext, &policy)?;
        let dir = base.parent().unwrap_or(Path::new(".")).to_path_buf();
        std::fs::create_dir_all(&dir)
            .map_err(|e| format!("Failed to create output folder {}: {e}", dir.display()))?;
        let name = base
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();
        Ok(collision_suffixed_path(&dir, &name))
    }

    fn run_derived_action(&mut self, action: ContextAction, path: &Path, opts: &ProcessOptions) {
        match process_image(path, opts) {
            Ok(produced) => {
                add_or_update_asset_in_inventory(
                    &mut self.images,
                    produced.clone(),
                    AssetStatus::Processed,
                );
                self.record_action_outcome(action, path, None, Ok(Some(produced)));
            }
            Err(reason) => self.record_action_outcome(action, path, None, Err(reason)),
        }
    }

    fn action_copy_image(&mut self, path: &Path) {
        match copy_image_to_clipboard(path) {
            Ok(_status) => {
                self.record_action_outcome(ContextAction::CopyImage, path, None, Ok(None))
            }
            Err(reason) => {
                self.record_action_outcome(ContextAction::CopyImage, path, None, Err(reason))
            }
        }
    }

    /// After a successful move, advance the stage/selection to the next
    /// surviving image in the filtered list (edge case: never a stale frame).
    /// Only advances when the MOVED file was the staged/selected one (018
    /// Batch 3: "Move to…" is now reachable from any list row, not just the
    /// staged image — moving a different row must not yank the stage/
    /// selection away from what the user was actually looking at).
    fn advance_stage_after_move(&mut self, moved_path: &Path) {
        let was_selected = self.selected.as_deref() == Some(moved_path);
        let (_, indices_before) = self.compute_list_indices();
        let pos = indices_before
            .iter()
            .position(|&i| self.images[i].path == moved_path);
        self.images.retain(|e| e.path != moved_path);
        // cache invariant: bump on every self.images mutation
        self.images_revision = self.images_revision.wrapping_add(1);
        if !was_selected {
            return;
        }
        let (_, indices_after) = self.compute_list_indices();
        self.selected = if indices_after.is_empty() {
            None
        } else {
            let next_pos = pos.unwrap_or(0).min(indices_after.len() - 1);
            Some(self.images[indices_after[next_pos]].path.clone())
        };
    }

    /// Log + surface every action outcome (FR-010); failures also raise a
    /// native error dialog naming the image and cause (SC-007).
    fn record_action_outcome(
        &mut self,
        action: ContextAction,
        image: &Path,
        destination: Option<PathBuf>,
        result: Result<Option<PathBuf>, String>,
    ) {
        let is_err = result.is_err();
        let outcome = ActionOutcome {
            action: ActionKind::Context(action),
            image: image.to_path_buf(),
            destination,
            result: match result {
                Ok(produced) => ActionResult::Ok { produced },
                Err(reason) => ActionResult::Err { reason },
            },
        };
        let line = format_action_outcome(&outcome);
        self.log(line.clone());
        self.status = line.clone();
        if is_err {
            rfd::MessageDialog::new()
                .set_title("Action failed")
                .set_description(&line)
                .set_level(rfd::MessageLevel::Error)
                .show();
        }
    }
}

impl App for RustFehApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        self.apply_startup_window_prefs(ctx);
        self.maybe_load_start_folder();
        // Edge-detect no-folder ↔ folder-loaded AFTER the start-folder auto-load
        // has settled `current_dir` (018 FIX-1).
        self.sync_auto_expand_folder_edge();
        self.poll_scan_complete(ctx);
        self.poll_subfolders(ctx);
        self.poll_tools_job(ctx);
        Self::emit_startup_notice_once();
        self.sync_frame_input_state(ctx);
        self.poll_round_trip_viewers(ctx);
        self.kick_stage_decode_if_selection_changed(ctx);
        self.poll_stage_decode(ctx);

        self.render_top_menu_bar(ctx);
        self.render_inspector_side_panel(ctx);
        self.render_central_image_panel(ctx);

        self.render_detached_inspector_windows(ctx);
        self.request_repaint_if_busy(ctx);
    }
}

impl RustFehApp {
    fn select_image(&mut self, path: PathBuf) {
        let disp = path.display().to_string();
        self.selected = Some(path);
        self.log(format!("Selected image: {}", disp));
        self.status = format!(
            "Selected: {}. Use Image actions to open in feh, or right-click for Resize/Convert.",
            disp
        );
    }

    fn apply_scan_partial(&mut self, entries: Vec<ImageEntry>, skipped: usize) {
        self.images = entries;
        // cache invariant: bump on every self.images mutation
        self.images_revision = self.images_revision.wrapping_add(1);
        if self.selected.is_none() {
            if let Some(p) = self.images.first().map(|e| e.path.clone()) {
                self.selected = Some(p);
            }
        }
        self.status = format!(
            "Scanning… {} images found ({} other files skipped) — Open in feh anytime",
            self.images.len(),
            skipped
        );
    }

    fn handle_scan_msg(&mut self, msg: ScanMsg, ctx: &egui::Context, still_scanning: &mut bool) {
        match msg {
            ScanMsg::Partial {
                generation,
                entries,
                skipped,
            } if generation == self.scan_generation => {
                self.apply_scan_partial(entries, skipped);
                ctx.request_repaint();
            }
            ScanMsg::Complete {
                generation,
                dir_label,
                result,
            } if generation == self.scan_generation => {
                self.apply_scan_result(&dir_label, result);
                *still_scanning = false;
                self.scanning = false;
            }
            ScanMsg::Converted {
                generation,
                entries,
                non_image_skipped,
                magick_truncated,
            } if generation == self.scan_generation => {
                // Merge the background converted snapshot BY PATH instead of a wholesale
                // replace, so user mutations to self.images (rename/move/processed-add) that
                // raced this async message are not silently discarded (018 F1).
                merge_converted_statuses(&mut self.images, &entries);
                // Rebuild the inventory from the LIVE list AFTER the merge, not the stale
                // scan-time snapshot (018 FIX-2): otherwise a move/tools-op that raced this
                // message reverts the tree/inventory counts and breaks SC-005. The scan-level
                // metadata (skipped / truncated) still comes from the scan message.
                let inventory =
                    ScanInventory::from_entries(&self.images, non_image_skipped, magick_truncated);
                // cache invariant: bump on every self.images mutation
                self.images_revision = self.images_revision.wrapping_add(1);
                self.scan_inventory = Some(inventory);
                self.log("Converted-status metadata updated (background)");
            }
            _ => {}
        }
    }

    fn poll_scan_complete(&mut self, ctx: &egui::Context) {
        let mut messages = Vec::new();
        if let Some(rx) = &self.scan_rx {
            while let Ok(msg) = rx.try_recv() {
                messages.push(msg);
            }
        }
        // Each queued Partial carries the FULL accumulated list so far, so an
        // earlier Partial in the same drain batch is always superseded by a
        // later one in that batch — apply only the last one to avoid redundant
        // self.images replaces (and the cache-invalidation bumps that follow).
        let last_partial_idx = messages
            .iter()
            .rposition(|m| matches!(m, ScanMsg::Partial { .. }));
        let mut still_scanning = self.scanning;
        for (i, msg) in messages.into_iter().enumerate() {
            if matches!(msg, ScanMsg::Partial { .. }) && Some(i) != last_partial_idx {
                continue;
            }
            self.handle_scan_msg(msg, ctx, &mut still_scanning);
        }
        self.scanning = still_scanning;
        if self.scanning {
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
        }
    }

    /// Kick an off-thread, non-recursive listing of `dir`'s immediate
    /// subfolders (feature 017 drill-down nav). Coalesced/superseded via
    /// subfolder_generation the same way scan_directory supersedes scans.
    fn request_subfolders(&mut self, dir: &Path) {
        self.subfolder_generation = self.subfolder_generation.wrapping_add(1);
        let generation = self.subfolder_generation;
        self.subfolders.clear();
        self.subfolders_dir = Some(dir.to_path_buf());
        self.subfolders_pending = true;
        let dir_path = dir.to_path_buf();
        let (tx, rx) = mpsc::channel();
        self.subfolder_rx = Some(rx);
        thread::spawn(move || {
            let folders = list_subfolders(&dir_path);
            let _ = tx.send(SubfolderMsg {
                generation,
                dir: dir_path,
                folders,
            });
        });
    }

    /// Poll the off-thread subfolder listing; coalesces multiple queued
    /// messages (keeps only the last) and discards results from a
    /// superseded generation (GEN GUARD).
    fn poll_subfolders(&mut self, ctx: &egui::Context) {
        let Some(rx) = &self.subfolder_rx else {
            return;
        };
        let mut latest = None;
        while let Ok(msg) = rx.try_recv() {
            latest = Some(msg);
        }
        if let Some(msg) = latest {
            if msg.generation == self.subfolder_generation {
                self.subfolders = msg.folders;
                self.subfolders_dir = Some(msg.dir);
                self.subfolders_pending = false;
            }
        }
        if self.subfolders_pending {
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
        }
    }

    fn scan_directory(&mut self, dir: &Path) {
        self.cancel_tools_job();
        self.cleanup_prepare_fast_temp();
        self.scanning = true;
        self.status = "Scanning…".to_string();
        self.selected = None;
        self.images.clear();
        self.pending_select_path = None;
        // Landing-scroll is re-armed AFTER navigate_to_folder by the round-trip
        // path (stage_selection_from_round_trip); clearing it here in the scan
        // reset stops a stale pending scroll from an earlier folder outliving the
        // rescan (018 FIX-6).
        self.pending_scroll_path = None;
        // cache invariant: bump on every self.images mutation
        self.images_revision = self.images_revision.wrapping_add(1);
        // Scan start = rising edge for Session status (018 FIX-1): a new scan
        // attempt is a fresh scope (lift any prior user-close latch), then open
        // Session status + expand the drawer once.
        self.auto_expand
            .begin_scope(InspectorSection::SessionStatus);
        self.auto_expand.request_open(
            InspectorSection::SessionStatus,
            &mut self.inspector_open,
            &mut self.inspector_drawer_collapsed,
        );
        self.selected_tree_folder = None;
        self.scan_inventory = None;
        self.tree_expanded_paths = default_tree_expanded();
        self.scan_generation = self.scan_generation.wrapping_add(1);
        self.scroll_generation = self.scroll_generation.wrapping_add(1);
        let generation = self.scan_generation;

        let dir_path = dir.to_path_buf();
        let recursive = self.recursive;
        let on_network = is_network_mount_path(dir);
        let magick_identify =
            self.deep_scan_magick && scan_magick_enabled(self.tool_caps.magick_available, dir);
        let dir_label = dir.display().to_string();
        self.scan_cancel.store(true, Ordering::Relaxed);
        let cancel = Arc::new(AtomicBool::new(false));
        self.scan_cancel = cancel.clone();
        let (tx, rx) = mpsc::channel();
        self.scan_rx = Some(rx);

        thread::spawn(move || {
            let result = scan_images_streaming(
                &dir_path,
                recursive,
                magick_identify,
                &cancel,
                |entries, skipped, _| {
                    let _ = tx.send(ScanMsg::Partial {
                        generation,
                        entries: entries.to_vec(),
                        skipped,
                    });
                },
            );
            let skipped = result.inventory.non_image_skipped;
            let truncated = result.inventory.magick_identify_truncated;
            let mut entries = result.entries.clone();
            let _ = tx.send(ScanMsg::Complete {
                generation,
                dir_label: dir_label.clone(),
                result,
            });
            thread::spawn(move || {
                if cancel.load(Ordering::Relaxed) {
                    return;
                }
                apply_converted_detection_cancellable(&mut entries, &cancel);
                let _ = tx.send(ScanMsg::Converted {
                    generation,
                    entries,
                    non_image_skipped: skipped,
                    magick_truncated: truncated,
                });
            });
        });

        if on_network {
            self.log(
                "Network folder — fast extension scan only (ImageMagick identify disabled)"
                    .to_string(),
            );
        } else if !magick_identify {
            self.log(
                "Fast scan: common image extensions only (enable “Detect exotic formats” for slow deep scan)"
                    .to_string(),
            );
        }
        self.log(format!(
            "Scan started for '{}' (recursive={})",
            dir.display(),
            self.recursive
        ));
    }

    fn apply_scan_result(&mut self, dir_label: &str, result: ScanResult) {
        for w in &result.warnings {
            self.log(w.clone());
        }

        let (entries, inventory) = finalize_scan_entries_fast(
            result.entries,
            result.inventory.non_image_skipped,
            result.inventory.magick_identify_truncated,
        );
        self.images = entries;
        // cache invariant: bump on every self.images mutation
        self.images_revision = self.images_revision.wrapping_add(1);
        self.scan_inventory = Some(inventory);

        if let Some(ref inv) = self.scan_inventory {
            self.log(format!(
                "Inventory: native={} magick={} converted={} awaiting={} skipped={}",
                inv.native_listed,
                inv.magick_detected,
                inv.converted,
                inv.awaiting_convert,
                inv.non_image_skipped
            ));
        }

        self.log(format!(
            "Scanned '{}' (recursive={}), found {} supported images",
            dir_label,
            self.recursive,
            self.images.len()
        ));

        if !self.images.is_empty() {
            let sample: Vec<_> = self
                .images
                .iter()
                .take(3)
                .map(|e| e.path.file_name().unwrap_or_default().to_string_lossy())
                .collect();
            self.log(format!("Sample files: {:?}", sample));
        }

        if self.images.is_empty() {
            self.status = post_scan_status("No images found", self.feh_available);
            self.selected = None;
            self.pending_select_path = None;
        } else {
            let target = self
                .pending_select_path
                .take()
                .filter(|p| self.images.iter().any(|e| &e.path == p));
            let p = target.unwrap_or_else(|| self.images[0].path.clone());
            self.selected = Some(p.clone());
            self.status = post_scan_status(
                &format!("Loaded {} images — Open in feh to view.", self.images.len()),
                self.feh_available,
            );
            self.log(format!("Auto-selected first image: {}", p.display()));
        }

        // Scan complete = falling edge for Session status (018 FIX-1): retract it
        // if the machine still owns it (a user who opened it stays), returning the
        // drawer to collapsed when nothing else is open (SC-001 / US1-AS2). Only
        // reached for the current generation's Complete message.
        self.auto_expand.retract(
            InspectorSection::SessionStatus,
            &mut self.inspector_open,
            &mut self.inspector_drawer_collapsed,
        );
    }

    /// Open the current filtered list in a round-trip feh viewer (feature 016,
    /// US2): rust-feh retains the `Child`, and when the user closes it, the
    /// image they landed on is selected and staged back in rust-feh. Other
    /// feh launches (`launch_entry_feh`/Launch All, `open_feh_on_prepared_fast`)
    /// are unaffected and keep using `spawn_feh_viewer` (fire-and-forget,
    /// byte-identical to pre-feature behavior — US3/T021; this codebase has no
    /// wallpaper `--bg-fill` spawn site yet, so there's nothing to verify there).
    fn open_in_feh(&mut self, path: &Path) {
        let (_, indices) = self.compute_list_indices();
        if indices.is_empty() {
            self.status = "No images in filtered list".to_owned();
            return;
        }
        if !indices
            .iter()
            .any(|&i| self.images[i].path.as_path() == path)
        {
            self.status = "Selected image is not in the filtered filelist".to_owned();
            return;
        }

        let paths: Vec<PathBuf> = indices
            .iter()
            .map(|&i| self.images[i].path.clone())
            .collect();

        let list_path = feh_filelist_temp_path();
        let count = match write_feh_filelist(&paths) {
            Ok(n) => n,
            Err(e) => {
                self.log(format!("Failed to write feh filelist: {e}"));
                self.status = format!("Failed to prepare feh filelist: {e}");
                return;
            }
        };

        self.spawn_round_trip_viewer(&list_path, path, paths, count);
    }

    /// Open a PINNED image in feh, bypassing `try_open_in_feh`/`resolve_feh_start_path`
    /// (which mutate `self.selected` as a side effect on fallback) — a pinned action
    /// must never disturb the live selection. Reuses `open_in_feh`'s existing
    /// filtered-list membership gate verbatim: if the pinned path is not in the
    /// live folder's filtered list (e.g. pin survived a folder navigation), this
    /// fails CLOSED with the existing "not in the filtered filelist" status rather
    /// than opening the wrong file.
    fn open_in_feh_pinned(&mut self, path: &Path) {
        if !self.feh_available {
            self.status = feh_missing_status();
            return;
        }
        // Defensive pinned-case gate (018 FIX-3): the detached window already
        // hides the action when the pin is not in the live list, but check here
        // too so a stale pin can never fall through to open_in_feh's generic
        // "not in the filtered filelist" status (which surfaces in the MAIN
        // window and reads as a mystery no-op). Reword for the pinned case.
        if !self.pinned_path_in_filtered_list(path) {
            self.status = format!(
                "Pinned image is not in the current folder/filter: {} — unpin or load its folder",
                file_name_display(path)
            );
            return;
        }
        self.open_in_feh(path);
    }

    fn spawn_round_trip_viewer(
        &mut self,
        list_path: &Path,
        start_at: &Path,
        filelist: Vec<PathBuf>,
        count: usize,
    ) {
        self.next_viewer_id = self.next_viewer_id.wrapping_add(1);
        let viewer_id = self.next_viewer_id;
        let handoff = handoff_path(std::process::id(), viewer_id);
        let profile_dir = viewer_profile_dir();
        let (program, args, envs) =
            viewer_spawn_command(list_path, start_at, &handoff, &profile_dir);

        let mut cmd = Command::new(&program);
        cmd.args(&args);
        for (key, value) in &envs {
            cmd.env(key, value);
        }
        self.log(format!(
            "Spawning round-trip feh with filelist ({count} images)"
        ));
        match cmd.spawn() {
            Ok(child) => {
                self.log(format!(
                    "feh launched (pid {:?}, round-trip viewer {viewer_id})",
                    child.id()
                ));
                self.status = format!("Launched feh on {}", start_at.display());
                self.round_trips.push(ViewerRoundTrip {
                    child,
                    handoff_path: handoff,
                    launched_with: start_at.to_path_buf(),
                    filelist,
                    viewer_id,
                });
            }
            Err(e) => {
                self.log(format!("Failed to spawn feh: {e}"));
                if feh_spawn_unavailable(&e) {
                    self.mark_feh_unavailable();
                } else {
                    self.status = format!("Failed to launch feh (is it installed?): {e}");
                }
            }
        }
    }

    /// Poll every live round-trip viewer once per frame (contract: "retains
    /// the Child and polls try_wait() every frame"); any status (success or
    /// signal) counts as a close and triggers handoff handling.
    fn poll_round_trip_viewers(&mut self, ctx: &egui::Context) {
        if self.round_trips.is_empty() {
            return;
        }
        ctx.request_repaint_after(std::time::Duration::from_millis(200));
        let mut exited_at = Vec::new();
        for (i, rt) in self.round_trips.iter_mut().enumerate() {
            if matches!(rt.child.try_wait(), Ok(Some(_))) {
                exited_at.push(i);
            }
        }
        // Remove back-to-front so earlier indices stay valid; process in the
        // order they were detected (close order) per contract "each exit is
        // processed independently in close order; the most recent close wins".
        for &i in exited_at.iter().rev() {
            let rt = self.round_trips.remove(i);
            self.handle_round_trip_exit(rt);
        }
    }

    fn handle_round_trip_exit(&mut self, rt: ViewerRoundTrip) {
        let content = std::fs::read_to_string(&rt.handoff_path).unwrap_or_default();
        let landed = validate_handoff(&content, &rt.filelist);
        let _ = std::fs::remove_file(&rt.handoff_path);
        match landed {
            Some(path) => {
                self.log(format!(
                    "Round trip (viewer {}): {} -> {}",
                    rt.viewer_id,
                    rt.launched_with.display(),
                    path.display()
                ));
                self.stage_selection_from_round_trip(&path);
            }
            None => {
                self.log(format!(
                    "Round trip (viewer {}) closed without a usable handoff (content length {})",
                    rt.viewer_id,
                    content.len()
                ));
            }
        }
    }

    /// Select + stage the landed image, scrolling the list to it (US2 AS1).
    /// If it no longer passes the active filter, clear the filter rather than
    /// silently dropping the handoff (US2-3).
    fn stage_selection_from_round_trip(&mut self, path: &Path) {
        let parent = path.parent();
        if self.current_dir.as_deref() != parent {
            // Landed on an image outside the currently-loaded folder (feature
            // 017 Phase 4): switch folders first. navigate_to_folder's
            // synchronous scan_directory reset clears pending_select_path, so
            // we set it AFTER calling navigate_to_folder — apply_scan_result
            // will pick it up once the async rescan completes and land the
            // selection there instead of defaulting to images[0].
            if let Some(parent) = parent {
                self.navigate_to_folder(parent);
            }
            self.pending_select_path = Some(path.to_path_buf());
            self.pending_scroll_path = Some(path.to_path_buf());
            let name = file_name_display(path);
            self.status = format!("Round trip landed on {name} — switched to its folder");
            return;
        }
        self.selected = Some(path.to_path_buf());
        self.pending_scroll_path = Some(path.to_path_buf());
        if self.scanning {
            // A rescan of this same folder is already in flight; apply_scan_result's
            // Complete arm would otherwise default to images[0] once it lands, clobbering
            // this selection. Arm pending_select_path so it lands here instead (018 F6).
            self.pending_select_path = Some(path.to_path_buf());
        }
        let (_, indices) = self.compute_list_indices();
        let in_filter = indices
            .iter()
            .any(|&i| self.images[i].path.as_path() == path);
        let name = file_name_display(path);
        if in_filter {
            self.status = format!("Round trip landed on {name}");
        } else {
            self.search.clear();
            self.status =
                format!("Round trip landed on {name} — cleared the active filter to show it");
        }
    }
}
