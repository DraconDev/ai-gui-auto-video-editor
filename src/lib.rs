pub mod analyzer;
pub mod batch_processor;
pub mod config;
pub mod editor;
pub mod exporter;
pub mod hwaccel;
pub mod ml;
pub mod preset_rules;
pub mod preview;
pub mod progress;
pub mod scene_detection;
pub mod stt_analyzer;
pub mod thumbnail;
pub mod utils;
pub mod watch;
pub mod watermark;

// gui module conditionally compiled to avoid circular dependency during crate compilation
#[cfg(feature = "gui")]
pub mod gui;

pub use analyzer::FfmpegAnalyzer;
pub use batch_processor::{
    FfmpegDurationGetter, ProcessingProgress, process_batch_dir, process_batch_dir_parallel,
    process_single_file, process_single_file_with_intro_outro,
    process_single_file_with_intro_outro_progress,
};
pub use config::{
    Config, FolderSettings, JoinMode, Preset, ProcessingConfig, SilenceMode, VideoResolution,
    WatchFolder,
};
pub use editor::FfmpegEditor;
pub use hwaccel::HwAccel;
pub use ml::{AutoReframeProcessor, FaceDetector, FrameExtractor, PersonSegmenter};
pub use preview::{generate_preview, preview_path};
pub use watch::{ConfigWatcher, WatchFolderConfig, run_watch_loop};

/// Shared test helpers (available to both unit and integration tests).
pub mod tests_common;
