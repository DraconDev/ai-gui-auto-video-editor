mod common;

use ai_vid_editor::config::{Config, Preset, SilenceMode, VideoResolution};
use ai_vid_editor::gui::FolderState;
use ai_vid_editor::gui::processing::build_folder_config;
use ai_vid_editor::gui::processing::make_test_folder_state;

fn make_folder_state() -> FolderState {
    make_test_folder_state()
}

#[test]
fn test_build_folder_config_no_overrides() {
    let config = Config::default();
    let mut folder = make_folder_state();
    folder.preset = String::new();

    let result = build_folder_config(&config, &folder);
    assert_eq!(result.silence.threshold_db, config.silence.threshold_db);
    assert_eq!(result.audio.enhance, config.audio.enhance);
}

#[test]
fn test_build_folder_config_youtube_preset() {
    let config = Config::default();
    eprintln!(
        "default config.export.chapters = {}",
        config.export.chapters
    );

    // Check if Preset::parse_name works
    eprintln!(
        "Preset::parse_name(\"youtube\") = {:?}",
        Preset::parse_name("youtube")
    );
    eprintln!(
        "Preset::parse_name(\"Youtube\") = {:?}",
        Preset::parse_name("Youtube")
    );

    let preset = Preset::parse_name("youtube").unwrap();
    let preset_config = preset.to_config();
    eprintln!(
        "youtube preset export.chapters = {}",
        preset_config.export.chapters
    );

    let merged = preset_config.clone().merge(config.clone());
    eprintln!("direct merge.export.chapters = {}", merged.export.chapters);

    let mut folder = make_folder_state();
    folder.preset = "youtube".to_string();
    eprintln!("folder.preset = {:?}", folder.preset);

    let result = build_folder_config(&config, &folder);
    eprintln!(
        "build_folder_config result.export.chapters = {}",
        result.export.chapters
    );
    assert!(result.export.chapters, "chapters");
}

#[test]
fn test_build_folder_config_shorts_preset() {
    let config = Config::default();
    let mut folder = make_folder_state();
    folder.preset = "shorts".to_string();

    let result = build_folder_config(&config, &folder);
    assert_eq!(result.silence.mode, SilenceMode::Speedup);
    assert_eq!(result.silence.speedup_factor, 3.0);
    assert!(result.video.reframe);
    assert_eq!(
        result.video.target_resolution,
        VideoResolution::Vertical1080p
    );
    assert!(result.export.clips);
}

#[test]
fn test_build_folder_config_silence_mode_cut() {
    let config = Config::default();
    let mut folder = make_folder_state();
    folder.settings.silence_mode = Some(SilenceMode::Cut);

    let result = build_folder_config(&config, &folder);
    assert_eq!(result.silence.mode, SilenceMode::Cut);
    assert_eq!(result.silence.min_duration, config.silence.min_duration);
}

#[test]
fn test_build_folder_config_silence_mode_keep() {
    let config = Config::default();
    let mut folder = make_folder_state();
    folder.settings.silence_mode = Some(SilenceMode::Keep);

    let result = build_folder_config(&config, &folder);
    assert_eq!(result.silence.mode, SilenceMode::Keep);
    assert_eq!(result.silence.min_duration, config.silence.min_duration);
}

#[test]
fn test_build_folder_config_threshold_override() {
    let mut config = Config::default();
    config.silence.threshold_db = -30.0;

    let mut folder = make_folder_state();
    folder.settings.silence_threshold_db = Some(-45.0);

    let result = build_folder_config(&config, &folder);
    assert_eq!(result.silence.threshold_db, -45.0);
}

#[test]
fn test_build_folder_config_enhance_audio_override() {
    let mut folder = make_folder_state();
    folder.settings.enhance_audio = Some(false);

    let result = build_folder_config(&Config::default(), &folder);
    assert!(!result.audio.enhance);
}

#[test]
fn test_build_folder_config_target_lufs_override() {
    let mut folder = make_folder_state();
    folder.settings.target_lufs = Some(-14.0);

    let result = build_folder_config(&Config::default(), &folder);
    assert_eq!(result.audio.target_lufs, -14.0);
}

#[test]
fn test_build_folder_config_stabilize_override() {
    let mut folder = make_folder_state();
    folder.settings.stabilize = Some(true);

    let result = build_folder_config(&Config::default(), &folder);
    assert!(result.video.stabilize);
}

#[test]
fn test_build_folder_config_color_correct_override() {
    let mut folder = make_folder_state();
    folder.settings.color_correct = Some(true);

    let result = build_folder_config(&Config::default(), &folder);
    assert!(result.video.color_correct);
}

#[test]
fn test_build_folder_config_reframe_override() {
    let mut folder = make_folder_state();
    folder.settings.reframe = Some(true);

    let result = build_folder_config(&Config::default(), &folder);
    assert!(result.video.reframe);
}

