// SPDX-License-Identifier: MIT
//! Tool capability matrix: dependency detection, format routing, and speed tiers.
//! Pure logic — no egui dependency (constitution §III).

/// How fast an operation typically feels for the user.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpeedTier {
    /// In-process, sub-second even at 10k scale (list/filter/sort).
    Instant,
    /// Subprocess launch + GPU viewer; per-image ops.
    Fast,
    /// In-process decode/encode (image crate).
    Medium,
    /// External convert pipeline or batch work.
    Slow,
}

impl SpeedTier {
    pub fn label(self) -> &'static str {
        match self {
            SpeedTier::Instant => "Instant",
            SpeedTier::Fast => "Fast",
            SpeedTier::Medium => "Medium",
            SpeedTier::Slow => "Slower",
        }
    }

    pub fn detail(self) -> &'static str {
        match self {
            SpeedTier::Instant => "in-memory; 10k filter <200ms",
            SpeedTier::Fast => "feh subprocess + GPU viewer",
            SpeedTier::Medium => "Rust image crate decode/encode",
            SpeedTier::Slow => "ImageMagick convert / batch",
        }
    }
}

/// Which component handles a capability.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Handler {
    RustFeh,
    Feh,
    ImageCrate,
    ImageMagick,
}

impl Handler {
    pub fn label(self) -> &'static str {
        match self {
            Handler::RustFeh => "rust-feh",
            Handler::Feh => "feh",
            Handler::ImageCrate => "image crate",
            Handler::ImageMagick => "ImageMagick",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DepKind {
    Required,
    Optional,
}

/// External binary the app may delegate to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DependencyStatus {
    pub name: &'static str,
    pub binaries: &'static [&'static str],
    pub kind: DepKind,
    pub role: &'static str,
    pub install_cmd: &'static str,
    pub installed: bool,
    pub resolved_binary: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperationTiming {
    pub operation: &'static str,
    pub handler: Handler,
    pub speed: SpeedTier,
    pub note: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormatRoute {
    pub extensions: &'static str,
    pub scan: Handler,
    pub view: Handler,
    pub resize: Handler,
    pub view_speed: SpeedTier,
    pub note: &'static str,
}

impl FormatRoute {
    /// One-line summary for collapsed format-route rows in the Tools panel.
    pub fn summary_line(&self) -> String {
        format!(
            "{} · View {} · {}",
            self.extensions,
            self.view.label(),
            self.view_speed.label()
        )
    }
}

/// Snapshot of detected tools on PATH.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolCapabilities {
    pub feh_available: bool,
    pub magick_available: bool,
    pub magick_binary: Option<String>,
}

impl ToolCapabilities {
    pub fn detect() -> Self {
        let feh_available = which::which("feh").is_ok();
        let magick_binary = which::which("magick")
            .ok()
            .or_else(|| which::which("convert").ok())
            .map(|p| p.display().to_string());
        let magick_available = magick_binary.is_some();
        Self {
            feh_available,
            magick_available,
            magick_binary,
        }
    }

    pub fn dependencies(&self) -> Vec<DependencyStatus> {
        vec![
            DependencyStatus {
                name: "feh",
                binaries: &["feh"],
                kind: DepKind::Required,
                role: "View, slideshow",
                install_cmd: "sudo apt install feh",
                installed: self.feh_available,
                resolved_binary: self
                    .feh_available
                    .then(|| "feh".to_string()),
            },
            DependencyStatus {
                name: "ImageMagick",
                binaries: &["magick", "convert"],
                kind: DepKind::Optional,
                role: "Magick-detect unlisted formats; convert fallback (010)",
                install_cmd: "sudo apt install imagemagick",
                installed: self.magick_available,
                resolved_binary: self.magick_binary.clone(),
            },
        ]
    }

