// SPDX-License-Identifier: MIT
// rust-feh — Fast lightweight GUI for feh + simple image tools
// All original nfeh / old maintainer code and traces have been archived (see archive/original-nfeh/).

use eframe::{egui, App, Frame};
use rust_feh::image_proc::{process_image, ProcessOptions};
use rust_feh::scanner::{scan_images, ScanResult};
use rust_feh::tool_caps::{feh_spawn_unavailable, DepKind, FormatRoute, ToolCapabilities};
use rust_feh::types::{ImageEntry, ListViewMode, ScanInventory, SortMode, WindowSizePreset};
use rust_feh::ui_logic::{
    clamp_window_size, feh_missing_status, file_name_display, file_status_label,
    finalize_scan_entries, folder_line_suffix, folder_tree_display_name, format_inventory_bar,
    refresh_entry_and_inventory,
    feh_filelist_temp_path, inventory_magick_hint, is_network_mount_path, join_activity_log,
    scan_magick_enabled,
    list_indices,
    list_view_mode_label, post_scan_status, relative_folder, showing_count_label, sort_mode_label,
    tree_file_glyph, tree_visible_rows, write_feh_filelist, TreeRowKind,
    default_tree_expanded, window_preset_dimensions, window_preset_label, FEH_VIEWER_GEOMETRY,
    FEH_VIEWER_ZOOM, WINDOW_MAX_RESIZABLE, WINDOW_MIN_RESIZABLE,
};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::mpsc::{self, Receiver};
use std::sync::Once;
use std::thread;

static STARTUP_NOTICE: Once = Once::new();

fn main() -> eframe::Result<()> {
    let (w, h) = window_preset_dimensions(WindowSizePreset::default());
    let (min_w, min_h) = WINDOW_MIN_RESIZABLE;
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([w, h])
            .with_min_inner_size([min_w, min_h])
            .with_resizable(true)
            .with_title("rust-feh"),
        ..Default::default()
    };

    let tool_caps = ToolCapabilities::detect();
    let feh_available = tool_caps.feh_available;
    let deps_section_open = tool_caps.has_missing_required();
    let tools_panel_ok = !tool_caps.has_missing_required();
    let status = if feh_available {
        String::new()
    } else {
        "feh not found — install with `sudo apt install feh`".to_string()
    };

    eframe::run_native(
        "rust-feh",
        options,
        Box::new(move |_cc| -> std::result::Result<_, Box<dyn std::error::Error + Send + Sync>> {
            Ok(Box::new(RustFehApp {
                current_dir: None,
                images: vec![],
                selected: None,
                status,
                debug_logs: vec![],
                search: String::new(),
                prior_search: String::new(),
                recursive: true,
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
                deps_section_open,
                browse_section_open: true,
                image_actions_section_open: true,
                activity_log_open: false,
                session_status_open: false,
                format_discovery_open: !tools_panel_ok,
                format_route_open: HashSet::new(),
            }) as Box<dyn App>)
        }),
    )?;
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
    scan_rx: Option<Receiver<ScanComplete>>,
    activity_log_detached: bool,
    session_status_detached: bool,
    deps_detached: bool,
    format_discovery_detached: bool,
    browse_detached: bool,
    image_actions_detached: bool,
    activity_log_open: bool,
    session_status_open: bool,
    browse_section_open: bool,
    image_actions_section_open: bool,
    /// Collapsed by default once required dependencies are OK.
    deps_section_open: bool,
    format_discovery_open: bool,
    format_route_open: HashSet<String>,
}

struct ScanComplete {
    generation: u64,
    dir_label: String,
    result: ScanResult,
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
        self.feh_available && !self.scanning && !self.compute_list_indices().1.is_empty()
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
        if self.scanning {
            self.status = "Scan in progress — wait for load to finish".to_owned();
            return;
        }
        let Some(path) = self.resolve_feh_start_path() else {
            self.status = "No images in filtered list".to_owned();
            return;
        };
        self.open_in_feh(&path);
    }

    fn run_quick_resize_demo(&mut self, path: &Path) {
        let opts = ProcessOptions {
            width: None,
            height: None,
            percent: Some(50.0),
            target_format: Some("jpg".into()),
            quality: Some(80),
            output_dir: None,
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
            });
        if actions_response.header_response.clicked() {
            self.image_actions_section_open = !self.image_actions_section_open;
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
}

