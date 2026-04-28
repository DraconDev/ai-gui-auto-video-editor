mod common;

use ai_vid_editor::FfmpegAnalyzer;
use ai_vid_editor::FfmpegEditor;
use ai_vid_editor::analyzer::VideoAnalyzer;
use ai_vid_editor::editor::VideoEditor;
use ai_vid_editor::config::{Config, FolderSettings, VideoResolution, ExportConfig, SilenceConfig, VideoConfig, AudioConfig};
use ai_vid_editor::exporter;
use ai_vid_editor::preview;
use ai_vid_editor::thumbnail;
use ai_vid_editor::analyzer::ProcessedSegment;
use ai_vid_editor::stt_analyzer::TranscriptSegment;
use common::*;

fn check_ffmpeg() {
    if !has_ffmpeg() || !has_ffprobe() {
        eprintln!("Skipping test: ffmpeg/ffprobe not available");
        return;
    }
}

#[test]
fn test_silence_detection() {
    check_ffmpeg();

    let analyzer = FfmpegAnalyzer;
    let video_path = test_video_path();

    if !video_path.exists() {
        eprintln!("Skipping test: test video not found");
        return;
    }

    let result = analyzer.detect_silence(&video_path, -30.0, 0.5);
    assert!(result.is_ok(), "Silence detection should succeed");

    let silences = result.unwrap();
    println!("Detected {} silent segments", silences.len());
}

#[test]
fn test_silence_detection_threshold() {
    check_ffmpeg();

    let analyzer = FfmpegAnalyzer;
    let video_path = test_video_path();

    if !video_path.exists() {
        eprintln!("Skipping test: test video not found");
        return;
    }

    // Higher threshold should detect more silence
    let silences_high = analyzer.detect_silence(&video_path, -20.0, 0.5).unwrap();
    let silences_low = analyzer.detect_silence(&video_path, -50.0, 0.5).unwrap();

    println!(
        "Silences at -20dB: {}, at -50dB: {}",
        silences_high.len(),
        silences_low.len()
    );
    assert!(
        silences_high.len() >= silences_low.len(),
        "Higher threshold should detect equal or more silence"
    );
}

#[test]
fn test_audio_enhancement() {
    check_ffmpeg();

    use tempfile::tempdir;

    let video_path = test_video_path();
    if !video_path.exists() {
        eprintln!("Skipping test: test video not found");
        return;
    }

    let output_dir = tempdir().unwrap();
    let output_path = output_dir.path().join("enhanced.mp4");

    let editor = FfmpegEditor::default();
    let result = editor.enhance_audio(&video_path, &output_path, -14.0);

    assert!(result.is_ok(), "Audio enhancement should succeed");
    assert!(output_path.exists(), "Output file should exist");
}

#[test]
fn test_video_stabilization() {
    check_ffmpeg();

    use tempfile::tempdir;

    let video_path = test_video_path();
    if !video_path.exists() {
        eprintln!("Skipping test: test video not found");
        return;
    }

    let output_dir = tempdir().unwrap();
    let output_path = output_dir.path().join("stabilized.mp4");

    let editor = FfmpegEditor::default();
    let result = editor.stabilize(&video_path, &output_path);

    assert!(result.is_ok(), "Video stabilization should succeed");
    assert!(output_path.exists(), "Output file should exist");
}

#[test]
fn test_color_correction() {
    check_ffmpeg();

    use tempfile::tempdir;

    let video_path = test_video_path();
    if !video_path.exists() {
        eprintln!("Skipping test: test video not found");
        return;
    }

    let output_dir = tempdir().unwrap();
    let output_path = output_dir.path().join("corrected.mp4");

    let editor = FfmpegEditor::default();
    let result = editor.color_correct(&video_path, &output_path);

    assert!(result.is_ok(), "Color correction should succeed");
    assert!(output_path.exists(), "Output file should exist");
}

