use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// How to handle detected silences
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum SilenceMode {
    /// Cut out silences completely (default)
    #[default]
    Cut,
    /// Speed up silences instead of cutting
    Speedup,
}

/// Preset profiles for common use cases
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Preset {
    /// YouTube long-form: silence cut + audio enhance + chapters
    Youtube,
    /// YouTube Shorts/TikTok: speedup mode + audio enhance
    Shorts,
    /// TikTok-specific: vertical 9:16, fast cuts, trending style
    Tiktok,
    /// Instagram Reels: vertical 9:16, 90s max, engaging style
    Reels,
    /// Podcast: silence cut + audio enhance + SRT subtitles
    Podcast,
    /// Twitter/X: 2:20 max, landscape 16:9
    Twitter,
    /// Minimal: just silence detection, no enhancement
    Minimal,
}

impl Preset {
    /// Apply preset to create a config
    pub fn to_config(&self) -> Config {
        let mut config = Config::default();

        match self {
            Preset::Youtube => {
                // Long-form YouTube: cut silences, enhance audio, generate chapters
                config.silence.mode = SilenceMode::Cut;
                config.silence.padding = 0.15; // Slightly more padding for natural flow
                config.audio.enhance = true;
                config.export.chapters = true;
                config.export.fcpxml = true;
                config.video.target_resolution = VideoResolution::Fhd1080p;
            }
            Preset::Shorts => {
                // Short-form: speedup silences, enhance audio, extract clips
                config.silence.mode = SilenceMode::Speedup;
                config.silence.speedup_factor = 3.0;
                config.silence.padding = 0.05; // Tighter for fast-paced content
                config.audio.enhance = true;
                config.export.clips = true;
                config.video.reframe = true; // Auto vertical
                config.video.target_resolution = VideoResolution::Vertical1080p;
            }
            Preset::Tiktok => {
                // TikTok: 9:16 vertical, fast cuts, max 3min, trending style
                config.silence.mode = SilenceMode::Speedup;
                config.silence.speedup_factor = 4.0;
                config.silence.padding = 0.03;
                config.audio.enhance = true;
                config.audio.target_lufs = -12.0; // Louder for mobile
                config.video.reframe = true;
                config.video.target_resolution = VideoResolution::Vertical1080p;
                config.export.captions = true; // Burn captions for accessibility
            }
            Preset::Reels => {
                // Instagram Reels: 9:16, 90s max, engaging
                config.silence.mode = SilenceMode::Speedup;
                config.silence.speedup_factor = 3.5;
                config.silence.padding = 0.05;
                config.audio.enhance = true;
                config.video.reframe = true;
                config.video.target_resolution = VideoResolution::Vertical1080p;
                config.export.clips = true;
                config.export.clip_max_duration = 90.0;
            }
            Preset::Podcast => {
                // Podcast: cut silences, enhance audio, generate subtitles
                config.silence.mode = SilenceMode::Cut;
                config.silence.padding = 0.2; // More padding for conversational flow
                config.audio.enhance = true;
                config.audio.target_lufs = -16.0; // Podcast standard
                config.export.subtitles = true;
                config.export.captions = true;
            }
            Preset::Twitter => {
                // Twitter/X: 2:20 max, landscape 16:9
                config.silence.mode = SilenceMode::Cut;
                config.silence.padding = 0.1;
                config.audio.enhance = true;
                config.video.target_resolution = VideoResolution::Fhd1080p;
                config.export.clips = true;
                config.export.clip_max_duration = 140.0; // 2:20
            }
            Preset::Minimal => {
                // Just silence detection, nothing else
                config.silence.mode = SilenceMode::Cut;
                config.audio.enhance = false;
            }
        }

        config
    }

    /// Get preset name as string
    pub fn as_str(&self) -> &'static str {
        match self {
            Preset::Youtube => "youtube",
            Preset::Shorts => "shorts",
            Preset::Tiktok => "tiktok",
            Preset::Reels => "reels",
            Preset::Podcast => "podcast",
            Preset::Twitter => "twitter",
            Preset::Minimal => "minimal",
        }
    }

    /// Parse preset from string
    pub fn parse_name(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "youtube" => Some(Preset::Youtube),
            "shorts" | "ytshorts" => Some(Preset::Shorts),
            "tiktok" => Some(Preset::Tiktok),
            "reels" | "instagram" => Some(Preset::Reels),
            "podcast" => Some(Preset::Podcast),
            "twitter" | "x" => Some(Preset::Twitter),
            "minimal" => Some(Preset::Minimal),
            _ => None,
        }
    }
}

/// Configuration for silence detection and handling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SilenceConfig {
    /// Silence detection threshold in dB (e.g., -30.0)
    #[serde(default = "default_threshold_db")]
    pub threshold_db: f32,

    /// Minimum silence duration to detect (seconds)
    #[serde(default = "default_min_duration")]
    pub min_duration: f32,

    /// Padding around cuts (seconds)
    #[serde(default = "default_padding")]
    pub padding: f32,

    /// How to handle silences: "cut" or "speedup"
    #[serde(default)]
    pub mode: SilenceMode,

    /// Speed multiplier when mode = "speedup"
    #[serde(default = "default_speedup_factor")]
    pub speedup_factor: f32,

    /// Only speedup silences longer than this (seconds)
    #[serde(default = "default_min_silence_for_speedup")]
    pub min_silence_for_speedup: f32,

    /// Enable scene-change detection to augment silence-based cuts
    #[serde(default)]
    pub scene_detect: bool,

    /// Scene detection threshold (0.0-1.0, higher = fewer scenes)
    #[serde(default = "default_scene_threshold")]
    pub scene_threshold: f32,
}

