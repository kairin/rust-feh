// SPDX-License-Identifier: MIT
//! Pure UI/business logic testable without egui (feature 001 validation).

use crate::types::{
    AssetStatus, FehLaunchEntry, FehLaunchList, FileStatus, ImageEntry, ListViewMode,
    OutputPolicy, ProcessedResult, ScanInventory, SortMode, WindowSizePreset,
};
use std::collections::{BTreeMap, HashSet};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

pub const FEH_MISSING_MSG: &str = "feh not found — install with `sudo apt install feh`";
/// Per-entry launch panel indicator when feh is absent (FR-011).
pub const FEH_NOT_INSTALLED_LAUNCH_MSG: &str = "feh not installed";

pub const WINDOW_MIN_RESIZABLE: (f32, f32) = (640.0, 480.0);
pub const WINDOW_MAX_RESIZABLE: (f32, f32) = (8192.0, 8192.0);

/// feh viewer: fixed geometry so the window never shrinks to tiny image dimensions.
pub const FEH_VIEWER_GEOMETRY: &str = "1280x960";
/// Upscale small images to fit the fixed window (5px icons stay visible, window stays put).
pub const FEH_VIEWER_ZOOM: &str = "max";

/// Never allow the rust-feh window below this floor (avoids accidental "vanished" windows).
pub fn clamp_window_size(width: f32, height: f32) -> (f32, f32) {
    let (min_w, min_h) = WINDOW_MIN_RESIZABLE;
    (width.max(min_w), height.max(min_h))
}

pub fn window_preset_label(preset: WindowSizePreset) -> &'static str {
    match preset {
        WindowSizePreset::Compact => "Compact (720 × 540)",
        WindowSizePreset::Default => "Default (960 × 720)",
        WindowSizePreset::Large => "Large (1280 × 960)",
    }
}

pub fn window_preset_dimensions(preset: WindowSizePreset) -> (f32, f32) {
    match preset {
        WindowSizePreset::Compact => (720.0, 540.0),
        WindowSizePreset::Default => (960.0, 720.0),
        WindowSizePreset::Large => (1280.0, 960.0),
    }
}

pub fn feh_missing_status() -> String {
    FEH_MISSING_MSG.to_string()
}

pub fn feh_not_installed_launch_status() -> String {
    FEH_NOT_INSTALLED_LAUNCH_MSG.to_string()
}

/// Temp file path for feh `--filelist` (overwritten each launch).
pub fn feh_filelist_temp_path() -> PathBuf {
    std::env::temp_dir().join(format!("rust-feh-filelist-{}.txt", std::process::id()))
}

/// Persistence location for configured multi-feh launch entries.
pub fn launch_list_path() -> PathBuf {
    let base = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    base.join(".config")
        .join("rust-feh")
        .join("launch-entries.json")
}

/// Persist launch entries using a temp-file + rename write.
pub fn save_launch_list(list: &FehLaunchList) -> Result<(), String> {
    let path = launch_list_path();
    let Some(parent) = path.parent() else {
        return Err(format!("Invalid launch-list path: {}", path.display()));
    };
    std::fs::create_dir_all(parent)
        .map_err(|e| format!("Failed to create config directory {}: {e}", parent.display()))?;
    let temp = path.with_extension(format!("json.tmp.{}", std::process::id()));
    let data = serde_json::to_vec_pretty(list)
        .map_err(|e| format!("Failed to serialize launch entries: {e}"))?;
    std::fs::write(&temp, data)
        .map_err(|e| format!("Failed to write launch entries {}: {e}", temp.display()))?;
    std::fs::rename(&temp, &path).map_err(|e| {
        let _ = std::fs::remove_file(&temp);
        format!("Failed to save launch entries {}: {e}", path.display())
    })?;
    Ok(())
}

/// Load persisted launch entries; missing or corrupt files recover to an empty list.
pub fn load_launch_list() -> FehLaunchList {
    let path = launch_list_path();
    let Ok(data) = std::fs::read(&path) else {
        return FehLaunchList::default();
    };
    match serde_json::from_slice(&data) {
        Ok(list) => list,
        Err(e) => {
            eprintln!(
                "[rust-feh] warning: corrupt launch-entries.json at {}: {e}; starting with empty list",
                path.display()
            );
            FehLaunchList::default()
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntryLaunchState {
    pub launchable: bool,
    pub status: String,
}

/// Decode full image bytes to native-dimension RGBA8 pixels for clipboard copy.
pub fn decode_image_to_rgba(bytes: &[u8]) -> Result<(u32, u32, Vec<u8>), String> {
    let img = image::load_from_memory(bytes)
        .map_err(|e| format!("Failed to decode image: {e}"))?;
    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();
    Ok((width, height, rgba.into_raw()))
}

/// Copy decoded image data to the system clipboard.
pub fn copy_image_to_clipboard(path: &Path) -> Result<String, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("Failed to read image: {e}"))?;
    let (width, height, rgba) = decode_image_to_rgba(&bytes)?;
    let mut clipboard = arboard::Clipboard::new()
        .map_err(|e| format!("Clipboard unavailable: {e}"))?;
    let image = arboard::ImageData {
        width: width as usize,
        height: height as usize,
        bytes: std::borrow::Cow::Owned(rgba),
    };
    clipboard
        .set_image(image)
        .map_err(|e| format!("Clipboard copy failed: {e}"))?;
    Ok(format!(
        "Copied image to clipboard: {}",
        path.file_name()
            .unwrap_or_default()
            .to_string_lossy()
    ))
}

fn entry_folder_images<'a>(entry: &FehLaunchEntry, images: &'a [ImageEntry]) -> Vec<&'a Path> {
    let Some(folder) = entry.folder_path.as_deref() else {
        return Vec::new();
    };
    images
        .iter()
        .filter(|image| image.path.parent().is_some_and(|p| p == folder))
        .map(|image| image.path.as_path())
        .collect()
}

