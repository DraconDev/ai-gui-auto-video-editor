#![allow(clippy::too_many_arguments)]
#![allow(clippy::should_implement_trait)]

pub mod gui;
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
pub mod watermark;

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