fn default_threshold_db() -> f32 {
    -30.0
}
fn default_min_duration() -> f32 {
    0.5
}
fn default_padding() -> f32 {
    0.1
}
fn default_speedup_factor() -> f32 {
    4.0
}
fn default_min_silence_for_speedup() -> f32 {
    0.5
}
fn default_scene_threshold() -> f32 {
    0.3
}
fn default_watermark_position() -> String {
    "bottom-right".to_string()
}
fn default_watermark_scale() -> f32 {
    1.0
}

impl Default for SilenceConfig {
    fn default() -> Self {
        Self {
            threshold_db: default_threshold_db(),
            min_duration: default_min_duration(),
            padding: default_padding(),
            mode: SilenceMode::Cut,
            speedup_factor: default_speedup_factor(),
            min_silence_for_speedup: default_min_silence_for_speedup(),
            scene_detect: false,
            scene_threshold: default_scene_threshold(),
        }
    }
}

/// Configuration for filler word removal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FillerWordsConfig {
    /// Enable filler word removal (requires STT)
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Words to remove
    #[serde(default = "default_filler_words")]
    pub words: Vec<String>,

    /// Padding around filler cuts (seconds)
    #[serde(default = "default_filler_padding")]
    pub padding: f32,
}

fn default_true() -> bool {
    true
}
fn default_filler_words() -> Vec<String> {
    vec!["um".into(), "uh".into(), "ah".into(), "er".into()]
}
fn default_filler_padding() -> f32 {
    0.05
}

impl Default for FillerWordsConfig {
    fn default() -> Self {
        Self {
            enabled: default_true(),
            words: default_filler_words(),
            padding: default_filler_padding(),
        }
    }
}

/// Configuration for audio processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfig {
    /// Enable audio enhancement
    #[serde(default = "default_true")]
    pub enhance: bool,

    /// Enable noise reduction
    #[serde(default)]
    pub noise_reduction: bool,

    /// Target loudness (LUFS) - YouTube standard is -14
    #[serde(default = "default_target_lufs")]
    pub target_lufs: f32,

    /// Path to background music file
    #[serde(default)]
    pub music_file: Option<PathBuf>,

    /// Volume reduction during speech (0.0-1.0)
    #[serde(default = "default_duck_volume")]
    pub duck_volume: f32,
}

fn default_target_lufs() -> f32 {
    -14.0
}
fn default_duck_volume() -> f32 {
    0.2
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            enhance: default_true(),
            noise_reduction: false,
            target_lufs: default_target_lufs(),
            music_file: None,
            duck_volume: default_duck_volume(),
        }
    }
}

/// Configuration for export options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportConfig {
    /// Generate SRT subtitles (raw text with timestamps)
    #[serde(default)]
    pub subtitles: bool,

    /// Burn styled subtitles into the video
    #[serde(default)]
    pub captions: bool,

    /// Generate YouTube chapters
    #[serde(default)]
    pub chapters: bool,

    /// Extract highlight clips for Shorts/Reels
    #[serde(default)]
    pub clips: bool,

    /// Number of clips to extract
    #[serde(default = "default_clip_count")]
    pub clip_count: u32,

    /// Minimum clip duration in seconds
    #[serde(default = "default_clip_min_duration")]
    pub clip_min_duration: f32,

    /// Maximum clip duration in seconds
    #[serde(default = "default_clip_max_duration")]
    pub clip_max_duration: f32,

    /// Generate FCPXML for DaVinci/Premiere
    #[serde(default)]
    pub fcpxml: bool,

    /// Generate EDL
    #[serde(default)]
    pub edl: bool,

    /// Generate thumbnail image for the video
    #[serde(default)]
    pub thumbnail: bool,

    /// Thumbnail width in pixels
    #[serde(default = "default_thumbnail_width")]
    pub thumbnail_width: u32,

    /// Thumbnail height in pixels
    #[serde(default = "default_thumbnail_height")]
    pub thumbnail_height: u32,

    /// Generate multiple format outputs simultaneously
    #[serde(default)]
    pub multi_format: bool,

    /// Additional resolutions to output (when multi_format is true)
    #[serde(default)]
    pub extra_resolutions: Vec<VideoResolution>,

    /// Generate a quick low-resolution preview file alongside the main output
    #[serde(default)]
    pub preview: bool,

    /// Preview duration in seconds
    #[serde(default = "default_preview_duration")]
    pub preview_duration: f32,
}

impl Default for ExportConfig {
    fn default() -> Self {
        Self {
            subtitles: false,
            captions: false,
            chapters: false,
            clips: false,
            clip_count: default_clip_count(),
            clip_min_duration: default_clip_min_duration(),
            clip_max_duration: default_clip_max_duration(),
            fcpxml: false,
            edl: false,
            thumbnail: false,
            thumbnail_width: default_thumbnail_width(),
            thumbnail_height: default_thumbnail_height(),
            multi_format: false,
            extra_resolutions: Vec::new(),
            preview: false,
            preview_duration: default_preview_duration(),
        }
    }
}