/// Build deterministic feh filelist paths for one launch entry from current scanned images.
pub fn build_entry_filelist(entry: &FehLaunchEntry, images: &[ImageEntry]) -> Vec<PathBuf> {
    entry_folder_images(entry, images)
        .into_iter()
        .map(Path::to_path_buf)
        .collect()
}

fn entry_launch_block_reason(entry: &FehLaunchEntry) -> Option<&'static str> {
    let folder = entry.folder_path.as_deref()?;
    if folder.is_dir() {
        None
    } else {
        Some("Folder not found")
    }
}

/// Explain whether a configured launch entry can be launched now.
pub fn entry_is_launchable(
    entry: &FehLaunchEntry,
    images: &[ImageEntry],
    feh_available: bool,
) -> EntryLaunchState {
    if !feh_available {
        return EntryLaunchState {
            launchable: false,
            status: feh_not_installed_launch_status(),
        };
    }
    if entry.folder_path.is_none() {
        return EntryLaunchState {
            launchable: false,
            status: "Select a folder".to_string(),
        };
    }
    if let Some(reason) = entry_launch_block_reason(entry) {
        return EntryLaunchState {
            launchable: false,
            status: reason.to_string(),
        };
    }
    let count = entry_folder_images(entry, images).len();
    if count == 0 {
        return EntryLaunchState {
            launchable: false,
            status: "No images".to_string(),
        };
    }
    EntryLaunchState {
        launchable: true,
        status: format!("{count} images"),
    }
}

/// Write one absolute path per line for feh `--filelist`.
pub fn write_feh_filelist(paths: impl IntoIterator<Item = impl AsRef<Path>>) -> std::io::Result<usize> {
    write_feh_filelist_to(feh_filelist_temp_path(), paths)
}

/// Write a feh `--filelist` to an explicit destination.
pub fn write_feh_filelist_to(
    dest: impl AsRef<Path>,
    paths: impl IntoIterator<Item = impl AsRef<Path>>,
) -> std::io::Result<usize> {
    let mut file = std::fs::File::create(dest)?;
    let mut count = 0usize;
    for path in paths {
        writeln!(file, "{}", path.as_ref().display())?;
        count += 1;
    }
    Ok(count)
}

/// Join activity log lines for display and clipboard copy.
/// GVFS/SMB/NFS paths are slow for per-file subprocess identify during scan.
pub fn is_network_mount_path(path: &Path) -> bool {
    let s = path.display().to_string();
    s.contains("/gvfs/")
        || s.contains("smb-share:")
        || s.contains("/nfs/")
        || s.starts_with("//")
}

/// Whether ImageMagick identify may run during directory scan (FR-001 / network policy).
pub fn scan_magick_enabled(magick_on_path: bool, root: &Path) -> bool {
    magick_on_path && !is_network_mount_path(root)
}

pub fn join_activity_log(lines: &[String]) -> String {
    if lines.is_empty() {
        "(no activity yet)".to_string()
    } else {
        lines.join("\n")
    }
}

/// FR-008a: keep feh warning in post-scan status when feh absent.
pub fn post_scan_status(base: &str, feh_available: bool) -> String {
    if feh_available {
        base.to_string()
    } else {
        format!("{base} — {FEH_MISSING_MSG}")
    }
}

pub fn sort_mode_label(mode: SortMode) -> &'static str {
    match mode {
        SortMode::Path => "Path",
        SortMode::Name => "Name",
        SortMode::Folder => "Folder",
    }
}

pub fn file_name_display(path: &Path) -> String {
    path.file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned()
}

/// Folder path relative to the scanned root (`"."` when the file is in the root folder).
pub fn relative_folder(root: Option<&Path>, path: &Path) -> String {
    let Some(root) = root else {
        return path
            .parent()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| ".".to_string());
    };

    match path.strip_prefix(root) {
        Ok(rel) => match rel.parent() {
            Some(parent) if !parent.as_os_str().is_empty() => parent.display().to_string(),
            _ => ".".to_string(),
        },
        Err(_) => path
            .parent()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| ".".to_string()),
    }
}

fn entry_matches_search(entry: &ImageEntry, root: Option<&Path>, needle: &str) -> bool {
    let path = &entry.path;
    file_name_display(path).to_lowercase().contains(needle)
        || relative_folder(root, path).to_lowercase().contains(needle)
        || path.display().to_string().to_lowercase().contains(needle)
}

/// FR-005/FR-006: case-insensitive UTF-8 substring filter on filename and folder path.
pub fn filter_indices(images: &[ImageEntry], root: Option<&Path>, search: &str) -> Vec<usize> {
    if search.is_empty() {
        return (0..images.len()).collect();
    }
    let needle = search.to_lowercase();
    images
        .iter()
        .enumerate()
        .filter(|(_, e)| entry_matches_search(e, root, &needle))
        .map(|(i, _)| i)
        .collect()
}

