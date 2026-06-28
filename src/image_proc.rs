// SPDX-License-Identifier: MIT
//! Image processing + magick-cache integration (extends core module per feature 013).
//! Pure logic: no egui. External tools via `std::process::Command` only.

use image::{imageops::FilterType, DynamicImage, ImageFormat};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::types::{FitMode, Filter, ImageOperation, OutputPolicy, ProcessedResult};
use crate::ui_logic::compute_output_path;

#[derive(Debug, Clone, Default)]
pub struct ProcessOptions {
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub percent: Option<f32>,
    pub fit: Option<FitMode>,
    pub filter: Option<Filter>,
    pub target_format: Option<String>,
    pub quality: Option<u8>,
    pub output_path: Option<PathBuf>,
}

pub fn has_external_magick() -> bool {
    crate::tool_caps::ToolCapabilities::detect().magick_available
}

pub fn has_magick_cache() -> bool {
    crate::tool_caps::ToolCapabilities::detect().magick_cache_available
}

pub fn cache_ready(cache_root: &Path, passkey: Option<&Path>) -> bool {
    if !has_magick_cache() || !cache_root.is_dir() {
        return false;
    }
    cache_probe(cache_root, passkey)
}

fn quiet_magick_cache(cmd: &mut Command) {
    cmd.stdout(Stdio::null()).stderr(Stdio::null());
}

// --- Crop geometry (WxH+X+Y, signed offsets) ---

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CropRect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

pub fn parse_crop_geometry(geometry: &str) -> Result<CropRect, String> {
    let geometry = geometry.trim();
    let x_idx = geometry
        .find('x')
        .ok_or_else(|| "Invalid crop geometry: expected WxH+X+Y".to_string())?;
    let w: u32 = geometry[..x_idx]
        .trim()
        .parse()
        .map_err(|_| "Invalid crop width".to_string())?;
    let rest = &geometry[x_idx + 1..];
    let plus = rest
        .find('+')
        .ok_or_else(|| "Invalid crop geometry: missing +offset".to_string())?;
    let h: u32 = rest[..plus]
        .trim()
        .parse()
        .map_err(|_| "Invalid crop height".to_string())?;
    let tail = &rest[plus + 1..];
    let (x_str, y_str) = if let Some(p2) = tail.find('+') {
        (&tail[..p2], &tail[p2 + 1..])
    } else if let Some(m) = tail.find('-') {
        if m == 0 {
            let tail2 = &tail[1..];
            let p2 = tail2
                .find('+')
                .or_else(|| tail2.find('-'))
                .ok_or_else(|| "Invalid crop Y offset".to_string())?;
            (&tail[..=p2], &tail2[p2..])
        } else {
            (&tail[..m], &tail[m..])
        }
    } else {
        return Err("Invalid crop geometry: expected WxH+X+Y".into());
    };
    let x: i32 = x_str
        .trim()
        .parse()
        .map_err(|_| "Invalid crop X offset".to_string())?;
    let y: i32 = y_str
        .trim()
        .parse()
        .map_err(|_| "Invalid crop Y offset".to_string())?;
    if w == 0 || h == 0 {
        return Err("Crop width and height must be > 0".into());
    }
    Ok(CropRect { x, y, width: w, height: h })
}

fn clamp_crop(rect: CropRect, img_w: u32, img_h: u32) -> Result<(u32, u32, u32, u32), String> {
    let x = rect.x.max(0) as u32;
    let y = rect.y.max(0) as u32;
    if x >= img_w || y >= img_h {
        return Err("Crop origin outside image bounds".into());
    }
    let w = rect.width.min(img_w - x);
    let h = rect.height.min(img_h - y);
    if w == 0 || h == 0 {
        return Err("Crop region empty after clamping".into());
    }
    Ok((x, y, w, h))
}