fn default_thumbnail_width() -> u32 {
    1280
}
fn default_thumbnail_height() -> u32 {
    720
}
fn default_preview_duration() -> f32 {
    30.0
}

fn default_clip_count() -> u32 {
    3
}
fn default_clip_min_duration() -> f32 {
    15.0
}
fn default_clip_max_duration() -> f32 {
    60.0
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FolderSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enhance_audio: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remove_silence: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub silence_threshold_db: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_lufs: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stabilize: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color_correct: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reframe: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blur_background: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub noise_reduction: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preview: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scene_detect: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub multi_format: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hw_accel: Option<crate::hwaccel::HwAccel>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_resolution: Option<VideoResolution>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtitles: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chapters: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub captions: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clips: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchFolder {
    pub input: PathBuf,
    pub output: PathBuf,
    #[serde(default = "default_preset")]
    pub preset: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default, skip_serializing_if = "FolderSettings::is_default")]
    pub settings: FolderSettings,
}

impl FolderSettings {
    fn is_default(&self) -> bool {
        self.enhance_audio.is_none()
            && self.remove_silence.is_none()
            && self.silence_threshold_db.is_none()
            && self.target_lufs.is_none()
            && self.stabilize.is_none()
            && self.color_correct.is_none()
            && self.reframe.is_none()
            && self.blur_background.is_none()
            && self.noise_reduction.is_none()
            && self.preview.is_none()
            && self.scene_detect.is_none()
            && self.multi_format.is_none()
            && self.hw_accel.is_none()
            && self.target_resolution.is_none()
            && self.subtitles.is_none()
            && self.chapters.is_none()
            && self.captions.is_none()
            && self.clips.is_none()
    }
}

fn default_preset() -> String {
    "youtube".to_string()
}

impl Default for WatchFolder {
    fn default() -> Self {
        Self {
            input: PathBuf::from("videos"),
            output: PathBuf::from("videos/output"),
            preset: default_preset(),
            enabled: false, // Default to disabled so watch mode doesn't start unexpectedly
            settings: FolderSettings::default(),
        }
    }
}

/// Configuration for paths
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathsConfig {
    /// Input video file (single file mode)
    #[serde(default)]
    pub input: Option<PathBuf>,

    /// Input directory (batch mode)
    #[serde(default)]
    pub input_dir: Option<PathBuf>,

    /// Output video file (single file mode)
    #[serde(default)]
    pub output: Option<PathBuf>,

    /// Output directory (batch mode)
    #[serde(default)]
    pub output_dir: Option<PathBuf>,

    /// Background music file
    #[serde(default)]
    pub music: Option<PathBuf>,

    /// Background music directory
    #[serde(default)]
    pub music_dir: Option<PathBuf>,

    /// Intro video
    #[serde(default)]
    pub intro: Option<PathBuf>,

    /// Outro video
    #[serde(default)]
    pub outro: Option<PathBuf>,

    /// Watch folders for GUI mode
    #[serde(default = "default_watch_folders")]
    pub watch_folders: Vec<WatchFolder>,
}

fn default_watch_folders() -> Vec<WatchFolder> {
    vec![WatchFolder::default()]
}

impl Default for PathsConfig {
    fn default() -> Self {
        Self {
            input: None,
            input_dir: Some(PathBuf::from("watch")),
            output: None,
            output_dir: Some(PathBuf::from("output")),
            music: None,
            music_dir: Some(PathBuf::from("music")),
            intro: None,
            outro: None,
            watch_folders: default_watch_folders(),
        }
    }
}

/// Configuration for watch mode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchConfig {
    /// Enable watch mode
    #[serde(default)]
    pub enabled: bool,

    /// Polling interval in seconds
    #[serde(default = "default_watch_interval")]
    pub interval: u64,
}

fn default_watch_interval() -> u64 {
    5
}

impl Default for WatchConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            interval: default_watch_interval(),
        }
    }
}

/// Target video resolution for output
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum VideoResolution {
    /// 720p HD (1280x720)
    Hd720p,
    /// 1080p Full HD (1920x1080)
    #[default]
    Fhd1080p,
    /// 1440p QHD (2560x1440)
    Qhd1440p,
    /// 4K UHD (3840x2160)
    Uhd4k,
    /// Vertical 1080p (1080x1920) for Shorts/Reels/TikTok
    Vertical1080p,
    /// Vertical 720p (720x1280)
    Vertical720p,
}

impl VideoResolution {
    /// Get resolution as (width, height)
    pub fn dimensions(&self) -> (u32, u32) {
        match self {
            VideoResolution::Hd720p => (1280, 720),
            VideoResolution::Fhd1080p => (1920, 1080),
            VideoResolution::Qhd1440p => (2560, 1440),
            VideoResolution::Uhd4k => (3840, 2160),
            VideoResolution::Vertical1080p => (1080, 1920),
            VideoResolution::Vertical720p => (720, 1280),
        }
    }

    /// Get as ffmpeg scale string
    pub fn to_ffmpeg_scale(&self) -> String {
        let (w, h) = self.dimensions();
        format!("{}:{}", w, h)
    }