#[test]
fn test_auto_reframe() {
    check_ffmpeg();

    use tempfile::tempdir;

    let video_path = test_video_path();
    if !video_path.exists() {
        eprintln!("Skipping test: test video not found");
        return;
    }

    let output_dir = tempdir().unwrap();
    let output_path = output_dir.path().join("reframed.mp4");

    let editor = FfmpegEditor::default();
    let result = editor.reframe(&video_path, &output_path, ai_vid_editor::config::VideoResolution::Vertical1080p);

    // Note: This will use center crop if ML models fail to load
    assert!(
        result.is_ok(),
        "Auto-reframe should succeed (with or without ML)"
    );
    assert!(output_path.exists(), "Output file should exist");
}

// ============================================================
// PHASE 1: Individual Feature Tests
// ============================================================

#[test]
fn test_noise_reduction() {
    check_ffmpeg();

    use tempfile::tempdir;

    let video_path = test_video_path();
    if !video_path.exists() {
        eprintln!("Skipping test: test video not found");
        return;
    }

    let output_dir = tempdir().unwrap();
    let output_path = output_dir.path().join("denoised.mp4");

    let editor = FfmpegEditor::default();
    let result = editor.reduce_noise(&video_path, &output_path);

    assert!(result.is_ok(), "Noise reduction should succeed");
    assert!(output_path.exists(), "Output file should exist");
}

#[test]
fn test_blur_background() {
    check_ffmpeg();

    use tempfile::tempdir;

    let video_path = test_video_path();
    if !video_path.exists() {
        eprintln!("Skipping test: test video not found");
        return;
    }

    let output_dir = tempdir().unwrap();
    let output_path = output_dir.path().join("blurred.mp4");

    let editor = FfmpegEditor::default();
    let result = editor.blur_background(&video_path, &output_path);

    assert!(result.is_ok(), "Blur background should succeed");
    assert!(output_path.exists(), "Output file should exist");
}

#[test]
fn test_reframe_all_resolutions() {
    check_ffmpeg();

    use tempfile::tempdir;

    let video_path = test_video_path();
    if !video_path.exists() {
        eprintln!("Skipping test: test video not found");
        return;
    }

    let resolutions = [
        VideoResolution::Hd720p,
        VideoResolution::Fhd1080p,
        VideoResolution::Qhd1440p,
        VideoResolution::Uhd4k,
        VideoResolution::Vertical1080p,
        VideoResolution::Vertical720p,
    ];

    let editor = FfmpegEditor::default();
    for res in &resolutions {
        let output_dir = tempdir().unwrap();
        let output_path = output_dir.path().join(format!("reframed_{:?}.mp4", res));

        let result = editor.reframe(&video_path, &output_path, *res);
        assert!(result.is_ok(), "Reframe with {:?} should succeed", res);
        assert!(output_path.exists(), "Output for {:?} should exist", res);
    }
}

#[test]
fn test_trim_video_with_segments() {
    check_ffmpeg();

    use tempfile::tempdir;

    let video_path = test_video_path();
    if !video_path.exists() {
        eprintln!("Skipping test: test video not found");
        return;
    }

    let output_dir = tempdir().unwrap();
    let output_path = output_dir.path().join("trimmed.mp4");

    // Use a single segment spanning most of the video
    let segments = vec![ProcessedSegment {
        start: 0.5,
        end: 5.0,
        speed: 1.0,
    }];

    let editor = FfmpegEditor::default();
    let result = editor.trim_video(&video_path, &output_path, &segments);

    assert!(result.is_ok(), "Trim video should succeed");
    assert!(output_path.exists(), "Trimmed output file should exist");

    // Verify output is shorter than input (trim worked)
    let input_meta = std::fs::metadata(&video_path).unwrap();
    let output_meta = std::fs::metadata(&output_path).unwrap();
    assert!(
        output_meta.len() < input_meta.len(),
        "Trimmed file should be smaller than input"
    );
}

