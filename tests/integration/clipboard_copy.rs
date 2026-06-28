// SPDX-License-Identifier: MIT
use std::fs;
use std::path::PathBuf;

use rust_feh::ui_logic::copy_image_to_clipboard;

fn display_available() -> bool {
    std::env::var_os("DISPLAY").is_some() || std::env::var_os("WAYLAND_DISPLAY").is_some()
}

fn make_test_png(dir: &PathBuf) -> (PathBuf, u32, u32) {
    let path = dir.join("clipboard-fixture.png");
    let img = image::RgbaImage::from_pixel(17, 11, image::Rgba([40, 80, 120, 255]));
    img.save(&path).unwrap();
    (path, 17, 11)
}

#[test]
fn clipboard_copy_roundtrip_preserves_dimensions() {
    if !display_available() {
        eprintln!("skip clipboard_copy_roundtrip: no DISPLAY/WAYLAND_DISPLAY");
        return;
    }

    let dir = std::env::temp_dir().join(format!("rust-feh-clipboard-it-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let (path, width, height) = make_test_png(&dir);

    if let Err(e) = copy_image_to_clipboard(&path) {
        if e.contains("Clipboard unavailable") {
            eprintln!("skip clipboard_copy_roundtrip: {e}");
            let _ = fs::remove_dir_all(&dir);
            return;
        }
        panic!("copy to clipboard: {e}");
    }

    let mut clipboard = match arboard::Clipboard::new() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("skip clipboard_copy_roundtrip: clipboard unavailable: {e}");
            let _ = fs::remove_dir_all(&dir);
            return;
        }
    };
    let image = match clipboard.get_image() {
        Ok(image) => image,
        Err(arboard::Error::ContentNotAvailable) => {
            eprintln!(
                "skip clipboard_copy_roundtrip: clipboard manager does not expose image readback on this display"
            );
            let _ = fs::remove_dir_all(&dir);
            return;
        }
        Err(e) => panic!("read image from clipboard: {e}"),
    };
    assert_eq!(image.width, width as usize);
    assert_eq!(image.height, height as usize);
    assert_eq!(image.bytes.len(), (width as usize) * (height as usize) * 4);

    let _ = fs::remove_dir_all(&dir);
}