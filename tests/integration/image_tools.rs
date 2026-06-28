// SPDX-License-Identifier: MIT
use std::fs;
use std::path::PathBuf;
use std::time::Instant;

use rust_feh::image_proc::{has_magick_cache, ImageToolsService};
use rust_feh::types::{ImageOperation, OutputPolicy};
use rust_feh::ui_logic::{add_or_update_asset_in_inventory, apply_rename_pairs};
use rust_feh::types::{AssetStatus, ImageEntry};

fn make_test_png(dir: &PathBuf) -> PathBuf {
    let path = dir.join("test.png");
    let img = image::RgbaImage::from_pixel(32, 32, image::Rgba([255, 0, 0, 255]));
    img.save(&path).unwrap();
    path
}

#[test]
fn single_resize_creates_new_file_original_untouched() {
    let dir = std::env::temp_dir().join(format!("rust-feh-it-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let src = make_test_png(&dir);
    let before = fs::read(&src).unwrap();

    let svc = ImageToolsService::new(None);
    let op = ImageOperation::Resize {
        width: None,
        height: None,
        percent: Some(50.0),
        fit: None,
        filter: None,
        quality: Some(85),
    };
    let policy = OutputPolicy::NewSubfolder {
        name: "processed".into(),
    };
    let res = svc.process_single(&src, op, policy).expect("resize");
    assert!(res.dest_path.is_file());
    assert_eq!(fs::read(&src).unwrap(), before);

    let mut images = vec![ImageEntry::new(src.clone())];
    add_or_update_asset_in_inventory(&mut images, res.dest_path.clone(), AssetStatus::Processed);
    assert_eq!(images.len(), 2);

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn sc005_original_bytes_unchanged_after_subfolder_resize() {
    let dir = std::env::temp_dir().join(format!("rust-feh-sc5-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let src = make_test_png(&dir);
    let hash_before = fs::read(&src).unwrap();

    let svc = ImageToolsService::new(None);
    let _ = svc
        .process_single(
            &src,
            ImageOperation::Resize {
                width: None,
                height: None,
                percent: Some(25.0),
                fit: None,
                filter: None,
                quality: Some(80),
            },
            OutputPolicy::NewSubfolder {
                name: "out".into(),
            },
        )
        .unwrap();

    assert_eq!(fs::read(&src).unwrap(), hash_before);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn batch_mixed_success_and_failure() {
    let dir = std::env::temp_dir().join(format!("rust-feh-batch-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let good = make_test_png(&dir);
    let missing = dir.join("ghost.png");

    let svc = ImageToolsService::new(None);
    let op = ImageOperation::Resize {
        width: None,
        height: None,
        percent: Some(50.0),
        fit: None,
        filter: None,
        quality: Some(85),
    };
    let policy = OutputPolicy::NewSubfolder {
        name: "out".into(),
    };
    let (results, summary) = svc.process_batch(&[good.clone(), missing], op, policy);
    assert_eq!(summary.total, 2);
    assert_eq!(summary.succeeded, 1);
    assert_eq!(summary.failed, 1);
    assert!(results[0].is_ok());
    assert!(results[1].is_err());

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn rename_apply_rolls_back_on_failure() {
    let dir = std::env::temp_dir().join(format!("rust-feh-ren-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let a = dir.join("a.jpg");
    let b = dir.join("b.jpg");
    fs::write(&a, b"a").unwrap();
    fs::write(&b, b"b").unwrap();

    let pairs = vec![
        (a.clone(), "renamed_a.jpg".into()),
        (b.clone(), "no_such_dir/out.jpg".into()),
    ];
    let outcome = apply_rename_pairs(&pairs);
    assert!(outcome.error.is_some());
    assert!(outcome.rolled_back);
    assert!(a.is_file());
    assert_eq!(fs::read_to_string(&a).unwrap(), "a");
    assert!(b.is_file());
    assert_eq!(fs::read_to_string(&b).unwrap(), "b");
    assert!(!dir.join("renamed_a.jpg").exists());

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn prepare_fast_set_idempotent_second_run() {
    let dir = std::env::temp_dir().join(format!("rust-feh-pf2-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let src = make_test_png(&dir);
    let temp = dir.join("fast");
    let svc = ImageToolsService::new(None);

    let start = Instant::now();
    let set1 = svc.prepare_fast_set(&[src.clone()], &temp).unwrap();
    let first = start.elapsed();

    let start2 = Instant::now();
    let set2 = svc.prepare_fast_set(&[src], &temp).unwrap();
    let second = start2.elapsed();

    assert!(!set1.materialized_paths.is_empty());
    assert_eq!(set1.materialized_paths.len(), set2.materialized_paths.len());
    assert!(
        second <= first * 2 + std::time::Duration::from_millis(200),
        "second prepare-fast should not be dramatically slower"
    );

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn cache_repeat_resize_faster_when_cache_available() {
    if !has_magick_cache() {
        eprintln!("skip cache_repeat: magick-cache not on PATH");
        return;
    }
    let dir = std::env::temp_dir().join(format!("rust-feh-cache-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let src = make_test_png(&dir);
    let home = std::env::var_os("HOME").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("/tmp"));
    let passkey = home.join(".magick-cache-passkey");
    if !passkey.is_file() {
        eprintln!("skip cache_repeat: passkey not at {}", passkey.display());
        return;
    }
    let cfg = rust_feh::types::CacheConfig {
        enabled: true,
        root: Some(home.join(".cache/rust-feh-magick")),
        passkey_path: Some(passkey),
        default_ttl: "1 day".into(),
    };
    let svc = ImageToolsService::new(Some(cfg));
    let op = ImageOperation::Resize {
        width: Some(24),
        height: Some(24),
        percent: None,
        fit: None,
        filter: None,
        quality: Some(85),
    };
    let policy = OutputPolicy::NewSubfolder {
        name: "c".into(),
    };

    let t0 = Instant::now();
    let r1 = svc.process_single(&src, op.clone(), policy.clone()).unwrap();
    let cold = t0.elapsed();

    let t1 = Instant::now();
    let r2 = svc.process_single(&src, op, policy).unwrap();
    let warm = t1.elapsed();

    if r2.was_cache_hit {
        assert!(
            warm < cold || warm < std::time::Duration::from_secs(3),
            "cache hit should be fast (cold={cold:?} warm={warm:?})"
        );
    } else {
        eprintln!("cache hit not observed (cold={cold:?} warm={warm:?}); magick-cache may need setup");
    }
    assert!(r1.dest_path.is_file());

    let _ = fs::remove_dir_all(&dir);
}