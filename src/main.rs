// SPDX-License-Identifier: MIT
// rust-feh — Fast lightweight GUI for feh + simple image tools
// All original nfeh / old maintainer code and traces have been archived (see archive/original-nfeh/).

use eframe::{egui, App, Frame};
use rust_feh::image_proc::{process_image, ImageToolsService, ProcessOptions};
use rust_feh::scanner::{scan_images_streaming, ScanResult};
use rust_feh::tool_caps::{feh_spawn_unavailable, DepKind, FormatRoute, ToolCapabilities};
use rust_feh::types::{
    AssetStatus, CacheConfig, FehLaunchEntry, FehLaunchList, FitMode, Filter, ImageEntry,
    ImageOperation, ListViewMode, OutputPolicy, PreparedFastSet, ProcessedResult, ScanInventory,
    SortMode, WindowSizePreset,
};
use rust_feh::ui_logic::{
    add_or_update_asset_in_inventory, apply_converted_detection, apply_rename_pairs,
    build_entry_filelist, clamp_window_size, copy_image_to_clipboard, crop_preview_pixels,
    EntryLaunchState,
    default_tree_expanded, entry_is_launchable, expand_rename_pattern, feh_filelist_temp_path,
    feh_entry_filelist_path, feh_missing_status, feh_not_installed_launch_status,
    file_name_display, file_status_label, prepare_fast_work_dir,
    finalize_scan_entries_fast,
    folder_line_suffix, folder_tree_display_name, format_image_tools_log, format_inventory_bar,
    inventory_magick_hint, is_network_mount_path, join_activity_log, list_indices,
    list_view_mode_label, load_launch_list, post_scan_status, refresh_entry_and_inventory,
    relative_folder, save_launch_list, scan_magick_enabled, showing_count_label, sort_mode_label,
    spawn_job, tree_file_glyph, tree_visible_rows, window_preset_dimensions, window_preset_label,
    write_feh_filelist, write_feh_filelist_to, JobMsg, TreeRow, TreeRowKind, FEH_VIEWER_GEOMETRY,
    FEH_VIEWER_ZOOM,
    WINDOW_MAX_RESIZABLE, WINDOW_MIN_RESIZABLE,
};
use std::collections::HashSet;
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
    Box::new(RustFehApp {
        current_dir: None,
        images: vec![],
        selected: None,
        status,
        debug_logs: vec![],
        search: String::new(),
        prior_search: String::new(),
        recursive: true,
        deep_scan_magick: false,
        feh_available,
        tool_caps,
        scanning: false,
        scroll_generation: 0,
        sort_mode: SortMode::default(),
        prior_sort_mode: SortMode::default(),
        window_size: WindowSizePreset::default(),
        prior_window_size: WindowSizePreset::default(),
        window_resizable: true,
        prior_window_resizable: true,
        scan_inventory: None,
        list_view_mode: ListViewMode::default(),
        tree_expanded_paths: default_tree_expanded(),
        scan_generation: 0,
        scan_rx: None,
        activity_log_detached: false,
        session_status_detached: false,
        deps_detached: false,
        format_discovery_detached: false,
        browse_detached: false,
        image_actions_detached: false,
        feh_instances_detached: false,
        deps_section_open,
        browse_section_open: true,
        image_actions_section_open: true,
        feh_instances_section_open: true,
        activity_log_open: false,
        session_status_open: false,
        format_discovery_open: !tools_panel_ok,
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
        clipboard_context_menu: None,
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

#[derive(Clone)]
struct ClipboardContextMenu {
    image_path: PathBuf,
    anchor_pos: egui::Pos2,
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
    (status, feh_available, tool_caps, deps_section_open, tools_panel_ok)
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
        eprintln!("[rust-feh] e.g. some SSH sessions, broken sockets, or misconfigured Wayland setups).");
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
    scan_inventory: Option<ScanInventory>,
    list_view_mode: ListViewMode,
    tree_expanded_paths: HashSet<String>,
    scan_generation: u64,
    scan_rx: Option<Receiver<ScanMsg>>,
    activity_log_detached: bool,
    session_status_detached: bool,
    deps_detached: bool,
    format_discovery_detached: bool,
    browse_detached: bool,
    image_actions_detached: bool,
    feh_instances_detached: bool,
    activity_log_open: bool,
    session_status_open: bool,
    browse_section_open: bool,
    image_actions_section_open: bool,
    feh_instances_section_open: bool,
    image_tools: ImageToolsService,
    cache_config: CacheConfig,
    tools_panel: ImageToolsPanelState,
    tools_job: Option<ActiveToolsJob>,
    prepared_fast: Option<PreparedFastSet>,
    prepare_fast_temp: Option<PathBuf>,
    launch_entries: FehLaunchList,
    selected_tree_folder: Option<PathBuf>,
    clipboard_context_menu: Option<ClipboardContextMenu>,
    /// Collapsed by default once required dependencies are OK.
    deps_section_open: bool,
    format_discovery_open: bool,
    format_route_open: HashSet<String>,
    /// Dev/test: auto-load `RUST_FEH_START_FOLDER` once on first frame.
    start_folder_loaded: bool,
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
        let total = self.images.len();
        let indices = list_indices(
            &self.images,
            self.current_dir.as_deref(),
            &self.search,
            self.sort_mode,
        );
        (total, indices)
    }

    fn pick_folder(&mut self) {
        if let Some(dir) = rfd::FileDialog::new().pick_folder() {
            self.log(format!("User chose folder: {}", dir.display()));
            self.current_dir = Some(dir.clone());
            self.scan_directory(&dir);
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
        self.current_dir = Some(path.clone());
        self.scan_directory(&path);
    }

    fn feh_button(ui: &mut egui::Ui, label: &str, available: bool, enabled: bool) -> egui::Response {
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
                "Selected: {}. Use Tools → Open in feh or Quick resize.",
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

    fn run_quick_resize_demo(&mut self, path: &Path) {
        let stem = path.file_stem().unwrap_or_default().to_string_lossy();
        let out = path
            .parent()
            .unwrap_or(Path::new("."))
            .join(format!("{stem}_processed.jpg"));
        let opts = ProcessOptions {
            width: None,
            height: None,
            percent: Some(50.0),
            fit: None,
            filter: None,
            target_format: Some("jpg".into()),
            quality: Some(80),
            output_path: Some(out),
        };
        match process_image(path, &opts) {
            Ok(out) => {
                if let Some(ref inv) = self.scan_inventory {
                    let inventory = refresh_entry_and_inventory(
                        &mut self.images,
                        path,
                        inv.non_image_skipped,
                        inv.magick_identify_truncated,
                    );
                    self.scan_inventory = Some(inventory);
                }
                self.status = format!("Processed → {} (inventory updated)", out.display());
                self.log(format!("Resize demo created: {}", out.display()));
            }
            Err(e) => {
                self.status = format!("Process error: {}", e);
                self.log(format!("Resize error: {}", e));
            }
        }
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
        ctx.send_viewport_cmd(egui::ViewportCommand::MinInnerSize(egui::vec2(min_w, min_h)));
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

    fn refresh_tool_caps(&mut self) {
        self.tool_caps = ToolCapabilities::detect();
        self.feh_available = self.tool_caps.feh_available;
        self.deps_section_open = self.tool_caps.has_missing_required();
        if self.tool_caps.has_missing_required() {
            self.format_discovery_open = true;
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
        ui.small(format!("{segment} is in a separate window. Close it with X to return here."));
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
        Self::header_with_detach_suffix(base, self.activity_log_detached)
    }

    fn deps_header_label(&self) -> String {
        let base = if !self.tool_caps.has_missing_required() {
            "✅ Dependencies — all required tools OK".to_string()
        } else {
            "⚠ Dependencies — action needed".to_string()
        };
        Self::header_with_detach_suffix(base, self.deps_detached)
    }

    fn format_discovery_header_label(&self) -> String {
        let routes = self.tool_caps.format_routes();
        let base = format!("Format discovery — {} groups", routes.len());
        Self::header_with_detach_suffix(base, self.format_discovery_detached)
    }

    fn render_inspector_activity_log(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        if self.activity_log_detached {
            Self::render_detached_placeholder(ui, "Activity log");
            return;
        }

        if Self::render_segment_detach_toolbar(
            ui,
            "Scan events, feh commands, warnings",
            "Detach window",
        ) {
            self.activity_log_detached = true;
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
        if self.deps_detached {
            Self::render_detached_placeholder(ui, "Dependencies");
            return;
        }

        if Self::render_segment_detach_toolbar(
            ui,
            "feh, ImageMagick, and other PATH tools",
            "Detach window",
        ) {
            self.deps_detached = true;
        }
        self.render_deps_section_body(ui, ctx);
    }

    fn render_format_discovery_body(&mut self, ui: &mut egui::Ui) {
        ui.small(
            "Scan = native listed or magick-detected; View = feh; Resize = quick resize demo.",
        );
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
        if self.format_discovery_detached {
            Self::render_detached_placeholder(ui, "Format discovery");
            return;
        }

        if Self::render_segment_detach_toolbar(
            ui,
            "Per-format scan, view, and resize routing",
            "Detach window",
        ) {
            self.format_discovery_detached = true;
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
        Self::header_with_detach_suffix(base, self.browse_detached)
    }

    fn render_inspector_browse(&mut self, ui: &mut egui::Ui) {
        if self.browse_detached {
            Self::render_detached_placeholder(ui, "Browse");
            return;
        }

        if Self::render_segment_detach_toolbar(
            ui,
            "Folder, filter, sort, and list view mode",
            "Detach window",
        ) {
            self.browse_detached = true;
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
        Self::header_with_detach_suffix(base, self.image_actions_detached)
    }

    fn render_image_actions_body(&mut self, ui: &mut egui::Ui) {
        let has_folder = self.current_dir.is_some();
        let has_selection = self.selected.is_some();
        let feh_ready = self.feh_open_ready();

        if Self::feh_button(ui, "Open in feh", self.feh_available, feh_ready).clicked() {
            self.log("User clicked 'Open in feh' (inspector)");
            self.try_open_in_feh();
        }
        if ui
            .add_enabled(
                has_folder && has_selection,
                egui::Button::new("Quick resize 50% (demo)"),
            )
            .clicked()
        {
            if let Some(path) = self.selected.clone() {
                self.log("User clicked resize demo (inspector)");
                self.run_quick_resize_demo(&path);
            }
        }
    }

    fn render_inspector_image_actions(&mut self, ui: &mut egui::Ui) {
        if self.image_actions_detached {
            Self::render_detached_placeholder(ui, "Image actions");
            return;
        }

        if Self::render_segment_detach_toolbar(
            ui,
            "Open selected image in feh or run quick resize",
            "Detach window",
        ) {
            self.image_actions_detached = true;
        }
        self.render_image_actions_body(ui);
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
        for image in &self.images {
            if let Some(parent) = image.path.parent() {
                if seen.insert(parent.to_path_buf()) {
                    folders.push(parent.to_path_buf());
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
        Self::header_with_detach_suffix(base, self.feh_instances_detached)
    }

    fn persist_launch_entries(&mut self) {
        if let Err(e) = save_launch_list(&self.launch_entries) {
            self.log(format!("Failed to save launch entries: {e}"));
            self.status = e;
        }
    }

    fn add_launch_entry(&mut self) {
        let folder = self.resolve_default_entry_folder();
        let created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let id = format!(
            "{created_at:x}-{}",
            self.launch_entries.entries.len()
        );
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
        let state = entry_is_launchable(entry, &self.images, self.feh_available);
        if !state.launchable {
            self.status = format!("Cannot launch: {}", state.status);
            return;
        }
        let paths = build_entry_filelist(entry, &self.images);
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
            format!(
                "Spawning feh for entry {} ({count} images)",
                entry.id
            ),
            format!("Launched feh on {}", state.status),
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
            .filter(|e| entry_is_launchable(e, &self.images, self.feh_available).launchable)
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
                .any(|e| entry_is_launchable(e, &self.images, feh_available).launchable);
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
                    let state = entry_is_launchable(entry, &self.images, feh_available);
                    Self::render_feh_entry_card(ui, idx, entry, &state, &candidates, &mut action);
                }
            });
        if let Some(action) = action {
            self.apply_feh_entry_action(action);
        }
    }

    fn render_inspector_feh_instances(&mut self, ui: &mut egui::Ui) {
        if self.feh_instances_detached {
            Self::render_detached_placeholder(ui, "Feh instances");
            return;
        }
        if Self::render_segment_detach_toolbar(
            ui,
            "Manage multiple feh launch configurations",
            "Detach window",
        ) {
            self.feh_instances_detached = true;
        }
        self.render_feh_instances_body(ui);
    }

    fn open_clipboard_context_menu(&mut self, image_path: PathBuf, anchor_pos: egui::Pos2) {
        self.clipboard_context_menu = Some(ClipboardContextMenu {
            image_path,
            anchor_pos,
        });
    }

    fn render_clipboard_context_menu(&mut self, ctx: &egui::Context) {
        let Some(menu) = self.clipboard_context_menu.clone() else {
            return;
        };

        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.clipboard_context_menu = None;
            return;
        }
        if ctx.input(|i| i.pointer.primary_clicked()) {
            if let Some(pos) = ctx.input(|i| i.pointer.interact_pos()) {
                let popup_rect = egui::Rect::from_min_size(menu.anchor_pos, egui::vec2(220.0, 44.0));
                if !popup_rect.contains(pos) {
                    self.clipboard_context_menu = None;
                    return;
                }
            }
        }

        egui::Area::new(egui::Id::new("clipboard_context_menu"))
            .order(egui::Order::Foreground)
            .fixed_pos(menu.anchor_pos)
            .show(ctx, |ui| {
                egui::Frame::popup(ui.style()).show(ui, |ui| {
                    if ui.button("📋 Copy image to clipboard").clicked() {
                        match copy_image_to_clipboard(&menu.image_path) {
                            Ok(status) => {
                                self.status = status.clone();
                                self.log(status);
                            }
                            Err(err) => {
                                self.status = err.clone();
                                self.log(format!("Clipboard copy failed: {err}"));
                            }
                        }
                        self.clipboard_context_menu = None;
                    }
                });
            });
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
        self.log(format!("Prepare Fast job started for {} images", paths.len()));
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
                    self.status = format!(
                        "Batch {}/{}: {}",
                        p.current + 1,
                        p.total,
                        p.message
                    );
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
                    self.status = format!(
                        "Pre-cache {}/{}: {}",
                        p.current + 1,
                        p.total,
                        p.message
                    );
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
                    self.status = format!(
                        "Pre-cache done: {}/{} cached",
                        job.precache_ok, job.total
                    );
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
                    self.status = format!(
                        "Prepare Fast {}/{}: {}",
                        p.current + 1,
                        p.total,
                        p.message
                    );
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
                if outcome.rolled_back { " (rolled back)" } else { "" }
            ));
            self.status = format!("Rename failed: {e}");
        } else {
            for (old, dest) in &outcome.applied {
                if let Some(e) = self.images.iter_mut().find(|e| e.path == *old) {
                    e.path = dest.clone();
                }
                self.log(format!("Renamed {} -> {}", old.display(), dest.display()));
            }
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

    fn render_tools_crop_preview(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.label("Geometry WxH+X+Y:");
        if ui
            .text_edit_singleline(&mut self.tools_panel.crop_geometry)
            .changed()
        {
            self.tools_panel.last_crop_key.clear();
        }
        let Some(sel) = &self.selected else {
            return;
        };
        let key = format!("{}:{}", sel.display(), self.tools_panel.crop_geometry);
        if self.tools_panel.last_crop_key != key {
            if let Ok(px) = crop_preview_pixels(sel, &self.tools_panel.crop_geometry, 256) {
                let img = egui::ColorImage::from_rgba_unmultiplied(
                    [px.width as usize, px.height as usize],
                    &px.rgba,
                );
                self.tools_panel.crop_texture = Some(ctx.load_texture(
                    "crop_preview",
                    img,
                    egui::TextureOptions::LINEAR,
                ));
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

    fn render_tools_single_batch_section(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
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
            ToolsSingleOp::Crop => self.render_tools_crop_preview(ui, ctx),
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
            .add_enabled(!paths.is_empty() && !job_busy, egui::Button::new("Pre-cache folder"))
            .clicked()
        {
            self.tools_start_precache_job();
        }
        if ui
            .add_enabled(!paths.is_empty() && !job_busy, egui::Button::new("Prepare Fast feh"))
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
                .add_enabled(self.feh_available, egui::Button::new("Launch feh on optimized"))
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
        let ok = self.tools_panel.rename_error.is_none()
            && !self.tools_panel.rename_preview.is_empty();
        if ui.add_enabled(ok, egui::Button::new("Apply rename…")).clicked() {
            self.tools_panel.rename_confirm_open = true;
        }
        if self.tools_panel.rename_confirm_open && ok && ui.button("Confirm rename").clicked() {
            self.tools_apply_rename();
        }
    }

    fn render_tools_action_buttons(&mut self, ui: &mut egui::Ui) {
        ui.separator();
        match self.tools_panel.section {
            ToolsSection::Single => {
                if ui
                    .add_enabled(self.selected.is_some(), egui::Button::new("Apply"))
                    .clicked()
                {
                    if let Some(sel) = self.selected.clone() {
                        self.tools_apply_single(&sel);
                    }
                }
            }
            ToolsSection::Batch => self.render_tools_batch_actions(ui),
            ToolsSection::Rename => self.render_tools_rename_actions(ui),
            ToolsSection::Cache => {}
        }
    }

    fn render_inspector_image_tools(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.vertical(|ui| {
            ui.label(egui::RichText::new("Image Tools").strong());
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.tools_panel.section, ToolsSection::Single, "Single");
                ui.selectable_value(&mut self.tools_panel.section, ToolsSection::Batch, "Batch");
                ui.selectable_value(&mut self.tools_panel.section, ToolsSection::Rename, "Rename");
                ui.selectable_value(&mut self.tools_panel.section, ToolsSection::Cache, "Cache");
            });
            ui.separator();
            match self.tools_panel.section {
                ToolsSection::Single | ToolsSection::Batch => {
                    self.render_tools_single_batch_section(ui, ctx);
                }
                ToolsSection::Rename => self.render_tools_rename_section(ui),
                ToolsSection::Cache => self.render_tools_cache_section(ui),
            }
            self.render_tools_job_status(ui);
            self.render_tools_action_buttons(ui);
        });
    }

    fn render_browse_controls_body(&mut self, ui: &mut egui::Ui) {
        let has_folder = self.current_dir.is_some();

        ui.vertical(|ui| {
            if ui.button("Choose folder").clicked() {
                self.pick_folder();
            }

            if let Some(dir) = &self.current_dir {
                ui.add(
                    egui::Label::new(dir.display().to_string())
                        .selectable(true)
                        .wrap_mode(egui::TextWrapMode::Wrap),
                );
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
                    if let Some(d) = self.current_dir.clone() {
                        self.scan_directory(&d);
                    }
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
                    if let Some(d) = self.current_dir.clone() {
                        self.scan_directory(&d);
                    }
                }

                if ui.add_enabled(has_folder, egui::Button::new("Rescan")).clicked() {
                    if let Some(d) = self.current_dir.clone() {
                        self.scan_directory(&d);
                    }
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

            ui.horizontal(|ui| {
                ui.label("View:");
                ui.add_enabled_ui(has_folder, |ui| {
                    ui.horizontal(|ui| {
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
                });
            });
        });
    }

    fn render_inspector_panel(&mut self, ui: &mut egui::Ui, ctx: &egui::Context, time: f64) {
        let (total, filtered) = self.compute_list_indices();
        let shown = filtered.len();

        ui.heading("Inspector");
        ui.label("Browse, image actions, session, and format routing.");
        ui.separator();

        let browse_header = self.browse_header_label();
        let browse_response = egui::CollapsingHeader::new(browse_header)
            .id_salt("inspector_browse")
            .open(Some(self.browse_section_open))
            .show(ui, |ui| {
                self.render_inspector_browse(ui);
            });
        if browse_response.header_response.clicked() {
            self.browse_section_open = !self.browse_section_open;
        }

        let actions_header = self.image_actions_header_label();
        let actions_response = egui::CollapsingHeader::new(actions_header)
            .id_salt("inspector_image_actions")
            .open(Some(self.image_actions_section_open))
            .show(ui, |ui| {
                self.render_inspector_image_actions(ui);
                ui.separator();
                self.render_inspector_image_tools(ui, ctx);
            });
        if actions_response.header_response.clicked() {
            self.image_actions_section_open = !self.image_actions_section_open;
        }

        let feh_instances_header = self.feh_instances_header_label();
        let feh_instances_response = egui::CollapsingHeader::new(feh_instances_header)
            .id_salt("inspector_feh_instances")
            .open(Some(self.feh_instances_section_open))
            .show(ui, |ui| {
                self.render_inspector_feh_instances(ui);
            });
        if feh_instances_response.header_response.clicked() {
            self.feh_instances_section_open = !self.feh_instances_section_open;
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
            .open(Some(self.session_status_open))
            .show(ui, |ui| {
                if self.scanning && !self.session_status_detached {
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
            self.session_status_open = !self.session_status_open;
        }
        if self.scanning && !self.session_status_open {
            self.session_status_open = true;
        }

        let log_header = self.activity_log_header_label();
        let log_response = egui::CollapsingHeader::new(log_header)
            .id_salt("inspector_activity_log")
            .open(Some(self.activity_log_open))
            .show(ui, |ui| {
                self.render_inspector_activity_log(ui, ctx);
            });
        if log_response.header_response.clicked() {
            self.activity_log_open = !self.activity_log_open;
        }

        let deps_header = self.deps_header_label();
        let deps_response = egui::CollapsingHeader::new(deps_header)
            .id_salt("tool_deps")
            .open(Some(self.deps_section_open))
            .show(ui, |ui| {
                self.render_inspector_dependencies(ui, ctx);
            });
        if deps_response.header_response.clicked() {
            self.deps_section_open = !self.deps_section_open;
        }

        let fd_header = self.format_discovery_header_label();
        let fd_response = egui::CollapsingHeader::new(fd_header)
            .id_salt("tool_format_discovery")
            .open(Some(self.format_discovery_open))
            .show(ui, |ui| {
                self.render_inspector_format_discovery(ui);
            });
        if fd_response.header_response.clicked() {
            self.format_discovery_open = !self.format_discovery_open;
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

        if self.session_status_detached {
            label = Self::header_with_detach_suffix(label, true);
        }

        let mut rich = egui::RichText::new(label);
        if self.scanning && !self.session_status_detached {
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
        let viewport_w = ctx.input(|i| {
            i.viewport()
                .inner_rect
                .map(|r| r.width())
                .unwrap_or(720.0)
        });
        // Inspector must not exceed the central image-list panel (each gets at least half).
        (viewport_w * 0.5).max(260.0)
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
                                .wrap_mode(egui::TextWrapMode::Wrap),
                        );
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
                            egui::Label::new(tip)
                                .selectable(true)
                                .wrap_mode(egui::TextWrapMode::Wrap),
                        );
                    });
                });
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
        if self.session_status_detached {
            Self::render_detached_placeholder(ui, "Session status");
            return;
        }

        if Self::render_segment_detach_toolbar(
            ui,
            "Image count, current status, operation speed tips",
            "Detach window",
        ) {
            self.session_status_detached = true;
        }
        self.render_session_status_body(ui, ctx, shown, total, time);
    }

    fn render_detached_inspector_windows(&mut self, ctx: &egui::Context) {
        let time = ctx.input(|i| i.time);
        let (total, filtered) = self.compute_list_indices();
        let shown = filtered.len();

        if self.browse_detached {
            let mut open = self.browse_detached;
            egui::Window::new("Browse")
                .open(&mut open)
                .collapsible(true)
                .resizable(true)
                .default_width(520.0)
                .show(ctx, |ui| {
                    self.render_browse_controls_body(ui);
                });
            self.browse_detached = open;
        }

        if self.image_actions_detached {
            let mut open = self.image_actions_detached;
            egui::Window::new("Image actions")
                .open(&mut open)
                .collapsible(true)
                .resizable(true)
                .default_width(360.0)
                .show(ctx, |ui| {
                    self.render_image_actions_body(ui);
                });
            self.image_actions_detached = open;
        }

        if self.feh_instances_detached {
            let mut open = self.feh_instances_detached;
            egui::Window::new("Feh instances")
                .open(&mut open)
                .collapsible(true)
                .resizable(true)
                .default_width(420.0)
                .show(ctx, |ui| {
                    self.render_feh_instances_body(ui);
                });
            self.feh_instances_detached = open;
        }

        if self.session_status_detached {
            let mut open = self.session_status_detached;
            let pulse_fill = if self.scanning {
                Self::activity_pulse_color(time, true)
            } else {
                egui::Color32::TRANSPARENT
            };
            egui::Window::new("Session status")
                .open(&mut open)
                .collapsible(true)
                .resizable(true)
                .default_width(480.0)
                .show(ctx, |ui| {
                    egui::Frame::none().fill(pulse_fill).show(ui, |ui| {
                        self.render_session_status_body(ui, ctx, shown, total, time);
                    });
                });
            self.session_status_detached = open;
        }

        if self.activity_log_detached {
            let mut open = self.activity_log_detached;
            egui::Window::new("Activity log")
                .open(&mut open)
                .collapsible(true)
                .resizable(true)
                .default_width(520.0)
                .show(ctx, |ui| {
                    self.render_activity_log_body(ui, ctx);
                });
            self.activity_log_detached = open;
        }

        if self.deps_detached {
            let mut open = self.deps_detached;
            egui::Window::new("Dependencies")
                .open(&mut open)
                .collapsible(true)
                .resizable(true)
                .default_width(420.0)
                .show(ctx, |ui| {
                    self.render_deps_section_body(ui, ctx);
                });
            self.deps_detached = open;
        }

        if self.format_discovery_detached {
            let mut open = self.format_discovery_detached;
            egui::Window::new("Format discovery")
                .open(&mut open)
                .collapsible(true)
                .resizable(true)
                .default_width(480.0)
                .show(ctx, |ui| {
                    self.render_format_discovery_body(ui);
                });
            self.format_discovery_detached = open;
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
            self.log(format!(
                "Window size set to {}",
                window_preset_label(self.window_size)
            ));
        }
        if self.window_resizable != self.prior_window_resizable {
            self.prior_window_resizable = self.window_resizable;
            let lock_size = self.current_viewport_size(ctx);
            self.apply_window_resize_policy(ctx, lock_size);
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
        if ui.checkbox(&mut self.recursive, "Include subfolders").changed() {
            ui.close_menu();
            self.rescan_current_folder_if_any();
        }
        if ui
            .checkbox(
                &mut self.deep_scan_magick,
                "Detect exotic formats (slow)",
            )
            .changed()
        {
            ui.close_menu();
            self.rescan_current_folder_if_any();
        }
        ui.menu_button("Window size", |ui| {
            for preset in [
                WindowSizePreset::Compact,
                WindowSizePreset::Default,
                WindowSizePreset::Large,
            ] {
                if ui
                    .selectable_value(
                        &mut self.window_size,
                        preset,
                        window_preset_label(preset),
                    )
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
                ui.menu_button("File", |ui| {
                    if ui.button("Choose folder...").clicked() {
                        ui.close_menu();
                        self.pick_folder();
                    }
                    if ui.button("Rescan").clicked() {
                        ui.close_menu();
                        self.rescan_current_folder_if_any();
                    }
                });
                ui.menu_button("View", |ui| self.render_view_menu(ui));
            });
        });
    }

    fn render_inspector_side_panel(&mut self, ctx: &egui::Context) {
        let inspector_max_w = Self::inspector_max_width(ctx);
        egui::SidePanel::right("inspector")
            .resizable(true)
            .default_width(320.0_f32.min(inspector_max_w))
            .min_width(260.0)
            .max_width(inspector_max_w)
            .show(ctx, |ui| {
                ui.set_max_width(inspector_max_w);
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        let time = ctx.input(|i| i.time);
                        self.render_inspector_panel(ui, ctx, time);
                    });
            });
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

    fn handle_image_row_click(
        &mut self,
        response: &egui::Response,
        path: PathBuf,
    ) {
        if response.secondary_clicked() {
            let anchor_pos = response
                .interact_pointer_pos()
                .unwrap_or(response.rect.left_bottom());
            self.open_clipboard_context_menu(path.clone(), anchor_pos);
        }
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
        let is_selected = self.selected.as_ref() == Some(&path);
        ui.horizontal(|ui| {
            ui.allocate_ui(egui::vec2(metrics.folder_col_w, metrics.row_h), |ui| {
                ui.label(egui::RichText::new(folder).weak());
            });
            let response = ui.selectable_label(is_selected, &name);
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
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .max_height(metrics.list_height)
            .id_salt(self.scroll_generation)
            .show_rows(ui, metrics.row_h, filtered.len(), |ui, row_range| {
                for row in row_range {
                    if row >= filtered.len() {
                        break;
                    }
                    self.render_flat_list_row(ui, filtered[row], list_root, metrics);
                }
            });
    }

    fn toggle_tree_folder(&mut self, folder_path: &str) {
        self.selected_tree_folder = Some(PathBuf::from(folder_path));
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

    fn render_tree_file_row(
        &mut self,
        ui: &mut egui::Ui,
        tree_row: &TreeRow,
        indent: f32,
    ) {
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
        let label = if self.images[idx].status == rust_feh::types::FileStatus::Converted {
            format!("{glyph} {name}  [{status}]")
        } else {
            format!("{glyph} {name}")
        };
        let is_selected = self.selected.as_ref() == Some(&path);
        ui.horizontal(|ui| {
            ui.add_space(indent);
            let response = ui.selectable_label(is_selected, label);
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
        let tree_rows = tree_visible_rows(
            &self.images,
            list_root,
            &self.search,
            self.sort_mode,
            &self.tree_expanded_paths,
            root_skipped,
        );
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
    }

    fn render_central_image_panel(
        &mut self,
        ctx: &egui::Context,
        filtered: &[usize],
        list_root: Option<&Path>,
    ) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::Frame::group(ui.style())
                .inner_margin(6.0)
                .show(ui, |ui| {
                    ui.label("Images (filter matches folder or filename; click a row to select):");
                });
            self.render_scan_inventory_banner(ui, list_root);

            let row_h = 18.0;
            let header_h = row_h + ui.spacing().item_spacing.y;
            let inventory_h = if self.scan_inventory.is_some() {
                120.0
            } else {
                0.0
            };
            let total_w = ui.available_width();
            let metrics = ImageListMetrics {
                list_height: (ui.available_height() - header_h - inventory_h).max(row_h * 4.0),
                folder_col_w: total_w * 0.35,
                status_col_w: total_w * 0.25,
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
        });
    }

    fn request_repaint_if_busy(&self, ctx: &egui::Context) {
        let busy = self.is_activity_busy();
        let tip_animating = !self.tool_caps.operation_timings().is_empty();
        if busy || tip_animating {
            ctx.request_repaint_after(std::time::Duration::from_millis(200));
        }
    }
}

impl App for RustFehApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        self.maybe_load_start_folder();
        self.poll_scan_complete(ctx);
        self.poll_tools_job(ctx);
        Self::emit_startup_notice_once();
        self.sync_frame_input_state(ctx);

        self.render_top_menu_bar(ctx);
        self.render_inspector_side_panel(ctx);

        // Recompute after menus/inspector — Rescan clears images mid-frame.
        let list_root = self.current_dir.clone();
        let (_total, filtered) = self.compute_list_indices();
        self.render_central_image_panel(ctx, &filtered, list_root.as_deref());

        self.render_clipboard_context_menu(ctx);
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
            "Selected: {}. Use Tools → Open in feh or Quick resize.",
            disp
        );
    }

    fn apply_scan_partial(&mut self, entries: Vec<ImageEntry>, skipped: usize) {
        self.images = entries;
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

    fn poll_scan_complete(&mut self, ctx: &egui::Context) {
        let mut messages = Vec::new();
        if let Some(rx) = &self.scan_rx {
            while let Ok(msg) = rx.try_recv() {
                messages.push(msg);
            }
        }
        let mut still_scanning = self.scanning;
        for msg in messages {
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
                    still_scanning = false;
                    self.scanning = false;
                }
                ScanMsg::Converted {
                    generation,
                    entries,
                    non_image_skipped,
                    magick_truncated,
                } if generation == self.scan_generation => {
                    let inventory =
                        ScanInventory::from_entries(&entries, non_image_skipped, magick_truncated);
                    self.images = entries;
                    self.scan_inventory = Some(inventory);
                    self.log("Converted-status metadata updated (background)");
                }
                _ => {}
            }
        }
        self.scanning = still_scanning;
        if !still_scanning && self.scan_rx.is_some() {
            // Keep rx open for Converted messages until folder changes.
        }
        if self.scanning {
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
        self.scan_inventory = None;
        self.tree_expanded_paths = default_tree_expanded();
        self.scan_generation = self.scan_generation.wrapping_add(1);
        self.scroll_generation = self.scroll_generation.wrapping_add(1);
        let generation = self.scan_generation;

        let dir_path = dir.to_path_buf();
        let recursive = self.recursive;
        let on_network = is_network_mount_path(dir);
        let magick_identify = self.deep_scan_magick
            && scan_magick_enabled(self.tool_caps.magick_available, dir);
        let dir_label = dir.display().to_string();
        let (tx, rx) = mpsc::channel();
        self.scan_rx = Some(rx);

        thread::spawn(move || {
            let result = scan_images_streaming(&dir_path, recursive, magick_identify, |entries, skipped, _| {
                let _ = tx.send(ScanMsg::Partial {
                    generation,
                    entries: entries.to_vec(),
                    skipped,
                });
            });
            let skipped = result.inventory.non_image_skipped;
            let truncated = result.inventory.magick_identify_truncated;
            let mut entries = result.entries.clone();
            let _ = tx.send(ScanMsg::Complete {
                generation,
                dir_label: dir_label.clone(),
                result,
            });
            thread::spawn(move || {
                apply_converted_detection(&mut entries);
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
        } else {
            let p = self.images[0].path.clone();
            self.selected = Some(p.clone());
            self.status = post_scan_status(
                &format!(
                    "Loaded {} images — Open in feh to view.",
                    self.images.len()
                ),
                self.feh_available,
            );
            self.log(format!("Auto-selected first image: {}", p.display()));
        }
    }

    fn open_in_feh(&mut self, path: &Path) {
        let (_, indices) = self.compute_list_indices();
        if indices.is_empty() {
            self.status = "No images in filtered list".to_owned();
            return;
        }
        if !indices.iter().any(|&i| self.images[i].path.as_path() == path) {
            self.status = "Selected image is not in the filtered filelist".to_owned();
            return;
        }

        let paths: Vec<&Path> = indices
            .iter()
            .map(|&i| self.images[i].path.as_path())
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

        self.spawn_feh_viewer(
            &list_path,
            path,
            format!("Spawning feh with filelist ({count} images)"),
            format!("Launched feh on {}", path.display()),
        );
    }

}