fn sort_key(images: &[ImageEntry], idx: usize, mode: SortMode, root: Option<&Path>) -> String {
    let path = &images[idx].path;
    match mode {
        SortMode::Path => path.display().to_string().to_lowercase(),
        SortMode::Name => file_name_display(path).to_lowercase(),
        SortMode::Folder => format!(
            "{}/{}",
            relative_folder(root, path).to_lowercase(),
            file_name_display(path).to_lowercase()
        ),
    }
}

/// Apply filter then sort for the virtualized image list.
pub fn list_indices(
    images: &[ImageEntry],
    root: Option<&Path>,
    search: &str,
    sort: SortMode,
) -> Vec<usize> {
    let mut indices = filter_indices(images, root, search);
    indices.sort_by(|&a, &b| {
        sort_key(images, a, sort, root).cmp(&sort_key(images, b, sort, root))
    });
    indices
}

pub fn showing_count_label(shown: usize, total: usize) -> String {
    format!("Showing {shown} / {total} images")
}

pub fn list_view_mode_label(mode: ListViewMode) -> &'static str {
    match mode {
        ListViewMode::FlatList => "Flat list",
        ListViewMode::FolderTree => "Folder tree",
    }
}

pub fn file_status_label(status: FileStatus) -> &'static str {
    match status {
        FileStatus::NativeListed => "native",
        FileStatus::MagickDetected => "magick · awaiting convert",
        FileStatus::Converted => "converted",
    }
}

pub fn tree_file_glyph(status: FileStatus) -> &'static str {
    match status {
        FileStatus::MagickDetected => "○",
        _ => "●",
    }
}

pub fn inventory_magick_hint(magick_on_path: bool, root: Option<&Path>) -> Option<&'static str> {
    if root.is_some_and(is_network_mount_path) {
        return Some(
            "Network folder — ImageMagick identify skipped during scan for responsiveness.",
        );
    }
    if magick_on_path {
        None
    } else {
        Some("Install ImageMagick to detect more formats (Tools panel).")
    }
}

/// Lines for the scan inventory summary bar (feature 005 contract).
pub fn format_inventory_bar(inv: &ScanInventory, root_label: &str) -> Vec<String> {
    vec![
        format!("Root: {root_label}"),
        format!(
            "Images listed (native) .............. {}   jpg png webp gif bmp",
            inv.native_listed
        ),
        format!(
            "Magick-detected (unlisted) ............. {}",
            inv.magick_detected
        ),
        format!(
            "Converted (processed output exists) ..... {}",
            inv.converted
        ),
        format!("Awaiting convert ....................... {}", inv.awaiting_convert),
        format!(
            "Non-image files skipped ............... {}",
            inv.non_image_skipped
        ),
    ]
}

/// FR-011: awaiting equals magick-detected minus converted-from-magick entries.
pub fn inventory_awaiting_invariant_holds(inv: &ScanInventory) -> bool {
    inv.awaiting_convert <= inv.magick_detected
        && inv.magick_detected.saturating_sub(inv.awaiting_convert) <= inv.converted
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FolderTreeNode {
    pub relative_path: String,
    pub listed_count: usize,
    pub magick_count: usize,
    pub skipped_count: usize,
    pub children: BTreeMap<String, FolderTreeNode>,
    pub file_indices: Vec<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TreeRowKind {
    Folder,
    File,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeRow {
    pub kind: TreeRowKind,
    pub depth: usize,
    pub folder_path: String,
    pub listed: usize,
    pub magick: usize,
    pub skipped: usize,
    pub expanded: bool,
    pub entry_index: Option<usize>,
}

fn bump_folder_counts(node: &mut FolderTreeNode, status: FileStatus) {
    if status == FileStatus::NativeListed {
        node.listed_count += 1;
    }
    if status == FileStatus::MagickDetected {
        node.magick_count += 1;
    }
}

fn descend_folder<'a>(
    root: &'a mut FolderTreeNode,
    folder: &str,
    status: FileStatus,
) -> &'a mut FolderTreeNode {
    let mut current = root;
    if folder == "." {
        return current;
    }
    let mut built = String::new();
    for segment in folder.split('/') {
        built = if built.is_empty() {
            segment.to_string()
        } else {
            format!("{built}/{segment}")
        };
        current = current
            .children
            .entry(segment.to_string())
            .or_insert_with(|| FolderTreeNode {
                relative_path: built.clone(),
                ..Default::default()
            });
        bump_folder_counts(current, status);
    }
    current
}

fn add_entry_to_tree(root: &mut FolderTreeNode, folder: &str, idx: usize, status: FileStatus) {
    bump_folder_counts(root, status);
    let leaf = descend_folder(root, folder, status);
    leaf.file_indices.push(idx);
}

/// Build folder hierarchy from filtered/sorted entry indices (FR-009).
pub fn build_folder_tree(
    images: &[ImageEntry],
    root: Option<&Path>,
    indices: &[usize],
) -> FolderTreeNode {
    let mut tree = FolderTreeNode {
        relative_path: ".".to_string(),
        ..Default::default()
    };
    for &idx in indices {
        let folder = relative_folder(root, &images[idx].path);
        add_entry_to_tree(&mut tree, &folder, idx, images[idx].status);
    }
    tree
}

pub fn folder_tree_display_name(relative_path: &str, scan_root: Option<&Path>) -> String {
    if relative_path == "." {
        scan_root
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| ".".to_string())
    } else {
        format!("{relative_path}/")
    }
}

