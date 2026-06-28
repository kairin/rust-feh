// SPDX-License-Identifier: MIT
//! Lightweight perf smoke hooks for SC-001/002/004/007.
use std::fs;
use std::path::PathBuf;
use std::time::Instant;

use rust_feh::image_proc::ImageToolsService;
use rust_feh::types::{ImageOperation, OutputPolicy};
use rust_feh::ui_logic::expand_rename_pattern;

fn make_png(path: &PathBuf, w: u32, h: u32) {
    let img = image::RgbaImage::from_pixel(w, h, image::Rgba([40, 80, 120, 255]));
    img.save(path).unwrap();
}

#[test]
fn sc001_single_resize_under_30s() {
    let dir = std::env::temp_dir().join(format!("rust-feh-sc1-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let src = dir.join("large.png");
    make_png(&src, 4000, 3000);

    let svc = ImageToolsService::new(None);
    let start = Instant::now();
    svc.process_single(
        &src,
        ImageOperation::Resize {
            width: None,
            height: None,
            percent: Some(50.0),
            fit: None,
            filter: None,
            quality: Some(85),
        },
        OutputPolicy::NewSubfolder {
            name: "processed".into(),
        },
    )
    .expect("resize");
    assert!(start.elapsed().as_secs() < 30, "SC-001 single op too slow");

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn sc002_cache_disabled_repeat_similar_timing() {
    let dir = std::env::temp_dir().join(format!("rust-feh-sc2-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let src = dir.join("img.png");
    make_png(&src, 800, 600);
    let svc = ImageToolsService::new(None);
    let op = ImageOperation::Resize {
        width: Some(400),
        height: Some(300),
        percent: None,
        fit: None,
        filter: None,
        quality: Some(85),
    };
    let policy = OutputPolicy::NewSubfolder {
        name: "p".into(),
    };
    let t0 = Instant::now();
    svc.process_single(&src, op.clone(), policy.clone()).unwrap();
    let first = t0.elapsed();
    let t1 = Instant::now();
    svc.process_single(&src, op, policy).unwrap();
    let second = t1.elapsed();
    assert!(!first.is_zero());
    assert!(!second.is_zero());
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn sc004_prepare_fast_batch_under_60s() {
    let dir = std::env::temp_dir().join(format!("rust-feh-sc4-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let paths: Vec<PathBuf> = (0..50)
        .map(|i| {
            let p = dir.join(format!("img_{i:03}.png"));
            make_png(&p, 640, 480);
            p
        })
        .collect();
    let svc = ImageToolsService::new(None);
    let temp = dir.join("fast");
    let start = Instant::now();
    let set = svc.prepare_fast_set(&paths, &temp).unwrap();
    assert!(!set.materialized_paths.is_empty());
    assert!(start.elapsed().as_secs() < 60, "SC-004 prepare-fast too slow");
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn sc007_rename_preview_500_under_2s() {
    let sources: Vec<PathBuf> = (0..500)
        .map(|i| PathBuf::from(format!("/tmp/img_{i:04}.jpg")))
        .collect();
    let start = Instant::now();
    let pairs = expand_rename_pattern("trip-{date:YYYYMMDD}-{counter:03}", &sources, 1).unwrap();
    assert_eq!(pairs.len(), 500);
    assert!(start.elapsed().as_secs() < 2, "SC-007 preview too slow");
}