impl App for RustFehApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        self.poll_scan_complete(ctx);

        STARTUP_NOTICE.call_once(|| {
            eprintln!(
                "[rust-feh] App started. Use 'Choose folder' to load images. Debug logs appear in the UI after actions."
            );
        });

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

        let list_root = self.current_dir.clone();

        egui::TopBottomPanel::top("controls").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Choose folder...").clicked() {
                        ui.close_menu();
                        self.pick_folder();
                    }
                    if ui.button("Rescan").clicked() {
                        ui.close_menu();
                        if let Some(d) = self.current_dir.clone() {
                            self.scan_directory(&d);
                        }
                    }
                });
                ui.menu_button("View", |ui| {
                    let changed = ui.checkbox(&mut self.recursive, "Include subfolders").changed();
                    if changed {
                        ui.close_menu();
                        if let Some(d) = self.current_dir.clone() {
                            self.scan_directory(&d);
                        }
                    }

                    ui.menu_button("Window size", |ui| {
                        if ui
                            .selectable_value(
                                &mut self.window_size,
                                WindowSizePreset::Compact,
                                window_preset_label(WindowSizePreset::Compact),
                            )
                            .clicked()
                        {
                            ui.close_menu();
                        }
                        if ui
                            .selectable_value(
                                &mut self.window_size,
                                WindowSizePreset::Default,
                                window_preset_label(WindowSizePreset::Default),
                            )
                            .clicked()
                        {
                            ui.close_menu();
                        }
                        if ui
                            .selectable_value(
                                &mut self.window_size,
                                WindowSizePreset::Large,
                                window_preset_label(WindowSizePreset::Large),
                            )
                            .clicked()
                        {
                            ui.close_menu();
                        }
                    });

                    if ui
                        .checkbox(&mut self.window_resizable, "Resizable window")
                        .changed()
                    {
                        ui.close_menu();
                    }
                });
            });
        });

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

        // Recompute after menus/inspector — Rescan clears images mid-frame.
        let (_total, filtered) = self.compute_list_indices();

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::Frame::group(ui.style())
                .inner_margin(6.0)
                .show(ui, |ui| {
                    ui.label("Images (filter matches folder or filename; click a row to select):");
                });

            if let (Some(ref inv), Some(ref dir)) = (&self.scan_inventory, &self.current_dir) {
                egui::Frame::group(ui.style())
                    .inner_margin(6.0)
                    .show(ui, |ui| {
                        ui.strong("Scan inventory");
                        for line in format_inventory_bar(inv, &dir.display().to_string()) {
                            ui.label(line);
                        }
                        if let Some(hint) = inventory_magick_hint(
                            self.tool_caps.magick_available,
                            list_root.as_deref(),
                        ) {
                            ui.small(hint);
                        }
                        if inv.magick_identify_truncated {
                            ui.small("ImageMagick identify capped at 500 files this scan.");
                        }
                    });
                ui.add_space(4.0);
            }

            let row_h = 18.0;
            let header_h = row_h + ui.spacing().item_spacing.y;
            let inventory_h = if self.scan_inventory.is_some() {
                120.0
            } else {
                0.0
            };
            let list_height =
                (ui.available_height() - header_h - inventory_h).max(row_h * 4.0);
            let total_w = ui.available_width();
            let folder_col_w = total_w * 0.35;
            let status_col_w = total_w * 0.25;

            egui::Frame::group(ui.style())
                .inner_margin(4.0)
                .show(ui, |ui| {
            if self.list_view_mode == ListViewMode::FlatList {
                ui.horizontal(|ui| {
                    ui.allocate_ui(egui::vec2(folder_col_w, row_h), |ui| {
                        ui.strong("Folder");
                    });
                    ui.strong("Filename");
                    ui.allocate_ui(egui::vec2(status_col_w, row_h), |ui| {
                        ui.strong("Status");
                    });
                });

                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .max_height(list_height)
                    .id_salt(self.scroll_generation)
                    .show_rows(ui, row_h, filtered.len(), |ui, row_range| {
                        for row in row_range {
                            if row >= filtered.len() {
                                break;
                            }
                            let i = filtered[row];
                            if i >= self.images.len() {
                                continue;
                            }
                            let path = self.images[i].path.clone();
                            let folder = relative_folder(list_root.as_deref(), &path);
                            let name = file_name_display(&path);
                            let status = file_status_label(self.images[i].status);

                            let is_selected = self.selected.as_ref() == Some(&path);
                            ui.horizontal(|ui| {
                                ui.allocate_ui(egui::vec2(folder_col_w, row_h), |ui| {
                                    ui.label(egui::RichText::new(folder).weak());
                                });
                                if ui.selectable_label(is_selected, &name).clicked() {
                                    self.select_image(path);
                                }
                                ui.allocate_ui(egui::vec2(status_col_w, row_h), |ui| {
                                    ui.small(status);
                                });
                            });
                        }
                    });
            } else {
                let root_skipped = self
                    .scan_inventory
                    .as_ref()
                    .map(|i| i.non_image_skipped)
                    .unwrap_or(0);
                let tree_rows = tree_visible_rows(
                    &self.images,
                    list_root.as_deref(),
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
                                    let prefix = if tree_row.expanded { "▼" } else { "▶" };
                                    let name = folder_tree_display_name(
                                        &tree_row.folder_path,
                                        list_root.as_deref(),
                                    );
                                    let suffix = folder_line_suffix(
                                        tree_row.listed,
                                        tree_row.magick,
                                        tree_row.skipped,
                                    );
                                    let label = format!("{prefix} {name}  — {suffix}");
                                    if ui
                                        .selectable_label(false, label)
                                        .clicked()
                                    {
                                        let path_key = tree_row.folder_path.clone();
                                        if self.tree_expanded_paths.contains(&path_key) {
                                            self.tree_expanded_paths.remove(&path_key);
                                        } else {
                                            self.tree_expanded_paths.insert(path_key);
                                        }
                                        self.scroll_generation =
                                            self.scroll_generation.wrapping_add(1);
                                    }
                                }
                                TreeRowKind::File => {
                                    let Some(idx) = tree_row.entry_index else {
                                        continue;
                                    };
                                    let path = self.images[idx].path.clone();
                                    let name = file_name_display(&path);
                                    let glyph = tree_file_glyph(self.images[idx].status);
                                    let status = file_status_label(self.images[idx].status);
                                    let label = if self.images[idx].status
                                        == rust_feh::types::FileStatus::Converted
                                    {
                                        format!("{glyph} {name}  [{status}]")
                                    } else {
                                        format!("{glyph} {name}")
                                    };
                                    let is_selected = self.selected.as_ref() == Some(&path);
                                    ui.horizontal(|ui| {
                                        ui.add_space(indent);
                                        if ui.selectable_label(is_selected, label).clicked() {
                                            self.select_image(path);
                                        }
                                    });
                                }
                            }
                        }
                    });
            }
                });
        });

        self.render_detached_inspector_windows(ctx);

        let busy = self.is_activity_busy();
        let tip_animating = !self.tool_caps.operation_timings().is_empty();
        if busy || tip_animating {
            ctx.request_repaint_after(std::time::Duration::from_millis(200));
        }
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

    fn poll_scan_complete(&mut self, ctx: &egui::Context) {
        let mut completed = None;
        if let Some(rx) = &self.scan_rx {
            while let Ok(msg) = rx.try_recv() {
                if msg.generation == self.scan_generation {
                    completed = Some(msg);
                }
            }
        }
        if let Some(msg) = completed {
            self.apply_scan_result(&msg.dir_label, msg.result);
            self.scanning = false;
            self.scan_rx = None;
        } else if self.scanning {
            ctx.request_repaint_after(std::time::Duration::from_millis(200));
        }
    }

    fn scan_directory(&mut self, dir: &Path) {
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
        let magick_available = scan_magick_enabled(self.tool_caps.magick_available, dir);
        let dir_label = dir.display().to_string();
        let (tx, rx) = mpsc::channel();
        self.scan_rx = Some(rx);

        thread::spawn(move || {
            let result = scan_images(&dir_path, recursive, magick_available);
            let _ = tx.send(ScanComplete {
                generation,
                dir_label,
                result,
            });
        });

        if on_network {
            self.log(
                "Network path detected — skipping ImageMagick identify during scan for responsiveness"
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

        let (entries, inventory) = finalize_scan_entries(
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
                    "Loaded {} images. First selected (Tools → Open in feh to view).",
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

        let mut cmd = Command::new("feh");
        cmd.arg("--geometry")
            .arg(FEH_VIEWER_GEOMETRY)
            .arg("--scale-down")
            .arg("--zoom")
            .arg(FEH_VIEWER_ZOOM)
            .arg("--filelist")
            .arg(&list_path)
            .arg("--start-at")
            .arg(path);

        self.log(format!(
            "Spawning feh with filelist ({count} images): {:?}",
            cmd
        ));

        match cmd.spawn() {
            Ok(child) => {
                self.log(format!(
                    "feh launched (pid {:?}) for {}",
                    child.id(),
                    path.display()
                ));
                self.status = format!("Launched feh on {}", path.display());
            }
            Err(e) => {
                self.log(format!("Failed to spawn feh: {}", e));
                if feh_spawn_unavailable(&e) {
                    self.mark_feh_unavailable();
                } else {
                    self.status = format!("Failed to launch feh (is it installed?): {}", e);
                }
            }
        }
    }

}