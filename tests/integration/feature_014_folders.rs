// SPDX-License-Identifier: MIT
//! Gated integration validation for feature 014 using real folder fixtures.
//! Run: RUST_FEH_FOLDER_A='...' RUST_FEH_FOLDER_B='...' cargo test --test feature_014_folders -- --nocapture

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

use rust_feh::scanner::scan_images;
use rust_feh::types::{FehLaunchEntry, FehLaunchList, ImageEntry};
use rust_feh::ui_logic::{
    build_entry_filelist, copy_image_to_clipboard, entry_is_launchable, feh_not_installed_launch_status,
    load_launch_list, save_launch_list, write_feh_filelist_to,
};

fn fixture_folders() -> Option<(PathBuf, PathBuf)> {
    let a = std::env::var_os("RUST_FEH_FOLDER_A").map(PathBuf::from)?;
    let b = std::env::var_os("RUST_FEH_FOLDER_B").map(PathBuf::from)?;
    Some((a, b))
}

fn require_fixture_folders() -> Option<(PathBuf, PathBuf)> {
    let (a, b) = fixture_folders()?;
    if !a.is_dir() {
        eprintln!("skip folder validation: folder A not found: {}", a.display());
        return None;
    }
    if !b.is_dir() {
        eprintln!("skip folder validation: folder B not found: {}", b.display());
        return None;
    }
    Some((a, b))
}

fn skip_unless_fixtures() -> Option<(PathBuf, PathBuf)> {
    require_fixture_folders().or_else(|| {
        eprintln!(
            "skip folder validation: set RUST_FEH_FOLDER_A and RUST_FEH_FOLDER_B to enable"
        );
        None
    })
}

fn scan_folder(dir: &Path) -> Vec<ImageEntry> {
    let result = scan_images(dir, false, false);
    assert!(
        !result.entries.is_empty(),
        "no images found in {}",
        dir.display()
    );
    result.entries
}

fn launch_entry(id: &str, folder: PathBuf) -> FehLaunchEntry {
    FehLaunchEntry {
        id: id.into(),
        label: None,
        folder_path: Some(folder),
        created_at: 1,
    }
}

