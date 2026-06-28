// SPDX-License-Identifier: MIT
//! Variation tests for image tools + multi-feh using real desktop folder fixtures.
//! Outputs land under `<folder>/_rust-feh-tests/<test-name>/` (never overwrites originals).
//!
//! Run:
//!   RUST_FEH_FOLDER_A='/home/kkk/Desktop/Tree Original' \
//!   RUST_FEH_FOLDER_B='/home/kkk/Desktop/treeeeeeeeeee' \
//!   cargo test --test desktop_tools_variations -- --nocapture

use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use rust_feh::image_proc::{has_magick_cache, ImageToolsService};
use rust_feh::scanner::scan_images;
use rust_feh::types::{AssetStatus, ImageEntry, ImageOperation, OutputPolicy};
use rust_feh::ui_logic::{
    add_or_update_asset_in_inventory, apply_rename_pairs, build_entry_filelist, crop_preview_pixels,
    entry_is_launchable, expand_rename_pattern,
};

const TEST_ROOT: &str = "_rust-feh-tests";

fn fixture_folders() -> Option<(PathBuf, PathBuf)> {
    let a = std::env::var_os("RUST_FEH_FOLDER_A").map(PathBuf::from)?;
    let b = std::env::var_os("RUST_FEH_FOLDER_B").map(PathBuf::from)?;
    Some((a, b))
}

fn skip_unless_fixtures() -> Option<(PathBuf, PathBuf)> {
    let (a, b) = fixture_folders()?;
    if !a.is_dir() {
        eprintln!("skip: folder A missing: {}", a.display());
        return None;
    }
    if !b.is_dir() {
        eprintln!("skip: folder B missing: {}", b.display());
        return None;
    }
    Some((a, b))
}

fn scan_folder(dir: &Path) -> Vec<ImageEntry> {
    let result = scan_images(dir, false, false);
    assert!(
        !result.entries.is_empty(),
        "no images in {}",
        dir.display()
    );
    result.entries
}

fn first_image(dir: &Path) -> PathBuf {
    scan_folder(dir)[0].path.clone()
}

/// Policy subfolder path segment under the source image's parent directory.
fn out_policy(test_name: &str) -> OutputPolicy {
    OutputPolicy::NewSubfolder {
        name: format!("{TEST_ROOT}/{test_name}/processed"),
    }
}

fn test_dir(folder: &Path, test_name: &str) -> PathBuf {
    folder.join(TEST_ROOT).join(test_name)
}

fn copy_into_staging(folder: &Path, test_name: &str, sources: &[PathBuf]) -> Vec<PathBuf> {
    let staging = test_dir(folder, test_name);
    fs::create_dir_all(&staging).expect("create staging");
    sources
        .iter()
        .enumerate()
        .map(|(i, src)| {
            let name = src
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            let dest = staging.join(format!("{i:02}-{name}"));
            fs::copy(src, &dest).expect("copy to staging");
            dest
        })
        .collect()
}

fn image_dims(path: &Path) -> (u32, u32) {
    let img = image::open(path).expect("open image");
    (img.width(), img.height())
}

fn safe_crop_geometry(path: &Path) -> String {
    let (w, h) = image_dims(path);
    let cw = w.min(480).max(32);
    let ch = h.min(360).max(32);
    format!("{cw}x{ch}+10+10")
}

#[test]
fn var01_single_resize_50pct_tree_original() {
    let Some((folder_a, _)) = skip_unless_fixtures() else {
        return;
    };
    let src = first_image(&folder_a);
    let before = fs::read(&src).unwrap();
    let (w, h) = image_dims(&src);

    let svc = ImageToolsService::new(None);
    let res = svc
        .process_single(
            &src,
            ImageOperation::Resize {
                width: None,
                height: None,
                percent: Some(50.0),
                fit: None,
                filter: None,
                quality: Some(85),
            },
            out_policy("01-single-resize-50pct"),
        )
        .expect("resize");

    assert!(res.dest_path.is_file());
    assert_eq!(fs::read(&src).unwrap(), before, "original untouched");
    let (dw, dh) = image_dims(&res.dest_path);
    assert!(dw <= w && dh <= h);
    assert!(dw > 0 && dh > 0);
    eprintln!("var01 wrote {}", res.dest_path.display());
}

#[test]
fn var02_crop_geometry_tree_original() {
    let Some((folder_a, _)) = skip_unless_fixtures() else {
        return;
    };
    let src = first_image(&folder_a);
    let before = fs::read(&src).unwrap();
    let geometry = safe_crop_geometry(&src);

    let preview = crop_preview_pixels(&src, &geometry, 256).expect("crop preview");
    assert!(preview.width > 0 && preview.height > 0);

    let svc = ImageToolsService::new(None);
    let res = svc
        .process_single(
            &src,
            ImageOperation::Crop {
                geometry: geometry.clone(),
            },
            out_policy("02-crop-geometry"),
        )
        .expect("crop");

    assert!(res.dest_path.is_file());
    assert_eq!(fs::read(&src).unwrap(), before);
    eprintln!("var02 crop {geometry} -> {}", res.dest_path.display());
}