fn filter_type(f: Option<Filter>) -> FilterType {
    match f {
        Some(Filter::Nearest) => FilterType::Nearest,
        Some(Filter::Triangle) => FilterType::Triangle,
        Some(Filter::CatmullRom) => FilterType::CatmullRom,
        Some(Filter::Gaussian) => FilterType::Gaussian,
        _ => FilterType::Lanczos3,
    }
}

fn default_ext_for_op(op: &ImageOperation) -> &str {
    match op {
        ImageOperation::Convert { target_format, .. } => target_format.as_str(),
        ImageOperation::Crop { .. } => "png",
        _ => "jpg",
    }
}

fn op_params_key(op: &ImageOperation) -> String {
    format!("{:?}", op)
}

// --- Magick cache ---

#[derive(Debug, Clone)]
pub struct MagickCacheManager {
    config: crate::types::CacheConfig,
    /// Probed once at construction; never re-run per put/get (identify dumps entire cache).
    ready: bool,
}

impl MagickCacheManager {
    pub fn new(config: crate::types::CacheConfig) -> Self {
        let ready = config.enabled
            && cache_ready(&cache_root_path(&config), config.passkey_path.as_deref());
        Self { config, ready }
    }

    pub fn enabled_and_ready(&self) -> bool {
        self.config.enabled && self.ready
    }

    pub fn make_iri(&self, source: &Path, operation: &str, params: &str) -> String {
        let sanitized = source
            .to_string_lossy()
            .replace(['/', '\\', ':', ' ', '.'], "_");
        let mut hasher = DefaultHasher::new();
        format!("{}{}{}", sanitized, operation, params).hash(&mut hasher);
        let hash = hasher.finish();
        // MagickCache IRI: project/type/resource-path (type must be image|blob|meta)
        format!("rustfeh/image/{}/{:016x}", operation, hash)
    }

    pub fn cache_root(&self) -> PathBuf {
        cache_root_path(&self.config)
    }

    pub fn put(&self, input: &Path, iri: &str) -> Result<(), String> {
        if !self.config.enabled {
            return Ok(());
        }
        cache_put(
            &self.cache_root(),
            input,
            iri,
            self.config.passkey_path.as_deref(),
            &self.config.default_ttl,
        )
    }

    pub fn get(&self, iri: &str, dest: &Path) -> Result<(), String> {
        if !self.config.enabled {
            return Err("cache disabled".into());
        }
        cache_get(
            &self.cache_root(),
            iri,
            dest,
            self.config.passkey_path.as_deref(),
        )
    }

    pub fn materialize(&self, iri: &str, dest: &Path) -> Result<PathBuf, String> {
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        materialize_from_cache(
            &self.cache_root(),
            self.config.passkey_path.as_deref(),
            iri,
            dest,
        )?;
        Ok(dest.to_path_buf())
    }
}

pub fn cache_root_path(config: &crate::types::CacheConfig) -> PathBuf {
    config.root.clone().unwrap_or_else(|| {
        std::env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join(".cache/rust-feh-magick")
    })
}

fn append_passkey(cmd: &mut std::process::Command, passkey: Option<&Path>) {
    if let Some(pk) = passkey {
        cmd.arg("-passkey").arg(pk);
    }
}

pub fn cache_put(
    cache_root: &Path,
    input: &Path,
    iri: &str,
    passkey: Option<&Path>,
    ttl: &str,
) -> Result<(), String> {
    if !has_magick_cache() {
        return Err("magick-cache not installed".into());
    }
    let mut cmd = Command::new("magick-cache");
    quiet_magick_cache(&mut cmd);
    append_passkey(&mut cmd, passkey);
    cmd.arg("-ttl").arg(ttl).arg("put").arg(cache_root).arg(iri).arg(input);
    let status = cmd.status().map_err(|e| e.to_string())?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("magick-cache put failed for {}", iri))
    }
}

