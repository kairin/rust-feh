// SPDX-License-Identifier: MIT
//! Integration tests for feature 016: context-menu file actions
//! (save-copy, move, resize-copy, convert) without GUI.

use std::fs;
use std::path::PathBuf;

use rust_feh::image_proc::{process_image, ProcessOptions};
use rust_feh::ui_logic::{execute_move_plan, plan_loss_proof_move, save_copy_to};

/// Create a small test PNG with the given color.
fn make_test_png(dir: &std::path::Path, color: [u8; 4]) -> PathBuf {
    let path = dir.join("test.png");
    let img = image::RgbaImage::from_pixel(32, 32, image::Rgba(color));
    img.save(&path).unwrap();
    path
}

/// Create a larger test PNG for resize testing.
fn make_large_test_png(dir: &std::path::Path, color: [u8; 4]) -> PathBuf {
    let path = dir.join("large.png");
    let img = image::RgbaImage::from_pixel(200, 200, image::Rgba(color));
    img.save(&path).unwrap();
    path
}

#[test]
fn save_copy_basic_and_collision() {
    let dir = std::env::temp_dir().join(format!("rust-feh-016-save-copy-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();

    let src_dir = dir.join("src");
    let dest_dir = dir.join("dest");
    fs::create_dir_all(&src_dir).unwrap();
    fs::create_dir_all(&dest_dir).unwrap();

    let src = make_test_png(&src_dir, [255, 0, 0, 255]);
    let src_content = fs::read(&src).unwrap();

    // First save: should succeed
    let copy1 = save_copy_to(&src, &dest_dir).expect("first save_copy_to");
    assert!(copy1.is_file(), "first copy should exist");
    assert_eq!(
        fs::read(&copy1).unwrap(),
        src_content,
        "first copy content should match source"
    );
    assert!(src.is_file(), "source should still exist");

    // Second save to same dest (collision): should get a -1 suffix
    let copy2 = save_copy_to(&src, &dest_dir).expect("second save_copy_to");
    assert!(copy2.is_file(), "second copy should exist");
    assert_ne!(copy1, copy2, "second copy should have different path");
    assert!(
        copy2.to_string_lossy().contains("-1"),
        "suffix should be -1"
    );

    // First copy should be untouched
    assert!(copy1.is_file(), "first copy should still exist");
    assert_eq!(
        fs::read(&copy1).unwrap(),
        src_content,
        "first copy content should be unchanged"
    );

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn move_to_advances_and_is_loss_proof() {
    let dir = std::env::temp_dir().join(format!("rust-feh-016-move-basic-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();

    let src_dir = dir.join("src");
    let dest_dir = dir.join("dest");
    fs::create_dir_all(&src_dir).unwrap();
    fs::create_dir_all(&dest_dir).unwrap();

    let src = make_test_png(&src_dir, [0, 255, 0, 255]);
    let src_content_before = fs::read(&src).unwrap();

    // Plan and execute move
    let plan = plan_loss_proof_move(&src, &dest_dir).expect("plan_loss_proof_move");
    let moved = execute_move_plan(&plan).expect("execute_move_plan");

    // Verify: source is gone, destination exists with correct content
    assert!(!src.is_file(), "source should no longer exist");
    assert!(moved.is_file(), "moved file should exist");
    assert_eq!(
        fs::read(&moved).unwrap(),
        src_content_before,
        "moved file content should match original"
    );

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn move_to_duplicate_name_destination_gets_suffixed() {
    let dir = std::env::temp_dir().join(format!(
        "rust-feh-016-move-collision-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();

    let src_dir = dir.join("src");
    let dest_dir = dir.join("dest");
    fs::create_dir_all(&src_dir).unwrap();
    fs::create_dir_all(&dest_dir).unwrap();

    // Create source: red pixel
    let src = make_test_png(&src_dir, [255, 0, 0, 255]);
    let src_content = fs::read(&src).unwrap();

    // Create pre-existing file in dest with same name but different content
    let existing = dest_dir.join("test.png");
    let existing_img = image::RgbaImage::from_pixel(32, 32, image::Rgba([0, 0, 255, 255]));
    existing_img.save(&existing).unwrap();
    let existing_content = fs::read(&existing).unwrap();

    // Plan and execute move
    let plan = plan_loss_proof_move(&src, &dest_dir).expect("plan_loss_proof_move");
    let moved = execute_move_plan(&plan).expect("execute_move_plan");

    // Verify: source gone, pre-existing file untouched, moved file at -1 suffix
    assert!(!src.is_file(), "source should be gone");
    assert!(
        moved.to_string_lossy().contains("-1"),
        "moved file should have -1 suffix"
    );
    assert!(moved.is_file(), "moved file should exist at suffixed path");
    assert_eq!(
        fs::read(&moved).unwrap(),
        src_content,
        "moved file content should match source"
    );

    // Pre-existing should be exactly as it was
    assert!(existing.is_file(), "pre-existing file should still exist");
    assert_eq!(
        fs::read(&existing).unwrap(),
        existing_content,
        "pre-existing file should be untouched (SC-003: no overwrites)"
    );

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn resize_copy_produces_smaller_output_original_untouched() {
    let dir = std::env::temp_dir().join(format!("rust-feh-016-resize-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();

    let src = make_large_test_png(&dir, [0, 128, 255, 255]);
    let src_content_before = fs::read(&src).unwrap();
    let src_img = image::open(&src).unwrap();
    let orig_width = src_img.width();
    let orig_height = src_img.height();
    assert_eq!(orig_width, 200, "source should be 200x200");
    assert_eq!(orig_height, 200, "source should be 200x200");

    let output_path = dir.join("photo_resized.jpg");
    let opts = ProcessOptions {
        width: None,
        height: None,
        percent: Some(50.0),
        fit: None,
        filter: None,
        target_format: Some("jpg".into()),
        quality: Some(80),
        output_path: Some(output_path.clone()),
    };

    let result = process_image(&src, &opts).expect("process_image");
    assert!(result.is_file(), "output file should exist");
    assert_eq!(
        result, output_path,
        "output path should match requested output_path"
    );

    // Check output dimensions (50% of 200 = 100, with some rounding tolerance)
    let resized_img = image::open(&result).unwrap();
    let resized_width = resized_img.width();
    let resized_height = resized_img.height();
    assert!(
        (95..=105).contains(&resized_width),
        "resized width should be ~100, got {resized_width}"
    );
    assert!(
        (95..=105).contains(&resized_height),
        "resized height should be ~100, got {resized_height}"
    );

    // Source should be untouched
    assert_eq!(
        fs::read(&src).unwrap(),
        src_content_before,
        "source should be unchanged"
    );

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn convert_format_png_to_jpg() {
    let dir = std::env::temp_dir().join(format!("rust-feh-016-convert-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();

    let src = make_test_png(&dir, [128, 64, 192, 255]);
    let src_content_before = fs::read(&src).unwrap();

    let output_path = dir.join("photo.jpg");
    let opts = ProcessOptions {
        width: None,
        height: None,
        percent: None,
        fit: None,
        filter: None,
        target_format: Some("jpg".into()),
        quality: None,
        output_path: Some(output_path.clone()),
    };

    let result = process_image(&src, &opts).expect("process_image");
    assert!(result.is_file(), "output file should exist");
    assert_eq!(
        result, output_path,
        "output path should match requested output_path"
    );

    // Verify it's a valid image (can be opened)
    let converted = image::open(&result).expect("output should be decodable as image");
    assert!(
        converted.width() > 0,
        "converted image should have valid width"
    );
    assert!(
        converted.height() > 0,
        "converted image should have valid height"
    );

    // Source should be untouched
    assert_eq!(
        fs::read(&src).unwrap(),
        src_content_before,
        "source PNG should be unchanged"
    );

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn actions_work_with_spaces_and_non_ascii_names() {
    let dir = std::env::temp_dir().join(format!("rust-feh-016-unicode-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();

    // Test save_copy with spaces and non-ASCII
    let src_copy_dir = dir.join("src_copy");
    let dest_copy_dir = dir.join("dest_copy");
    fs::create_dir_all(&src_copy_dir).unwrap();
    fs::create_dir_all(&dest_copy_dir).unwrap();

    let src_copy_path = src_copy_dir.join("vacation café day.png");
    let img = image::RgbaImage::from_pixel(32, 32, image::Rgba([100, 150, 200, 255]));
    img.save(&src_copy_path).unwrap();
    let src_copy_content = fs::read(&src_copy_path).unwrap();

    let copy_result = save_copy_to(&src_copy_path, &dest_copy_dir).expect("save_copy_to");
    assert!(copy_result.is_file(), "copied file should exist");
    assert_eq!(
        fs::read(&copy_result).unwrap(),
        src_copy_content,
        "copied content should match"
    );

    // The filename should preserve the original name (possibly with suffix if collision)
    let copy_filename = copy_result
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap();
    assert!(
        copy_filename.starts_with("vacation café day"),
        "copied filename should preserve non-ASCII characters, got: {copy_filename}"
    );

    // Test move with spaces and non-ASCII
    let src_move_dir = dir.join("src_move");
    let dest_move_dir = dir.join("dest_move");
    fs::create_dir_all(&src_move_dir).unwrap();
    fs::create_dir_all(&dest_move_dir).unwrap();

    let src_move_path = src_move_dir.join("vacation café day.png");
    let img2 = image::RgbaImage::from_pixel(32, 32, image::Rgba([200, 150, 100, 255]));
    img2.save(&src_move_path).unwrap();
    let src_move_content = fs::read(&src_move_path).unwrap();

    let plan = plan_loss_proof_move(&src_move_path, &dest_move_dir).expect("plan_loss_proof_move");
    let moved = execute_move_plan(&plan).expect("execute_move_plan");

    // Source gone, destination exists with matching content
    assert!(!src_move_path.is_file(), "source should be gone after move");
    assert!(moved.is_file(), "moved file should exist");
    assert_eq!(
        fs::read(&moved).unwrap(),
        src_move_content,
        "moved content should match original"
    );

    // Filename should preserve non-ASCII
    let move_filename = moved
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap();
    assert!(
        move_filename.starts_with("vacation café day"),
        "moved filename should preserve non-ASCII characters, got: {move_filename}"
    );

    let _ = fs::remove_dir_all(&dir);
}
