// SPDX-License-Identifier: MIT
//! Simple image processor using the `image` crate (pure Rust, open source).
//! Supports resize (percent or exact), basic format conversion.
//! Optional external magick detection for more formats later.

use image::{imageops::FilterType, ImageFormat};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct ProcessOptions {
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub percent: Option<f32>,
    pub target_format: Option<String>, // "jpg", "png", "webp" ...
    pub quality: Option<u8>,
    pub output_dir: Option<PathBuf>,
}

pub fn process_image(input: &Path, opts: &ProcessOptions) -> Result<PathBuf, String> {
    let img = image::open(input).map_err(|e| e.to_string())?;

    let mut out = img;

    // Resize logic (simple)
    if let Some(pct) = opts.percent {
        let w = (out.width() as f32 * pct / 100.0) as u32;
        let h = (out.height() as f32 * pct / 100.0) as u32;
        out = out.resize(w, h, FilterType::Lanczos3);
    } else if let (Some(w), Some(h)) = (opts.width, opts.height) {
        out = out.resize_exact(w, h, FilterType::Lanczos3);
    } else if let Some(w) = opts.width {
        let h = (out.height() as f32 * (w as f32 / out.width() as f32)) as u32;
        out = out.resize(w, h, FilterType::Lanczos3);
    } else if let Some(h) = opts.height {
        let w = (out.width() as f32 * (h as f32 / out.height() as f32)) as u32;
        out = out.resize(w, h, FilterType::Lanczos3);
    }

    // Determine output path + format
    let stem = input.file_stem().unwrap_or_default().to_string_lossy();
    let ext = opts.target_format.as_deref().unwrap_or("png");
    let out_dir = opts.output_dir.clone().unwrap_or_else(|| input.parent().unwrap().to_path_buf());
    let out_path = out_dir.join(format!("{}_processed.{}", stem, ext));

    std::fs::create_dir_all(&out_dir).ok();

    let file = std::fs::File::create(&out_path).map_err(|e| e.to_string())?;
    let mut writer = std::io::BufWriter::new(file);

    match ext {
        "jpg" | "jpeg" => {
            let quality = opts.quality.unwrap_or(85);
            let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut writer, quality);
            out.write_with_encoder(encoder).map_err(|e| e.to_string())?;
        }
        "png" => {
            out.write_to(&mut writer, ImageFormat::Png).map_err(|e| e.to_string())?;
        }
        "webp" => {
            out.write_to(&mut writer, ImageFormat::WebP).map_err(|e| e.to_string())?;
        }
        _ => {
            out.write_to(&mut writer, ImageFormat::Png).map_err(|e| e.to_string())?;
        }
    }

    Ok(out_path)
}

/// Detect if an external ImageMagick is available (lightweight, never required).
pub fn has_external_magick() -> bool {
    crate::tool_caps::ToolCapabilities::detect().magick_available
}
