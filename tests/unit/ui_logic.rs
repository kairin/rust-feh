// SPDX-License-Identifier: MIT
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

use rust_feh::types::{
    AssetStatus, FehLaunchEntry, FehLaunchList, ImageEntry, OutputPolicy, WindowPreferences,
    WindowSizePreset,
};
use rust_feh::ui_logic::{
    add_or_update_asset_in_inventory, aggregate_batch_results, build_entry_filelist,
    compute_output_path, copy_image_to_clipboard, crop_preview_pixels, decode_image_to_rgba,
    entry_is_launchable, expand_rename_pattern, load_launch_list, load_window_prefs,
    save_launch_list, save_window_prefs, write_feh_filelist_to,
};

static HOME_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn compute_output_path_subfolder() {
    let src = PathBuf::from("/tmp/photos/vacation.jpg");
    let p = compute_output_path(
        &src,
        "_resized",
        "jpg",
        &OutputPolicy::NewSubfolder {
            name: "processed".into(),
        },
    )
    .unwrap();
    assert!(p.ends_with("processed/vacation_resized.jpg"));
}

#[test]
fn compute_output_path_suffixed() {
    let src = PathBuf::from("/tmp/a.png");
    let p = compute_output_path(
        &src,
        "",
        "png",
        &OutputPolicy::SuffixedSibling {
            suffix: "_out".into(),
        },
    )
    .unwrap();
    assert!(p.ends_with("a_out.png"));
}

#[test]
fn expand_rename_counter_and_date() {
    let sources = vec![
        PathBuf::from("/tmp/a.jpg"),
        PathBuf::from("/tmp/b.jpg"),
    ];
    let pairs = expand_rename_pattern("trip-{counter:03}", &sources, 1).unwrap();
    assert_eq!(pairs[0].1, "trip-001.jpg");
    assert_eq!(pairs[1].1, "trip-002.jpg");
}

#[test]
fn expand_rename_collision_fails() {
    let sources = vec![
        PathBuf::from("/tmp/a.jpg"),
        PathBuf::from("/tmp/b.jpg"),
    ];
    assert!(expand_rename_pattern("same", &sources, 1).is_err());
}