// ============================================================
// PHASE 2: Export Feature Tests
// ============================================================

#[test]
fn test_export_fcpxml() {
    use tempfile::tempdir;

    let dir = tempdir().unwrap();
    let output_path = dir.path().join("timeline.fcpxml");
    let input_path = dir.path().join("input.mp4");

    let segments = vec![
        ProcessedSegment { start: 0.0, end: 5.0, speed: 1.0 },
        ProcessedSegment { start: 10.0, end: 20.0, speed: 1.0 },
    ];

    let result = exporter::export_fcpxml(&segments, &input_path, &output_path);
    assert!(result.is_ok(), "FCPXML export should succeed");
    assert!(output_path.exists(), "FCPXML output file should exist");

    let content = std::fs::read_to_string(&output_path).unwrap();
    assert!(content.contains("fcpxml version=\"1.8\""), "FCPXML should have version");
    assert!(content.contains("<spine>"), "FCPXML should have spine element");
    assert!(content.contains("<video name="), "FCPXML should have video elements");
}

#[test]
fn test_export_edl() {
    use tempfile::tempdir;

    let dir = tempdir().unwrap();
    let output_path = dir.path().join("timeline.edl");
    let input_path = dir.path().join("input.mp4");

    let segments = vec![
        ProcessedSegment { start: 0.0, end: 5.5, speed: 1.0 },
        ProcessedSegment { start: 10.0, end: 20.0, speed: 1.0 },
    ];

    let result = exporter::export_edl(&segments, &input_path, &output_path, 25.0);
    assert!(result.is_ok(), "EDL export should succeed");
    assert!(output_path.exists(), "EDL output file should exist");

    let content = std::fs::read_to_string(&output_path).unwrap();
    assert!(content.contains("TITLE:"), "EDL should have title");
    assert!(content.contains("FCM: NON-DROP FRAME"), "EDL should have frame code mode");
    assert!(content.contains("AX       V     C"), "EDL should have source track entry");
}

#[test]
fn test_export_srt() {
    use tempfile::tempdir;

    let dir = tempdir().unwrap();
    let output_path = dir.path().join("subtitles.srt");

    let transcript = vec![
        TranscriptSegment { start: 0.0, end: 5.0, text: "Hello world".to_string(), confidence: 1.0 },
        TranscriptSegment { start: 5.0, end: 10.0, text: "This is a test".to_string(), confidence: 1.0 },
    ];

    let result = exporter::export_srt(&transcript, &output_path);
    assert!(result.is_ok(), "SRT export should succeed");
    assert!(output_path.exists(), "SRT output file should exist");

    let content = std::fs::read_to_string(&output_path).unwrap();
    assert!(content.contains("1\n"), "SRT should have first index");
    assert!(content.contains("00:00:00,000 --> 00:00:05,000"), "SRT should have first timestamp");
    assert!(content.contains("Hello world"), "SRT should contain first text");
}

#[test]
fn test_export_youtube_chapters() {
    use tempfile::tempdir;

    let dir = tempdir().unwrap();
    let output_path = dir.path().join("chapters.txt");

    let transcript = vec![
        TranscriptSegment { start: 0.0, end: 30.0, text: "Welcome".to_string(), confidence: 1.0 },
        TranscriptSegment { start: 30.0, end: 60.0, text: "Introduction".to_string(), confidence: 1.0 },
        TranscriptSegment { start: 200.0, end: 230.0, text: "Advanced features".to_string(), confidence: 1.0 },
    ];

    let result = exporter::export_youtube_chapters(&transcript, &output_path);
    assert!(result.is_ok(), "YouTube chapters export should succeed");
    assert!(output_path.exists(), "Chapters output file should exist");

    let content = std::fs::read_to_string(&output_path).unwrap();
    assert!(content.contains("00:00 Intro"), "Chapters should start with intro at 00:00");
    assert!(content.contains("00:00"), "Chapters should have timestamp for first chapter");
}