    /// Human-readable label for UI dropdowns.
    pub fn display_name(&self) -> &'static str {
        match self {
            VideoResolution::Hd720p => "720p HD",
            VideoResolution::Fhd1080p => "1080p Full HD",
            VideoResolution::Qhd1440p => "1440p QHD",
            VideoResolution::Uhd4k => "4K UHD",
            VideoResolution::Vertical1080p => "1080p Vertical",
            VideoResolution::Vertical720p => "720p Vertical",
        }
    }
}

/// Configuration for video processing
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VideoConfig {
    /// Enable video stabilization (vidstab filter)
    #[serde(default)]
    pub stabilize: bool,

    /// Enable auto color correction
    #[serde(default)]
    pub color_correct: bool,

    /// Enable auto-reframe (horizontal to vertical, follows face)
    #[serde(default)]
    pub reframe: bool,

    /// Enable background blur (person segmentation)
    #[serde(default)]
    pub blur_background: bool,

    /// Target output resolution
    #[serde(default)]
    pub target_resolution: VideoResolution,

    /// Hardware acceleration for encoding
    #[serde(default)]
    pub hw_accel: crate::hwaccel::HwAccel,

    /// Path to watermark image (PNG with alpha recommended)
    #[serde(default)]
    pub watermark: Option<PathBuf>,

    /// Watermark position
    #[serde(default = "default_watermark_position")]
    pub watermark_position: String,

    /// Watermark scale factor
    #[serde(default = "default_watermark_scale")]
    pub watermark_scale: f32,
}

/// Join mode for combining processed videos
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum JoinMode {
    /// Don't join videos
    #[default]
    Off,
    /// Join videos by date (newest first)
    ByDate,
    /// Join videos alphabetically by name
    ByName,
    /// Join after N videos processed
    AfterCount,
}

/// Configuration for processing options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingConfig {
    /// How to join processed videos
    #[serde(default)]
    pub join_mode: JoinMode,

    /// Number of videos after which to join (when join_mode = AfterCount)
    #[serde(default = "default_join_after_count")]
    pub join_after_count: u32,

    /// Output filename pattern for joined videos
    /// Supports: {date}, {time}, {count}
    #[serde(default = "default_join_pattern")]
    pub join_output_pattern: String,
}

fn default_join_after_count() -> u32 {
    5
}
fn default_join_pattern() -> String {
    "joined_{date}.mp4".to_string()
}

impl Default for ProcessingConfig {
    fn default() -> Self {
        Self {
            join_mode: JoinMode::Off,
            join_after_count: default_join_after_count(),
            join_output_pattern: default_join_pattern(),
        }
    }
}

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    /// Paths for input/output/music/intro/outro
    #[serde(default)]
    pub paths: PathsConfig,

    /// Silence detection and handling
    #[serde(default)]
    pub silence: SilenceConfig,

    /// Filler word removal
    #[serde(default)]
    pub filler_words: FillerWordsConfig,

    /// Audio processing
    #[serde(default)]
    pub audio: AudioConfig,

    /// Video processing
    #[serde(default)]
    pub video: VideoConfig,

    /// Processing options (join mode, etc.)
    #[serde(default)]
    pub processing: ProcessingConfig,

    /// Export options
    #[serde(default)]
    pub export: ExportConfig,

    /// Watch mode settings
    #[serde(default)]
    pub watch: WatchConfig,
}

impl Config {
    /// Load configuration from a file
    pub fn from_file(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {:?}", path))?;

        let config: Config = toml::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {:?}", path))?;

        config.validate()?;

        Ok(config)
    }

    /// Save configuration to a file
    pub fn to_file(&self, path: &Path) -> Result<()> {
        let content = toml::to_string_pretty(self).context("Failed to serialize config")?;

        fs::write(path, content)
            .with_context(|| format!("Failed to write config file: {:?}", path))?;

        Ok(())
    }

    /// Get the default config file path in user's config directory
    pub fn default_config_path() -> Option<PathBuf> {
        directories::ProjectDirs::from("com", "ai-vid-editor", "ai-vid-editor")
            .map(|dirs| dirs.config_dir().join("config.toml"))
    }

    /// Get the project-local config path
    pub fn project_config_path() -> PathBuf {
        PathBuf::from("ai-vid-editor.toml")
    }

    /// Load configuration with precedence: CLI > project > global > defaults
    pub fn load_with_precedence(
        cli_config_path: Option<&Path>,
        cli_threshold: Option<f32>,
        cli_duration: Option<f32>,
        cli_padding: Option<f32>,
        cli_speedup: bool,
    ) -> Result<Self> {
        let mut config = Config::default();

        // Try to load global config first
        if let Some(global_path) = Self::default_config_path()
            && global_path.exists()
        {
            config = Self::from_file(&global_path)?;
        }

        // Then try project config (overrides global)
        let project_path = Self::project_config_path();
        if project_path.exists() {
            let project_config = Self::from_file(&project_path)?;
            config = config.merge(project_config);
        }

        // Then try explicitly specified config (overrides project)
        if let Some(path) = cli_config_path
            && path.exists()
        {
            let file_config = Self::from_file(path)?;
            config = config.merge(file_config);
        }

        // Finally, apply CLI overrides (highest precedence)
        if let Some(threshold) = cli_threshold {
            config.silence.threshold_db = threshold;
        }
        if let Some(duration) = cli_duration {
            config.silence.min_duration = duration;
        }
        if let Some(padding) = cli_padding {
            config.silence.padding = padding;
        }
        if cli_speedup {
            config.silence.mode = SilenceMode::Speedup;
        }

        config.validate()?;

        Ok(config)
    }