pub fn folder_line_suffix(listed: usize, magick: usize, skipped: usize) -> String {
    let mut parts = vec![format!("{listed} listed")];
    if magick > 0 {
        parts.push(format!("{magick} magick"));
    }
    if skipped > 0 {
        parts.push(format!("{skipped} skipped"));
    }
    parts.join(" │ ")
}

fn flatten_tree_node(
    node: &FolderTreeNode,
    depth: usize,
    expanded: &HashSet<String>,
    _scan_root: Option<&Path>,
    root_skipped: usize,
    rows: &mut Vec<TreeRow>,
) {
    let is_expanded = expanded.contains(&node.relative_path);
    let skipped = if node.relative_path == "." {
        root_skipped
    } else {
        node.skipped_count
    };
    rows.push(TreeRow {
        kind: TreeRowKind::Folder,
        depth,
        folder_path: node.relative_path.clone(),
        listed: node.listed_count,
        magick: node.magick_count,
        skipped,
        expanded: is_expanded,
        entry_index: None,
    });

    if !is_expanded {
        return;
    }

    for child in node.children.values() {
        flatten_tree_node(child, depth + 1, expanded, _scan_root, root_skipped, rows);
    }
    for &idx in &node.file_indices {
        rows.push(TreeRow {
            kind: TreeRowKind::File,
            depth: depth + 1,
            folder_path: node.relative_path.clone(),
            listed: 0,
            magick: 0,
            skipped: 0,
            expanded: false,
            entry_index: Some(idx),
        });
    }
}

fn folder_ancestor_paths(folder: &str) -> Vec<String> {
    let mut paths = vec![".".to_string()];
    if folder == "." {
        return paths;
    }
    let mut built = String::new();
    for segment in folder.split('/') {
        built = if built.is_empty() {
            segment.to_string()
        } else {
            format!("{built}/{segment}")
        };
        paths.push(built.clone());
    }
    paths
}

fn effective_expanded_paths(
    images: &[ImageEntry],
    root: Option<&Path>,
    search: &str,
    indices: &[usize],
    expanded: &HashSet<String>,
) -> HashSet<String> {
    let mut effective = expanded.clone();
    if !search.is_empty() {
        for &idx in indices {
            let folder = relative_folder(root, &images[idx].path);
            for path in folder_ancestor_paths(&folder) {
                effective.insert(path);
            }
        }
    }
    effective
}

/// Lazy-expand tree rows for virtualization (research R4).
pub fn tree_visible_rows(
    images: &[ImageEntry],
    root: Option<&Path>,
    search: &str,
    sort: SortMode,
    expanded: &HashSet<String>,
    root_skipped: usize,
) -> Vec<TreeRow> {
    let indices = list_indices(images, root, search, sort);
    let effective = effective_expanded_paths(images, root, search, &indices, expanded);
    let tree = build_folder_tree(images, root, &indices);
    let mut rows = Vec::new();
    flatten_tree_node(&tree, 0, &effective, root, root_skipped, &mut rows);
    rows
}

pub fn default_tree_expanded() -> HashSet<String> {
    HashSet::from([".".to_string()])
}

const PROCESSED_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "webp"];

/// True when `path` is a `*_processed.*` artifact or has a matching processed sibling (FR-012).
pub fn detect_converted_status(path: &Path) -> bool {
    if is_processed_artifact(path) {
        return true;
    }
    processed_sibling_exists(path)
}

fn is_processed_artifact(path: &Path) -> bool {
    path.file_stem()
        .and_then(|s| s.to_str())
        .is_some_and(|stem| stem.ends_with("_processed"))
}

fn processed_sibling_exists(path: &Path) -> bool {
    let Some(parent) = path.parent() else {
        return false;
    };
    let stem = path
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    PROCESSED_EXTENSIONS
        .iter()
        .any(|ext| parent.join(format!("{stem}_processed.{ext}")).is_file())
}

/// After in-app resize, update selected entry status and rebuild inventory (no full rescan).
pub fn refresh_entry_and_inventory(
    entries: &mut [ImageEntry],
    path: &Path,
    non_image_skipped: usize,
    magick_truncated: bool,
) -> ScanInventory {
    if let Some(entry) = entries.iter_mut().find(|e| e.path == path) {
        if detect_converted_status(path) {
            entry.status = FileStatus::Converted;
        }
    }
    ScanInventory::from_entries(entries, non_image_skipped, magick_truncated)
}

/// Direct add new/optimized/processed asset to in-memory list without full re-scan (T014, FR-005, clarify).
/// Used after tool ops and prepare-fast materialize so assets appear immediately.
pub fn add_or_update_asset_in_inventory(
    images: &mut Vec<ImageEntry>,
    new_path: PathBuf,
    asset_status: AssetStatus,
) {
    if images.iter().any(|e| e.path == new_path) {
        // update status if exists
        if let Some(e) = images.iter_mut().find(|e| e.path == new_path) {
            e.asset_status = asset_status;
        }
        return;
    }
    // new entry, lazy size etc; status native for now
    images.push(ImageEntry::with_asset_status(
        new_path,
        FileStatus::NativeListed,
        asset_status,
    ));
}

