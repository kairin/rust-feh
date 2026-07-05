// SPDX-License-Identifier: MIT
//! Integration tests for feature 016: round-trip handoff protocol
//! (validation and resolution at pure-logic level, no GUI, no subprocess spawn).

use std::fs;
use std::path::PathBuf;

use rust_feh::ui_logic::{handoff_path, validate_handoff};

/// SC-002 logic tier: handoff file resolution against a trusted filelist.
/// When feh writes a handoff file with "landed" image path, validate_handoff
/// resolves it back correctly, matching against the original launch filelist.
#[test]
fn handoff_resolves_to_correct_landed_image() {
    let dir = std::env::temp_dir().join(format!(
        "rust-feh-handoff-correct-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();

    // Create 3 fake image files in the tempdir
    let a_jpg = dir.join("a.jpg");
    let b_jpg = dir.join("b.jpg");
    let c_jpg = dir.join("c.jpg");
    fs::write(&a_jpg, b"fake image a").unwrap();
    fs::write(&b_jpg, b"fake image b").unwrap();
    fs::write(&c_jpg, b"fake image c").unwrap();

    let filelist = vec![a_jpg.clone(), b_jpg.clone(), c_jpg.clone()];

    // Simulate handoff content: "feh wrote b.jpg then exited"
    // (echo %F outputs the path with a trailing newline)
    let handoff_content = format!("{}\n", b_jpg.display());

    // Validate and resolve
    let resolved = validate_handoff(&handoff_content, &filelist);
    assert_eq!(
        resolved,
        Some(b_jpg.clone()),
        "handoff should resolve to b.jpg (US2-3: correct landed image)"
    );

    let _ = fs::remove_dir_all(&dir);
}

/// US2-2 no spurious change: when user never navigates from launch image,
/// handoff content equals launch image path, and resolution round-trips to
/// itself (idempotent, no spurious "changed" event).
#[test]
fn handoff_unchanged_when_launched_with_equals_landed() {
    let dir = std::env::temp_dir().join(format!(
        "rust-feh-handoff-unchanged-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();

    // Create 3 fake image files
    let a_jpg = dir.join("a.jpg");
    let b_jpg = dir.join("b.jpg");
    let c_jpg = dir.join("c.jpg");
    fs::write(&a_jpg, b"fake image a").unwrap();
    fs::write(&b_jpg, b"fake image b").unwrap();
    fs::write(&c_jpg, b"fake image c").unwrap();

    let filelist = vec![a_jpg.clone(), b_jpg.clone(), c_jpg.clone()];

    // Simulate "user never navigated": handoff content is the FIRST file (launch point)
    let handoff_content = format!("{}\n", a_jpg.display());

    // Validate and resolve
    let resolved = validate_handoff(&handoff_content, &filelist);
    assert_eq!(
        resolved,
        Some(a_jpg.clone()),
        "handoff should resolve to a.jpg (no spurious change when unchanged)"
    );

    let _ = fs::remove_dir_all(&dir);
}

/// SC-002 logic tier: reject tampered handoff content (not in filelist).
/// If untrusted handoff content names a path NOT in the original launch filelist,
/// validate_handoff returns None and the handoff is silently ignored.
#[test]
fn handoff_tampered_content_is_ignored() {
    let dir = std::env::temp_dir().join(format!(
        "rust-feh-handoff-tampered-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();

    // Create 3 legit files for the filelist
    let a_jpg = dir.join("a.jpg");
    let b_jpg = dir.join("b.jpg");
    let c_jpg = dir.join("c.jpg");
    fs::write(&a_jpg, b"fake image a").unwrap();
    fs::write(&b_jpg, b"fake image b").unwrap();
    fs::write(&c_jpg, b"fake image c").unwrap();

    let filelist = vec![a_jpg, b_jpg, c_jpg];

    // Simulate tampered handoff: path not in filelist (escape attempt or corruption)
    let tampered_content = "/tmp/totally/unrelated/path/not-in-list.jpg\n";

    // Validate and resolve
    let resolved = validate_handoff(tampered_content, &filelist);
    assert_eq!(
        resolved, None,
        "tampered handoff should be rejected (R4: membership gate)"
    );

    // Also test with garbage/binary content
    let garbage_content = "garbage \x00 content\n";
    let resolved_garbage = validate_handoff(garbage_content, &filelist);
    assert_eq!(
        resolved_garbage, None,
        "garbage handoff should be rejected"
    );

    let _ = fs::remove_dir_all(&dir);
}

/// SC-002 logic tier: reject empty handoff (zero bytes or whitespace-only).
/// validate_handoff returns None for empty or whitespace-only content.
#[test]
fn handoff_empty_content_is_ignored() {
    let dir = std::env::temp_dir().join(format!(
        "rust-feh-handoff-empty-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();

    let a_jpg = dir.join("a.jpg");
    fs::write(&a_jpg, b"fake image a").unwrap();
    let filelist = vec![a_jpg];

    // Empty content
    let resolved_empty = validate_handoff("", &filelist);
    assert_eq!(resolved_empty, None, "empty handoff should be rejected");

    // Whitespace-only content
    let resolved_whitespace = validate_handoff("   \n  \n", &filelist);
    assert_eq!(
        resolved_whitespace, None,
        "whitespace-only handoff should be rejected"
    );

    // Single newline (first line empty after trimming)
    let resolved_newline = validate_handoff("\n", &filelist);
    assert_eq!(
        resolved_newline, None,
        "newline-only handoff should be rejected"
    );

    let _ = fs::remove_dir_all(&dir);
}

/// SC-002 logic tier: two independent round-trips resolve independently.
/// This test proves the necessary precondition for "most recent close wins"
/// (the GUI-level ordering in main.rs::handle_round_trip_exit): at the logic
/// tier, each round-trip's handoff resolution is fully stateless and
/// independent — a viewer's handoff can only ever resolve against ITS OWN
/// filelist, never another viewer's. There is no cross-contamination.
#[test]
fn two_independent_round_trips_resolve_independently() {
    let dir1 = std::env::temp_dir().join(format!(
        "rust-feh-handoff-rt1-{}",
        std::process::id()
    ));
    let dir2 = std::env::temp_dir().join(format!(
        "rust-feh-handoff-rt2-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir1);
    let _ = fs::remove_dir_all(&dir2);
    fs::create_dir_all(&dir1).unwrap();
    fs::create_dir_all(&dir2).unwrap();

    // Round-trip 1: files x1.jpg, x2.jpg with handoff pointing to x2.jpg
    let x1_jpg = dir1.join("x1.jpg");
    let x2_jpg = dir1.join("x2.jpg");
    fs::write(&x1_jpg, b"fake image x1").unwrap();
    fs::write(&x2_jpg, b"fake image x2").unwrap();
    let filelist_1 = vec![x1_jpg.clone(), x2_jpg.clone()];
    let handoff_content_1 = format!("{}\n", x2_jpg.display());

    // Round-trip 2: files y1.jpg, y2.jpg with handoff pointing to y1.jpg
    let y1_jpg = dir2.join("y1.jpg");
    let y2_jpg = dir2.join("y2.jpg");
    fs::write(&y1_jpg, b"fake image y1").unwrap();
    fs::write(&y2_jpg, b"fake image y2").unwrap();
    let filelist_2 = vec![y1_jpg.clone(), y2_jpg.clone()];
    let handoff_content_2 = format!("{}\n", y1_jpg.display());

    // Validate each independently
    let resolved_1 = validate_handoff(&handoff_content_1, &filelist_1);
    let resolved_2 = validate_handoff(&handoff_content_2, &filelist_2);

    assert_eq!(
        resolved_1, Some(x2_jpg.clone()),
        "round-trip 1 should resolve to x2.jpg independently"
    );
    assert_eq!(
        resolved_2, Some(y1_jpg.clone()),
        "round-trip 2 should resolve to y1.jpg independently"
    );

    // Prove no cross-contamination: rt1's handoff against rt2's filelist should fail
    let cross_1_2 = validate_handoff(&handoff_content_1, &filelist_2);
    assert_eq!(
        cross_1_2, None,
        "rt1's handoff (x2.jpg) should NOT resolve against rt2's filelist (y1, y2)"
    );

    let cross_2_1 = validate_handoff(&handoff_content_2, &filelist_1);
    assert_eq!(
        cross_2_1, None,
        "rt2's handoff (y1.jpg) should NOT resolve against rt1's filelist (x1, x2)"
    );

    let _ = fs::remove_dir_all(&dir1);
    let _ = fs::remove_dir_all(&dir2);
}

/// SC-002 logic tier: handoff_path returns a pid/viewer_id-scoped real file path.
/// The path resolves under ~/.cache/rust-feh/, can be written to and read back,
/// and is unique per (pid, viewer_id) pair.
#[test]
fn handoff_path_is_pid_and_viewer_scoped_real_file() {
    let pid = std::process::id();
    let viewer_id_1 = 9001u64;
    let viewer_id_2 = 9002u64;

    // Get two distinct handoff paths (same pid, different viewer_ids)
    let path_1 = handoff_path(pid, viewer_id_1);
    let path_2 = handoff_path(pid, viewer_id_2);

    // Both should be under ~/.cache/rust-feh/
    let cache_dir = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".cache")
        .join("rust-feh");

    assert!(
        path_1.starts_with(&cache_dir),
        "path_1 should be under ~/.cache/rust-feh/"
    );
    assert!(
        path_2.starts_with(&cache_dir),
        "path_2 should be under ~/.cache/rust-feh/"
    );

    // Paths should be distinct
    assert_ne!(path_1, path_2, "different viewer_ids should produce different paths");

    // Both should contain their respective viewer_ids in the filename
    assert!(
        path_1.to_string_lossy().contains("9001"),
        "path_1 should contain viewer_id 9001"
    );
    assert!(
        path_2.to_string_lossy().contains("9002"),
        "path_2 should contain viewer_id 9002"
    );

    // Create the parent directory if needed, write content to path_1, read it back
    let parent = path_1.parent().unwrap();
    fs::create_dir_all(parent).unwrap();

    let test_content = "round-trip test content";
    fs::write(&path_1, test_content).unwrap();
    let read_back = fs::read_to_string(&path_1).unwrap();
    assert_eq!(
        read_back, test_content,
        "handoff file should round-trip content correctly"
    );

    // Clean up
    let _ = fs::remove_file(&path_1);

    // Verify same pid + viewer_id produces same path (idempotent)
    let path_1_again = handoff_path(pid, viewer_id_1);
    assert_eq!(
        path_1, path_1_again,
        "handoff_path should be deterministic for same (pid, viewer_id)"
    );
}