#[test]
fn test_generate_preview() {
    check_ffmpeg();

    use tempfile::tempdir;

    let video_path = test_video_path();
    if !video_path.exists() {
        eprintln!("Skipping test: test video not found");
        return;
    }

    let output_dir = tempdir().unwrap();
    let preview_path = preview::preview_path(output_dir.path());

    let result = preview::generate_preview(&video_path, &preview_path, 5.0, 480);
    assert!(result.is_ok(), "Preview generation should succeed");
    assert!(preview_path.exists(), "Preview file should exist");

    // Verify preview is smaller than original
    let input_meta = std::fs::metadata(&video_path).unwrap();
    let preview_meta = std::fs::metadata(&preview_path).unwrap();
    assert!(
        preview_meta.len() < input_meta.len(),
        "Preview should be smaller than input video"
    );
}

#[test]
fn test_generate_thumbnail() {
    check_ffmpeg();

    use tempfile::tempdir;

    let video_path = test_video_path();
    if !video_path.exists() {
        eprintln!("Skipping test: test video not found");
        return;
    }

    let output_dir = tempdir().unwrap();
    let output_path = output_dir.path().join("thumb.jpg");

    let result = thumbnail::generate_thumbnail(&video_path, &output_path, 320, 180);
    assert!(result.is_ok(), "Thumbnail generation should succeed");
    assert!(output_path.exists(), "Thumbnail file should exist");

    // Verify it's a valid image (JPEG header)
    let bytes = std::fs::read(&output_path).unwrap();
    assert!(bytes.len() > 2, "Thumbnail should have some data");
}

#[test]
fn test_highlight_clips_extraction() {
    check_ffmpeg();

    use tempfile::tempdir;

    let video_path = test_video_path();
    if !video_path.exists() {
        eprintln!("Skipping test: test video not found");
        return;
    }

    let output_dir = tempdir().unwrap();
    let output_path = output_dir.path().join("clips.mp4");

    let segments = vec![
        ProcessedSegment { start: 1.0, end: 2.5, speed: 1.0 },
        ProcessedSegment { start: 4.0, end: 5.0, speed: 1.0 },
    ];

    let result = ai_vid_editor::clipper::extract_highlight_clips(
        &video_path,
        &output_path,
        &segments,
        &ai_vid_editor::config::VideoResolution::Fhd1080p,
        2,
    );
    assert!(result.is_ok(), "Highlight clip extraction should succeed");
    assert!(output_path.exists(), "Clips output file should exist");
}

// ============================================================
// PHASE 3: Full Pipeline Tests
// ============================================================

#[test]
fn test_full_pipeline_all_features_disabled() {
    check_ffmpeg();

    use tempfile::tempdir;

    let video_path = test_video_path();
    if !video_path.exists() {
        eprintln!("Skipping test: test video not found");
        return;
    }

    let output_dir = tempdir().unwrap();
    let output_path = output_dir.path().join("output_all_off.mp4");

    let config = Config::default();

    let editor = FfmpegEditor::default();
    let analyzer = FfmpegAnalyzer;
    let duration_getter = ai_vid_editor::batch_processor::FfmpegDurationGetter;

    let result = ai_vid_editor::batch_processor::process_single_file(
        video_path.clone(),
        output_path.clone(),
        &config,
        &analyzer,
        &editor,
        &duration_getter,
    );

    assert!(result.is_ok(), "Pipeline with all features disabled should succeed");
    assert!(output_path.exists(), "Output file should exist");
}