    pub fn operation_timings(&self) -> Vec<OperationTiming> {
        let view_handler = if self.feh_available {
            Handler::Feh
        } else {
            Handler::RustFeh
        };
        let exotic_view = if self.magick_available && self.feh_available {
            Handler::ImageMagick
        } else if self.feh_available {
            Handler::Feh
        } else {
            Handler::RustFeh
        };

        vec![
            OperationTiming {
                operation: "Browse / filter / sort list",
                handler: Handler::RustFeh,
                speed: SpeedTier::Instant,
                note: "Flat list or folder tree; scan inventory bar",
            },
            OperationTiming {
                operation: "Open / slideshow / navigate",
                handler: view_handler,
                speed: SpeedTier::Fast,
                note: if self.feh_available {
                    "Delegated to feh (not in-app viewer)"
                } else {
                    "feh missing — install to enable"
                },
            },
            OperationTiming {
                operation: "Quick resize (jpg/png/webp)",
                handler: Handler::ImageCrate,
                speed: SpeedTier::Medium,
                note: "Always available; no external deps",
            },
            OperationTiming {
                operation: "Exotic format view (svg/heic/raw…)",
                handler: exotic_view,
                speed: if self.magick_available {
                    SpeedTier::Slow
                } else {
                    SpeedTier::Medium
                },
                note: if self.magick_available {
                    "feh --conversion-timeout + ImageMagick"
                } else {
                    "Limited without ImageMagick on PATH"
                },
            },
        ]
    }

    pub fn format_routes(&self) -> Vec<FormatRoute> {
        let magick_view = if self.magick_available {
            (
                Handler::ImageMagick,
                SpeedTier::Slow,
                "Magick-detected in inventory; feh + magick convert",
            )
        } else {
            (Handler::Feh, SpeedTier::Fast, "Not magick-detected without ImageMagick")
        };

        vec![
            FormatRoute {
                extensions: "jpg, jpeg, png, webp, gif, bmp",
                scan: Handler::RustFeh,
                view: Handler::Feh,
                resize: Handler::ImageCrate,
                view_speed: SpeedTier::Fast,
                note: "Native listed in inventory; feh + image crate",
            },
            FormatRoute {
                extensions: "tiff, tif, pnm, ppm",
                scan: if self.magick_available {
                    Handler::ImageMagick
                } else {
                    Handler::RustFeh
                },
                view: Handler::Feh,
                resize: Handler::ImageMagick,
                view_speed: SpeedTier::Fast,
                note: if self.magick_available {
                    "Magick-detected in inventory when identify succeeds"
                } else {
                    "Not listed without ImageMagick identify"
                },
            },
            FormatRoute {
                extensions: "svg, xcf, otf, psd",
                scan: Handler::RustFeh,
                view: magick_view.0,
                resize: Handler::ImageMagick,
                view_speed: magick_view.1,
                note: magick_view.2,
            },
            FormatRoute {
                extensions: "heic, heif, avif, jxl",
                scan: Handler::RustFeh,
                view: if self.magick_available {
                    Handler::ImageMagick
                } else {
                    Handler::RustFeh
                },
                resize: Handler::ImageMagick,
                view_speed: if self.magick_available {
                    SpeedTier::Slow
                } else {
                    SpeedTier::Instant
                },
                note: if self.magick_available {
                    "Magick-detected (unlisted) in inventory; awaiting convert until processed"
                } else {
                    "Hidden from list/inventory without ImageMagick on PATH"
                },
            },
            FormatRoute {
                extensions: "raw, cr2, nef, arw",
                scan: Handler::RustFeh,
                view: if self.magick_available {
                    Handler::ImageMagick
                } else {
                    Handler::Feh
                },
                resize: Handler::ImageMagick,
                view_speed: SpeedTier::Slow,
                note: "Magick-detected when identify succeeds; feh may use dcraw/magick",
            },
        ]
    }