/// Root tree folder listed count should match inventory native_listed (SC-005).
pub fn tree_root_listed_matches_inventory(tree: &FolderTreeNode, inventory: &ScanInventory) -> bool {
    tree.relative_path == "." && tree.listed_count == inventory.native_listed
}

/// Mark converted rows and rebuild inventory after scanner walk.
/// Fast path after scan: no per-file converted-sibling stat storm (feh-first).
pub fn finalize_scan_entries_fast(
    entries: Vec<ImageEntry>,
    non_image_skipped: usize,
    magick_truncated: bool,
) -> (Vec<ImageEntry>, ScanInventory) {
    let inventory = ScanInventory::from_entries(&entries, non_image_skipped, magick_truncated);
    (entries, inventory)
}

/// Background pass: mark converted / *_processed siblings (can be slow on huge folders).
pub fn apply_converted_detection(entries: &mut [ImageEntry]) -> usize {
    let mut n = 0usize;
    for entry in entries.iter_mut() {
        if detect_converted_status(&entry.path) {
            entry.status = FileStatus::Converted;
            n += 1;
        }
    }
    n
}

pub fn finalize_scan_entries(
    mut entries: Vec<ImageEntry>,
    non_image_skipped: usize,
    magick_truncated: bool,
) -> (Vec<ImageEntry>, ScanInventory) {
    apply_converted_detection(&mut entries);
    finalize_scan_entries_fast(entries, non_image_skipped, magick_truncated)
}

/// Compute final output path for an image operation (pure, FR-003).
pub fn compute_output_path(
    source: &Path,
    stem_suffix: &str,
    ext: &str,
    policy: &OutputPolicy,
) -> Result<PathBuf, String> {
    let parent = source.parent().unwrap_or(Path::new("."));
    let stem = source
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy();
    let ext = ext.trim_start_matches('.');
    match policy {
        OutputPolicy::NewSubfolder { name } => {
            let dir = parent.join(name);
            Ok(dir.join(format!("{stem}{stem_suffix}.{ext}")))
        }
        OutputPolicy::SuffixedSibling { suffix } => Ok(parent.join(format!("{stem}{suffix}.{ext}"))),
        OutputPolicy::InPlaceWithBackup { .. } => {
            let orig_ext = source
                .extension()
                .map(|e| e.to_string_lossy().into_owned())
                .unwrap_or_else(|| ext.to_string());
            Ok(parent.join(format!("{stem}.{orig_ext}")))
        }
    }
}

/// Raw RGBA preview buffer for crop (no egui types).
pub struct CropPreviewPixels {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

pub fn crop_preview_pixels(source: &Path, geometry: &str, max_dim: u32) -> Result<CropPreviewPixels, String> {
    use crate::image_proc::parse_crop_geometry;
    let rect = parse_crop_geometry(geometry)?;
    let img = image::open(source).map_err(|e| e.to_string())?;
    let (x, y, w, h) = {
        let x = rect.x.max(0) as u32;
        let y = rect.y.max(0) as u32;
        if x >= img.width() || y >= img.height() {
            return Err("Crop origin outside image".into());
        }
        let w = rect.width.min(img.width() - x);
        let h = rect.height.min(img.height() - y);
        if w == 0 || h == 0 {
            return Err("Crop region empty".into());
        }
        (x, y, w, h)
    };
    let cropped = img.crop_imm(x, y, w, h);
    let scale = (max_dim as f32 / cropped.width().max(cropped.height()) as f32).min(1.0);
    let preview = if scale < 1.0 {
        let nw = (cropped.width() as f32 * scale).max(1.0) as u32;
        let nh = (cropped.height() as f32 * scale).max(1.0) as u32;
        cropped.resize(nw, nh, image::imageops::FilterType::Triangle)
    } else {
        cropped
    };
    Ok(CropPreviewPixels {
        width: preview.width(),
        height: preview.height(),
        rgba: preview.to_rgba8().into_raw(),
    })
}

/// Expand rename pattern for ordered paths (FR-007).
pub fn expand_rename_pattern(
    pattern: &str,
    sources: &[PathBuf],
    counter_start: u32,
) -> Result<Vec<(PathBuf, String)>, String> {
    let mut out = Vec::with_capacity(sources.len());
    let mut seen = HashSet::new();
    for (i, src) in sources.iter().enumerate() {
        let name = expand_one_rename_token(pattern, src, counter_start + i as u32)?;
        if !seen.insert(name.clone()) {
            return Err(format!("Name collision: {name}"));
        }
        out.push((src.clone(), name));
    }
    Ok(out)
}

fn expand_one_rename_token(pattern: &str, source: &Path, counter: u32) -> Result<String, String> {
    let stem = source
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy();
    let ext = source
        .extension()
        .map(|e| e.to_string_lossy().into_owned())
        .unwrap_or_default();
    let date = chrono_date_yyyymmdd();
    let mut result = pattern.to_string();
    result = replace_counter(&result, counter)?;
    result = result.replace("{original}", &stem);
    result = result.replace("{ext}", &ext);
    result = result.replace("{date:YYYYMMDD}", &date);
    result = result.replace("{date:YYYY}", &date[..4.min(date.len())]);
    Ok(if ext.is_empty() || result.ends_with(&format!(".{ext}")) {
        result
    } else {
        format!("{result}.{ext}")
    })
}

fn replace_counter(s: &str, counter: u32) -> Result<String, String> {
    let mut out = s.to_string();
    while let Some(start) = out.find("{counter:") {
        let rest = &out[start + 10..];
        let end = rest
            .find('}')
            .ok_or_else(|| "unclosed {counter:NN}".to_string())?;
        let width: usize = rest[..end]
            .parse()
            .map_err(|_| "invalid counter width".to_string())?;
        let padded = format!("{counter:0width$}");
        out.replace_range(start..start + 10 + end + 1, &padded);
    }
    if out.contains("{counter}") {
        out = out.replace("{counter}", &counter.to_string());
    }
    Ok(out)
}

fn chrono_date_yyyymmdd() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    // Simple UTC date without chrono dep
    let days = secs / 86400;
    let (y, m, d) = days_to_ymd(days);
    format!("{y:04}{m:02}{d:02}")
}