fn feh_available() -> bool {
    Command::new("feh")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn spawn_feh_filelist(list_path: &Path, start_at: &Path) -> std::process::Child {
    Command::new("feh")
        .arg("--geometry")
        .arg("1280x960")
        .arg("--scale-down")
        .arg("--zoom")
        .arg("max")
        .arg("--filelist")
        .arg(list_path)
        .arg("--start-at")
        .arg(start_at)
        .spawn()
        .expect("spawn feh")
}

#[test]
fn vs_scan_and_launch_entries_for_two_folders() {
    let Some((folder_a, folder_b)) = skip_unless_fixtures() else {
        return;
    };
    if !feh_available() {
        eprintln!("skip vs_scan_and_launch_entries: feh not installed");
        return;
    }

    let images_a = scan_folder(&folder_a);
    let images_b = scan_folder(&folder_b);
    let mut combined = images_a.clone();
    combined.extend(images_b.clone());

    let entry_a = launch_entry("val-a", folder_a.clone());
    let entry_b = launch_entry("val-b", folder_b.clone());

    for (entry, images) in [(&entry_a, &combined), (&entry_b, &combined)] {
        let state = entry_is_launchable(entry, images, true);
        assert!(state.launchable, "{}: {}", entry.id, state.status);
        let paths = build_entry_filelist(entry, images);
        assert!(!paths.is_empty());
    }
}

#[test]
fn vs_multi_feh_spawn_two_instances() {
    let Some((folder_a, folder_b)) = skip_unless_fixtures() else {
        return;
    };
    if !feh_available() {
        eprintln!("skip vs_multi_feh_spawn: feh not installed");
        return;
    }

    let images_a = scan_folder(&folder_a);
    let images_b = scan_folder(&folder_b);
    let mut combined = images_a;
    combined.extend(images_b);

    let mut children = Vec::new();
    for (id, folder) in [("val-a", &folder_a), ("val-b", &folder_b)] {
        let entry = launch_entry(id, folder.clone());
        let paths = build_entry_filelist(&entry, &combined);
        let list_path = std::env::temp_dir().join(format!("rust-feh-validate-{id}.txt"));
        write_feh_filelist_to(&list_path, &paths).unwrap();
        let child = spawn_feh_filelist(&list_path, &paths[0]);
        children.push((list_path, child));
    }

    std::thread::sleep(Duration::from_millis(400));
    for (_, child) in &mut children {
        assert!(
            child.try_wait().unwrap().is_none(),
            "feh exited unexpectedly"
        );
    }

    for (list_path, mut child) in children {
        let _ = child.kill();
        let _ = child.wait();
        let _ = fs::remove_file(list_path);
    }
}

#[test]
fn sc001_launch_all_five_entries_stays_fast() {
    let Some((folder_a, folder_b)) = skip_unless_fixtures() else {
        return;
    };
    if !feh_available() {
        eprintln!("skip sc001: feh not installed");
        return;
    }

    let mut combined = scan_folder(&folder_a);
    combined.extend(scan_folder(&folder_b));

    let entries: Vec<FehLaunchEntry> = (0..5)
        .map(|i| {
            let folder = if i % 2 == 0 {
                folder_a.clone()
            } else {
                folder_b.clone()
            };
            launch_entry(&format!("bulk-{i}"), folder)
        })
        .collect();

    let start = Instant::now();
    let mut children = Vec::new();
    for entry in &entries {
        let state = entry_is_launchable(entry, &combined, true);
        if !state.launchable {
            continue;
        }
        let paths = build_entry_filelist(entry, &combined);
        let list_path = std::env::temp_dir().join(format!("rust-feh-sc001-{}.txt", entry.id));
        write_feh_filelist_to(&list_path, &paths).unwrap();
        let child = spawn_feh_filelist(&list_path, &paths[0]);
        children.push((list_path, child));
    }

    assert_eq!(children.len(), 5);
    assert!(
        start.elapsed() < Duration::from_secs(3),
        "launch-all spawn took too long: {:?}",
        start.elapsed()
    );

    for (list_path, mut child) in children {
        let _ = child.kill();
        let _ = child.wait();
        let _ = fs::remove_file(list_path);
    }
}

#[test]
fn vs_clipboard_copy_first_image_from_each_folder() {
    if std::env::var_os("DISPLAY").is_none() && std::env::var_os("WAYLAND_DISPLAY").is_none() {
        eprintln!("skip clipboard validation: no display");
        return;
    }

    let Some((folder_a, folder_b)) = skip_unless_fixtures() else {
        return;
    };
    for folder in [folder_a, folder_b] {
        let images = scan_folder(&folder);
        let path = &images[0].path;
        match copy_image_to_clipboard(path) {
            Ok(msg) => {
                assert!(msg.contains("Copied image to clipboard"));
                let (w, h, _) = rust_feh::ui_logic::decode_image_to_rgba(&fs::read(path).unwrap())
                    .unwrap();
                if let Ok(out) = Command::new("wl-paste")
                    .args(["--type", "image/png"])
                    .output()
                {
                    if out.status.success() && !out.stdout.is_empty() {
                        let pasted = image::load_from_memory(&out.stdout).expect("paste png");
                        assert_eq!(pasted.width(), w);
                        assert_eq!(pasted.height(), h);
                    } else {
                        eprintln!(
                            "clipboard copy ok; wl-paste unavailable for {}",
                            path.display()
                        );
                    }
                }
            }
            Err(e) if e.contains("Clipboard unavailable") => {
                eprintln!("skip clipboard for {}: {e}", path.display());
            }
            Err(e) => panic!("clipboard copy failed for {}: {e}", path.display()),
        }
    }
}

#[test]
fn vs_persistence_roundtrip_two_entries() {
    let Some((folder_a, folder_b)) = skip_unless_fixtures() else {
        return;
    };
    let home = std::env::temp_dir().join(format!(
        "rust-feh-persist-val-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&home);
    fs::create_dir_all(&home).unwrap();
    std::env::set_var("HOME", &home);

    let list = FehLaunchList {
        version: 1,
        entries: vec![
            launch_entry("p-a", folder_a),
            launch_entry("p-b", folder_b),
        ],
    };
    save_launch_list(&list).unwrap();
    assert_eq!(load_launch_list(), list);

    let _ = fs::remove_dir_all(&home);
}

#[test]
fn vs_single_open_in_feh_filelist_pattern() {
    let Some((folder_a, _)) = skip_unless_fixtures() else {
        return;
    };
    if !feh_available() {
        eprintln!("skip single open in feh: feh not installed");
        return;
    }

    let images = scan_folder(&folder_a);
    let paths: Vec<_> = images.iter().map(|e| e.path.as_path()).collect();
    let list_path = std::env::temp_dir().join("rust-feh-single-open.txt");
    write_feh_filelist_to(&list_path, &paths).unwrap();
    let mut child = spawn_feh_filelist(&list_path, &paths[0]);
    std::thread::sleep(Duration::from_millis(300));
    assert!(child.try_wait().unwrap().is_none());
    let _ = child.kill();
    let _ = child.wait();
    let _ = fs::remove_file(list_path);
}

#[test]
fn fr011_feh_not_installed_indicator() {
    let Some((folder_a, _)) = skip_unless_fixtures() else {
        return;
    };
    let images = scan_folder(&folder_a);
    let entry = launch_entry("fr011", folder_a);
    let state = entry_is_launchable(&entry, &images, false);
    assert_eq!(state.status, feh_not_installed_launch_status());
    assert!(!state.launchable);
}