#[test]
fn crop_preview_pixels_dimensions() {
    let dir = std::env::temp_dir().join(format!("rust-feh-crop-prev-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join("src.png");
    let img = image::RgbaImage::from_pixel(100, 80, image::Rgba([0, 128, 255, 255]));
    img.save(&path).unwrap();

    let px = crop_preview_pixels(&path, "40x30+10+5", 256).unwrap();
    assert_eq!(px.width, 40);
    assert_eq!(px.height, 30);
    assert_eq!(px.rgba.len(), 40 * 30 * 4);

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn aggregate_batch_results_counts() {
    use rust_feh::types::{ImageOperation, ProcessedResult};
    let ok = ProcessedResult {
        source_path: PathBuf::from("/a.jpg"),
        dest_path: PathBuf::from("/b.jpg"),
        operation: ImageOperation::Convert {
            target_format: "jpg".into(),
            quality: Some(90),
        },
        cache_iri: None,
        was_cache_hit: false,
        materialized_for_fast: None,
    };
    let results = vec![Ok(ok), Err("skip".into())];
    let summary = aggregate_batch_results(&results);
    assert_eq!(summary.total, 2);
    assert_eq!(summary.succeeded, 1);
    assert_eq!(summary.failed, 1);
}

#[test]
fn inventory_add_processed() {
    let mut images = vec![ImageEntry::new(PathBuf::from("/tmp/x.jpg"))];
    add_or_update_asset_in_inventory(
        &mut images,
        PathBuf::from("/tmp/y.jpg"),
        AssetStatus::Processed,
    );
    assert_eq!(images.len(), 2);
    assert_eq!(images[1].asset_status, AssetStatus::Processed);
}

fn temp_test_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "rust-feh-{name}-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn launch_entry(folder: Option<PathBuf>) -> FehLaunchEntry {
    FehLaunchEntry {
        id: "entry-1".into(),
        label: Some("Work".into()),
        folder_path: folder,
        created_at: 42,
    }
}

#[test]
fn test_launch_list_roundtrip() {
    let _guard = HOME_LOCK.lock().unwrap();
    let home = temp_test_dir("launch-roundtrip");
    std::env::set_var("HOME", &home);

    let list = FehLaunchList {
        version: 1,
        entries: vec![launch_entry(Some(home.join("photos")))],
    };
    save_launch_list(&list).unwrap();
    assert_eq!(load_launch_list(), list);

    let _ = fs::remove_dir_all(&home);
}

#[test]
fn test_launch_list_empty_load() {
    let _guard = HOME_LOCK.lock().unwrap();
    let home = temp_test_dir("launch-empty");
    std::env::set_var("HOME", &home);

    assert_eq!(load_launch_list(), FehLaunchList::default());

    let _ = fs::remove_dir_all(&home);
}

#[test]
fn test_launch_list_corrupt_json() {
    let _guard = HOME_LOCK.lock().unwrap();
    let home = temp_test_dir("launch-corrupt");
    std::env::set_var("HOME", &home);
    let config = home.join(".config/rust-feh");
    fs::create_dir_all(&config).unwrap();
    fs::write(config.join("launch-entries.json"), b"not-json").unwrap();

    assert_eq!(load_launch_list(), FehLaunchList::default());

    let _ = fs::remove_dir_all(&home);
}

#[test]
fn window_prefs_round_trip() {
    let _guard = HOME_LOCK.lock().unwrap();
    let home = temp_test_dir("window-prefs-roundtrip");
    std::env::set_var("HOME", &home);

    let prefs = WindowPreferences {
        version: 1,
        preset: WindowSizePreset::Large,
        resizable: false,
    };
    save_window_prefs(&prefs).unwrap();
    assert_eq!(load_window_prefs(), prefs);

    let _ = fs::remove_dir_all(&home);
}

#[test]
fn window_prefs_missing_returns_default() {
    let _guard = HOME_LOCK.lock().unwrap();
    let home = temp_test_dir("window-prefs-missing");
    std::env::set_var("HOME", &home);

    assert_eq!(load_window_prefs(), WindowPreferences::default());

    let _ = fs::remove_dir_all(&home);
}

#[test]
fn window_prefs_corrupt_returns_default() {
    let _guard = HOME_LOCK.lock().unwrap();
    let home = temp_test_dir("window-prefs-corrupt");
    std::env::set_var("HOME", &home);
    let config = home.join(".config/rust-feh");
    fs::create_dir_all(&config).unwrap();
    fs::write(config.join("window-prefs.json"), b"not-json").unwrap();

    assert_eq!(load_window_prefs(), WindowPreferences::default());

    let _ = fs::remove_dir_all(&home);
}

#[test]
fn test_decode_image_to_rgba_known_png() {
    let mut img = image::RgbaImage::new(2, 1);
    img.put_pixel(0, 0, image::Rgba([255, 0, 0, 255]));
    img.put_pixel(1, 0, image::Rgba([0, 255, 0, 128]));
    let mut bytes = Vec::new();
    image::DynamicImage::ImageRgba8(img)
        .write_to(&mut std::io::Cursor::new(&mut bytes), image::ImageFormat::Png)
        .unwrap();

    let (width, height, rgba) = decode_image_to_rgba(&bytes).unwrap();
    assert_eq!((width, height), (2, 1));
    assert_eq!(rgba.len(), 2 * 1 * 4);
}

#[test]
fn test_clipboard_missing_file_error() {
    let err = copy_image_to_clipboard(&PathBuf::from("/definitely/missing/rust-feh.png")).unwrap_err();
    assert!(err.starts_with("Failed to read image:"));
}

#[test]
fn test_clipboard_non_image_error() {
    let dir = temp_test_dir("clipboard-non-image");
    let path = dir.join("not-image.txt");
    fs::write(&path, b"not an image").unwrap();

    let err = copy_image_to_clipboard(&path).unwrap_err();
    assert!(err.starts_with("Failed to decode image:"));

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn test_entry_filelist_and_launchability() {
    let dir = temp_test_dir("entry-filelist");
    let other = temp_test_dir("entry-filelist-other");
    let img_a = dir.join("a.png");
    let img_b = dir.join("b.jpg");
    let img_other = other.join("c.png");
    fs::write(&img_a, b"a").unwrap();
    fs::write(&img_b, b"b").unwrap();
    fs::write(&img_other, b"c").unwrap();
    let images = vec![
        ImageEntry::new(img_a.clone()),
        ImageEntry::new(img_b.clone()),
        ImageEntry::new(img_other),
    ];
    let entry = launch_entry(Some(dir.clone()));

    let paths = build_entry_filelist(&entry, &images);
    assert_eq!(paths, vec![img_a.clone(), img_b.clone()]);
    let state = entry_is_launchable(&entry, &images, true);
    assert!(state.launchable);
    assert_eq!(state.status, "2 images");

    let filelist = dir.join("filelist.txt");
    assert_eq!(write_feh_filelist_to(&filelist, &paths).unwrap(), 2);
    let filelist_text = fs::read_to_string(filelist).unwrap();
    assert!(filelist_text.contains(&img_a.display().to_string()));
    assert!(filelist_text.contains(&img_b.display().to_string()));

    let _ = fs::remove_dir_all(&dir);
    let _ = fs::remove_dir_all(&other);
}

#[test]
fn test_entry_launchability_missing_empty_unassigned_and_feh() {
    let dir = temp_test_dir("entry-empty");
    let image_dir = temp_test_dir("entry-images");
    let img = image_dir.join("a.png");
    fs::write(&img, b"a").unwrap();
    let images = vec![ImageEntry::new(img)];

    let unassigned = launch_entry(None);
    assert_eq!(entry_is_launchable(&unassigned, &images, true).status, "Select a folder");

    let missing = launch_entry(Some(dir.join("missing")));
    assert_eq!(entry_is_launchable(&missing, &images, true).status, "Folder not found");

    let empty = launch_entry(Some(dir.clone()));
    assert_eq!(entry_is_launchable(&empty, &images, true).status, "No images");

    let unavailable = launch_entry(Some(image_dir));
    assert_eq!(
        entry_is_launchable(&unavailable, &images, false).status,
        "feh not installed"
    );

    let _ = fs::remove_dir_all(&dir);
}