    /// Merge another config into this one (other takes precedence)
    #[must_use]
    pub fn merge(mut self, other: Self) -> Self {
        // Silence config - always take other's value for mode enum
        if other.silence.threshold_db != default_threshold_db() {
            self.silence.threshold_db = other.silence.threshold_db;
        }
        if other.silence.min_duration != default_min_duration() {
            self.silence.min_duration = other.silence.min_duration;
        }
        if other.silence.padding != default_padding() {
            self.silence.padding = other.silence.padding;
        }
        // Always take other's mode (allows switching between Cut and Speedup in both directions)
        self.silence.mode = other.silence.mode;
        if other.silence.speedup_factor != default_speedup_factor() {
            self.silence.speedup_factor = other.silence.speedup_factor;
        }
        if other.silence.min_silence_for_speedup != default_min_silence_for_speedup() {
            self.silence.min_silence_for_speedup = other.silence.min_silence_for_speedup;
        }
        self.silence.scene_detect = other.silence.scene_detect;
        if other.silence.scene_threshold != default_scene_threshold() {
            self.silence.scene_threshold = other.silence.scene_threshold;
        }

        // Filler words config - always take other's boolean values
        self.filler_words.enabled = other.filler_words.enabled;
        if !other.filler_words.words.is_empty() {
            self.filler_words.words = other.filler_words.words;
        }
        if other.filler_words.padding != default_filler_padding() {
            self.filler_words.padding = other.filler_words.padding;
        }

        // Audio config - always take other's boolean values
        self.audio.enhance = other.audio.enhance;
        self.audio.noise_reduction = other.audio.noise_reduction;
        if other.audio.target_lufs != default_target_lufs() {
            self.audio.target_lufs = other.audio.target_lufs;
        }
        if other.audio.music_file.is_some() {
            self.audio.music_file = other.audio.music_file;
        }
        if other.audio.duck_volume != default_duck_volume() {
            self.audio.duck_volume = other.audio.duck_volume;
        }

        // Export config - always take other's boolean values
        self.export.subtitles = other.export.subtitles;
        self.export.captions = other.export.captions;
        self.export.chapters = other.export.chapters;
        self.export.clips = other.export.clips;
        self.export.fcpxml = other.export.fcpxml;
        self.export.edl = other.export.edl;
        self.export.thumbnail = other.export.thumbnail;
        self.export.multi_format = other.export.multi_format;
        self.export.preview = other.export.preview;
        self.export.preview_duration = other.export.preview_duration;
        if other.export.thumbnail_width != default_thumbnail_width() {
            self.export.thumbnail_width = other.export.thumbnail_width;
        }
        if other.export.thumbnail_height != default_thumbnail_height() {
            self.export.thumbnail_height = other.export.thumbnail_height;
        }
        if !other.export.extra_resolutions.is_empty() {
            self.export.extra_resolutions = other.export.extra_resolutions.clone();
        }
        if other.export.clip_count != default_clip_count() {
            self.export.clip_count = other.export.clip_count;
        }
        if other.export.clip_min_duration != default_clip_min_duration() {
            self.export.clip_min_duration = other.export.clip_min_duration;
        }
        if other.export.clip_max_duration != default_clip_max_duration() {
            self.export.clip_max_duration = other.export.clip_max_duration;
        }

        // Paths config
        if other.paths.input.is_some() {
            self.paths.input = other.paths.input;
        }
        if other.paths.input_dir.is_some() {
            self.paths.input_dir = other.paths.input_dir;
        }
        if other.paths.output.is_some() {
            self.paths.output = other.paths.output;
        }
        if other.paths.output_dir.is_some() {
            self.paths.output_dir = other.paths.output_dir;
        }
        if other.paths.music.is_some() {
            self.paths.music = other.paths.music;
        }
        if other.paths.music_dir.is_some() {
            self.paths.music_dir = other.paths.music_dir;
        }
        if other.paths.intro.is_some() {
            self.paths.intro = other.paths.intro;
        }
        if other.paths.outro.is_some() {
            self.paths.outro = other.paths.outro;
        }
        if !other.paths.watch_folders.is_empty() {
            self.paths.watch_folders = other.paths.watch_folders;
        }

        // Watch config - always take other's boolean values
        self.watch.enabled = other.watch.enabled;
        if other.watch.interval != default_watch_interval() {
            self.watch.interval = other.watch.interval;
        }

        // Video config - always take other's boolean values
        self.video.stabilize = other.video.stabilize;
        self.video.color_correct = other.video.color_correct;
        self.video.reframe = other.video.reframe;
        self.video.blur_background = other.video.blur_background;
        self.video.target_resolution = other.video.target_resolution;
        if other.video.hw_accel != crate::hwaccel::HwAccel::None {
            self.video.hw_accel = other.video.hw_accel;
        }
        if other.video.watermark.is_some() {
            self.video.watermark = other.video.watermark.clone();
        }
        if other.video.watermark_scale != default_watermark_scale() {
            self.video.watermark_scale = other.video.watermark_scale;
        }
        if other.video.watermark_position != default_watermark_position() {
            self.video.watermark_position = other.video.watermark_position.clone();
        }

        // Processing config
        self.processing.join_mode = other.processing.join_mode;
        if other.processing.join_after_count != default_join_after_count() {
            self.processing.join_after_count = other.processing.join_after_count;
        }
        if other.processing.join_output_pattern != default_join_pattern() {
            self.processing.join_output_pattern = other.processing.join_output_pattern;
        }

        self
    }