#[test]
fn var03_convert_jpeg_tree_original() {
    let Some((folder_a, _)) = skip_unless_fixtures() else {
        return;
    };
    let src = first_image(&folder_a);
    let before = fs::read(&src).unwrap();

    let svc = ImageToolsService::new(None);
    let res = svc
        .process_single(
            &src,
            ImageOperation::Convert {
                target_format: "jpg".into(),
                quality: Some(88),
            },
            out_policy("03-convert-jpeg"),
        )
        .expect("convert");

    assert!(res.dest_path.is_file());
    assert_eq!(fs::read(&src).unwrap(), before);
    assert_eq!(
        res.dest_path.extension().unwrap().to_string_lossy().to_lowercase(),
        "jpg"
    );
    eprintln!("var03 wrote {}", res.dest_path.display());
}

#[test]
fn var04_batch_resize_cross_folder() {
    let Some((folder_a, folder_b)) = skip_unless_fixtures() else {
        return;
    };
    let src_a = first_image(&folder_a);
    let src_b = first_image(&folder_b);
    let hash_a = fs::read(&src_a).unwrap();
    let hash_b = fs::read(&src_b).unwrap();

    let svc = ImageToolsService::new(None);
    let op = ImageOperation::Resize {
        width: None,
        height: None,
        percent: Some(75.0),
        fit: None,
        filter: None,
        quality: Some(80),
    };
    let policy = out_policy("04-batch-resize-cross-folder");
    let (results, summary) = svc.process_batch(&[src_a.clone(), src_b.clone()], op, policy);

    assert_eq!(summary.total, 2);
    assert_eq!(summary.succeeded, 2);
    assert!(results.iter().all(|r| r.is_ok()));
    assert_eq!(fs::read(&src_a).unwrap(), hash_a);
    assert_eq!(fs::read(&src_b).unwrap(), hash_b);

    for r in results {
        let ok = r.unwrap();
        assert!(ok.dest_path.starts_with(folder_a.join(TEST_ROOT)) || ok.dest_path.starts_with(folder_b.join(TEST_ROOT)));
        eprintln!("var04 wrote {}", ok.dest_path.display());
    }
}

#[test]
fn var05_suffixed_sibling_policy() {
    let Some((folder_a, _)) = skip_unless_fixtures() else {
        return;
    };
    let src = first_image(&folder_a);
    let staged = copy_into_staging(&folder_a, "05-suffixed-staging", &[src])
        .into_iter()
        .next()
        .unwrap();
    let before = fs::read(&staged).unwrap();

    let policy = OutputPolicy::SuffixedSibling {
        suffix: "_half".into(),
    };

    let svc = ImageToolsService::new(None);
    let res = svc
        .process_single(
            &staged,
            ImageOperation::Resize {
                width: None,
                height: None,
                percent: Some(50.0),
                fit: None,
                filter: None,
                quality: Some(85),
            },
            policy,
        )
        .expect("suffixed resize");

    assert!(res.dest_path.is_file());
    assert!(res.dest_path.parent() == staged.parent());
    let name = res.dest_path.file_name().unwrap().to_string_lossy();
    assert!(name.contains("_half"), "expected suffixed output, got {name}");
    assert_eq!(fs::read(&staged).unwrap(), before);
    eprintln!("var05 wrote {}", res.dest_path.display());
}

#[test]
fn var06_rename_pattern_staging() {
    let Some((folder_a, _)) = skip_unless_fixtures() else {
        return;
    };
    let images = scan_folder(&folder_a);
    let sources: Vec<PathBuf> = images.iter().take(2).map(|e| e.path.clone()).collect();
    let staged = copy_into_staging(&folder_a, "06-rename-staging", &sources);

    let pairs = expand_rename_pattern("tree-{counter:03}", &staged, 1).expect("expand");
    assert_eq!(pairs.len(), 2);
    let outcome = apply_rename_pairs(&pairs);
    assert!(outcome.error.is_none());
    assert!(!outcome.rolled_back);

    for (_, dest) in outcome.applied {
        assert!(dest.is_file());
        let name = dest.file_name().unwrap().to_string_lossy();
        assert!(name.starts_with("tree-"), "renamed to {name}");
        eprintln!("var06 renamed -> {}", dest.display());
    }
}

#[test]
fn var07_prepare_fast_materialized() {
    let Some((folder_a, _)) = skip_unless_fixtures() else {
        return;
    };
    let paths: Vec<PathBuf> = scan_folder(&folder_a)
        .into_iter()
        .take(3)
        .map(|e| e.path)
        .collect();
    let temp = test_dir(&folder_a, "07-prepare-fast/materialized");
    fs::create_dir_all(&temp).unwrap();

    let svc = ImageToolsService::new(None);
    let set = svc.prepare_fast_set(&paths, &temp).expect("prepare-fast");
    assert!(!set.materialized_paths.is_empty());
    let filelist_path = temp.join("feh-filelist.txt");
    let n = rust_feh::ui_logic::write_feh_filelist_to(&filelist_path, &set.materialized_paths).unwrap();
    assert_eq!(n, set.materialized_paths.len());
    let body = fs::read_to_string(&filelist_path).unwrap();
    assert!(body.lines().count() >= set.materialized_paths.len());
    for p in &set.materialized_paths {
        assert!(p.is_file());
        eprintln!("var07 materialized {}", p.display());
    }
}

