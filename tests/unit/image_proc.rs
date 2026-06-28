// SPDX-License-Identifier: MIT
use std::fs;
use std::path::PathBuf;

use rust_feh::image_proc::{parse_crop_geometry, MagickCacheManager, ImageToolsService};
use rust_feh::types::{CacheConfig, FitMode, ImageOperation, OutputPolicy, Filter};
use rust_feh::ui_logic::compute_output_path;

#[test]
fn parse_crop_geometry_signed() {
    let r = parse_crop_geometry("100x200+10+20").unwrap();
    assert_eq!(r.width, 100);
    assert_eq!(r.x, 10);
}

#[test]
fn make_iri_hierarchical() {
    let m = MagickCacheManager::new(CacheConfig::default());
    let p = PathBuf::from("/tmp/test/image.jpg");
    let iri = m.make_iri(&p, "resize", "w100");
    assert!(iri.starts_with("rustfeh/image/resize/"));
}

#[test]
fn output_policy_paths_all_variants() {
    let src = PathBuf::from("/data/photo.jpg");
    let _ = compute_output_path(&src, "_x", "jpg", &OutputPolicy::default()).unwrap();
    let _ = compute_output_path(
        &src,
        "_x",
        "jpg",
        &OutputPolicy::SuffixedSibling {
            suffix: "_r".into(),
        },
    )
    .unwrap();
    let _ = compute_output_path(
        &src,
        "_x",
        "jpg",
        &OutputPolicy::InPlaceWithBackup {
            backup_suffix: ".bak".into(),
        },
    )
    .unwrap();
}

#[test]
fn cache_iri_stable_for_same_inputs() {
    let m = MagickCacheManager::new(CacheConfig::default());
    let p = PathBuf::from("/tmp/a/photo.jpg");
    let a = m.make_iri(&p, "resize", "p1");
    let b = m.make_iri(&p, "resize", "p1");
    assert_eq!(a, b);
    assert_ne!(a, m.make_iri(&p, "resize", "p2"));
}

#[test]
fn cache_disabled_reports_not_ready() {
    let m = MagickCacheManager::new(CacheConfig {
        enabled: false,
        ..CacheConfig::default()
    });
    assert!(!m.enabled_and_ready());
}

#[test]
fn prepare_fast_one_writes_jpeg() {
    let dir = std::env::temp_dir().join(format!("rust-feh-pf1-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let src = dir.join("in.png");
    image::RgbaImage::from_pixel(64, 48, image::Rgba([10, 20, 30, 255]))
        .save(&src)
        .unwrap();
    let out_dir = dir.join("fast");
    let svc = ImageToolsService::new(None);
    let dest = svc.prepare_fast_one(&src, &out_dir).unwrap();
    assert!(dest.is_file());
    assert!(dest.extension().unwrap().to_string_lossy() == "jpg");
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn service_smoke_types() {
    let svc = ImageToolsService::new(None);
    let _op = ImageOperation::Resize {
        width: None,
        height: None,
        percent: Some(50.0),
        fit: Some(FitMode::Contain),
        filter: Some(Filter::Lanczos3),
        quality: Some(85),
    };
    let _ = svc;
}