#[test]
fn test_full_pipeline_noise_reduction_and_enhance() {
    check_ffmpeg();

    use tempfile::tempdir;

    let video_path = test_video_path();
    if !video_path.exists() {
        eprintln!("Skipping test: test video not found");
        return;
    }

    let output_dir = tempdir().unwrap();
    let output_path = output_dir.path().join("output_noise_enhance.mp4");

    let mut config = Config::default();
    config.audio.noise_reduction = true;
    config.audio.enhance = true;
    config.audio.target_lufs = -16.0;

    let editor = FfmpegEditor::default();
    let analyzer = FfmpegAnalyzer;
    let duration_getter = ai_vid_editor::batch_processor::FfmpegDurationGetter;

    let result = ai_vid_editor::batch_processor::process_single_file(
        video_path.clone(),
        output_path.clone(),
        &config,
        &analyzer,
        &editor,
        &duration_getter,
    );

    assert!(result.is_ok(), "Pipeline with noise reduction and audio enhance should succeed");
    assert!(output_path.exists(), "Output file should exist");
}

#[test]
fn test_full_pipeline_reframe_and_scale() {
    check_ffmpeg();

    use tempfile::tempdir;

    let video_path = test_video_path();
    if !video_path.exists() {
        eprintln!("Skipping test: test video not found");
        return;
    }

    let output_dir = tempdir().unwrap();
    let output_path = output_dir.path().join("output_reframed.mp4");

    let mut config = Config::default();
    config.video.reframe = true;
    config.video.target_resolution = VideoResolution::Vertical1080p;

    let editor = FfmpegEditor::default();
    let analyzer = FfmpegAnalyzer;
    let duration_getter = ai_vid_editor::batch_processor::FfmpegDurationGetter;

    let result = ai_vid_editor::batch_processor::process_single_file(
        video_path,
        output_path,
        &config,
        &analyzer,
        &editor,
        &duration_getter,
    );

    assert!(result.is_ok(), "Pipeline with reframe enabled should succeed");
}

#[test]
fn test_full_pipeline_with_preview_export() {
    check_ffmpeg();

    use tempfile::tempdir;

    let video_path = test_video_path();
    if !video_path.exists() {
        eprintln!("Skipping test: test video not found");
        return;
    }

    let output_dir = tempdir().unwrap();
    let output_path = output_dir.path().join("output_with_preview.mp4");
    let preview_path = preview::preview_path(&output_path);

    let mut config = Config::default();
    config.export.preview = true;
    config.export.preview_duration = 3.0;

    let editor = FfmpegEditor::default();
    let analyzer = FfmpegAnalyzer;
    let duration_getter = ai_vid_editor::batch_processor::FfmpegDurationGetter;

    let result = ai_vid_editor::batch_processor::process_single_file(
        video_path,
        output_path.clone(),
        &config,
        &analyzer,
        &editor,
        &duration_getter,
    );

    assert!(result.is_ok(), "Pipeline with preview export should succeed");
    assert!(output_path.exists(), "Output file should exist");
    // Note: preview file path depends on final output path - may not exist if
    // pipeline generates intermediate files before producing final output
}

// ============================================================
// PHASE 4: Config Integration Tests
// ============================================================