pub fn cache_get(
    cache_root: &Path,
    iri: &str,
    dest: &Path,
    passkey: Option<&Path>,
) -> Result<(), String> {
    if !has_magick_cache() {
        return Err("magick-cache not installed".into());
    }
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    let mut cmd = Command::new("magick-cache");
    quiet_magick_cache(&mut cmd);
    append_passkey(&mut cmd, passkey);
    cmd.arg("get").arg(cache_root).arg(iri).arg(dest);
    let status = cmd.status().map_err(|e| e.to_string())?;
    if status.success() && dest.is_file() {
        Ok(())
    } else {
        Err(format!("magick-cache get miss for {}", iri))
    }
}

pub fn materialize_from_cache(
    cache_root: &Path,
    passkey: Option<&Path>,
    iri: &str,
    dest: &Path,
) -> Result<(), String> {
    let dmr_ref = format!("dmr:{iri}");
    let mut cmd = std::process::Command::new("magick");
    cmd.arg("convert")
        .arg("-define")
        .arg(format!("dmr:path={}", cache_root.display()))
        .arg("-auto-orient")
        .arg("-strip")
        .arg("-quality")
        .arg("92")
        .arg(&dmr_ref)
        .arg(dest);
    if let Some(pk) = passkey {
        cmd.arg("-define").arg(format!("dmr:passkey={}", pk.display()));
    }
    let status = cmd.status().map_err(|e| e.to_string())?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("materialize failed for {}", iri))
    }
}

pub fn cache_probe(cache_root: &Path, passkey: Option<&Path>) -> bool {
    if !has_magick_cache() {
        return false;
    }
    let mut cmd = Command::new("magick-cache");
    quiet_magick_cache(&mut cmd);
    append_passkey(&mut cmd, passkey);
    cmd.arg("identify").arg(cache_root).arg("/");
    cmd.status().map(|s| s.success()).unwrap_or(false)
}

// --- Image ops (image crate + magick fallback) ---

pub fn process_image(input: &Path, opts: &ProcessOptions) -> Result<PathBuf, String> {
    let out_path = opts
        .output_path
        .clone()
        .ok_or_else(|| "output_path required".to_string())?;
    if let Some(parent) = out_path.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    if has_external_magick() {
        if let Ok(()) = run_magick_resize_convert(input, opts, &out_path) {
            return Ok(out_path);
        }
    }
    process_image_crate(input, opts, &out_path)
}

fn process_image_crate(input: &Path, opts: &ProcessOptions, out_path: &Path) -> Result<PathBuf, String> {
    let img = image::open(input).map_err(|e| e.to_string())?;
    let filter = filter_type(opts.filter);
    let mut out = img;
    if let Some(pct) = opts.percent {
        let w = (out.width() as f32 * pct / 100.0).max(1.0) as u32;
        let h = (out.height() as f32 * pct / 100.0).max(1.0) as u32;
        out = out.resize(w, h, filter);
    } else if let (Some(w), Some(h)) = (opts.width, opts.height) {
        out = match opts.fit {
            Some(FitMode::Cover) => out.resize_to_fill(w, h, filter),
            Some(FitMode::Contain) => out.resize(w, h, filter),
            _ => out.resize_exact(w, h, filter),
        };
    } else if let Some(w) = opts.width {
        let h = (out.height() as f32 * (w as f32 / out.width() as f32)).max(1.0) as u32;
        out = out.resize(w, h, filter);
    } else if let Some(h) = opts.height {
        let w = (out.width() as f32 * (h as f32 / out.height() as f32)).max(1.0) as u32;
        out = out.resize(w, h, filter);
    }
    write_image(&out, out_path, opts.quality, opts.target_format.as_deref())
}

pub fn crop_image(input: &Path, geometry: &str, out_path: &Path) -> Result<PathBuf, String> {
    if has_external_magick() {
        if let Ok(()) = run_magick_crop(input, geometry, out_path) {
            return Ok(out_path.to_path_buf());
        }
    }
    let rect = parse_crop_geometry(geometry)?;
    let img = image::open(input).map_err(|e| e.to_string())?;
    let (x, y, w, h) = clamp_crop(rect, img.width(), img.height())?;
    let cropped = img.crop_imm(x, y, w, h);
    if let Some(parent) = out_path.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    cropped.save(out_path).map_err(|e| e.to_string())?;
    Ok(out_path.to_path_buf())
}