#[test]
fn var08_cache_repeat_resize_when_available() {
    if !has_magick_cache() {
        eprintln!("skip var08: magick-cache not on PATH");
        return;
    }
    let Some((folder_a, _)) = skip_unless_fixtures() else {
        return;
    };
    let home = std::env::var_os("HOME").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("/tmp"));
    let passkey = home.join(".magick-cache-passkey");
    if !passkey.is_file() {
        eprintln!("skip var08: passkey missing at {}", passkey.display());
        return;
    }

    let src = first_image(&folder_a);
    let cfg = rust_feh::types::CacheConfig {
        enabled: true,
        root: Some(home.join(".cache/rust-feh-magick")),
        passkey_path: Some(passkey),
        default_ttl: "1 day".into(),
    };
    let svc = ImageToolsService::new(Some(cfg));
    let op = ImageOperation::Resize {
        width: Some(128),
        height: Some(128),
        percent: None,
        fit: None,
        filter: None,
        quality: Some(85),
    };
    let policy = out_policy("08-cache-repeat");

    let cold = Instant::now();
    let r1 = svc.process_single(&src, op.clone(), policy.clone()).expect("cold");
    let cold_elapsed = cold.elapsed();

    let warm = Instant::now();
    let r2 = svc.process_single(&src, op, policy).expect("warm");
    let warm_elapsed = warm.elapsed();

    assert!(r1.dest_path.is_file());
    assert!(r2.dest_path.is_file());
    if r2.was_cache_hit {
        assert!(warm_elapsed < cold_elapsed || warm_elapsed < std::time::Duration::from_secs(3));
        eprintln!("var08 cache hit cold={cold_elapsed:?} warm={warm_elapsed:?}");
    } else {
        eprintln!("var08 no cache hit observed (setup may need refresh)");
    }
}

#[test]
fn var09_inventory_updates_after_processing() {
    let Some((folder_a, _)) = skip_unless_fixtures() else {
        return;
    };
    let src = first_image(&folder_a);
    let mut inventory = scan_folder(&folder_a);

    let svc = ImageToolsService::new(None);
    let res = svc
        .process_single(
            &src,
            ImageOperation::Resize {
                width: None,
                height: None,
                percent: Some(40.0),
                fit: None,
                filter: None,
                quality: Some(80),
            },
            out_policy("09-inventory-resize"),
        )
        .expect("resize");

    let before = inventory.len();
    add_or_update_asset_in_inventory(&mut inventory, res.dest_path.clone(), AssetStatus::Processed);
    assert_eq!(inventory.len(), before + 1);
    assert!(inventory.iter().any(|e| e.path == res.dest_path));
}

#[test]
fn var10_multi_feh_filelists_both_folders() {
    let Some((folder_a, folder_b)) = skip_unless_fixtures() else {
        return;
    };
    let mut combined = scan_folder(&folder_a);
    combined.extend(scan_folder(&folder_b));

    for (id, folder) in [("mf-a", &folder_a), ("mf-b", &folder_b)] {
        let entry = rust_feh::types::FehLaunchEntry {
            id: id.into(),
            label: None,
            folder_path: Some(folder.clone()),
            created_at: 1,
        };
        let state = entry_is_launchable(&entry, &combined, true);
        assert!(state.launchable, "{id}: {}", state.status);
        let list = build_entry_filelist(&entry, &combined);
        assert!(!list.is_empty());
        let out = test_dir(folder, "10-multi-feh-filelist");
        fs::create_dir_all(&out).unwrap();
        let list_path = out.join(format!("{id}.txt"));
        let n = rust_feh::ui_logic::write_feh_filelist_to(&list_path, &list).unwrap();
        assert_eq!(n, list.len());
        eprintln!("var10 filelist {id}: {} lines -> {}", n, list_path.display());
    }
}

#[test]
fn var11_second_folder_convert_and_resize() {
    let Some((_, folder_b)) = skip_unless_fixtures() else {
        return;
    };
    let src = first_image(&folder_b);
    let before = fs::read(&src).unwrap();
    let svc = ImageToolsService::new(None);

    let resized = svc
        .process_single(
            &src,
            ImageOperation::Resize {
                width: None,
                height: None,
                percent: Some(60.0),
                fit: None,
                filter: None,
                quality: Some(82),
            },
            out_policy("11-resize-60pct"),
        )
        .expect("resize b");

    let converted = svc
        .process_single(
            &src,
            ImageOperation::Convert {
                target_format: "webp".into(),
                quality: Some(80),
            },
            out_policy("11-convert-webp"),
        )
        .expect("convert b");

    assert_eq!(fs::read(&src).unwrap(), before);
    assert!(resized.dest_path.is_file());
    assert!(converted.dest_path.is_file());
    eprintln!("var11 wrote {} and {}", resized.dest_path.display(), converted.dest_path.display());
}