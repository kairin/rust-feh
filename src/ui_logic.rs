// SPDX-License-Identifier: MIT
//! Pure UI/business logic testable without egui (feature 001 validation).

use crate::types::{FileStatus, ImageEntry, ListViewMode, ScanInventory, SortMode, WindowSizePreset};
use std::collections::{BTreeMap, HashSet};
use std::io::Write;
use std::path::{Path, PathBuf};

pub const FEH_MISSING_MSG: &str = "feh not found — install with `sudo apt install feh`";

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

/// Temp file path for feh `--filelist` (overwritten each launch).
pub fn feh_filelist_temp_path() -> PathBuf {
    std::env::temp_dir().join(format!("rust-feh-filelist-{}.txt", std::process::id()))
}

/// Write one absolute path per line for feh `--filelist`.
pub fn write_feh_filelist(paths: impl IntoIterator<Item = impl AsRef<Path>>) -> std::io::Result<usize> {
    let dest = feh_filelist_temp_path();
    let mut file = std::fs::File::create(&dest)?;
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

/// Root tree folder listed count should match inventory native_listed (SC-005).
pub fn tree_root_listed_matches_inventory(tree: &FolderTreeNode, inventory: &ScanInventory) -> bool {
    tree.relative_path == "." && tree.listed_count == inventory.native_listed
}

/// Mark converted rows and rebuild inventory after scanner walk.
pub fn finalize_scan_entries(
    mut entries: Vec<ImageEntry>,
    non_image_skipped: usize,
    magick_truncated: bool,
) -> (Vec<ImageEntry>, ScanInventory) {
    for entry in &mut entries {
        if detect_converted_status(&entry.path) {
            entry.status = FileStatus::Converted;
        }
    }
    let inventory = ScanInventory::from_entries(&entries, non_image_skipped, magick_truncated);
    (entries, inventory)
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