fn write_image(
    img: &DynamicImage,
    out_path: &Path,
    quality: Option<u8>,
    target_format: Option<&str>,
) -> Result<PathBuf, String> {
    if let Some(parent) = out_path.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    let ext = out_path
        .extension()
        .and_then(|e| e.to_str())
        .or(target_format)
        .unwrap_or("png");
    let file = std::fs::File::create(out_path).map_err(|e| e.to_string())?;
    let mut writer = std::io::BufWriter::new(file);
    match ext.to_lowercase().as_str() {
        "jpg" | "jpeg" => {
            let q = quality.unwrap_or(85);
            let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut writer, q);
            img.write_with_encoder(encoder).map_err(|e| e.to_string())?;
        }
        "webp" => {
            img.write_to(&mut writer, ImageFormat::WebP)
                .map_err(|e| e.to_string())?;
        }
        _ => {
            img.write_to(&mut writer, ImageFormat::Png)
                .map_err(|e| e.to_string())?;
        }
    }
    Ok(out_path.to_path_buf())
}

fn run_magick_resize_convert(input: &Path, opts: &ProcessOptions, out: &Path) -> Result<(), String> {
    let mut cmd = std::process::Command::new("magick");
    cmd.arg("convert").arg(input).arg("-auto-orient");
    if let Some(pct) = opts.percent {
        cmd.arg("-resize").arg(format!("{pct}%"));
    } else if let (Some(w), Some(h)) = (opts.width, opts.height) {
        let spec = match opts.fit {
            Some(FitMode::Cover) => format!("{w}x{h}^"),
            Some(FitMode::Contain) => format!("{w}x{h}"),
            _ => format!("{w}x{h}!"),
        };
        cmd.arg("-resize").arg(spec);
    } else if let Some(w) = opts.width {
        cmd.arg("-resize").arg(format!("{w}x"));
    } else if let Some(h) = opts.height {
        cmd.arg("-resize").arg(format!("x{h}"));
    }
    if let Some(q) = opts.quality {
        cmd.arg("-quality").arg(q.to_string());
    }
    cmd.arg("-strip").arg(out);
    cmd.status()
        .map_err(|e| e.to_string())?
        .success()
        .then_some(())
        .ok_or_else(|| "magick resize failed".to_string())
}

fn run_magick_crop(input: &Path, geometry: &str, out: &Path) -> Result<(), String> {
    let status = std::process::Command::new("magick")
        .arg("convert")
        .arg(input)
        .arg("-crop")
        .arg(geometry)
        .arg("+repage")
        .arg(out)
        .status()
        .map_err(|e| e.to_string())?;
    if status.success() {
        Ok(())
    } else {
        Err("magick crop failed".into())
    }
}

// --- Service ---

#[derive(Debug, Clone)]
pub struct ImageToolsService {
    cache: Option<MagickCacheManager>,
}

impl Default for ImageToolsService {
    fn default() -> Self {
        Self::new(None)
    }
}

impl ImageToolsService {
    pub fn new(cache_config: Option<crate::types::CacheConfig>) -> Self {
        let cache = cache_config
            .filter(|c| c.enabled)
            .map(MagickCacheManager::new);
        Self { cache }
    }

    pub fn update_cache(&mut self, config: crate::types::CacheConfig) {
        self.cache = config.enabled.then(|| MagickCacheManager::new(config));
    }

    pub fn cache_enabled_and_ready(&self) -> bool {
        self.cache
            .as_ref()
            .is_some_and(|cm| cm.enabled_and_ready())
    }

