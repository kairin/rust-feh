// SPDX-License-Identifier: MIT
//! Pure UI/business logic testable without egui (feature 001 validation).

use crate::types::{
    ActionKind, ActionOutcome, ActionPrefs, ActionResult, AssetStatus, ContextAction,
    FehLaunchEntry, FehLaunchList, FileStatus, ImageEntry, ListViewMode, OutputPolicy,
    ProcessedResult, ScanInventory, SortMode, WindowPreferences, WindowSizePreset,
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

fn runtime_cache_dir() -> PathBuf {
    let base = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    base.join(".cache").join("rust-feh")
}

/// Writable path for feh `--filelist` (overwritten each launch).
pub fn feh_filelist_temp_path() -> PathBuf {
    runtime_cache_dir().join(format!("filelist-{}.txt", std::process::id()))
}

/// Per-entry feh filelist path so Launch All can write distinct lists concurrently.
pub fn feh_entry_filelist_path(entry_id: &str) -> PathBuf {
    runtime_cache_dir().join(format!("filelist-{}-{}.txt", std::process::id(), entry_id))
}

/// POSIX single-quote escaping: wrap `s` in single quotes and replace every
/// embedded `'` with the `'\''` sequence (close-quote, escaped literal quote,
/// reopen-quote). The result is a single shell word that a POSIX `/bin/sh -c`
/// reproduces byte-for-byte, no matter what metacharacters `s` contains.
///
/// This is the ONLY place rust-feh emits shell syntax, and only for the VALUE
/// of feh's `--info` argument (which feh itself later runs via its own internal
/// `/bin/sh -c`). rust-feh never hands a shell a command line of its own.
fn shell_single_quote(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('\'');
    for ch in s.chars() {
        if ch == '\'' {
            out.push_str("'\\''");
        } else {
            out.push(ch);
        }
    }
    out.push('\'');
    out
}

/// Build the exact round-trip viewer invocation (contract:
/// `contracts/viewer-roundtrip.md`, research R1/R2) as an **argument vector**.
///
/// Returns `(program, args, envs)` where every element of `args` is one
/// separate argument, to be passed to `std::process::Command::arg`/`args`
/// individually. rust-feh NEVER concatenates these into a single shell command
/// line — no shell parses them on the rust-feh side, so `filelist_path` and
/// `start_at` need (and receive) no escaping and pass through verbatim even
/// when they contain shell metacharacters, spaces, or quotes.
///
/// The single `--info` argument's VALUE is a shell command string, because feh
/// runs it through feh's own `/bin/sh -c` for every displayed image. Only the
/// `handoff_path` interpolated into that value is shell-quoted (defensively; in
/// practice it is always rust-feh-generated from `runtime_cache_dir()`).
pub fn viewer_spawn_command(
    filelist_path: &Path,
    start_at: &Path,
    handoff_path: &Path,
    profile_dir: &Path,
) -> (String, Vec<String>, Vec<(String, String)>) {
    let info_command = format!(
        "echo %F > {}",
        shell_single_quote(&handoff_path.display().to_string())
    );
    let args = vec![
        "--geometry".to_string(),
        FEH_VIEWER_GEOMETRY.to_string(),
        "--scale-down".to_string(),
        "--zoom".to_string(),
        FEH_VIEWER_ZOOM.to_string(),
        "--info".to_string(),
        info_command,
        "--filelist".to_string(),
        filelist_path.display().to_string(),
        "--start-at".to_string(),
        start_at.display().to_string(),
    ];
    let envs = vec![(
        "XDG_CONFIG_HOME".to_string(),
        profile_dir.display().to_string(),
    )];
    ("feh".to_string(), args, envs)
}

/// Find the index of the filelist entry that a handoff candidate path resolves
/// to, or `None`. Membership in the filelist rust-feh itself wrote is the trust
/// gate (research R4): a handoff value is accepted ONLY if it names a path
/// already in the launched filelist.
///
/// Primary check canonicalizes both the candidate and each entry (resolving
/// symlinks and `..`) and compares — this is what defeats symlink-escape and
/// path-traversal: a candidate that superficially sits "inside" a trusted
/// directory but canonically resolves elsewhere matches no entry and is
/// rejected. If the candidate cannot be canonicalized (e.g. the file was
/// deleted between display and read-back — a legitimate failure mode), fall
/// back to a RAW path comparison against the raw filelist, which still bounds
/// acceptance to paths already in the trusted list and so does not reopen the
/// symlink hole.
fn matching_filelist_index(candidate: &Path, filelist: &[PathBuf]) -> Option<usize> {
    match candidate.canonicalize() {
        Ok(canon_candidate) => filelist
            .iter()
            .position(|entry| entry.canonicalize().is_ok_and(|c| c == canon_candidate)),
        Err(_) => filelist.iter().position(|entry| entry == candidate),
    }
}

/// Starting at `index`, return the nearest filelist entry that still exists on
/// disk, expanding outward as index-1, index+1, index-2, index+2, … (contract
/// "handoff image deleted meanwhile" → nearest surviving neighbor). Returns the
/// entry at `index` itself if it exists; `None` if nothing in the list survives.
fn nearest_surviving_neighbor(filelist: &[PathBuf], index: usize) -> Option<PathBuf> {
    if index >= filelist.len() {
        return None;
    }
    if filelist[index].exists() {
        return Some(filelist[index].clone());
    }
    let n = filelist.len();
    let mut offset = 1usize;
    loop {
        let mut progressed = false;
        if index >= offset {
            progressed = true;
            let li = index - offset;
            if filelist[li].exists() {
                return Some(filelist[li].clone());
            }
        }
        if index + offset < n {
            progressed = true;
            let ri = index + offset;
            if filelist[ri].exists() {
                return Some(filelist[ri].clone());
            }
        }
        if !progressed {
            return None;
        }
        offset += 1;
    }
}

/// Validate the UNTRUSTED handoff file content feh wrote (contract "Exit
/// handling"/"Failure modes", research R4). `content` is raw bytes-as-str read
/// from a file an external process wrote; it is never trusted blindly.
///
/// Returns the trusted filelist `PathBuf` to select (never the raw untrusted
/// string), or `None` when the content should be ignored. Steps:
/// 1. take only the first line, trimmed; empty → `None`;
/// 2. resolve it to a filelist entry via `matching_filelist_index`
///    (canonicalize-and-compare, with a raw-compare fallback for deleted files);
///    no match → `None`;
/// 3. return that entry if it still exists, else its nearest surviving
///    neighbor; `None` if the whole list is gone.
///
/// Logging of rejected/accepted round-trips is the caller's responsibility
/// (main.rs); this function never echoes untrusted content.
pub fn validate_handoff(content: &str, filelist: &[PathBuf]) -> Option<PathBuf> {
    let first_line = content.lines().next()?.trim();
    if first_line.is_empty() {
        return None;
    }
    let candidate = Path::new(first_line);
    let index = matching_filelist_index(candidate, filelist)?;
    nearest_surviving_neighbor(filelist, index)
}

/// Handoff file path for one round-trip viewer (contract:
/// `runtime_cache_dir()/handoff-<pid>-<viewer_id>`), unique per viewer.
pub fn handoff_path(pid: u32, viewer_id: u64) -> PathBuf {
    runtime_cache_dir().join(format!("handoff-{pid}-{viewer_id}"))
}

/// Remove any leftover `handoff-*` files under the runtime cache dir (e.g.
/// left behind if rust-feh exited while round-trip viewers were still open).
/// Called once at startup. Returns the number of files removed; best-effort
/// (I/O errors on individual files are silently skipped, not fatal).
pub fn cleanup_stale_handoffs() -> usize {
    let dir = runtime_cache_dir();
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return 0;
    };
    let mut removed = 0usize;
    for entry in entries.flatten() {
        let name = entry.file_name();
        if name.to_string_lossy().starts_with("handoff-") && std::fs::remove_file(entry.path()).is_ok() {
            removed += 1;
        }
    }
    removed
}

