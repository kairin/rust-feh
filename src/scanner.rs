// SPDX-License-Identifier: MIT
//! Directory scanner (MVP sync version; will become async + cached + walkdir).

use std::io;
use std::path::Path;
use std::process::Command;
use walkdir::WalkDir;

use crate::types::{FileStatus, ImageEntry, ScanInventory};

pub const MAGICK_IDENTIFY_CAP: usize = 500;
pub const SCAN_WARNING_CAP: usize = 50;

/// Format a walkdir error for the activity log (feature 004 / 011).
pub fn format_walk_warning(err: &walkdir::Error) -> String {
    let path = err
        .path()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| "(unknown path)".to_string());
    if err
        .io_error()
        .is_some_and(|io| io.kind() == io::ErrorKind::PermissionDenied)
    {
        format!("Permission denied, skipped: {path}")
    } else {
        format!("Scan skip: {err} ({path})")
    }
}

/// Cap warning volume; append summary when truncated (feature 004 edge case).
pub fn summarize_scan_warnings(mut warnings: Vec<String>) -> Vec<String> {
    if warnings.len() <= SCAN_WARNING_CAP {
        return warnings;
    }
    let omitted = warnings.len() - SCAN_WARNING_CAP;
    warnings.truncate(SCAN_WARNING_CAP);
    warnings.push(format!(
        "… {omitted} more scan warning(s) omitted (cap {SCAN_WARNING_CAP})"
    ));
    warnings
}

/// Result of scanning a directory (feature 005).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScanResult {
    pub entries: Vec<ImageEntry>,
    pub warnings: Vec<String>,
    pub inventory: ScanInventory,
}

/// Scan `dir` for images and inventory stats. Optional ImageMagick `identify` for unlisted types.
pub fn scan_images(dir: &Path, recursive: bool, magick_available: bool) -> ScanResult {
    let mut entries = Vec::new();
    let mut warnings = Vec::new();
    let mut non_image_skipped = 0usize;
    let mut magick_identify_calls = 0usize;
    let mut magick_truncated = false;
    let magick_bin = if magick_available {
        which::which("magick")
            .ok()
            .or_else(|| which::which("convert").ok())
    } else {
        None
    };

    let walker = if recursive {
        WalkDir::new(dir).follow_links(false)
    } else {
        WalkDir::new(dir).follow_links(false).max_depth(1)
    };

    for entry in walker.into_iter() {
        match entry {
            Ok(e) => {
                if !e.file_type().is_file() {
                    continue;
                }
                let path = e.path().to_path_buf();
                if is_native_image(&path) {
                    entries.push(ImageEntry::new(path));
                    continue;
                }
                if magick_available && magick_identify_calls < MAGICK_IDENTIFY_CAP {
                    if is_magick_image(&path, magick_bin.as_deref()) {
                        magick_identify_calls += 1;
                        entries.push(ImageEntry::with_status(
                            path,
                            FileStatus::MagickDetected,
                        ));
                        continue;
                    }
                } else if magick_available && magick_identify_calls >= MAGICK_IDENTIFY_CAP {
                    magick_truncated = true;
                }
                non_image_skipped += 1;
            }
            Err(e) => {
                warnings.push(format_walk_warning(&e));
            }
        }
    }

    entries.sort_by_key(|e| e.path.clone());
    let inventory = ScanInventory::from_entries(&entries, non_image_skipped, magick_truncated);
    let warnings = summarize_scan_warnings(warnings);
    ScanResult {
        entries,
        warnings,
        inventory,
    }
}

fn is_native_image(path: &Path) -> bool {
    match path.extension().and_then(|e| e.to_str()) {
        Some(ext) => {
            let e = ext.to_lowercase();
            matches!(
                e.as_str(),
                "jpg" | "jpeg" | "png" | "webp" | "gif" | "bmp"
            )
        }
        None => false,
    }
}

fn is_magick_image(path: &Path, magick_bin: Option<&Path>) -> bool {
    let Some(bin) = magick_bin else {
        return false;
    };
    Command::new(bin)
        .arg(path)
        .arg("-format")
        .arg("%m")
        .output()
        .map(|o| o.status.success() && !o.stdout.is_empty())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn scan_nonexistent_dir_returns_empty() {
        let r = scan_images(Path::new("/nonexistent-rust-feh-test-dir"), false, false);
        assert!(r.entries.is_empty());
        assert!(
            r.warnings.iter().any(|w| w.starts_with("Scan skip:")),
            "expected scan skip warning, got: {:?}",
            r.warnings
        );
        assert_eq!(r.inventory.non_image_skipped, 0);
    }

    #[test]
    fn scan_finds_supported_extension() {
        let dir = std::env::temp_dir().join("rust-feh-scanner-test");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("photo.jpg"), b"x").unwrap();
        fs::write(dir.join("notes.txt"), b"x").unwrap();

        let r = scan_images(&dir, false, false);
        assert_eq!(r.entries.len(), 1);
        assert!(r.entries[0].path.ends_with("photo.jpg"));
        assert_eq!(r.inventory.native_listed, 1);
        assert_eq!(r.inventory.non_image_skipped, 1);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn summarize_scan_warnings_truncates_with_summary() {
        let warnings: Vec<_> = (0..60).map(|i| format!("warn {i}")).collect();
        let out = summarize_scan_warnings(warnings);
        assert_eq!(out.len(), SCAN_WARNING_CAP + 1);
        assert!(out.last().unwrap().contains("10 more scan warning"));
    }

    #[test]
    fn summarize_scan_warnings_unchanged_under_cap() {
        let warnings = vec!["a".to_string(), "b".to_string()];
        let out = summarize_scan_warnings(warnings.clone());
        assert_eq!(out, warnings);
    }

    #[test]
    fn format_walk_warning_scan_skip_for_nonexistent_root() {
        let warning = WalkDir::new("/nonexistent-rust-feh-format-walk-test")
            .into_iter()
            .find_map(|e| e.err())
            .map(|e| format_walk_warning(&e))
            .expect("walkdir should error on missing root");
        assert!(warning.starts_with("Scan skip:"));
    }

    #[cfg(unix)]
    #[test]
    fn format_walk_warning_permission_denied_prefix() {
        use std::os::unix::fs::PermissionsExt;

        let dir = std::env::temp_dir().join("rust-feh-format-walk-perms");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let secret = dir.join("secret");
        fs::create_dir_all(&secret).unwrap();
        fs::set_permissions(&secret, fs::Permissions::from_mode(0o000)).unwrap();

        let result = scan_images(&dir, true, false);
        assert!(
            result
                .warnings
                .iter()
                .any(|w| w.starts_with("Permission denied")),
            "got: {:?}",
            result.warnings
        );

        fs::set_permissions(&secret, fs::Permissions::from_mode(0o755)).unwrap();
        let _ = fs::remove_dir_all(&dir);
    }
}