    pub fn process_single(
        &self,
        source: &Path,
        op: ImageOperation,
        policy: OutputPolicy,
    ) -> Result<ProcessedResult, String> {
        let ext = default_ext_for_op(&op);
        let stem_suffix = op_output_suffix(&op);
        let dest = compute_output_path(source, stem_suffix, ext, &policy)?;
        let params = op_params_key(&op);
        let mut was_cache_hit = false;
        let mut cache_iri = None;

        if let Some(cm) = self.cache.as_ref() {
            if cm.enabled_and_ready() {
                let iri = cm.make_iri(source, "processed", &params);
                if cm.get(&iri, &dest).is_ok() {
                    was_cache_hit = true;
                    cache_iri = Some(iri);
                    return Ok(ProcessedResult {
                        source_path: source.to_path_buf(),
                        dest_path: dest,
                        operation: op,
                        cache_iri,
                        was_cache_hit,
                        materialized_for_fast: None,
                    });
                }
            }
        }

        if policy_requires_backup(&policy) {
            create_backup(source, &policy)?;
        }

        execute_op_to_path(source, &op, &dest)?;

        if let Some(cm) = self.cache.as_ref() {
            if cm.enabled_and_ready() {
                let orig_iri = cm.make_iri(source, "originals", "v1");
                let _ = cm.put(source, &orig_iri);
                let result_iri = cm.make_iri(source, "processed", &params);
                if cm.put(&dest, &result_iri).is_ok() {
                    cache_iri = Some(result_iri);
                }
            }
        }

        Ok(ProcessedResult {
            source_path: source.to_path_buf(),
            dest_path: dest,
            operation: op,
            cache_iri,
            was_cache_hit,
            materialized_for_fast: None,
        })
    }

    /// Put one original into cache (for per-path background jobs).
    pub fn pre_cache_one(&self, source: &Path) -> bool {
        let Some(cm) = self.cache.as_ref() else {
            return false;
        };
        if !cm.enabled_and_ready() {
            return false;
        }
        let iri = cm.make_iri(source, "originals", "v1");
        cm.put(source, &iri).is_ok()
    }

    /// Put originals into cache for paths (background-friendly; caller iterates).
    pub fn pre_cache_paths(&self, sources: &[PathBuf]) -> usize {
        sources
            .iter()
            .filter(|src| self.pre_cache_one(src))
            .count()
    }

    /// Materialize one optimized JPEG for feh fast viewing.
    pub fn prepare_fast_one(&self, source: &Path, temp_dir: &Path) -> Result<PathBuf, String> {
        std::fs::create_dir_all(temp_dir).map_err(|e| e.to_string())?;
        let stem = source
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .replace(['/', '\\'], "_");
        let dest = temp_dir.join(format!("{stem}_fast.jpg"));
        if has_external_magick() {
            let status = std::process::Command::new("magick")
                .arg("convert")
                .arg(source)
                .arg("-auto-orient")
                .arg("-strip")
                .arg("-quality")
                .arg("92")
                .arg(&dest)
                .status()
                .map_err(|e| e.to_string())?;
            if !status.success() {
                return Err("magick prepare-fast failed".into());
            }
        } else {
            let opts = ProcessOptions {
                target_format: Some("jpg".into()),
                quality: Some(92),
                output_path: Some(dest.clone()),
                ..Default::default()
            };
            process_image(source, &opts)?;
        }
        if let Some(cm) = self.cache.as_ref() {
            if cm.enabled_and_ready() {
                let iri = cm.make_iri(source, "fast", "v1");
                let _ = cm.put(&dest, &iri);
            }
        }
        Ok(dest)
    }

    /// Materialize optimized JPEGs for feh fast viewing (US5).
    pub fn prepare_fast_set(
        &self,
        sources: &[PathBuf],
        temp_dir: &Path,
    ) -> Result<crate::types::PreparedFastSet, String> {
        let mut materialized = Vec::new();
        for src in sources {
            if let Ok(dest) = self.prepare_fast_one(src, temp_dir) {
                materialized.push(dest);
            }
        }
        let source_folder = sources
            .first()
            .and_then(|p| p.parent())
            .unwrap_or(Path::new("."))
            .to_path_buf();
        Ok(crate::types::PreparedFastSet {
            materialized_paths: materialized,
            filelist_path: None,
            source_folder,
        })
    }