fn days_to_ymd(mut days: u64) -> (u32, u32, u32) {
    let mut y = 1970u32;
    loop {
        let diy = if is_leap(y) { 366 } else { 365 };
        if days < diy as u64 {
            break;
        }
        days -= diy as u64;
        y += 1;
    }
    let months = if is_leap(y) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };
    let mut m = 1u32;
    for &md in &months {
        if days < md as u64 {
            break;
        }
        days -= md as u64;
        m += 1;
    }
    (y, m, days as u32 + 1)
}

fn is_leap(y: u32) -> bool {
    (y.is_multiple_of(4) && !y.is_multiple_of(100)) || y.is_multiple_of(400)
}

/// Aggregate per-item batch results into a summary (for UI + tests).
pub fn aggregate_batch_results(results: &[Result<ProcessedResult, String>]) -> crate::image_proc::BatchSummary {
    let total = results.len();
    let succeeded = results.iter().filter(|r| r.is_ok()).count();
    let failed = total.saturating_sub(succeeded);
    crate::image_proc::BatchSummary {
        total,
        succeeded,
        failed,
        skipped: 0,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenameApplyOutcome {
    pub applied: Vec<(PathBuf, PathBuf)>,
    pub rolled_back: bool,
    pub error: Option<String>,
}

/// Apply rename pairs atomically; rolls back prior renames on first failure.
pub fn apply_rename_pairs(pairs: &[(PathBuf, String)]) -> RenameApplyOutcome {
    let mut applied = Vec::new();
    for (old, new_name) in pairs {
        let parent = old.parent().unwrap_or(Path::new("."));
        let dest = parent.join(new_name);
        if let Err(e) = std::fs::rename(old, &dest) {
            for (from, to) in applied.iter().rev() {
                let _ = std::fs::rename(to, from);
            }
            return RenameApplyOutcome {
                applied,
                rolled_back: true,
                error: Some(format!("{}: {e}", old.display())),
            };
        }
        applied.push((old.clone(), dest));
    }
    RenameApplyOutcome {
        applied,
        rolled_back: false,
        error: None,
    }
}

pub fn format_image_tools_log(result: &ProcessedResult) -> String {
    let hit = if result.was_cache_hit { " [cache hit]" } else { "" };
    format!(
        "Image tools: {:?} {} -> {}{}",
        result.operation,
        result.source_path.display(),
        result.dest_path.display(),
        hit
    )
}

pub struct JobProgress {
    pub current: usize,
    pub total: usize,
    pub message: String,
}

pub enum JobMsg<T> {
    Progress(JobProgress),
    Item(T),
    Done,
    Cancelled,
}

pub fn spawn_job<F, T>(
    items: Vec<PathBuf>,
    cancel: Arc<AtomicBool>,
    work: F,
) -> mpsc::Receiver<JobMsg<T>>
where
    F: Fn(&Path) -> Result<T, String> + Send + 'static,
    T: Send + 'static,
{
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let total = items.len();
        for (i, path) in items.iter().enumerate() {
            if cancel.load(Ordering::Relaxed) {
                let _ = tx.send(JobMsg::Cancelled);
                return;
            }
            let _ = tx.send(JobMsg::Progress(JobProgress {
                current: i,
                total,
                message: path.display().to_string(),
            }));
            match work(path) {
                Ok(v) => {
                    let _ = tx.send(JobMsg::Item(v));
                }
                Err(e) => {
                    let _ = tx.send(JobMsg::Progress(JobProgress {
                        current: i + 1,
                        total,
                        message: format!("skip: {e}"),
                    }));
                }
            }
        }
        let _ = tx.send(JobMsg::Done);
    });
    rx
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn entry(path: &str) -> ImageEntry {
        ImageEntry::new(PathBuf::from(path))
    }

    #[test]
    fn post_scan_appends_feh_warning_when_unavailable() {
        let s = post_scan_status("Loaded 5 images.", false);
        assert!(s.contains("Loaded 5 images."));
        assert!(s.contains(FEH_MISSING_MSG));
    }

    #[test]
    fn post_scan_unchanged_when_feh_available() {
        assert_eq!(post_scan_status("No images found", true), "No images found");
    }

    #[test]
    fn filter_case_insensitive_substring_on_filename() {
        let images = vec![
            entry("/tmp/test/Vacation_001.jpg"),
            entry("/tmp/test/work.png"),
            entry("/tmp/test/VACATION_002.JPG"),
        ];
        let f = filter_indices(&images, Some(Path::new("/tmp/test")), "vacation");
        assert_eq!(f.len(), 2);
        assert_eq!(f, vec![0, 2]);
    }

    #[test]
    fn filter_matches_relative_folder() {
        let root = Path::new("/data/photos");
        let images = vec![
            entry("/data/photos/vacation/a.jpg"),
            entry("/data/photos/work/b.png"),
        ];
        let f = filter_indices(&images, Some(root), "vacation");
        assert_eq!(f, vec![0]);
    }

    #[test]
    fn filter_empty_returns_all() {
        let images = vec![entry("/tmp/a.jpg"), entry("/tmp/b.jpg")];
        assert_eq!(filter_indices(&images, None, ""), vec![0, 1]);
    }

    #[test]
    fn filter_zero_match() {
        let images = vec![entry("/tmp/a.jpg")];
        assert!(filter_indices(&images, None, "zzz").is_empty());
    }

    #[test]
    fn relative_folder_in_root() {
        let root = Path::new("/data/photos");
        assert_eq!(
            relative_folder(Some(root), Path::new("/data/photos/a.jpg")),
            "."
        );
    }

    #[test]
    fn relative_folder_in_subdir() {
        let root = Path::new("/data/photos");
        assert_eq!(
            relative_folder(Some(root), Path::new("/data/photos/vacation/a.jpg")),
            "vacation"
        );
    }

    #[test]
    fn feh_filelist_order_matches_list_indices_sorts() {
        let root = Path::new("/data");
        let images = vec![
            entry("/data/b/z.jpg"),
            entry("/data/a/m.jpg"),
            entry("/data/a/a.jpg"),
        ];
        for sort in [SortMode::Path, SortMode::Name, SortMode::Folder] {
            let indices = list_indices(&images, Some(root), "", sort);
            let ordered: Vec<PathBuf> = indices
                .iter()
                .map(|&i| images[i].path.clone())
                .collect();
            let _ = std::fs::remove_file(feh_filelist_temp_path());
            write_feh_filelist(&ordered).unwrap();
            let body = std::fs::read_to_string(feh_filelist_temp_path()).unwrap();
            let lines: Vec<&str> = body.lines().collect();
            assert_eq!(
                lines.len(),
                ordered.len(),
                "sort {:?} line count",
                sort
            );
            for (line, path) in lines.iter().zip(ordered.iter()) {
                assert_eq!(*line, path.display().to_string());
            }
            let _ = std::fs::remove_file(feh_filelist_temp_path());
        }
    }

    #[test]
    fn write_feh_filelist_one_path_per_line() {
        let dir = std::env::temp_dir().join("rust-feh-filelist-test");
        let _ = std::fs::remove_file(feh_filelist_temp_path());
        let a = dir.join("a.jpg");
        let b = dir.join("sub/b.jpg");
        let paths = [a.as_path(), b.as_path()];
        let n = write_feh_filelist(&paths).unwrap();
        assert_eq!(n, 2);
        let body = std::fs::read_to_string(feh_filelist_temp_path()).unwrap();
        assert!(body.contains(&format!("{}\n", a.display())));
        assert!(body.contains(&format!("{}\n", b.display())));
        let _ = std::fs::remove_file(feh_filelist_temp_path());
    }

    #[test]
    fn network_mount_path_detects_gvfs_smb() {
        let p = Path::new("/run/user/1000/gvfs/smb-share:server=ds1819.local,share=4tb/AI");
        assert!(is_network_mount_path(p));
        assert!(!is_network_mount_path(Path::new("/home/kkk/Pictures")));
    }

    #[test]
    fn network_mount_path_detects_nfs_and_unc() {
        assert!(is_network_mount_path(Path::new("/mnt/nfs/photos")));
        assert!(is_network_mount_path(Path::new("//server/share/AI")));
        assert!(!is_network_mount_path(Path::new("/mnt/local/photos")));
    }

    #[test]
    fn scan_magick_enabled_skips_network_paths() {
        let smb = Path::new("/run/user/1000/gvfs/smb-share:server=ds1819.local,share=4tb/AI");
        assert!(!scan_magick_enabled(true, smb));
        assert!(scan_magick_enabled(true, Path::new("/home/kkk/Pictures")));
        assert!(!scan_magick_enabled(false, Path::new("/home/kkk/Pictures")));
    }

    #[test]
    fn join_activity_log_empty_and_lines() {
        assert_eq!(join_activity_log(&[]), "(no activity yet)");
        assert_eq!(
            join_activity_log(&["a".into(), "b".into()]),
            "a\nb"
        );
    }

    #[test]
    fn sort_by_name_orders_filenames() {
        let images = vec![
            entry("/tmp/z.jpg"),
            entry("/tmp/a.jpg"),
            entry("/tmp/m.jpg"),
        ];
        let sorted = list_indices(&images, None, "", SortMode::Name);
        assert_eq!(sorted, vec![1, 2, 0]);
    }

    #[test]
    fn sort_by_folder_groups_directories() {
        let root = Path::new("/data");
        let images = vec![
            entry("/data/b/2.jpg"),
            entry("/data/a/1.jpg"),
            entry("/data/a/2.jpg"),
        ];
        let sorted = list_indices(&images, Some(root), "", SortMode::Folder);
        assert_eq!(sorted, vec![1, 2, 0]);
    }

    #[test]
    fn showing_count_format() {
        assert_eq!(showing_count_label(42, 10000), "Showing 42 / 10000 images");
        assert_eq!(showing_count_label(0, 10), "Showing 0 / 10 images");
    }

    #[test]
    fn clamp_window_size_enforces_floor() {
        assert_eq!(clamp_window_size(5.0, 5.0), WINDOW_MIN_RESIZABLE);
        assert_eq!(clamp_window_size(960.0, 720.0), (960.0, 720.0));
    }

    #[test]
    fn detect_converted_status_processed_artifact() {
        let dir = std::env::temp_dir().join("rust-feh-converted-test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("sunset_processed.jpg");
        std::fs::write(&path, b"x").unwrap();
        assert!(detect_converted_status(&path));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn detect_converted_status_sibling_output() {
        let dir = std::env::temp_dir().join("rust-feh-sibling-test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let source = dir.join("photo.jpg");
        std::fs::write(&source, b"x").unwrap();
        std::fs::write(dir.join("photo_processed.png"), b"x").unwrap();
        assert!(detect_converted_status(&source));
        assert!(!detect_converted_status(&dir.join("other.jpg")));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn finalize_scan_entries_updates_inventory() {
        let dir = std::env::temp_dir().join("rust-feh-finalize-test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let source = dir.join("a.jpg");
        std::fs::write(&source, b"x").unwrap();
        std::fs::write(dir.join("a_processed.png"), b"x").unwrap();
        let entries = vec![ImageEntry::new(source)];
        let (entries, inventory) = finalize_scan_entries(entries, 2, false);
        assert_eq!(entries[0].status, FileStatus::Converted);
        assert_eq!(inventory.converted, 1);
        assert_eq!(inventory.native_listed, 0);
        assert_eq!(inventory.non_image_skipped, 2);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn relative_folder_deep_nested() {
        let root = Path::new("/data/photos");
        assert_eq!(
            relative_folder(Some(root), Path::new("/data/photos/sub/deep/img.png")),
            "sub/deep"
        );
    }

    #[test]
    fn file_status_labels() {
        assert_eq!(file_status_label(FileStatus::NativeListed), "native");
        assert_eq!(
            file_status_label(FileStatus::MagickDetected),
            "magick · awaiting convert"
        );
        assert_eq!(file_status_label(FileStatus::Converted), "converted");
    }

    #[test]
    fn build_folder_tree_groups_by_folder() {
        let root = Path::new("/data");
        let images = vec![
            entry("/data/a.jpg"),
            entry("/data/sub/b.png"),
            entry("/data/sub/deep/c.webp"),
        ];
        let tree = build_folder_tree(&images, Some(root), &[0, 1, 2]);
        assert_eq!(tree.listed_count, 3);
        assert!(tree.children.contains_key("sub"));
        let sub = &tree.children["sub"];
        assert_eq!(sub.listed_count, 2);
        assert!(sub.children.contains_key("deep"));
    }

    #[test]
    fn tree_visible_rows_respects_filter() {
        let root = Path::new("/data");
        let images = vec![
            entry("/data/a.jpg"),
            entry("/data/sub/b.png"),
            entry("/data/sub/deep/c.webp"),
        ];
        let expanded = default_tree_expanded();
        let rows = tree_visible_rows(&images, Some(root), "deep", SortMode::Path, &expanded, 0);
        let files: Vec<_> = rows
            .iter()
            .filter(|r| r.kind == TreeRowKind::File)
            .collect();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].entry_index, Some(2));
    }

    #[test]
    fn native_converted_fr011() {
        let dir = std::env::temp_dir().join("rust-feh-fr011-test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let source = dir.join("photo.jpg");
        std::fs::write(&source, b"x").unwrap();
        std::fs::write(dir.join("photo_processed.jpg"), b"x").unwrap();
        let mut entries = vec![ImageEntry::new(source.clone())];
        let inventory = refresh_entry_and_inventory(&mut entries, &source, 0, false);
        assert_eq!(entries[0].status, FileStatus::Converted);
        assert_eq!(inventory.converted, 1);
        assert_eq!(inventory.native_listed, 0);
        assert_eq!(inventory.awaiting_convert, 0);
        assert_eq!(inventory.magick_detected, 0);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn sc005_tree_root_listed_matches_inventory() {
        let root = Path::new("/data");
        let images = vec![
            entry("/data/a.jpg"),
            entry("/data/b.png"),
            ImageEntry::with_status(
                PathBuf::from("/data/c.jpg"),
                FileStatus::Converted,
            ),
        ];
        let indices = vec![0, 1, 2];
        let tree = build_folder_tree(&images, Some(root), &indices);
        let inventory = ScanInventory::from_entries(&images, 0, false);
        assert_eq!(inventory.native_listed, 2);
        assert!(tree_root_listed_matches_inventory(&tree, &inventory));
    }

    #[test]
    fn inventory_awaiting_invariant() {
        let inv = ScanInventory {
            native_listed: 2,
            magick_detected: 3,
            converted: 1,
            awaiting_convert: 2,
            non_image_skipped: 5,
            magick_identify_truncated: false,
        };
        assert!(inventory_awaiting_invariant_holds(&inv));
    }

    #[test]
    fn window_preset_dimension_values() {
        assert_eq!(
            super::window_preset_dimensions(WindowSizePreset::Default),
            (960.0, 720.0)
        );
        assert_eq!(
            super::window_preset_dimensions(WindowSizePreset::Large),
            (1280.0, 960.0)
        );
    }
}