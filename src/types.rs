// SPDX-License-Identifier: MIT
//! Core domain types for rust-feh (clean, independent of GUI).

use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FileStatus {
    #[default]
    NativeListed,
    MagickDetected,
    Converted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ListViewMode {
    #[default]
    FlatList,
    FolderTree,
}

/// Aggregate counts after a directory scan (feature 005).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ScanInventory {
    pub native_listed: usize,
    pub magick_detected: usize,
    pub converted: usize,
    pub awaiting_convert: usize,
    pub non_image_skipped: usize,
    pub magick_identify_truncated: bool,
}

impl ScanInventory {
    pub fn from_entries(entries: &[ImageEntry], non_image_skipped: usize, magick_truncated: bool) -> Self {
        let native_listed = entries
            .iter()
            .filter(|e| e.status == FileStatus::NativeListed)
            .count();
        let converted = entries
            .iter()
            .filter(|e| e.status == FileStatus::Converted)
            .count();
        let awaiting_convert = entries
            .iter()
            .filter(|e| e.status == FileStatus::MagickDetected)
            .count();
        let magick_detected = awaiting_convert
            + entries
                .iter()
                .filter(|e| e.status == FileStatus::Converted && is_magick_origin(e))
                .count();
        Self {
            native_listed,
            magick_detected,
            converted,
            awaiting_convert,
            non_image_skipped,
            magick_identify_truncated: magick_truncated,
        }
    }
}

fn is_magick_origin(entry: &ImageEntry) -> bool {
    !is_native_extension(entry.path.extension().and_then(|e| e.to_str()))
}

fn is_native_extension(ext: Option<&str>) -> bool {
    ext.map(|e| e.to_lowercase())
        .is_some_and(|e| matches!(e.as_str(), "jpg" | "jpeg" | "png" | "webp" | "gif" | "bmp"))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageEntry {
    pub path: PathBuf,
    pub size_bytes: Option<u64>,
    pub status: FileStatus,
}

impl ImageEntry {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            size_bytes: None,
            status: FileStatus::NativeListed,
        }
    }

    pub fn with_status(path: PathBuf, status: FileStatus) -> Self {
        Self {
            path,
            size_bytes: None,
            status,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WindowSizePreset {
    /// 720 × 540
    Compact,
    /// 960 × 720
    #[default]
    Default,
    /// 1280 × 960
    Large,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortMode {
    /// Full path (scanner default).
    #[default]
    Path,
    /// Filename only, case-insensitive.
    Name,
    /// Relative folder, then filename.
    Folder,
}