    /// Validate config values are within sensible bounds.
    /// Returns Ok(()) if valid, or an error describing the first invalid value.
    pub fn validate(&self) -> Result<()> {
        if self.silence.threshold_db > 0.0 {
            anyhow::bail!(
                "silence.threshold_db must be negative (got {})",
                self.silence.threshold_db
            );
        }
        if self.silence.min_duration < 0.0 {
            anyhow::bail!(
                "silence.min_duration must be non-negative (got {})",
                self.silence.min_duration
            );
        }
        if self.silence.padding < 0.0 {
            anyhow::bail!(
                "silence.padding must be non-negative (got {})",
                self.silence.padding
            );
        }
        if self.silence.speedup_factor <= 0.0 {
            anyhow::bail!(
                "silence.speedup_factor must be positive (got {})",
                self.silence.speedup_factor
            );
        }
        if self.silence.min_silence_for_speedup < 0.0 {
            anyhow::bail!(
                "silence.min_silence_for_speedup must be non-negative (got {})",
                self.silence.min_silence_for_speedup
            );
        }
        if self.audio.duck_volume < 0.0 || self.audio.duck_volume > 1.0 {
            anyhow::bail!(
                "audio.duck_volume must be between 0.0 and 1.0 (got {})",
                self.audio.duck_volume
            );
        }
        if self.export.clip_min_duration < 0.0 {
            anyhow::bail!(
                "export.clip_min_duration must be non-negative (got {})",
                self.export.clip_min_duration
            );
        }
        if self.export.clip_max_duration < 0.0 {
            anyhow::bail!(
                "export.clip_max_duration must be non-negative (got {})",
                self.export.clip_max_duration
            );
        }
        if self.export.clip_min_duration > self.export.clip_max_duration {
            anyhow::bail!(
                "export.clip_min_duration ({}) must be <= clip_max_duration ({})",
                self.export.clip_min_duration,
                self.export.clip_max_duration
            );
        }
        if self.watch.interval == 0 {
            anyhow::bail!("watch.interval must be > 0 (got 0)");
        }

        // Warn about incompatible feature combinations
        if self.video.reframe && self.video.blur_background {
            tracing::warn!(
                "Both reframe and blur_background are enabled. Blur background will be applied after reframing."
            );
        }

        match self.video.target_resolution {
            VideoResolution::Vertical1080p | VideoResolution::Vertical720p => {
                if !self.video.reframe {
                    tracing::info!(
                        "Vertical resolution selected but reframe is not enabled. Output may have black bars."
                    );
                }
            }
            _ => {
                if self.video.reframe {
                    tracing::info!(
                        "Reframe is enabled but target resolution is landscape. Consider using a vertical resolution preset."
                    );
                }
            }
        }

        if self.export.captions && !self.export.subtitles {
            tracing::info!(
                "Captions export enabled without subtitles. Transcription will still be performed for captions."
            );
        }

        if self.export.multi_format && self.export.extra_resolutions.is_empty() {
            tracing::warn!(
                "Multi-format export enabled but no extra resolutions specified. Only the target resolution will be output."
            );
        }

        if self.audio.noise_reduction && !self.audio.enhance {
            tracing::info!(
                "Noise reduction enabled without audio enhancement. Consider enabling audio.enhance for better results."
            );
        }

        if self.silence.mode == SilenceMode::Speedup && self.silence.speedup_factor < 1.0 {
            tracing::warn!(
                "Speedup factor {} is less than 1.0. This will slow down silences instead of speeding them up.",
                self.silence.speedup_factor
            );
        }

        Ok(())
    }

    /// Generate a default config file content
    pub fn generate_default_toml() -> Result<String> {
        let config = Config::default();
        let toml = toml::to_string_pretty(&config).context("Failed to serialize default config")?;

        // Fix floating point precision artifacts (e.g., 0.10000000149011612 -> 0.1)
        // This happens because f32 values get serialized as f64
        // Only fix known float fields to avoid corrupting strings/paths
        const FLOAT_KEYS: &[&str] = &[
            "threshold_db",
            "min_duration",
            "padding",
            "speedup_factor",
            "min_silence_for_speedup",
            "target_lufs",
            "duck_volume",
            "clip_min_duration",
            "clip_max_duration",
        ];
        fn fix_floats(s: &str) -> String {
            let mut result = String::new();
            for line in s.lines() {
                if line.contains('=') && line.chars().any(|c| c == '.') {
                    let parts: Vec<&str> = line.splitn(2, '=').collect();
                    if parts.len() == 2 {
                        let key = parts[0].trim_end();
                        let value = parts[1].trim();
                        // Only round values for known float fields
                        let is_float_key = FLOAT_KEYS.iter().any(|&k| key.ends_with(k));
                        if is_float_key && let Ok(float_val) = value.parse::<f64>() {
                            let rounded = (float_val * 100.0).round() / 100.0;
                            if rounded == rounded.trunc() {
                                result.push_str(&format!("{} = {}\n", key, rounded as i64));
                            } else {
                                result.push_str(&format!("{} = {}\n", key, rounded));
                            }
                            continue;
                        }
                    }
                }
                result.push_str(line);
                result.push('\n');
            }
            result
        }

        Ok(fix_floats(&toml))
    }

