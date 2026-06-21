// SPDX-License-Identifier: MIT
//! Feature 005 list presentation — test harness and inventory fixtures.

use rust_feh::scanner::scan_images;
use rust_feh::types::FileStatus;
use rust_feh::ui_logic::{
    build_folder_tree, detect_converted_status, finalize_scan_entries,
    inventory_awaiting_invariant_holds, refresh_entry_and_inventory,
    relative_folder, tree_root_listed_matches_inventory,
};
use std::fs;
use std::path::{Path, PathBuf};

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("rust-feh-005-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

#[test]
fn feature_005_test_harness_exists() {
    assert!(true);
}

#[test]
fn scan_inventory_counts_native_and_skipped() {
    let dir = temp_dir("inventory");
    fs::write(dir.join("photo.jpg"), b"x").unwrap();
    fs::write(dir.join("notes.txt"), b"x").unwrap();
    fs::write(dir.join("clip.mp4"), b"x").unwrap();

    let result = scan_images(&dir, false, false);
    assert_eq!(result.entries.len(), 1);
    assert_eq!(result.inventory.native_listed, 1);
    assert_eq!(result.inventory.non_image_skipped, 2);

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn relative_folder_nested_path() {
    let root = Path::new("/data/photos");
    assert_eq!(
        relative_folder(Some(root), Path::new("/data/photos/sub/deep/img.png")),
        "sub/deep"
    );
}

#[test]
fn converted_detection_marks_source_and_artifact() {
    let dir = temp_dir("converted");
    let source = dir.join("vacation.heic");
    fs::write(&source, b"x").unwrap();
    fs::write(dir.join("vacation_processed.jpg"), b"x").unwrap();
    assert!(detect_converted_status(&source));

    let (entries, inventory) = finalize_scan_entries(
        vec![rust_feh::types::ImageEntry::with_status(
            source,
            FileStatus::MagickDetected,
        )],
        0,
        false,
    );
    assert_eq!(entries[0].status, FileStatus::Converted);
    assert_eq!(inventory.converted, 1);
    assert_eq!(inventory.awaiting_convert, 0);

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn mixed_fixture_inventory_counts() {
    let dir = temp_dir("mixed");
    fs::write(dir.join("readme.txt"), b"x").unwrap();
    fs::write(dir.join("photo.jpg"), b"x").unwrap();
    let sub = dir.join("sub");
    fs::create_dir_all(&sub).unwrap();
    fs::write(sub.join("a.png"), b"x").unwrap();
    let deep = sub.join("deep");
    fs::create_dir_all(&deep).unwrap();
    fs::write(deep.join("b.webp"), b"x").unwrap();

    let result = scan_images(&dir, true, false);
    let (entries, inventory) = finalize_scan_entries(
        result.entries,
        result.inventory.non_image_skipped,
        result.inventory.magick_identify_truncated,
    );

    assert_eq!(entries.len(), 3);
    assert_eq!(inventory.native_listed, 3);
    assert_eq!(inventory.non_image_skipped, 1);
    assert!(inventory_awaiting_invariant_holds(&inventory));

    let tree = build_folder_tree(&entries, Some(&dir), &[0, 1, 2]);
    assert_eq!(tree.listed_count, 3);
    assert_eq!(tree.children["sub"].listed_count, 2);

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn awaiting_convert_matches_magick_minus_converted_magick() {
    let dir = temp_dir("awaiting");
    let heic = dir.join("sample.heic");
    fs::write(&heic, b"x").unwrap();
    let (entries, inventory) = finalize_scan_entries(
        vec![
            rust_feh::types::ImageEntry::with_status(heic.clone(), FileStatus::MagickDetected),
            rust_feh::types::ImageEntry::with_status(
                dir.join("other.heic"),
                FileStatus::MagickDetected,
            ),
        ],
        0,
        false,
    );
    assert_eq!(inventory.magick_detected, 2);
    assert_eq!(inventory.awaiting_convert, 2);

    fs::write(dir.join("sample_processed.jpg"), b"x").unwrap();
    let (entries, inventory) = finalize_scan_entries(entries, 0, false);
    assert_eq!(entries[0].status, FileStatus::Converted);
    assert_eq!(inventory.converted, 1);
    assert_eq!(inventory.awaiting_convert, 1);
    assert_eq!(
        inventory.awaiting_convert,
        inventory.magick_detected - inventory.converted
    );
    assert!(inventory_awaiting_invariant_holds(&inventory));

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn sc005_mixed_fixture_tree_matches_inventory() {
    let dir = temp_dir("sc005");
    fs::write(dir.join("readme.txt"), b"x").unwrap();
    fs::write(dir.join("photo.jpg"), b"x").unwrap();
    let sub = dir.join("sub");
    fs::create_dir_all(&sub).unwrap();
    fs::write(sub.join("a.png"), b"x").unwrap();

    let result = scan_images(&dir, true, false);
    let (entries, inventory) = finalize_scan_entries(
        result.entries,
        result.inventory.non_image_skipped,
        result.inventory.magick_identify_truncated,
    );
    let indices: Vec<_> = (0..entries.len()).collect();
    let tree = build_folder_tree(&entries, Some(&dir), &indices);
    assert!(tree_root_listed_matches_inventory(&tree, &inventory));
    assert_eq!(tree.listed_count, inventory.native_listed);

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn post_resize_refresh_without_rescan() {
    let dir = temp_dir("resize-refresh");
    let source = dir.join("photo.jpg");
    fs::write(&source, b"x").unwrap();
    let mut entries = vec![rust_feh::types::ImageEntry::new(source.clone())];
    fs::write(dir.join("photo_processed.jpg"), b"x").unwrap();
    let inventory = refresh_entry_and_inventory(&mut entries, &source, 1, false);
    assert_eq!(entries[0].status, FileStatus::Converted);
    assert_eq!(inventory.converted, 1);
    assert_eq!(inventory.native_listed, 0);

    let _ = fs::remove_dir_all(&dir);
}

#[test]
#[ignore = "requires ImageMagick on PATH and a valid heic sample"]
fn heic_magick_detect_when_installed() {
    if !which::which("magick").is_ok() && !which::which("convert").is_ok() {
        return;
    }
    let dir = temp_dir("heic");
    let heic = dir.join("sample.heic");
    fs::write(&heic, b"\x00").unwrap();
    let result = scan_images(&dir, false, true);
    let _ = result.inventory;
    let _ = fs::remove_dir_all(&dir);
}