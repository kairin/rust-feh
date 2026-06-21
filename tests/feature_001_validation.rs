// SPDX-License-Identifier: MIT
//! Automated substitutes for quickstart.md V1–V10 / T067–T068 where GUI is not required.

use rust_feh::scanner::scan_images;
use rust_feh::types::ImageEntry;
use rust_feh::ui_logic::{filter_indices, post_scan_status, showing_count_label, FEH_MISSING_MSG};
use std::fs;
use std::path::PathBuf;
use std::time::Instant;

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("rust-feh-val-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

fn make_images(n: usize, prefix: &str) -> Vec<ImageEntry> {
    (0..n)
        .map(|i| ImageEntry::new(PathBuf::from(format!("/data/{prefix}_{i:05}.jpg"))))
        .collect()
}

/// V4 / FR-005 / FR-006 / SC-003: filter 10k entries under 200ms (last keystroke simulation).
#[test]
fn sc003_filter_10k_under_200ms() {
    let images = make_images(10_000, "img");
    let start = Instant::now();
    let filtered = filter_indices(&images, None, "img_050");
    let elapsed = start.elapsed();
    assert!(!filtered.is_empty());
    assert!(
        elapsed.as_millis() < 200,
        "filter took {}ms, SC-003 requires <200ms",
        elapsed.as_millis()
    );
}

/// V2 / FR-004: scan 10k files — metadata path only (no GUI RSS).
#[test]
fn scan_10k_supported_files() {
    let dir = temp_dir("10k");
    for i in 0..10_000 {
        fs::write(dir.join(format!("photo_{i:05}.jpg")), b"x").unwrap();
    }
    let start = Instant::now();
    let result = scan_images(&dir, false, false);
    let elapsed = start.elapsed();
    assert_eq!(result.entries.len(), 10_000);
    assert!(result.warnings.is_empty());
    assert!(
        elapsed.as_secs() < 10,
        "scan 10k took {:?}, FR-010 expects <10s",
        elapsed
    );
    let _ = fs::remove_dir_all(&dir);
}

/// V4: counter format for filtered and zero-match.
#[test]
fn v4_counter_formats() {
    let images = make_images(100, "x");
    let shown = filter_indices(&images, None, "no-such-name").len();
    assert_eq!(showing_count_label(shown, images.len()), "Showing 0 / 100 images");
    let all = filter_indices(&images, None, "");
    assert_eq!(
        showing_count_label(all.len(), images.len()),
        "Showing 100 / 100 images"
    );
}

/// V3 / SC-005: post-scan status does not imply feh spawn (logic-only).
#[test]
fn v3_status_after_load_no_feh_implication() {
    let status = post_scan_status(
        "Loaded 3 images. First selected (click Open in feh to view).",
        true,
    );
    assert!(!status.to_lowercase().contains("launched feh"));
    assert!(status.contains("click Open in feh"));
}

/// V8 / SC-007 / FR-008a: feh missing messaging.
#[test]
fn v8_feh_missing_status_strings() {
    let status = post_scan_status("Loaded 1 images.", false);
    assert!(status.contains(FEH_MISSING_MSG));
}

/// V5 / FR-010: recursive toggle changes count.
#[test]
fn v5_recursive_scan_includes_subdirs() {
    let dir = temp_dir("recursive");
    fs::write(dir.join("top.jpg"), b"x").unwrap();
    let sub = dir.join("sub");
    fs::create_dir_all(&sub).unwrap();
    fs::write(sub.join("nested.jpg"), b"x").unwrap();

    let flat = scan_images(&dir, false, false);
    let rec = scan_images(&dir, true, false);
    assert_eq!(flat.entries.len(), 1);
    assert_eq!(rec.entries.len(), 2);

    let _ = fs::remove_dir_all(&dir);
}

/// V9 / FR-012: empty dir → no selection implied by status helper.
#[test]
fn v9_empty_scan_status() {
    let dir = temp_dir("empty");
    let result = scan_images(&dir, false, false);
    assert!(result.entries.is_empty());
    let status = post_scan_status("No images found", true);
    assert_eq!(status, "No images found");
    let _ = fs::remove_dir_all(&dir);
}

/// T068 / FR-015: permission-denied subdirectory (Unix only).
#[cfg(unix)]
#[test]
fn t068_permission_denied_warning() {
    use std::os::unix::fs::PermissionsExt;

    let dir = temp_dir("perms");
    fs::write(dir.join("visible.jpg"), b"x").unwrap();
    let secret = dir.join("secret");
    fs::create_dir_all(&secret).unwrap();
    fs::write(secret.join("hidden.jpg"), b"x").unwrap();
    fs::set_permissions(&secret, fs::Permissions::from_mode(0o000)).unwrap();

    let result = scan_images(&dir, true, false);
    assert_eq!(result.entries.len(), 1);
    assert!(
        result
            .warnings
            .iter()
            .any(|w| w.contains("Permission denied")),
        "expected permission warning, got: {:?}",
        result.warnings
    );

    fs::set_permissions(&secret, fs::Permissions::from_mode(0o755)).unwrap();
    let _ = fs::remove_dir_all(&dir);
}

#[cfg(not(unix))]
#[test]
fn t068_permission_denied_skipped_non_unix() {
    // FR-015 Unix test; CI on non-Unix passes as no-op.
}

/// T069 / 004-FR-006 / 011: non-permission walkdir errors surface as Scan skip.
#[test]
fn t069_scan_skip_non_permission() {
    let result = scan_images(
        std::path::Path::new("/nonexistent-rust-feh-t069-dir"),
        false,
        false,
    );
    assert!(result.entries.is_empty());
    assert!(
        result
            .warnings
            .iter()
            .any(|w| w.starts_with("Scan skip:")),
        "expected Scan skip warning, got: {:?}",
        result.warnings
    );
}