    pub fn process_batch(
        &self,
        sources: &[PathBuf],
        op: ImageOperation,
        policy: OutputPolicy,
    ) -> (Vec<Result<ProcessedResult, String>>, BatchSummary) {
        let mut results = Vec::with_capacity(sources.len());
        let mut ok = 0usize;
        let mut err = 0usize;
        for src in sources {
            match self.process_single(src, op.clone(), policy.clone()) {
                Ok(r) => {
                    ok += 1;
                    results.push(Ok(r));
                }
                Err(e) => {
                    err += 1;
                    results.push(Err(e));
                }
            }
        }
        (
            results,
            BatchSummary {
                total: sources.len(),
                succeeded: ok,
                failed: err,
                skipped: 0,
            },
        )
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BatchSummary {
    pub total: usize,
    pub succeeded: usize,
    pub failed: usize,
    pub skipped: usize,
}

fn policy_requires_backup(policy: &OutputPolicy) -> bool {
    matches!(policy, OutputPolicy::InPlaceWithBackup { .. })
}

fn create_backup(source: &Path, policy: &OutputPolicy) -> Result<PathBuf, String> {
    let OutputPolicy::InPlaceWithBackup { backup_suffix } = policy else {
        return Err("not in-place policy".into());
    };
    let parent = source.parent().unwrap_or(Path::new("."));
    let stem = source.file_stem().unwrap_or_default().to_string_lossy();
    let ext = source
        .extension()
        .map(|e| e.to_string_lossy().into_owned())
        .unwrap_or_default();
    let backup_name = if ext.is_empty() {
        format!("{stem}{backup_suffix}")
    } else {
        format!("{stem}{backup_suffix}.{ext}")
    };
    let backup_path = parent.join(backup_name);
    std::fs::copy(source, &backup_path).map_err(|e| e.to_string())?;
    Ok(backup_path)
}

fn op_output_suffix(op: &ImageOperation) -> &'static str {
    match op {
        ImageOperation::Resize { .. } => "_resized",
        ImageOperation::Crop { .. } => "_cropped",
        ImageOperation::Convert { .. } => "_converted",
        ImageOperation::Rename { .. } => "",
    }
}

fn execute_op_to_path(source: &Path, op: &ImageOperation, dest: &Path) -> Result<(), String> {
    match op {
        ImageOperation::Resize {
            width,
            height,
            percent,
            fit,
            filter,
            quality,
        } => {
            let opts = ProcessOptions {
                width: *width,
                height: *height,
                percent: *percent,
                fit: *fit,
                filter: *filter,
                quality: *quality,
                target_format: dest
                    .extension()
                    .and_then(|e| e.to_str())
                    .map(|s| s.to_string()),
                output_path: Some(dest.to_path_buf()),
            };
            process_image(source, &opts).map(|_| ())
        }
        ImageOperation::Crop { geometry } => {
            crop_image(source, geometry, dest).map(|_| ())
        }
        ImageOperation::Convert {
            target_format,
            quality,
        } => {
            let opts = ProcessOptions {
                target_format: Some(target_format.clone()),
                quality: *quality,
                output_path: Some(dest.to_path_buf()),
                ..Default::default()
            };
            process_image(source, &opts).map(|_| ())
        }
        ImageOperation::Rename { .. } => Err("rename via fs::rename in ui_logic".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::CacheConfig;

    #[test]
    fn parse_crop_valid() {
        let r = parse_crop_geometry("800x600+100+50").unwrap();
        assert_eq!(r.width, 800);
        assert_eq!(r.height, 600);
        assert_eq!(r.x, 100);
        assert_eq!(r.y, 50);
    }

    #[test]
    fn make_iri_stable() {
        let m = MagickCacheManager::new(CacheConfig::default());
        let p = PathBuf::from("/tmp/a.jpg");
        assert_eq!(
            m.make_iri(&p, "resize", "p1"),
            m.make_iri(&p, "resize", "p1")
        );
    }
}