    /// Load a preset from a TOML file in the presets directory
    pub fn from_preset_file(preset_name: &str) -> Result<Self> {
        let preset_path = PathBuf::from("presets").join(format!("{}.toml", preset_name));
        if preset_path.exists() {
            Self::from_file(&preset_path)
        } else {
            anyhow::bail!("Preset file not found: {:?}", preset_path)
        }
    }

    /// Get list of available preset names from presets directory
    pub fn available_presets() -> Vec<String> {
        let presets_dir = PathBuf::from("presets");
        if !presets_dir.exists() {
            return vec![
                "youtube".to_string(),
                "shorts".to_string(),
                "podcast".to_string(),
                "minimal".to_string(),
            ];
        }

        let mut presets = Vec::new();
        if let Ok(entries) = fs::read_dir(&presets_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map(|e| e == "toml").unwrap_or(false)
                    && let Some(stem) = path.file_stem()
                {
                    presets.push(stem.to_string_lossy().to_string());
                }
            }
        }
        presets.sort();

        if presets.is_empty() {
            vec![
                "youtube".to_string(),
                "shorts".to_string(),
                "podcast".to_string(),
                "minimal".to_string(),
            ]
        } else {
            presets
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.silence.threshold_db, -30.0);
        assert_eq!(config.silence.min_duration, 0.5);
        assert_eq!(config.silence.padding, 0.1);
        assert_eq!(config.silence.mode, SilenceMode::Cut);
        assert_eq!(config.silence.speedup_factor, 4.0);
    }

    #[test]
    fn test_config_serialize_deserialize() {
        let config = Config::default();
        let toml_str = toml::to_string_pretty(&config).unwrap();
        let parsed: Config = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.silence.threshold_db, config.silence.threshold_db);
    }

    #[test]
    fn test_config_from_file() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("test_config.toml");

        let content = r#"
[silence]
threshold_db = -35.0
mode = "speedup"
speedup_factor = 2.0