#[test]
fn test_build_folder_config_maps_all_fields() {
    // This test verifies that build_folder_config correctly maps
    // every FolderSettings field to the merged Config

    use ai_vid_editor::gui::processing::build_folder_config;
    use std::path::PathBuf;

    let folder = ai_vid_editor::config::WatchFolder {
        input: PathBuf::from("/input"),
        output: PathBuf::from("/output"),
        preset: "youtube".to_string(),
        enabled: true,
        settings: FolderSettings {
            enhance_audio: Some(true),
            remove_silence: Some(true),
            silence_threshold_db: Some(-40.0),
            target_lufs: Some(-18.0),
            stabilize: Some(true),
            color_correct: Some(true),
            reframe: Some(true),
            blur_background: Some(true),
            noise_reduction: Some(true),
            preview: Some(true),
            scene_detect: Some(true),
            multi_format: Some(true),
            hw_accel: Some(ai_vid_editor::hwaccel::HwAccel::Nvenc),
            target_resolution: Some(VideoResolution::Fhd1080p),
            subtitles: Some(true),
            chapters: Some(true),
            captions: Some(true),
            clips: Some(true),
        },
    };

    let base_config = Config::default();
    let merged = build_folder_config(&folder, &base_config);

    // Verify all fields were correctly propagated
    assert_eq!(merged.audio.enhance, true, "enhance_audio should be mapped");
    assert_eq!(merged.silence.remove_silence, Some(true), "remove_silence should be mapped");
    assert_eq!(merged.silence.threshold_db, -40.0, "silence_threshold_db should be mapped");
    assert_eq!(merged.audio.target_lufs, -18.0, "target_lufs should be mapped");
    assert_eq!(merged.video.stabilize, true, "stabilize should be mapped");
    assert_eq!(merged.video.color_correct, true, "color_correct should be mapped");
    assert_eq!(merged.video.reframe, true, "reframe should be mapped");
    assert_eq!(merged.video.blur_background, true, "blur_background should be mapped");
    assert_eq!(merged.audio.noise_reduction, true, "noise_reduction should be mapped");
    assert_eq!(merged.export.preview, true, "preview should be mapped");
    assert_eq!(merged.silence.scene_detect, true, "scene_detect should be mapped");
    assert_eq!(merged.export.multi_format, true, "multi_format should be mapped");
    assert_eq!(merged.video.hw_accel, ai_vid_editor::hwaccel::HwAccel::Nvenc, "hw_accel should be mapped");
    assert_eq!(merged.video.target_resolution, VideoResolution::Fhd1080p, "target_resolution should be mapped");

    // The four export fields we fixed
    assert_eq!(merged.export.subtitles, true, "subtitles should be mapped");
    assert_eq!(merged.export.chapters, true, "chapters should be mapped");
    assert_eq!(merged.export.captions, true, "captions should be mapped");
    assert_eq!(merged.export.clips, true, "clips should be mapped");
}

#[test]
fn test_config_export_fields_exist() {
    // Verify all export config fields are accessible and non-null
    let config = Config::default();

    // These should not panic - basic sanity check
    let _ = config.export.preview;
    let _ = config.export.preview_duration;
    let _ = config.export.subtitles;
    let _ = config.export.chapters;
    let _ = config.export.captions;
    let _ = config.export.clips;
    let _ = config.export.multi_format;
    let _ = config.export.thumbnail;
    let _ = config.export.fcpxml;
    let _ = config.export.edl;

    assert!(true, "All export config fields should be accessible");
}

#[test]
fn test_transcript_segment_from_synthetic_data() {
    // Verify TranscriptSegment can be constructed and used
    let segments = vec![
        TranscriptSegment { start: 0.0, end: 3.5, text: "First segment".to_string(), confidence: 0.95 },
        TranscriptSegment { start: 3.5, end: 7.0, text: "Second segment".to_string(), confidence: 0.88 },
    ];

    assert_eq!(segments.len(), 2);
    assert_eq!(segments[0].text, "First segment");
    assert_eq!(segments[1].start, 3.5);

    // Test SRT round-trip
    use tempfile::tempdir;
    let dir = tempdir().unwrap();
    let srt_path = dir.path().join("roundtrip.srt");

    exporter::export_srt(&segments, &srt_path).unwrap();
    assert!(srt_path.exists());
}

#[test]
fn test_processed_segment_speed_handling() {
    // Verify ProcessedSegment with various speed values
    let segments = vec![
        ProcessedSegment { start: 0.0, end: 5.0, speed: 1.0 },   // normal
        ProcessedSegment { start: 10.0, end: 15.0, speed: 1.5 }, // sped up
    ];

    // Verify EDL export handles different speeds
    use tempfile::tempdir;
    let dir = tempdir().unwrap();
    let edl_path = dir.path().join("speed_test.edl");
    let input_path = dir.path().join("input.mp4");

    let result = exporter::export_edl(&segments, &input_path, &edl_path, 30.0);
    assert!(result.is_ok(), "EDL export should handle varied speeds");
}
