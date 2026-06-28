// SPDX-License-Identifier: MIT

pub mod image_proc;
pub mod scanner;
pub mod tool_caps;
pub mod types;
pub mod ui_logic;

pub use image_proc::{BatchSummary, ImageToolsService};
pub use types::{
    CacheConfig, FehLaunchEntry, FehLaunchList, ImageOperation, OutputPolicy, ProcessedResult,
    WindowPreferences, WindowSizePreset,
};
pub use ui_logic::{
    build_entry_filelist, copy_image_to_clipboard, decode_image_to_rgba, entry_is_launchable,
    launch_list_path, load_launch_list, load_window_prefs, save_launch_list, save_window_prefs,
    window_prefs_path,
};