#[test]
fn test_build_folder_config_blur_background_override() {
    let mut folder = make_folder_state();
    folder.settings.blur_background = Some(true);

    let result = build_folder_config(&Config::default(), &folder);
    assert!(result.video.blur_background);
}

#[test]
fn test_build_folder_config_noise_reduction_override() {
    let mut folder = make_folder_state();
    folder.settings.noise_reduction = Some(true);

    let result = build_folder_config(&Config::default(), &folder);
    assert!(result.audio.noise_reduction);
}

#[test]
fn test_build_folder_config_preview_override() {
    let mut folder = make_folder_state();
    folder.settings.preview = Some(true);

    let result = build_folder_config(&Config::default(), &folder);
    assert!(result.export.preview);
}

#[test]
fn test_build_folder_config_scene_detect_override() {
    let mut folder = make_folder_state();
    folder.settings.scene_detect = Some(true);

    let result = build_folder_config(&Config::default(), &folder);
    assert!(result.silence.scene_detect);
}

#[test]
fn test_build_folder_config_multi_format_override() {
    let mut folder = make_folder_state();
    folder.settings.multi_format = Some(true);

    let result = build_folder_config(&Config::default(), &folder);
    assert!(result.export.multi_format);
}

#[test]
fn test_build_folder_config_subtitles_override() {
    let mut folder = make_folder_state();
    folder.settings.subtitles = Some(true);

    let result = build_folder_config(&Config::default(), &folder);
    assert!(result.export.subtitles);
}

#[test]
fn test_build_folder_config_chapters_override() {
    let mut folder = make_folder_state();
    folder.settings.chapters = Some(true);

    let result = build_folder_config(&Config::default(), &folder);
    assert!(result.export.chapters);
}

#[test]
fn test_build_folder_config_captions_override() {
    let mut folder = make_folder_state();
    folder.settings.captions = Some(true);

    let result = build_folder_config(&Config::default(), &folder);
    assert!(result.export.captions);
}

#[test]
fn test_build_folder_config_clips_override() {
    let mut folder = make_folder_state();
    folder.settings.clips = Some(true);

    let result = build_folder_config(&Config::default(), &folder);
    assert!(result.export.clips);
}

#[test]
fn test_build_folder_config_unknown_preset_falls_back() {
    let mut config = Config::default();
    config.audio.enhance = true;

    let mut folder = make_folder_state();
    folder.preset = "nonexistent".to_string();
    folder.settings.enhance_audio = Some(false);

    let result = build_folder_config(&config, &folder);
    assert!(!result.audio.enhance);
}

#[test]
fn test_build_folder_config_all_settings_at_once() {
    let mut folder = make_folder_state();
    folder.preset = "youtube".to_string();
    folder.settings.enhance_audio = Some(false);
    folder.settings.silence_mode = Some(SilenceMode::Cut);
    folder.settings.silence_threshold_db = Some(-50.0);
    folder.settings.target_lufs = Some(-15.0);
    folder.settings.stabilize = Some(true);
    folder.settings.color_correct = Some(true);
    folder.settings.reframe = Some(true);
    folder.settings.blur_background = Some(true);
    folder.settings.noise_reduction = Some(true);
    folder.settings.preview = Some(true);
    folder.settings.scene_detect = Some(true);
    folder.settings.multi_format = Some(true);
    folder.settings.subtitles = Some(true);
    folder.settings.chapters = Some(true);
    folder.settings.captions = Some(true);
    folder.settings.clips = Some(true);

    let result = build_folder_config(&Config::default(), &folder);

    assert_eq!(result.silence.mode, SilenceMode::Cut);
    assert_eq!(result.silence.threshold_db, -50.0);
    assert!(!result.audio.enhance);
    assert_eq!(result.audio.target_lufs, -15.0);
    assert!(result.audio.noise_reduction);
    assert!(result.video.stabilize);
    assert!(result.video.color_correct);
    assert!(result.video.reframe);
    assert!(result.video.blur_background);
    assert!(result.export.preview);
    assert!(result.silence.scene_detect);
    assert!(result.export.multi_format);
    assert!(result.export.subtitles);
    assert!(result.export.chapters);
    assert!(result.export.captions);
    assert!(result.export.clips);
    assert_eq!(result.video.target_resolution, VideoResolution::Fhd1080p);
}

#[test]
fn test_build_folder_config_preset_then_folder_overrides() {
    let config = Config::default();
    let mut folder = make_folder_state();
    folder.preset = "shorts".to_string();
    folder.settings.enhance_audio = Some(false);
    folder.settings.remove_silence = Some(false);

    let result = build_folder_config(&config, &folder);

    assert_eq!(result.silence.mode, SilenceMode::Speedup);
    assert!(!result.audio.enhance);
}