[audio]
enhance = false
"#;
        fs::write(&config_path, content).unwrap();

        let config = Config::from_file(&config_path).unwrap();
        assert_eq!(config.silence.threshold_db, -35.0);
        assert_eq!(config.silence.mode, SilenceMode::Speedup);
        assert_eq!(config.silence.speedup_factor, 2.0);
        assert!(!config.audio.enhance);
    }

    #[test]
    fn test_config_to_file() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("output_config.toml");

        let mut config = Config::default();
        config.silence.threshold_db = -40.0;
        config.silence.mode = SilenceMode::Speedup;

        config.to_file(&config_path).unwrap();

        let loaded = Config::from_file(&config_path).unwrap();
        assert_eq!(loaded.silence.threshold_db, -40.0);
        assert_eq!(loaded.silence.mode, SilenceMode::Speedup);
    }

    #[test]
    fn test_merge_configs() {
        let base = Config::default();

        let mut override_config = Config::default();
        override_config.silence.threshold_db = -40.0;
        override_config.silence.mode = SilenceMode::Speedup;
        override_config.export.subtitles = true;

        let merged = base.merge(override_config);
        assert_eq!(merged.silence.threshold_db, -40.0);
        assert_eq!(merged.silence.mode, SilenceMode::Speedup);
        assert!(merged.export.subtitles);
    }

    #[test]
    fn test_cli_overrides() {
        let config = Config::load_with_precedence(
            None,
            Some(-50.0), // cli_threshold
            Some(1.0),   // cli_duration
            Some(0.2),   // cli_padding
            true,        // cli_speedup
        )
        .unwrap();

        assert_eq!(config.silence.threshold_db, -50.0);
        assert_eq!(config.silence.min_duration, 1.0);
        assert_eq!(config.silence.padding, 0.2);
        assert_eq!(config.silence.mode, SilenceMode::Speedup);
    }

    #[test]
    fn test_silence_mode_serde() {
        // Test serialization through a config struct
        let mut config = Config::default();
        config.silence.mode = SilenceMode::Speedup;

        let serialized = toml::to_string_pretty(&config).unwrap();
        assert!(serialized.contains("mode = \"speedup\""));

        let deserialized: Config = toml::from_str(&serialized).unwrap();
        assert_eq!(deserialized.silence.mode, SilenceMode::Speedup);

        // Test cut mode
        config.silence.mode = SilenceMode::Cut;
        let serialized = toml::to_string_pretty(&config).unwrap();
        let deserialized: Config = toml::from_str(&serialized).unwrap();
        assert_eq!(deserialized.silence.mode, SilenceMode::Cut);
    }

    #[test]
    fn test_preset_youtube() {
        let config = Preset::Youtube.to_config();
        assert_eq!(config.silence.mode, SilenceMode::Cut);
        assert_eq!(config.silence.padding, 0.15);
        assert!(config.audio.enhance);
        assert!(config.export.chapters);
        assert!(config.export.fcpxml);
    }

    #[test]
    fn test_preset_shorts() {
        let config = Preset::Shorts.to_config();
        assert_eq!(config.silence.mode, SilenceMode::Speedup);
        assert_eq!(config.silence.speedup_factor, 3.0);
        assert_eq!(config.silence.padding, 0.05);
        assert!(config.audio.enhance);
        assert!(config.export.clips);
        assert!(!config.export.captions);
        assert!(!config.export.subtitles);
    }

    #[test]
    fn test_preset_podcast() {
        let config = Preset::Podcast.to_config();
        assert_eq!(config.silence.mode, SilenceMode::Cut);
        assert_eq!(config.silence.padding, 0.2);
        assert!(config.audio.enhance);
        assert_eq!(config.audio.target_lufs, -16.0);
        assert!(config.export.subtitles);
        assert!(config.export.captions);
        assert!(!config.export.clips);
    }

    #[test]
    fn test_preset_minimal() {
        let config = Preset::Minimal.to_config();
        assert_eq!(config.silence.mode, SilenceMode::Cut);
        assert!(!config.audio.enhance);
        assert!(!config.export.clips);
        assert!(!config.export.captions);
        assert!(!config.export.subtitles);
    }

    #[test]
    fn test_export_config_defaults() {
        let config = Config::default();
        assert!(!config.export.subtitles);
        assert!(!config.export.chapters);
        assert!(!config.export.captions);
        assert!(!config.export.clips);
        assert!(!config.export.fcpxml);
        assert!(!config.export.edl);
        assert_eq!(config.export.clip_count, 3);
        assert_eq!(config.export.clip_min_duration, 15.0);
        assert_eq!(config.export.clip_max_duration, 60.0);
    }

    #[test]
    fn test_validate_positive_threshold_fails() {
        let mut config = Config::default();
        config.silence.threshold_db = 1.0;
        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("threshold_db"));
    }

    #[test]
    fn test_validate_negative_speedup_fails() {
        let mut config = Config::default();
        config.silence.speedup_factor = 0.0;
        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("speedup_factor"));
    }

    #[test]
    fn test_validate_zero_speedup_fails() {
        let mut config = Config::default();
        config.silence.speedup_factor = -1.0;
        let result = config.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_duck_volume_negative_fails() {
        let mut config = Config::default();
        config.audio.duck_volume = -0.1;
        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("duck_volume"));
    }

    #[test]
    fn test_validate_duck_volume_over_one_fails() {
        let mut config = Config::default();
        config.audio.duck_volume = 1.5;
        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("duck_volume"));
    }

    #[test]
    fn test_validate_duck_volume_boundary_zero_ok() {
        let mut config = Config::default();
        config.audio.duck_volume = 0.0;
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_duck_volume_boundary_one_ok() {
        let mut config = Config::default();
        config.audio.duck_volume = 1.0;
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_clip_duration_order_fails() {
        let mut config = Config::default();
        config.export.clip_min_duration = 30.0;
        config.export.clip_max_duration = 10.0;
        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("clip_min_duration"));
    }

    #[test]
    fn test_validate_clip_duration_equal_ok() {
        let mut config = Config::default();
        config.export.clip_min_duration = 10.0;
        config.export.clip_max_duration = 10.0;
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_watch_interval_zero_fails() {
        let mut config = Config::default();
        config.watch.interval = 0;
        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("watch.interval"));
    }

    #[test]
    fn test_validate_watch_interval_one_ok() {
        let mut config = Config::default();
        config.watch.interval = 1;
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_min_duration_negative_fails() {
        let mut config = Config::default();
        config.silence.min_duration = -0.1;
        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("min_duration"));
    }

    #[test]
    fn test_validate_padding_negative_fails() {
        let mut config = Config::default();
        config.silence.padding = -0.1;
        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("padding"));
    }

    #[test]
    fn test_validate_min_silence_for_speedup_negative_fails() {
        let mut config = Config::default();
        config.silence.min_silence_for_speedup = -0.1;
        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("min_silence_for_speedup"));
    }

    #[test]
    fn test_validate_clip_min_duration_negative_fails() {
        let mut config = Config::default();
        config.export.clip_min_duration = -1.0;
        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("clip_min_duration"));
    }

    #[test]
    fn test_validate_clip_max_duration_negative_fails() {
        let mut config = Config::default();
        config.export.clip_max_duration = -1.0;
        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("clip_max_duration"));
    }

    #[test]
    fn test_preset_from_str() {
        assert_eq!(Preset::from_str("youtube"), Some(Preset::Youtube));
        assert_eq!(Preset::from_str("SHORTS"), Some(Preset::Shorts));
        assert_eq!(Preset::from_str("tiktok"), Some(Preset::Tiktok));
        assert_eq!(Preset::from_str("reels"), Some(Preset::Reels));
        assert_eq!(Preset::from_str("podcast"), Some(Preset::Podcast));
        assert_eq!(Preset::from_str("twitter"), Some(Preset::Twitter));
        assert_eq!(Preset::from_str("minimal"), Some(Preset::Minimal));
        assert_eq!(Preset::from_str("invalid"), None);
    }
}
