// SPDX-License-Identifier: MIT
//! Core domain types for rust-feh (clean, independent of GUI).

use serde::{Deserialize, Serialize};
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
    fn count_status(entries: &[ImageEntry], status: FileStatus) -> usize {
        entries.iter().filter(|e| e.status == status).count()
    }

    pub fn from_entries(entries: &[ImageEntry], non_image_skipped: usize, magick_truncated: bool) -> Self {
        let native_listed = Self::count_status(entries, FileStatus::NativeListed);
        let converted = Self::count_status(entries, FileStatus::Converted);
        let awaiting_convert = Self::count_status(entries, FileStatus::MagickDetected);
        let magick_converted = entries
            .iter()
            .filter(|e| e.status == FileStatus::Converted && is_magick_origin(e))
            .count();
        Self {
            native_listed,
            magick_detected: awaiting_convert + magick_converted,
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
    /// Extended for 013: Optimized (from prepare-fast), Processed (from tools).
    pub asset_status: AssetStatus,
}

impl ImageEntry {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            size_bytes: None,
            status: FileStatus::NativeListed,
            asset_status: AssetStatus::Regular,
        }
    }

    pub fn with_status(path: PathBuf, status: FileStatus) -> Self {
        Self {
            path,
            size_bytes: None,
            status,
            asset_status: AssetStatus::Regular,
        }
    }

    pub fn with_asset_status(path: PathBuf, status: FileStatus, asset_status: AssetStatus) -> Self {
        Self {
            path,
            size_bytes: None,
            status,
            asset_status,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum WindowSizePreset {
    /// 720 × 540
    Compact,
    /// 960 × 720
    #[default]
    Default,
    /// 1280 × 960
    Large,
}

/// Persisted window sizing choice (feature 006, FR-008 / SC-005).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct WindowPreferences {
    pub version: u32,
    pub preset: WindowSizePreset,
    pub resizable: bool,
}

impl Default for WindowPreferences {
    fn default() -> Self {
        Self {
            version: 1,
            preset: WindowSizePreset::Default,
            resizable: true,
        }
    }
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

/// A persisted configuration entry for launching an independent feh instance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FehLaunchEntry {
    pub id: String,
    pub label: Option<String>,
    pub folder_path: Option<PathBuf>,
    pub created_at: u64,
}

/// Ordered persisted collection of feh launch profiles.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FehLaunchList {
    pub version: u32,
    pub entries: Vec<FehLaunchEntry>,
}

impl Default for FehLaunchList {
    fn default() -> Self {
        Self {
            version: 1,
            entries: Vec::new(),
        }
    }
}

/// Policy for where processed/renamed output goes (image tools feature 013).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OutputPolicy {
    /// Create new files in a subfolder (e.g. "processed").
    NewSubfolder { name: String },
    /// Create alongside originals with a suffix (e.g. "_resized").
    SuffixedSibling { suffix: String },
    /// Backup originals then modify in place (requires explicit confirmation).
    InPlaceWithBackup { backup_suffix: String },
}

impl Default for OutputPolicy {
    fn default() -> Self {
        Self::NewSubfolder {
            name: "processed".to_string(),
        }
    }
}

// ProcessOptions lives in image_proc (feature 013).
// We keep other new types here.

/// High level operation descriptor (for service layer and cache IRIs).
#[derive(Debug, Clone, PartialEq)]
pub enum ImageOperation {
    Resize {
        width: Option<u32>,
        height: Option<u32>,
        percent: Option<f32>,
        fit: Option<FitMode>,
        filter: Option<Filter>,
        quality: Option<u8>,
    },
    Crop {
        geometry: String, // "WxH+X+Y"
    },
    Convert {
        target_format: String,
        quality: Option<u8>,
    },
    Rename {
        pattern: String,
    },
}

/// Fit mode for resize (per plan I1 fix, contracts, data-model for professional tools).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FitMode {
    #[default]
    Contain,
    Cover,
    Stretch,
}

/// Filter for resize (re-export or wrapper; use image::imageops::FilterType under the hood in image_proc).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Filter {
    #[default]
    Lanczos3,
    Nearest,
    Triangle,
    CatmullRom,
    Gaussian,
}

/// Result of processing one image.
#[derive(Debug, Clone)]
pub struct ProcessedResult {
    pub source_path: std::path::PathBuf,
    pub dest_path: std::path::PathBuf,
    pub operation: ImageOperation,
    pub cache_iri: Option<String>,
    pub was_cache_hit: bool,
    pub materialized_for_fast: Option<std::path::PathBuf>,
}

/// Cache configuration (in-memory for v1 per spec).
#[derive(Debug, Clone, Default)]
pub struct CacheConfig {
    pub enabled: bool,
    pub root: Option<std::path::PathBuf>,
    pub passkey_path: Option<std::path::PathBuf>,
    pub default_ttl: String, // "90 days" or "never"
}

/// A set of prepared fast-viewing assets (for US5).
#[derive(Debug, Clone, Default)]
pub struct PreparedFastSet {
    pub materialized_paths: Vec<std::path::PathBuf>,
    pub filelist_path: Option<std::path::PathBuf>,
    pub source_folder: std::path::PathBuf,
}

/// Simple rename pattern (parsed).
#[derive(Debug, Clone, Default)]
pub struct RenamePattern {
    pub raw: String,
    // Tokens expanded at apply time.
}

/// Status extension for ImageEntry (for optimized assets).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AssetStatus {
    #[default]
    Regular,
    Optimized, // from Prepare Fast feh
    Processed, // from tools
}