/// Isolated feh config directory for round-trip viewers (research R2):
/// pointing `XDG_CONFIG_HOME` here means launched viewers never read, write,
/// or shadow the user's personal `~/.config/feh` — stock navigation defaults,
/// no personal-theme overlays. Created empty on first use; idempotent.
pub fn viewer_profile_dir() -> PathBuf {
    let base = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    let dir = base.join(".config").join("rust-feh").join("viewer-profile");
    let _ = std::fs::create_dir_all(&dir);
    dir
}

/// Scratch directory for Prepare Fast materialized JPEGs (session-scoped).
pub fn prepare_fast_work_dir() -> PathBuf {
    runtime_cache_dir().join(format!("prepare-fast-{}", std::process::id()))
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
    std::fs::create_dir_all(parent).map_err(|e| {
        format!(
            "Failed to create config directory {}: {e}",
            parent.display()
        )
    })?;
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

/// Persistence location for window sizing preferences (feature 006, FR-008).
pub fn window_prefs_path() -> PathBuf {
    let base = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    base.join(".config")
        .join("rust-feh")
        .join("window-prefs.json")
}

/// Persist window preferences using a temp-file + rename write.
pub fn save_window_prefs(prefs: &WindowPreferences) -> Result<(), String> {
    let path = window_prefs_path();
    let Some(parent) = path.parent() else {
        return Err(format!("Invalid window-prefs path: {}", path.display()));
    };
    std::fs::create_dir_all(parent).map_err(|e| {
        format!(
            "Failed to create config directory {}: {e}",
            parent.display()
        )
    })?;
    let temp = path.with_extension(format!("json.tmp.{}", std::process::id()));
    let data = serde_json::to_vec_pretty(prefs)
        .map_err(|e| format!("Failed to serialize window preferences: {e}"))?;
    std::fs::write(&temp, data)
        .map_err(|e| format!("Failed to write window preferences {}: {e}", temp.display()))?;
    std::fs::rename(&temp, &path).map_err(|e| {
        let _ = std::fs::remove_file(&temp);
        format!("Failed to save window preferences {}: {e}", path.display())
    })?;
    Ok(())
}

/// Load window preferences; missing or corrupt files recover to defaults.
pub fn load_window_prefs() -> WindowPreferences {
    let path = window_prefs_path();
    let Ok(data) = std::fs::read(&path) else {
        return WindowPreferences::default();
    };
    match serde_json::from_slice(&data) {
        Ok(prefs) => prefs,
        Err(e) => {
            eprintln!(
                "[rust-feh] warning: corrupt window-prefs.json at {}: {e}; using defaults",
                path.display()
            );
            WindowPreferences::default()
        }
    }
}

/// Persistence location for action preferences (feature 016, FR-003).
pub fn action_prefs_path() -> PathBuf {
    let base = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    base.join(".config")
        .join("rust-feh")
        .join("action-prefs.json")
}

/// Persist action preferences using a temp-file + rename write.
pub fn save_action_prefs(prefs: &ActionPrefs) -> Result<(), String> {
    let path = action_prefs_path();
    let Some(parent) = path.parent() else {
        return Err(format!("Invalid action-prefs path: {}", path.display()));
    };
    std::fs::create_dir_all(parent).map_err(|e| {
        format!(
            "Failed to create config directory {}: {e}",
            parent.display()
        )
    })?;
    let temp = path.with_extension(format!("json.tmp.{}", std::process::id()));
    let data = serde_json::to_vec_pretty(prefs)
        .map_err(|e| format!("Failed to serialize action preferences: {e}"))?;
    std::fs::write(&temp, data)
        .map_err(|e| format!("Failed to write action preferences {}: {e}", temp.display()))?;
    std::fs::rename(&temp, &path).map_err(|e| {
        let _ = std::fs::remove_file(&temp);
        format!("Failed to save action preferences {}: {e}", path.display())
    })?;
    Ok(())
}

/// Load action preferences; missing or corrupt files recover to defaults.
pub fn load_action_prefs() -> ActionPrefs {
    let path = action_prefs_path();
    let Ok(data) = std::fs::read(&path) else {
        return ActionPrefs::default();
    };
    match serde_json::from_slice(&data) {
        Ok(prefs) => prefs,
        Err(e) => {
            eprintln!(
                "[rust-feh] warning: corrupt action-prefs.json at {}: {e}; using defaults",
                path.display()
            );
            ActionPrefs::default()
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
    let img = image::load_from_memory(bytes).map_err(|e| format!("Failed to decode image: {e}"))?;
    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();
    Ok((width, height, rgba.into_raw()))
}

/// Copy decoded image data to the system clipboard.
pub fn copy_image_to_clipboard(path: &Path) -> Result<String, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("Failed to read image: {e}"))?;
    let (width, height, rgba) = decode_image_to_rgba(&bytes)?;
    let mut clipboard =
        arboard::Clipboard::new().map_err(|e| format!("Clipboard unavailable: {e}"))?;
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
        path.file_name().unwrap_or_default().to_string_lossy()
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
pub fn write_feh_filelist(
    paths: impl IntoIterator<Item = impl AsRef<Path>>,
) -> std::io::Result<usize> {
    write_feh_filelist_to(feh_filelist_temp_path(), paths)
}

/// Write a feh `--filelist` to an explicit destination.
pub fn write_feh_filelist_to(
    dest: impl AsRef<Path>,
    paths: impl IntoIterator<Item = impl AsRef<Path>>,
) -> std::io::Result<usize> {
    let dest = dest.as_ref();
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }
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
    s.contains("/gvfs/") || s.contains("smb-share:") || s.contains("/nfs/") || s.starts_with("//")
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
    indices.sort_by(|&a, &b| sort_key(images, a, sort, root).cmp(&sort_key(images, b, sort, root)));
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
        format!(
            "Awaiting convert ....................... {}",
            inv.awaiting_convert
        ),
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
pub fn tree_root_listed_matches_inventory(
    tree: &FolderTreeNode,
    inventory: &ScanInventory,
) -> bool {
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
    let stem = source.file_stem().unwrap_or_default().to_string_lossy();
    let ext = ext.trim_start_matches('.');
    match policy {
        OutputPolicy::NewSubfolder { name } => {
            let dir = parent.join(name);
            Ok(dir.join(format!("{stem}{stem_suffix}.{ext}")))
        }
        OutputPolicy::SuffixedSibling { suffix } => {
            Ok(parent.join(format!("{stem}{suffix}.{ext}")))
        }
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

pub fn crop_preview_pixels(
    source: &Path,
    geometry: &str,
    max_dim: u32,
) -> Result<CropPreviewPixels, String> {
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
    let stem = source.file_stem().unwrap_or_default().to_string_lossy();
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
pub fn aggregate_batch_results(
    results: &[Result<ProcessedResult, String>],
) -> crate::image_proc::BatchSummary {
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
    let hit = if result.was_cache_hit {
        " [cache hit]"
    } else {
        ""
    };
    format!(
        "Image tools: {:?} {} -> {}{}",
        result.operation,
        result.source_path.display(),
        result.dest_path.display(),
        hit
    )
}

fn action_kind_label(kind: &ActionKind) -> &'static str {
    match kind {
        ActionKind::Context(ContextAction::SaveCopyTo) => "Save a copy",
        ActionKind::Context(ContextAction::MoveTo) => "Move",
        ActionKind::Context(ContextAction::ResizeCopy) => "Resize copy",
        ActionKind::Context(ContextAction::ConvertFormat) => "Convert format",
        ActionKind::Context(ContextAction::CopyPath) => "Copy path",
        ActionKind::Context(ContextAction::CopyImage) => "Copy image",
        ActionKind::RoundTrip => "Round trip",
    }
}

/// Format one `ActionOutcome` as a single human-readable activity-log line
/// (feature 016, FR-010). Never echoes untrusted content verbatim; this only
/// ever receives already-validated paths/reasons produced by this codebase.
pub fn format_action_outcome(outcome: &ActionOutcome) -> String {
    let label = action_kind_label(&outcome.action);
    match &outcome.result {
        ActionResult::Ok { produced } => {
            let target = produced.as_ref().or(outcome.destination.as_ref());
            match target {
                Some(p) => format!("{label}: {} -> {}", outcome.image.display(), p.display()),
                None => format!("{label}: {}", outcome.image.display()),
            }
        }
        ActionResult::Err { reason } => {
            format!("{label} failed: {} — {reason}", outcome.image.display())
        }
    }
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

/// Find a free path in dest_dir for file_name, handling collisions with numeric suffixes.
/// If the path doesn't exist, return it unchanged. Otherwise return name-1, name-2, etc.,
/// filling gaps (e.g. if name and name-1 exist but name-2 doesn't, return name-2).
/// Handles dotfiles and files with/without extensions correctly.
pub fn collision_suffixed_path(dest_dir: &Path, file_name: &str) -> PathBuf {
    let target = dest_dir.join(file_name);
    if !target.exists() {
        return target;
    }

    let stem = Path::new(file_name)
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| file_name.to_string());
    let ext = Path::new(file_name)
        .extension()
        .map(|e| e.to_string_lossy().into_owned());

    let mut suffix = 1u32;
    loop {
        let new_name = if let Some(ref e) = ext {
            format!("{stem}-{suffix}.{e}")
        } else {
            format!("{stem}-{suffix}")
        };
        let candidate = dest_dir.join(&new_name);
        if !candidate.exists() {
            return candidate;
        }
        suffix += 1;
    }
}

/// Collision-safe copy of `src` into `dest_dir` (feature 016, FR-004). Verifies
/// the copy's byte length matches the source before returning; never overwrites
/// an existing file at the destination.
pub fn save_copy_to(src: &Path, dest_dir: &Path) -> Result<PathBuf, String> {
    let file_name = src
        .file_name()
        .ok_or_else(|| format!("Source has no file name: {}", src.display()))?
        .to_string_lossy()
        .into_owned();
    let dest = collision_suffixed_path(dest_dir, &file_name);
    let src_len = std::fs::metadata(src)
        .map_err(|e| format!("Failed to read source metadata {}: {e}", src.display()))?
        .len();
    std::fs::copy(src, &dest).map_err(|e| {
        format!(
            "Failed to copy {} to {}: {e}",
            src.display(),
            dest.display()
        )
    })?;
    let copied_len = std::fs::metadata(&dest)
        .map_err(|e| format!("Failed to verify copy at {}: {e}", dest.display()))?
        .len();
    if copied_len != src_len {
        let _ = std::fs::remove_file(&dest);
        return Err(format!(
            "Copy verification failed for {}: copied {copied_len} bytes, expected {src_len}",
            src.display()
        ));
    }
    Ok(dest)
}

/// A planned loss-proof move: the exact source, final destination (already
/// collision-safe), and a same-directory temp name used during the fallback
/// copy-verify-rename path (feature 016, R7).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MovePlan {
    pub source: PathBuf,
    pub final_dest: PathBuf,
    pub temp_dest: PathBuf,
}

/// Plan a loss-proof move of `src` into `dest_dir`, choosing a collision-safe
/// final name up front (feature 016, FR-004/FR-005).
pub fn plan_loss_proof_move(src: &Path, dest_dir: &Path) -> Result<MovePlan, String> {
    let file_name = src
        .file_name()
        .ok_or_else(|| format!("Source has no file name: {}", src.display()))?
        .to_string_lossy()
        .into_owned();
    let final_dest = collision_suffixed_path(dest_dir, &file_name);
    let temp_dest = dest_dir.join(format!(
        ".{file_name}.rustfeh-move-tmp-{}",
        std::process::id()
    ));
    Ok(MovePlan {
        source: src.to_path_buf(),
        final_dest,
        temp_dest,
    })
}

/// Execute a loss-proof move plan: try a fast same-filesystem atomic rename
/// first; on failure (e.g. cross-filesystem), fall back to copy → verify byte
/// length → atomic rename into place → remove source. The source is only
/// removed after the destination copy is verified to exist with the correct
/// size. On any failure the source is left intact; at most one orphaned temp
/// file may remain at `plan.temp_dest`, which is cleaned up on error here.
pub fn execute_move_plan(plan: &MovePlan) -> Result<PathBuf, String> {
    if std::fs::rename(&plan.source, &plan.final_dest).is_ok() {
        return Ok(plan.final_dest.clone());
    }

    let src_len = std::fs::metadata(&plan.source)
        .map_err(|e| {
            format!(
                "Failed to read source metadata {}: {e}",
                plan.source.display()
            )
        })?
        .len();

    if let Err(e) = std::fs::copy(&plan.source, &plan.temp_dest) {
        let _ = std::fs::remove_file(&plan.temp_dest);
        return Err(format!(
            "Failed to copy {} to destination: {e}",
            plan.source.display()
        ));
    }

    let copied_len = match std::fs::metadata(&plan.temp_dest) {
        Ok(m) => m.len(),
        Err(e) => {
            let _ = std::fs::remove_file(&plan.temp_dest);
            return Err(format!("Failed to verify copy at destination: {e}"));
        }
    };
    if copied_len != src_len {
        let _ = std::fs::remove_file(&plan.temp_dest);
        return Err(format!(
            "Move verification failed for {}: copied {copied_len} bytes, expected {src_len}",
            plan.source.display()
        ));
    }

    if let Err(e) = std::fs::rename(&plan.temp_dest, &plan.final_dest) {
        let _ = std::fs::remove_file(&plan.temp_dest);
        return Err(format!(
            "Failed to finalize move to {}: {e}",
            plan.final_dest.display()
        ));
    }

    std::fs::remove_file(&plan.source).map_err(|e| {
        format!(
            "Copied to {} but failed to remove source {}: {e} (source retained, no data lost)",
            plan.final_dest.display(),
            plan.source.display()
        )
    })?;

    Ok(plan.final_dest.clone())
}

/// Compute output dimensions for downscaling an image to fit max_edge.
/// Preserves aspect ratio and never upscales. Guards against zero inputs.
pub fn stage_decode_bounds(w: u32, h: u32, max_edge: u32) -> (u32, u32) {
    if w == 0 || h == 0 || max_edge == 0 {
        return (w, h);
    }

    let longest = w.max(h);
    if longest <= max_edge {
        return (w, h);
    }

    let scale = max_edge as f32 / longest as f32;
    let new_w = ((w as f32 * scale).round().max(1.0)) as u32;
    let new_h = ((h as f32 * scale).round().max(1.0)) as u32;
    (new_w, new_h)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::sync::Mutex;

    fn entry(path: &str) -> ImageEntry {
        ImageEntry::new(PathBuf::from(path))
    }

    /// `action_prefs_path()` resolves to one fixed real path (`~/.config/rust-feh/
    /// action-prefs.json`); the three `action_prefs_*` tests below read/write it
    /// directly and must not interleave under cargo's parallel test threads.
    static ACTION_PREFS_TEST_LOCK: Mutex<()> = Mutex::new(());

    /// `feh_filelist_temp_path()` is a single pid-scoped path shared by every test
    /// in this process; the two tests below both write/read it directly and must
    /// not interleave under cargo's parallel test threads (pre-existing latent
    /// flake, tightened while touching this file for feature 016).
    static FEH_FILELIST_TEST_LOCK: Mutex<()> = Mutex::new(());

    /// `cleanup_stale_handoffs()` operates on one real shared directory
    /// (`runtime_cache_dir()`); the tests below must not interleave their own
    /// create-then-cleanup sequences under cargo's parallel test threads, or
    /// one test's cleanup call can sweep away another's not-yet-asserted files
    /// (observed flake: `removed >= 2` failing when a concurrent thread's own
    /// `cleanup_stale_handoffs()` call won the race).
    static HANDOFF_CLEANUP_TEST_LOCK: Mutex<()> = Mutex::new(());

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
        let _guard = FEH_FILELIST_TEST_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let root = Path::new("/data");
        let images = vec![
            entry("/data/b/z.jpg"),
            entry("/data/a/m.jpg"),
            entry("/data/a/a.jpg"),
        ];
        for sort in [SortMode::Path, SortMode::Name, SortMode::Folder] {
            let indices = list_indices(&images, Some(root), "", sort);
            let ordered: Vec<PathBuf> = indices.iter().map(|&i| images[i].path.clone()).collect();
            let _ = std::fs::remove_file(feh_filelist_temp_path());
            write_feh_filelist(&ordered).unwrap();
            let body = std::fs::read_to_string(feh_filelist_temp_path()).unwrap();
            let lines: Vec<&str> = body.lines().collect();
            assert_eq!(lines.len(), ordered.len(), "sort {:?} line count", sort);
            for (line, path) in lines.iter().zip(ordered.iter()) {
                assert_eq!(*line, path.display().to_string());
            }
            let _ = std::fs::remove_file(feh_filelist_temp_path());
        }
    }

    #[test]
    fn write_feh_filelist_one_path_per_line() {
        let _guard = FEH_FILELIST_TEST_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
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
        assert_eq!(join_activity_log(&["a".into(), "b".into()]), "a\nb");
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
            ImageEntry::with_status(PathBuf::from("/data/c.jpg"), FileStatus::Converted),
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

    #[test]
    fn collision_suffixed_path_no_collision_returns_unchanged() {
        let dir = std::env::temp_dir().join("rust-feh-collision-test-1");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let result = collision_suffixed_path(&dir, "photo.jpg");
        assert_eq!(result, dir.join("photo.jpg"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn collision_suffixed_path_existing_name_gets_dash_one() {
        let dir = std::env::temp_dir().join("rust-feh-collision-test-2");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("photo.jpg"), b"x").unwrap();
        let result = collision_suffixed_path(&dir, "photo.jpg");
        assert_eq!(result, dir.join("photo-1.jpg"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn collision_suffixed_path_fills_gap() {
        let dir = std::env::temp_dir().join("rust-feh-collision-test-3");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("photo.jpg"), b"x").unwrap();
        std::fs::write(dir.join("photo-1.jpg"), b"x").unwrap();
        let result = collision_suffixed_path(&dir, "photo.jpg");
        assert_eq!(result, dir.join("photo-2.jpg"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn collision_suffixed_path_dotfile_no_extension_split() {
        let dir = std::env::temp_dir().join("rust-feh-collision-test-4");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(".bashrc"), b"x").unwrap();
        let result = collision_suffixed_path(&dir, ".bashrc");
        assert_eq!(result, dir.join(".bashrc-1"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn collision_suffixed_path_no_extension() {
        let dir = std::env::temp_dir().join("rust-feh-collision-test-5");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("README"), b"x").unwrap();
        let result = collision_suffixed_path(&dir, "README");
        assert_eq!(result, dir.join("README-1"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn collision_suffixed_path_non_ascii() {
        let dir = std::env::temp_dir().join("rust-feh-collision-test-6");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("café-été.jpg"), b"x").unwrap();
        let result = collision_suffixed_path(&dir, "café-été.jpg");
        assert_eq!(result, dir.join("café-été-1.jpg"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn stage_decode_bounds_landscape_downscales() {
        assert_eq!(stage_decode_bounds(4000, 2000, 2000), (2000, 1000));
    }

    #[test]
    fn stage_decode_bounds_portrait_downscales() {
        assert_eq!(stage_decode_bounds(2000, 4000, 2000), (1000, 2000));
    }

    #[test]
    fn stage_decode_bounds_small_image_no_upscale() {
        assert_eq!(stage_decode_bounds(400, 300, 2048), (400, 300));
    }

    #[test]
    fn stage_decode_bounds_exact_edge_no_upscale() {
        assert_eq!(stage_decode_bounds(2048, 1024, 2048), (2048, 1024));
    }

    #[test]
    fn stage_decode_bounds_zero_guards() {
        assert_eq!(stage_decode_bounds(0, 100, 2048), (0, 100));
        assert_eq!(stage_decode_bounds(100, 100, 0), (100, 100));
        assert_eq!(stage_decode_bounds(100, 0, 2048), (100, 0));
    }

    #[test]
    fn action_prefs_round_trip_save_and_load() {
        let _guard = ACTION_PREFS_TEST_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let path = action_prefs_path();
        // Save original if it exists
        let original_backup = std::fs::read(&path).ok();

        // Remove the file if it exists to start fresh
        let _ = std::fs::remove_file(&path);

        // Create and save test prefs
        let test_prefs = ActionPrefs {
            version: 1,
            last_destination: Some(PathBuf::from("/tmp/some/dir")),
        };

        save_action_prefs(&test_prefs).expect("save should succeed");

        // Load it back
        let loaded = load_action_prefs();

        // Verify round-trip equality
        assert_eq!(loaded, test_prefs);

        // Restore original or clean up
        let _ = std::fs::remove_file(&path);
        if let Some(backup_data) = original_backup {
            let _ = std::fs::write(&path, backup_data);
        }
    }

    #[test]
    fn action_prefs_missing_file_returns_default() {
        let _guard = ACTION_PREFS_TEST_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let path = action_prefs_path();
        // Save original if it exists
        let original_backup = std::fs::read(&path).ok();

        // Remove the file if it exists so path is absent
        let _ = std::fs::remove_file(&path);

        // Load should return default
        let loaded = load_action_prefs();
        assert_eq!(loaded, ActionPrefs::default());

        // Restore original or clean up
        let _ = std::fs::remove_file(&path);
        if let Some(backup_data) = original_backup {
            let _ = std::fs::write(&path, backup_data);
        }
    }

    #[test]
    fn action_prefs_corrupt_json_recovers_to_default() {
        let _guard = ACTION_PREFS_TEST_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let path = action_prefs_path();
        // Save original if it exists
        let original_backup = std::fs::read(&path).ok();

        // Create parent dirs if needed
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        // Write garbage bytes (not valid JSON)
        std::fs::write(&path, b"not valid json garbage{").expect("write garbage");

        // Load should return default and log warning
        let loaded = load_action_prefs();
        assert_eq!(loaded, ActionPrefs::default());

        // Restore original or clean up
        let _ = std::fs::remove_file(&path);
        if let Some(backup_data) = original_backup {
            let _ = std::fs::write(&path, backup_data);
        }
    }

    #[test]
    fn loss_proof_move_same_dir_or_same_fs_succeeds() {
        let temp_base = std::env::temp_dir().join("rust-feh-move-test-1");
        let _ = std::fs::remove_dir_all(&temp_base);
        std::fs::create_dir_all(&temp_base).unwrap();

        let src = temp_base.join("source.txt");
        let dest_dir = temp_base.join("dest");
        std::fs::create_dir_all(&dest_dir).unwrap();

        let content = b"test file content";
        std::fs::write(&src, content).unwrap();

        let plan = plan_loss_proof_move(&src, &dest_dir).unwrap();
        let result = execute_move_plan(&plan).unwrap();

        assert!(
            !src.exists(),
            "source should be removed after successful move"
        );
        assert!(result.exists(), "destination should exist");
        assert_eq!(
            std::fs::read(&result).unwrap(),
            content,
            "destination content should match source"
        );
        assert_eq!(
            result,
            dest_dir.join("source.txt"),
            "returned path should match dest_dir.join(original_name)"
        );

        let _ = std::fs::remove_dir_all(&temp_base);
    }

    #[test]
    fn loss_proof_move_collision_at_destination_gets_suffixed() {
        let temp_base = std::env::temp_dir().join("rust-feh-move-test-2");
        let _ = std::fs::remove_dir_all(&temp_base);
        std::fs::create_dir_all(&temp_base).unwrap();

        let src = temp_base.join("photo.jpg");
        let dest_dir = temp_base.join("dest");
        std::fs::create_dir_all(&dest_dir).unwrap();

        std::fs::write(&src, b"A").unwrap();
        std::fs::write(dest_dir.join("photo.jpg"), b"B").unwrap();

        let plan = plan_loss_proof_move(&src, &dest_dir).unwrap();
        let result = execute_move_plan(&plan).unwrap();

        assert!(!src.exists(), "source should be removed after move");
        assert_eq!(
            result,
            dest_dir.join("photo-1.jpg"),
            "collision should result in -1 suffix"
        );
        assert_eq!(
            std::fs::read(&result).unwrap(),
            b"A",
            "new file should have source content"
        );
        assert_eq!(
            std::fs::read(dest_dir.join("photo.jpg")).unwrap(),
            b"B",
            "existing photo.jpg should be untouched"
        );

        let _ = std::fs::remove_dir_all(&temp_base);
    }

    #[test]
    fn loss_proof_move_unwritable_destination_preserves_source() {
        let temp_base = std::env::temp_dir().join("rust-feh-move-test-3");
        let _ = std::fs::remove_dir_all(&temp_base);
        std::fs::create_dir_all(&temp_base).unwrap();

        let src = temp_base.join("source.txt");
        let dest_dir = temp_base.join("dest");
        std::fs::create_dir_all(&dest_dir).unwrap();

        let content = b"source content";
        std::fs::write(&src, content).unwrap();

        // Make destination unwritable
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(0o555);
            std::fs::set_permissions(&dest_dir, perms).unwrap();
        }

        let plan = plan_loss_proof_move(&src, &dest_dir).unwrap();
        let result = execute_move_plan(&plan);

        assert!(
            result.is_err(),
            "move should fail with unwritable destination"
        );
        assert!(src.exists(), "source should still exist after failed move");
        assert_eq!(
            std::fs::read(&src).unwrap(),
            content,
            "source content should be unchanged"
        );

        // Restore permissions for cleanup
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(0o755);
            let _ = std::fs::set_permissions(&dest_dir, perms);
        }

        let _ = std::fs::remove_dir_all(&temp_base);
    }

    #[test]
    fn loss_proof_move_source_preserved_on_verification_failure() {
        let temp_base = std::env::temp_dir().join("rust-feh-move-test-4");
        let _ = std::fs::remove_dir_all(&temp_base);
        std::fs::create_dir_all(&temp_base).unwrap();

        let src = temp_base.join("source.txt");
        let dest_dir = temp_base.join("dest");
        std::fs::create_dir_all(&dest_dir).unwrap();

        std::fs::write(&src, b"test").unwrap();

        let plan = plan_loss_proof_move(&src, &dest_dir).unwrap();

        // Verify plan structure
        assert_eq!(plan.source, src);
        assert_eq!(plan.final_dest, dest_dir.join("source.txt"));
        assert_eq!(plan.temp_dest.parent(), Some(dest_dir.as_path()));
        assert!(
            plan.temp_dest
                .file_name()
                .unwrap()
                .to_string_lossy()
                .contains(&std::process::id().to_string()),
            "temp filename should contain process id"
        );

        let _ = std::fs::remove_dir_all(&temp_base);
    }

    #[test]
    fn save_copy_to_basic_copy() {
        let temp_base = std::env::temp_dir().join("rust-feh-copy-test-1");
        let _ = std::fs::remove_dir_all(&temp_base);
        std::fs::create_dir_all(&temp_base).unwrap();

        let src = temp_base.join("source.txt");
        let dest_dir = temp_base.join("dest");
        std::fs::create_dir_all(&dest_dir).unwrap();

        let content = b"test file content";
        std::fs::write(&src, content).unwrap();

        let result = save_copy_to(&src, &dest_dir).unwrap();

        assert!(result.exists(), "destination should exist");
        assert!(src.exists(), "source should still exist after copy");
        assert_eq!(
            std::fs::read(&result).unwrap(),
            content,
            "destination content should match source"
        );
        assert_eq!(
            std::fs::read(&src).unwrap(),
            content,
            "source content should be unchanged"
        );

        let _ = std::fs::remove_dir_all(&temp_base);
    }

    #[test]
    fn save_copy_to_collision_gets_suffixed() {
        let temp_base = std::env::temp_dir().join("rust-feh-copy-test-2");
        let _ = std::fs::remove_dir_all(&temp_base);
        std::fs::create_dir_all(&temp_base).unwrap();

        let src = temp_base.join("photo.jpg");
        let dest_dir = temp_base.join("dest");
        std::fs::create_dir_all(&dest_dir).unwrap();

        let src_content = b"A";
        let existing_content = b"B";
        std::fs::write(&src, src_content).unwrap();
        std::fs::write(dest_dir.join("photo.jpg"), existing_content).unwrap();

        let result = save_copy_to(&src, &dest_dir).unwrap();

        assert_eq!(
            result,
            dest_dir.join("photo-1.jpg"),
            "collision should result in -1 suffix"
        );
        assert_eq!(
            std::fs::read(&result).unwrap(),
            src_content,
            "new file should have source content"
        );
        assert_eq!(
            std::fs::read(dest_dir.join("photo.jpg")).unwrap(),
            existing_content,
            "existing photo.jpg should be untouched"
        );
        assert!(src.exists(), "source should still exist after copy");

        let _ = std::fs::remove_dir_all(&temp_base);
    }

    #[test]
    fn save_copy_to_non_ascii_name() {
        let temp_base = std::env::temp_dir().join("rust-feh-copy-test-3");
        let _ = std::fs::remove_dir_all(&temp_base);
        std::fs::create_dir_all(&temp_base).unwrap();

        let src = temp_base.join("café.jpg");
        let dest_dir = temp_base.join("dest");
        std::fs::create_dir_all(&dest_dir).unwrap();

        let content = b"image data";
        std::fs::write(&src, content).unwrap();

        let result = save_copy_to(&src, &dest_dir).unwrap();

        assert!(result.exists(), "destination should exist");
        assert_eq!(
            result.file_name().unwrap().to_string_lossy(),
            "café.jpg",
            "filename should preserve non-ASCII characters"
        );
        assert_eq!(
            std::fs::read(&result).unwrap(),
            content,
            "destination content should match source"
        );

        let _ = std::fs::remove_dir_all(&temp_base);
    }

    #[test]
    fn save_copy_to_missing_source_returns_error() {
        let temp_base = std::env::temp_dir().join("rust-feh-copy-test-4");
        let _ = std::fs::remove_dir_all(&temp_base);
        std::fs::create_dir_all(&temp_base).unwrap();

        let src = temp_base.join("nonexistent.txt");
        let dest_dir = temp_base.join("dest");
        std::fs::create_dir_all(&dest_dir).unwrap();

        let result = save_copy_to(&src, &dest_dir);

        assert!(result.is_err(), "should return error for missing source");

        let _ = std::fs::remove_dir_all(&temp_base);
    }

    #[test]
    fn format_action_outcome_ok_with_produced() {
        let outcome = ActionOutcome {
            action: ActionKind::Context(ContextAction::SaveCopyTo),
            image: PathBuf::from("/a/photo.jpg"),
            destination: None,
            result: ActionResult::Ok {
                produced: Some(PathBuf::from("/b/photo.jpg")),
            },
        };

        let formatted = format_action_outcome(&outcome);

        assert!(formatted.contains("Save a copy"));
        assert!(formatted.contains("/a/photo.jpg"));
        assert!(formatted.contains("/b/photo.jpg"));
    }

    #[test]
    fn format_action_outcome_ok_no_produced_falls_back_to_destination() {
        let outcome = ActionOutcome {
            action: ActionKind::Context(ContextAction::SaveCopyTo),
            image: PathBuf::from("/a/photo.jpg"),
            destination: Some(PathBuf::from("/b")),
            result: ActionResult::Ok { produced: None },
        };

        let formatted = format_action_outcome(&outcome);

        assert!(formatted.contains("Save a copy"));
        assert!(formatted.contains("/b"));
    }

    #[test]
    fn format_action_outcome_err_includes_reason() {
        let outcome = ActionOutcome {
            action: ActionKind::Context(ContextAction::SaveCopyTo),
            image: PathBuf::from("/a/photo.jpg"),
            destination: None,
            result: ActionResult::Err {
                reason: "disk full".into(),
            },
        };

        let formatted = format_action_outcome(&outcome);

        assert!(formatted.contains("failed"));
        assert!(formatted.contains("disk full"));
    }

    #[test]
    fn format_action_outcome_round_trip_label() {
        let outcome = ActionOutcome {
            action: ActionKind::RoundTrip,
            image: PathBuf::from("/a/photo.jpg"),
            destination: None,
            result: ActionResult::Ok { produced: None },
        };

        let formatted = format_action_outcome(&outcome);

        assert!(formatted.contains("Round trip"));
    }

    #[test]
    fn format_action_outcome_covers_all_context_actions() {
        let actions = vec![
            ContextAction::SaveCopyTo,
            ContextAction::MoveTo,
            ContextAction::ResizeCopy,
            ContextAction::ConvertFormat,
            ContextAction::CopyPath,
            ContextAction::CopyImage,
        ];

        let mut labels = Vec::new();
        for action in actions {
            let outcome = ActionOutcome {
                action: ActionKind::Context(action),
                image: PathBuf::from("/a/photo.jpg"),
                destination: None,
                result: ActionResult::Ok { produced: None },
            };

            let formatted = format_action_outcome(&outcome);
            assert!(
                !formatted.is_empty(),
                "formatted string should not be empty"
            );
            labels.push(formatted);
        }

        // Verify all labels are distinct
        for i in 0..labels.len() {
            for j in (i + 1)..labels.len() {
                assert_ne!(labels[i], labels[j], "labels should be distinct");
            }
        }
    }

    // ---- Feature 016 T015: viewer_spawn_command (argument-vector safety) ----

    #[test]
    fn viewer_spawn_command_produces_expected_argument_vector() {
        let (program, args, envs) = viewer_spawn_command(
            Path::new("/tmp/rust-feh/filelist-1.txt"),
            Path::new("/home/user/pics/photo.jpg"),
            Path::new("/tmp/rust-feh/handoff-1"),
            Path::new("/home/user/.config/rust-feh/viewer-profile"),
        );
        assert_eq!(program, "feh");
        assert_eq!(
            args,
            vec![
                "--geometry".to_string(),
                FEH_VIEWER_GEOMETRY.to_string(),
                "--scale-down".to_string(),
                "--zoom".to_string(),
                FEH_VIEWER_ZOOM.to_string(),
                "--info".to_string(),
                "echo %F > '/tmp/rust-feh/handoff-1'".to_string(),
                "--filelist".to_string(),
                "/tmp/rust-feh/filelist-1.txt".to_string(),
                "--start-at".to_string(),
                "/home/user/pics/photo.jpg".to_string(),
            ]
        );
        assert_eq!(
            envs,
            vec![(
                "XDG_CONFIG_HOME".to_string(),
                "/home/user/.config/rust-feh/viewer-profile".to_string()
            )]
        );
    }

    #[test]
    fn viewer_spawn_command_never_builds_a_shell_string_for_the_whole_invocation() {
        let filelist = "/tmp/list.txt";
        let start_at = "/tmp/pics/start.jpg";
        let handoff = "/tmp/handoff-xyz";
        let (_program, args, _envs) = viewer_spawn_command(
            Path::new(filelist),
            Path::new(start_at),
            Path::new(handoff),
            Path::new("/tmp/profile"),
        );
        // The invocation is many separate arguments, never one joined shell line.
        assert!(
            args.len() > 1,
            "expected an argument vector, not one string"
        );
        // No single argument concatenates more than one of the three distinct
        // paths — proving they never get glued together into shell-parsed text.
        for arg in &args {
            let count = [filelist, start_at, handoff]
                .into_iter()
                .filter(|needle| arg.contains(*needle))
                .count();
            assert!(
                count <= 1,
                "argument {arg:?} concatenates multiple distinct paths"
            );
        }
    }

    #[test]
    fn viewer_spawn_command_handles_adversarial_filenames_in_filelist_and_start_at_paths() {
        let adversarial = "/tmp/weird; rm -rf ~ $(whoami) \"quoted\" 'single'.jpg";
        let filelist = "/tmp/evil dir; touch pwned/filelist.txt";
        let (_program, args, _envs) = viewer_spawn_command(
            Path::new(filelist),
            Path::new(adversarial),
            Path::new("/tmp/rust-feh/handoff-2"),
            Path::new("/tmp/profile"),
        );
        // --start-at value is the path VERBATIM: it is its own arg-vector element,
        // not shell-parsed text, so it receives no escaping and no truncation.
        let start_idx = args.iter().position(|a| a == "--start-at").unwrap();
        assert_eq!(args[start_idx + 1], adversarial);
        // --filelist value is likewise verbatim.
        let fl_idx = args.iter().position(|a| a == "--filelist").unwrap();
        assert_eq!(args[fl_idx + 1], filelist);
        // Same total arg count/shape as the ordinary case: the adversarial
        // content did not fracture into extra or fewer arguments.
        assert_eq!(args.len(), 11);
    }

    #[test]
    fn viewer_spawn_command_shell_quotes_handoff_path_in_info_command() {
        let handoff = "/tmp/rust-feh/handoff-o'brien-1";
        let (_program, args, _envs) = viewer_spawn_command(
            Path::new("/tmp/list.txt"),
            Path::new("/tmp/start.jpg"),
            Path::new(handoff),
            Path::new("/tmp/profile"),
        );
        let info_idx = args.iter().position(|a| a == "--info").unwrap();
        let info = &args[info_idx + 1];
        // POSIX single-quote escaping of the embedded quote: close, escaped
        // literal quote, reopen. A real `sh -c` would treat the path as one
        // intact argument.
        assert!(
            info.contains("'\\''"),
            "info value not POSIX-escaped: {info}"
        );
        assert_eq!(info, "echo %F > '/tmp/rust-feh/handoff-o'\\''brien-1'");
    }

    #[test]
    fn viewer_spawn_command_uses_geometry_and_zoom_constants() {
        let (_program, args, _envs) = viewer_spawn_command(
            Path::new("/tmp/list.txt"),
            Path::new("/tmp/start.jpg"),
            Path::new("/tmp/handoff"),
            Path::new("/tmp/profile"),
        );
        let g_idx = args.iter().position(|a| a == "--geometry").unwrap();
        assert_eq!(args[g_idx + 1], FEH_VIEWER_GEOMETRY);
        let z_idx = args.iter().position(|a| a == "--zoom").unwrap();
        assert_eq!(args[z_idx + 1], FEH_VIEWER_ZOOM);
    }

    // ---- Feature 016 T016: validate_handoff (untrusted-input validation) ----

    #[test]
    fn validate_handoff_accepts_exact_match_in_filelist() {
        let dir = std::env::temp_dir().join("rust-feh-handoff-accept");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let a = dir.join("a.jpg");
        let b = dir.join("b.jpg");
        std::fs::write(&a, b"x").unwrap();
        std::fs::write(&b, b"x").unwrap();
        let filelist = vec![a.clone(), b.clone()];
        let content = b.display().to_string();
        assert_eq!(validate_handoff(&content, &filelist), Some(b));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn validate_handoff_rejects_path_not_in_filelist() {
        let dir = std::env::temp_dir().join("rust-feh-handoff-notin");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let a = dir.join("a.jpg");
        let outsider = dir.join("outsider.jpg");
        std::fs::write(&a, b"x").unwrap();
        // Real, existing file — but membership in the filelist is the gate, not
        // mere existence.
        std::fs::write(&outsider, b"x").unwrap();
        let filelist = vec![a];
        let content = outsider.display().to_string();
        assert_eq!(validate_handoff(&content, &filelist), None);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn validate_handoff_rejects_garbage_content() {
        let filelist = vec![PathBuf::from("/tmp/whatever/a.jpg")];
        assert_eq!(
            validate_handoff("\x00not a path!! $(whoami)", &filelist),
            None
        );
    }

    #[test]
    fn validate_handoff_rejects_empty_content() {
        let filelist = vec![PathBuf::from("/tmp/a.jpg")];
        assert_eq!(validate_handoff("", &filelist), None);
        assert_eq!(validate_handoff("   \n  \t", &filelist), None);
    }

    #[test]
    fn validate_handoff_rejects_symlink_escape_attempt() {
        let base = std::env::temp_dir().join("rust-feh-handoff-symlink");
        let _ = std::fs::remove_dir_all(&base);
        let trusted = base.join("trusted");
        let outside = base.join("outside");
        std::fs::create_dir_all(&trusted).unwrap();
        std::fs::create_dir_all(&outside).unwrap();
        let real = trusted.join("real.jpg");
        std::fs::write(&real, b"x").unwrap();
        let secret = outside.join("secret.jpg");
        std::fs::write(&secret, b"x").unwrap();
        // A symlink that superficially sits inside the trusted dir but points at
        // the outside file.
        let link = trusted.join("link.jpg");
        std::os::unix::fs::symlink(&secret, &link).unwrap();
        // Filelist contains only the genuinely-trusted real file.
        let filelist = vec![real];
        let content = link.display().to_string();
        // link canonicalizes to outside/secret.jpg, which no filelist entry
        // canonically equals -> rejected. This is the canonicalize-and-compare
        // defense defeating the escape.
        assert_eq!(validate_handoff(&content, &filelist), None);
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn validate_handoff_deleted_image_falls_back_to_nearest_surviving_neighbor() {
        let dir = std::env::temp_dir().join("rust-feh-handoff-deleted");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let files: Vec<PathBuf> = (1..=5).map(|i| dir.join(format!("{i}.jpg"))).collect();
        for f in &files {
            std::fs::write(f, b"x").unwrap();
        }
        // Delete the 3rd file AFTER building the filelist: its raw path still
        // matches the filelist (canonicalize fails, raw fallback succeeds), but
        // the file itself is gone.
        std::fs::remove_file(&files[2]).unwrap();
        let content = files[2].display().to_string();
        let result = validate_handoff(&content, &files).expect("expected a surviving neighbor");
        assert_ne!(result, files[2], "must not return the deleted path");
        assert!(
            result == files[1] || result == files[3],
            "expected nearest surviving neighbor (files[1] or files[3]), got {result:?}"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn validate_handoff_trims_trailing_newline() {
        let dir = std::env::temp_dir().join("rust-feh-handoff-newline");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let a = dir.join("a.jpg");
        std::fs::write(&a, b"x").unwrap();
        let filelist = vec![a.clone()];
        // echo's trailing newline, the realistic case.
        let content = format!("{}\n", a.display());
        assert_eq!(validate_handoff(&content, &filelist), Some(a));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn validate_handoff_only_reads_first_line() {
        let dir = std::env::temp_dir().join("rust-feh-handoff-firstline");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let a = dir.join("a.jpg");
        std::fs::write(&a, b"x").unwrap();
        let filelist = vec![a.clone()];
        // Valid path on line 1, appended garbage after (corrupted/appended file).
        let content = format!("{}\n/etc/passwd\ngarbage line\n", a.display());
        assert_eq!(validate_handoff(&content, &filelist), Some(a));
        let _ = std::fs::remove_dir_all(&dir);
    }

    // ---- Feature 016 T017: handoff_path (handoff file path generation) ----

    #[test]
    fn handoff_path_includes_pid_and_viewer_id() {
        let path = handoff_path(1234, 7);
        let file_name = path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .into_owned();
        assert_eq!(file_name, "handoff-1234-7");
        assert_eq!(path.parent().unwrap(), runtime_cache_dir());
    }

    #[test]
    fn handoff_path_distinct_per_viewer_id() {
        let path1 = handoff_path(1234, 1);
        let path2 = handoff_path(1234, 2);
        assert_ne!(path1, path2);
    }

    #[test]
    fn cleanup_stale_handoffs_removes_only_handoff_prefixed_files() {
        let _guard = HANDOFF_CLEANUP_TEST_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let dir = runtime_cache_dir();
        let _ = std::fs::create_dir_all(&dir);

        // Use distinctive names to identify our test files
        let handoff_file_1 = dir.join("handoff-9999-cleanup-test-1");
        let handoff_file_2 = dir.join("handoff-9999-cleanup-test-2");
        let non_handoff_file = dir.join("filelist-9999.txt");

        // Create test files
        std::fs::write(&handoff_file_1, b"test").unwrap();
        std::fs::write(&handoff_file_2, b"test").unwrap();
        std::fs::write(&non_handoff_file, b"test").unwrap();

        // Call cleanup
        let removed = cleanup_stale_handoffs();

        // Both handoff files should be gone
        assert!(!handoff_file_1.exists(), "handoff file 1 should be removed");
        assert!(!handoff_file_2.exists(), "handoff file 2 should be removed");

        // Non-handoff file should still exist
        assert!(non_handoff_file.exists(), "non-handoff file should remain");

        // Cleanup our test file
        let _ = std::fs::remove_file(&non_handoff_file);

        // At least 2 files were removed (our test files, possibly others)
        assert!(removed >= 2, "should have removed at least 2 files, got {removed}");
    }

    #[test]
    fn cleanup_stale_handoffs_does_not_panic_when_called() {
        let _guard = HANDOFF_CLEANUP_TEST_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        // This tests that cleanup is safe to call; it should not panic
        // even if the directory exists and has stale handoffs.
        let _ = cleanup_stale_handoffs();
        // If we got here without panicking, the test passes.
    }

    #[test]
    fn viewer_profile_dir_creation_is_idempotent() {
        let dir1 = viewer_profile_dir();
        assert!(dir1.is_dir(), "viewer profile dir should exist after first call");
        let dir2 = viewer_profile_dir();
        assert_eq!(dir1, dir2, "path should be stable across calls");
        assert!(dir2.is_dir(), "viewer profile dir should still exist after second call");
    }
}