    pub fn has_missing_required(&self) -> bool {
        self.dependencies()
            .iter()
            .any(|d| d.kind == DepKind::Required && !d.installed)
    }
}

/// Spawn error looks like a missing feh executable (NotFound or platform message).
pub fn is_feh_not_found(err: &std::io::Error) -> bool {
    use std::io::ErrorKind;
    err.kind() == ErrorKind::NotFound || err.to_string().contains("No such file")
}

/// Re-lookup guard: only mark unavailable when feh is absent from PATH.
pub fn feh_confirmed_missing() -> bool {
    which::which("feh").is_err()
}

/// Combined check for FR-008 spawn-failure sync (research R3).
pub fn feh_spawn_unavailable(err: &std::io::Error) -> bool {
    is_feh_not_found(err) && feh_confirmed_missing()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_returns_feh_and_magick_fields() {
        let caps = ToolCapabilities::detect();
        let _ = (caps.feh_available, caps.magick_available);
    }

    #[test]
    fn dependencies_include_feh_and_imagemagick() {
        let caps = ToolCapabilities::detect();
        let deps = caps.dependencies();
        assert_eq!(deps.len(), 2);
        assert_eq!(deps[0].name, "feh");
        assert_eq!(deps[1].name, "ImageMagick");
        assert_eq!(deps[1].install_cmd, "sudo apt install imagemagick");
    }

    #[test]
    fn operation_timings_include_browse_and_view() {
        let caps = ToolCapabilities::detect();
        let ops = caps.operation_timings();
        assert!(ops.iter().any(|o| o.operation.contains("Browse")));
        assert!(ops.iter().any(|o| o.operation.contains("Open")));
    }

    #[test]
    fn format_routes_cover_common_and_exotic() {
        let caps = ToolCapabilities::detect();
        let routes = caps.format_routes();
        assert!(routes[0].extensions.contains("jpg"));
        assert!(routes.iter().any(|r| r.extensions.contains("svg")));
    }

    #[test]
    fn speed_tier_labels_are_stable() {
        assert_eq!(SpeedTier::Instant.label(), "Instant");
        assert_eq!(SpeedTier::Fast.label(), "Fast");
    }

    #[test]
    fn operation_timings_reflect_missing_feh() {
        let caps = ToolCapabilities {
            feh_available: false,
            magick_available: false,
            magick_binary: None,
        };
        let ops = caps.operation_timings();
        let view = ops
            .iter()
            .find(|o| o.operation.contains("Open"))
            .expect("view row");
        assert!(view.note.contains("feh missing"));
    }

    #[test]
    fn format_routes_reflect_missing_magick() {
        let caps = ToolCapabilities {
            feh_available: true,
            magick_available: false,
            magick_binary: None,
        };
        let heic = caps
            .format_routes()
            .into_iter()
            .find(|r| r.extensions.contains("heic"))
            .expect("heic group");
        assert!(heic.note.contains("without ImageMagick"));
    }

    #[test]
    fn format_routes_reflect_magick_present() {
        let caps = ToolCapabilities {
            feh_available: true,
            magick_available: true,
            magick_binary: Some("magick".to_string()),
        };
        let heic = caps
            .format_routes()
            .into_iter()
            .find(|r| r.extensions.contains("heic"))
            .expect("heic group");
        assert!(heic.note.contains("Magick-detected"));
    }

    #[test]
    fn is_feh_not_found_detects_not_found_kind() {
        use std::io::{Error, ErrorKind};
        let err = Error::new(ErrorKind::NotFound, "feh");
        assert!(super::is_feh_not_found(&err));
    }

    #[test]
    fn is_feh_not_found_detects_no_such_file_message() {
        use std::io::{Error, ErrorKind};
        let err = Error::new(ErrorKind::Other, "No such file or directory");
        assert!(super::is_feh_not_found(&err));
    }

    #[test]
    fn feh_spawn_unavailable_requires_path_confirm() {
        use std::io::{Error, ErrorKind};
        let err = Error::new(ErrorKind::NotFound, "feh");
        let missing = super::feh_spawn_unavailable(&err);
        assert_eq!(missing, super::feh_confirmed_missing());
    }

    #[test]
    fn detect_snapshot_matches_path_lookup() {
        let caps = ToolCapabilities::detect();
        assert_eq!(caps.feh_available, which::which("feh").is_ok());
        let magick_on_path =
            which::which("magick").is_ok() || which::which("convert").is_ok();
        assert_eq!(caps.magick_available, magick_on_path);
    }

    #[test]
    fn dependencies_mark_feh_required() {
        let caps = ToolCapabilities {
            feh_available: false,
            magick_available: false,
            magick_binary: None,
        };
        let deps = caps.dependencies();
        assert_eq!(deps[0].kind, DepKind::Required);
        assert!(!deps[0].installed);
        assert_eq!(deps[1].kind, DepKind::Optional);
        assert!(caps.has_missing_required());
    }
}