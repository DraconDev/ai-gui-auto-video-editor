use anyhow::{Context, Result};
use indicatif::{ProgressBar, ProgressStyle};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tracing::{debug, info, warn};

use crate::analyzer::ProcessedSegment;
use crate::analyzer::VideoAnalyzer;
use crate::config::Config;
use crate::editor::VideoEditor;
use crate::editor::calculate_keep_segments;
use crate::exporter;
use crate::progress::BatchProgress;
use crate::stt_analyzer::{CandleSttAnalyzer, TranscriptSegment, VideoSttAnalyzer};
use crate::utils::find_video_files;

/// RAII guard for cleaning up temporary video files on drop.
/// Tracks intermediate files and removes them when the guard goes out of scope.
struct TempFileGuard {
    temps: Vec<PathBuf>,
    output: PathBuf,
}

impl TempFileGuard {
    fn new(output: PathBuf) -> Self {
        Self {
            temps: Vec::new(),
            output,
        }
    }

    fn track(&mut self, path: PathBuf) {
        if path != self.output && !self.temps.contains(&path) {
            self.temps.push(path);
        }
    }

    fn untrack(&mut self, path: &Path) {
        self.temps.retain(|p| p != path);
    }
}

impl Drop for TempFileGuard {
    fn drop(&mut self) {
        for path in &self.temps {
            if let Err(e) = fs::remove_file(path) {
                debug!(path = ?path, error = %e, "Failed to remove temp file");
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProcessingProgress {
    pub fraction: f32,
    pub stage: String,
}

// Trait for getting video duration
pub trait DurationGetter: Send + Sync {
    fn get_duration(&self, path: &Path) -> Result<f32>;
}

// Concrete implementation using ffprobe
pub struct FfmpegDurationGetter;

impl DurationGetter for FfmpegDurationGetter {
    fn get_duration(&self, path: &Path) -> Result<f32> {
        let output = std::process::Command::new("ffprobe")
            .args([
                "-v",
                "error",
                "-show_entries",
                "format=duration",
                "-of",
                "default=noprint_wrappers=1:nokey=1",
                path.to_str().context("invalid path")?,
            ])
            .output()
            .context("failed to execute ffprobe")?;

        let val_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
        val_str.parse::<f32>().context("failed to parse duration")
    }
}

/// Concatenate intro/outro videos using ffmpeg
fn concatenate_videos(
    intro: Option<&Path>,
    main: &Path,
    outro: Option<&Path>,
    output: &Path,
) -> Result<()> {
    let has_intro = intro.is_some();
    let has_outro = outro.is_some();

    if !has_intro && !has_outro {
        // No intro/outro, just copy
        fs::copy(main, output)?;
        return Ok(());
    }

    // Build ffmpeg concat filter
    let mut args: Vec<String> = vec![];
    let mut concat_inputs = String::new();
    let mut input_idx = 0;

    if let Some(intro_path) = intro {
        args.push("-i".to_string());
        args.push(
            intro_path
                .to_str()
                .context("invalid intro path")?
                .to_string(),
        );
        concat_inputs.push_str(&format!("[{}:v][{}:a]", input_idx, input_idx));
        input_idx += 1;
    }

    args.push("-i".to_string());
    args.push(main.to_str().context("invalid main path")?.to_string());
    concat_inputs.push_str(&format!("[{}:v][{}:a]", input_idx, input_idx));
    input_idx += 1;

    if let Some(outro_path) = outro {
        args.push("-i".to_string());
        args.push(
            outro_path
                .to_str()
                .context("invalid outro path")?
                .to_string(),
        );
        concat_inputs.push_str(&format!("[{}:v][{}:a]", input_idx, input_idx));
    }

    let n = input_idx;
    let filter = format!("{}concat=n={}:v=1:a=1[outv][outa]", concat_inputs, n);

    args.push("-filter_complex".to_string());
    args.push(filter);
    args.push("-map".to_string());
    args.push("[outv]".to_string());
    args.push("-map".to_string());
    args.push("[outa]".to_string());
    args.push("-y".to_string());
    args.push(output.to_str().context("invalid output path")?.to_string());

    let status = std::process::Command::new("ffmpeg")
        .args(&args)
        .status()
        .context("failed to execute ffmpeg for concat")?;

    if !status.success() {
        anyhow::bail!("ffmpeg concat failed with status: {}", status);
    }

    Ok(())
}

pub fn process_single_file<A, E, D>(
    input_file: PathBuf,
    output_file: PathBuf,
    config: &Config,
    analyzer: &A,
    editor: &E,
    duration_getter: &D,
) -> Result<()>
where
    A: VideoAnalyzer,
    E: VideoEditor,
    D: DurationGetter,
{
    process_single_file_with_intro_outro(
        input_file,
        output_file,
        config,
        analyzer,
        editor,
        duration_getter,
        None,
        None,
    )
}

pub fn process_single_file_with_intro_outro<A, E, D>(
    input_file: PathBuf,
    output_file: PathBuf,
    config: &Config,
    analyzer: &A,
    editor: &E,
    duration_getter: &D,
    intro: Option<PathBuf>,
    outro: Option<PathBuf>,
) -> Result<()>
where
    A: VideoAnalyzer,
    E: VideoEditor,
    D: DurationGetter,
{
    process_single_file_with_intro_outro_progress(
        input_file,
        output_file,
        config,
        analyzer,
        editor,
        duration_getter,
        intro,
        outro,
        |_| {},
    )
}

pub fn process_single_file_with_intro_outro_progress<A, E, D, F>(
    input_file: PathBuf,
    output_file: PathBuf,
    config: &Config,
    analyzer: &A,
    editor: &E,
    duration_getter: &D,
    intro: Option<PathBuf>,
    outro: Option<PathBuf>,
    mut progress: F,
) -> Result<()>
where
    A: VideoAnalyzer,
    E: VideoEditor,
    D: DurationGetter,
    F: FnMut(ProcessingProgress),
{
    // Guard ensures temp files are cleaned up even on early return
    let mut guard = TempFileGuard::new(output_file.clone());

    report_progress(&mut progress, 0.02, "Analyzing silence");
    info!(file = ?input_file, "Analyzing video");
    debug!(mode = ?config.silence.mode, "Silence mode");

    let silences = analyzer
        .detect_silence(
            &input_file,
            config.silence.threshold_db,
            config.silence.min_duration,
        )
        .context("Failed to detect silence")?;

    info!(count = silences.len(), "Detected silent segments");

    report_progress(&mut progress, 0.08, "Planning edits");
    let video_duration = duration_getter.get_duration(&input_file)?;
    debug!(duration = video_duration, "Video duration");

    // Merge scene changes with silences if scene detection is enabled
    let silences = if config.silence.scene_detect {
        report_progress(&mut progress, 0.09, "Detecting scene changes");
        match crate::scene_detection::detect_scene_changes(&input_file, config.silence.scene_threshold) {
            Ok(scenes) => {
                info!(count = scenes.len(), "Detected scene changes");
                merge_silences_and_scenes(&silences, &scenes, video_duration)
            }
            Err(e) => {
                warn!(error = %e, "Scene detection failed, using silence only");
                silences
            }
        }
    } else {
        silences
    };

    report_progress(&mut progress, 0.1, "Planning edits");

    // Generate early preview if requested
    if config.export.preview {
        let preview_path = crate::preview::preview_path(&output_file);
        report_progress(&mut progress, 0.11, "Generating preview");
        if let Err(e) = crate::preview::generate_preview(
            &input_file,
            &preview_path,
            config.export.preview_duration,
            480,
        ) {
            warn!(error = %e, "Failed to generate preview");
        } else {
            info!(path = %preview_path.display(), "Preview generated");
        }
    }

    let processed_segments = calculate_keep_segments(
        &silences,
        video_duration,
        config.silence.padding,
        config.silence.mode,
        config.silence.speedup_factor,
        config.silence.min_silence_for_speedup,
    );
    debug!(count = processed_segments.len(), "Segments to process");

    let trimmed_file = if config.audio.enhance
        || config.audio.music_file.is_some()
        || intro.is_some()
        || outro.is_some()
    {
        let path = output_file.with_extension("trimmed.mp4");
        guard.track(path.clone());
        path
    } else {
        output_file.clone()
    };

    report_progress(&mut progress, 0.15, "Trimming video");
    editor
        .trim_video_with_progress(
            &input_file,
            &trimmed_file,
            &processed_segments,
            &mut |value| {
                let percent = 0.15 + (value * 0.6);
                report_progress(
                    &mut progress,
                    percent,
                    format!("Trimming video ({:.0}%)", value * 100.0),
                );
            },
        )
        .context("Failed to trim video")?;
    debug!(file = ?trimmed_file, "Trimmed video saved");

    let enhanced_file = if config.audio.enhance {
        let enhanced = output_file.with_extension("enhanced.mp4");
        report_progress(&mut progress, 0.78, "Enhancing audio");
        info!("Enhancing audio");
        editor
            .enhance_audio(&trimmed_file, &enhanced, config.audio.target_lufs)
            .context("Failed to enhance audio")?;

        if trimmed_file != output_file {
            guard.untrack(&trimmed_file);
            let _ = fs::remove_file(&trimmed_file);
        }
        guard.track(enhanced.clone());
        enhanced
    } else {
        trimmed_file
    };

    let with_music_file = if let Some(ref music_path) = config.audio.music_file {
        let with_music = output_file.with_extension("music.mp4");
        report_progress(&mut progress, 0.84, "Mixing background music");
        info!(music = ?music_path, "Mixing background music");

        let empty_transcript = vec![];
        editor
            .mix_with_music(
                &enhanced_file,
                music_path,
                &with_music,
                &empty_transcript,
                config.audio.duck_volume,
            )
            .context("Failed to mix music")?;

        if enhanced_file != output_file {
            guard.untrack(&enhanced_file);
            let _ = fs::remove_file(&enhanced_file);
        }
        guard.track(with_music.clone());
        with_music
    } else {
        enhanced_file
    };

    let concat_file = if intro.is_some() || outro.is_some() {
        report_progress(&mut progress, 0.88, "Adding intro/outro");
        info!("Adding intro/outro");
        concatenate_videos(
            intro.as_deref(),
            &with_music_file,
            outro.as_deref(),
            &output_file,
        )?;

        if with_music_file != output_file {
            guard.untrack(&with_music_file);
            let _ = fs::remove_file(&with_music_file);
        }
        guard.track(output_file.clone());
        output_file.clone()
    } else {
        if with_music_file != output_file {
            guard.untrack(&with_music_file);
            fs::rename(&with_music_file, &output_file)?;
        }
        output_file.clone()
    };

    let mut current_file = concat_file;

    if config.video.stabilize {
        let stabilized = output_file.with_extension("stabilized.mp4");
        report_progress(&mut progress, 0.9, "Stabilizing video");
        info!("Stabilizing video");
        editor.stabilize(&current_file, &stabilized)?;
        if current_file != output_file {
            guard.untrack(&current_file);
            let _ = fs::remove_file(&current_file);
        }
        guard.track(stabilized.clone());
        current_file = stabilized;
    }

    if config.video.color_correct {
        let corrected = output_file.with_extension("corrected.mp4");
        report_progress(&mut progress, 0.93, "Color correcting");
        info!("Color correcting");
        editor.color_correct(&current_file, &corrected)?;
        if current_file != output_file {
            guard.untrack(&current_file);
            let _ = fs::remove_file(&current_file);
        }
        guard.track(corrected.clone());
        current_file = corrected;
    }

    if config.video.reframe {
        let reframed = output_file.with_extension("reframed.mp4");
        report_progress(&mut progress, 0.95, "Auto-reframing");
        info!("Auto-reframing to vertical (9:16)");
        editor.reframe(&current_file, &reframed)?;
        if current_file != output_file {
            guard.untrack(&current_file);
            let _ = fs::remove_file(&current_file);
        }
        guard.track(reframed.clone());
        current_file = reframed;
    }

    if config.video.blur_background {
        let blurred = output_file.with_extension("blurred.mp4");
        report_progress(&mut progress, 0.97, "Blurring background");
        info!("Blurring background");
        editor.blur_background(&current_file, &blurred)?;
        if current_file != output_file {
            guard.untrack(&current_file);
            let _ = fs::remove_file(&current_file);
        }
        guard.track(blurred.clone());
        current_file = blurred;
    }

    // Apply watermark if configured
    if let Some(ref watermark_path) = config.video.watermark {
        let watermarked = output_file.with_extension("watermarked.mp4");
        report_progress(&mut progress, 0.98, "Adding watermark");
        info!(watermark = ?watermark_path, "Adding watermark");

        let position = crate::watermark::WatermarkPosition::from_str(&config.video.watermark_position)
            .unwrap_or(crate::watermark::WatermarkPosition::BottomRight);
        let scale = config.video.watermark_scale;

        crate::watermark::add_watermark(
            &current_file,
            watermark_path,
            &watermarked,
            position,
            scale,
        )?;

        if current_file != output_file {
            guard.untrack(&current_file);
            let _ = fs::remove_file(&current_file);
        }
        guard.track(watermarked.clone());
        current_file = watermarked;
    }

    // Move final temp file to output if needed
    if current_file != output_file {
        fs::rename(&current_file, &output_file)?;
    }
    guard.untrack(&output_file); // Don't delete the final output

    report_progress(&mut progress, 0.99, "Writing exports");
    export_additional_files(&input_file, &output_file, &processed_segments, config)?;

    report_progress(&mut progress, 1.0, "Done");
    info!(file = ?output_file, "Successfully saved video");
    Ok(())
}

/// Merge silence segments with scene-change boundaries.
/// Scene changes are treated as additional cut points - they split existing
/// silence segments or create new boundaries for trimming.
fn merge_silences_and_scenes(
    silences: &[crate::analyzer::Segment],
    scenes: &[f32],
    duration: f32,
) -> Vec<crate::analyzer::Segment> {
    if scenes.is_empty() {
        return silences.to_vec();
    }

    // Convert scene timestamps to segments (gaps between scenes are "keep" regions)
    let scene_segments = crate::scene_detection::scenes_to_segments(scenes, duration);

    // Merge: a silence segment is extended to nearest scene boundary if close
    let mut merged = Vec::new();
    for silence in silences {
        let mut start = silence.start;
        let mut end = silence.end;

        // Extend silence to nearest scene boundary if within 0.5s
        for scene in &scene_segments {
            if (scene.start - start).abs() < 0.5 {
                start = scene.start.min(start);
            }
            if (scene.end - end).abs() < 0.5 {
                end = scene.end.max(end);
            }
        }

        merged.push(crate::analyzer::Segment { start, end });
    }

    merged
}

fn report_progress<F>(progress: &mut F, fraction: f32, stage: impl Into<String>)
where
    F: FnMut(ProcessingProgress),
{
    progress(ProcessingProgress {
        fraction: fraction.clamp(0.0, 1.0),
        stage: stage.into(),
    });
}

/// Export additional files (SRT, chapters, FCPXML, EDL, clips) based on config
fn export_additional_files(
    input_file: &Path,
    output_file: &Path,
    segments: &[ProcessedSegment],
    config: &Config,
) -> Result<()> {
    let base_path = output_file.with_extension("");

    // Run transcription if we need transcript for any export
    let transcript = if config.export.subtitles
        || config.export.chapters
        || config.export.captions
        || config.export.clips
    {
        match CandleSttAnalyzer.transcribe(output_file) {
            Ok(t) => {
                info!(segments = t.len(), "Transcription complete");
                Some(t)
            }
            Err(e) => {
                warn!(error = %e, "Transcription failed");
                None
            }
        }
    } else {
        None
    };

    if config.export.subtitles {
        let srt_path = format!("{}.srt", base_path.display());
        debug!(path = %srt_path, "Exporting SRT subtitles");
        if let Some(ref t) = transcript {
            exporter::export_srt(t, Path::new(&srt_path))?;
        } else {
            fs::write(&srt_path, "# Transcription failed\n")?;
        }
    }

    if config.export.chapters {
        let chapters_path = format!("{}.chapters.txt", base_path.display());
        debug!(path = %chapters_path, "Exporting YouTube chapters");
        if let Some(ref t) = transcript {
            exporter::export_youtube_chapters(t, Path::new(&chapters_path))?;
        } else {
            fs::write(&chapters_path, "00:00 Intro\n")?;
        }
    }

    if config.export.captions {
        let ass_path = PathBuf::from(format!("{}.ass", base_path.display()));
        debug!(path = %ass_path.display(), "Generating styled captions");
        // Generate styled ASS captions from transcript
        if let Some(ref t) = transcript {
            if let Err(e) = generate_styled_captions(t, &ass_path) {
                warn!(error = %e, "Failed to generate styled captions");
            } else {
                // Burn captions into video
                info!("Burning captions into video");
                let captioned_path = output_file.with_extension("captioned.mp4");
                burn_subtitles_into_video(output_file, &ass_path, &captioned_path)?;
            }
        }
    }

    if config.export.clips
        && let Some(ref t) = transcript
    {
        let clips_output_dir = base_path.parent().unwrap_or_else(|| Path::new("."));
        let clip_pattern = format!(
            "{}_clip",
            base_path.file_stem().unwrap_or_default().to_string_lossy()
        );
        match extract_highlight_clips(
            output_file,
            t,
            config.export.clip_count,
            config.export.clip_min_duration,
            config.export.clip_max_duration,
            clips_output_dir,
            &clip_pattern,
        ) {
            Ok(clip_paths) => {
                info!(count = clip_paths.len(), "Extracted highlight clips");
            }
            Err(e) => {
                warn!(error = %e, "Failed to extract highlight clips");
            }
        }
    }

    if config.export.fcpxml {
        let fcpxml_path = format!("{}.fcpxml", base_path.display());
        debug!(path = %fcpxml_path, "Exporting FCPXML");
        exporter::export_fcpxml(segments, input_file, Path::new(&fcpxml_path))?;
    }

    if config.export.edl {
        let edl_path = format!("{}.edl", base_path.display());
        debug!(path = %edl_path, "Exporting EDL");
        exporter::export_edl(segments, input_file, Path::new(&edl_path), 25.0)?;
    }

    // Generate thumbnail
    if config.export.thumbnail {
        let thumb_path = format!("{}.jpg", base_path.display());
        debug!(path = %thumb_path, "Generating thumbnail");
        if let Err(e) = crate::thumbnail::generate_thumbnail(
            output_file,
            Path::new(&thumb_path),
            config.export.thumbnail_width,
            config.export.thumbnail_height,
        ) {
            warn!(error = %e, "Failed to generate thumbnail");
        }
    }

    // Multi-format output
    if config.export.multi_format && !config.export.extra_resolutions.is_empty() {
        debug!("Generating multi-format outputs");
        for resolution in &config.export.extra_resolutions {
            let (w, h) = resolution.dimensions();
            let ext = output_file
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("mp4");
            let multi_path = format!("{}_{}p.{}", base_path.display(), h, ext);
            debug!(path = %multi_path, resolution = ?resolution, "Generating alternate resolution");

            let status = std::process::Command::new("ffmpeg")
                .args([
                    "-i",
                    output_file.to_str().context("invalid output path")?,
                    "-vf",
                    &format!("scale={}:{}", w, h),
                    "-c:a",
                    "copy",
                    "-y",
                    &multi_path,
                ])
                .status()
                .context("failed to execute ffmpeg for multi-format")?;

            if !status.success() {
                warn!(path = %multi_path, "Multi-format ffmpeg failed");
            }
        }
    }

    // Generate quick preview
    if config.export.preview {
        let preview_path = crate::preview::preview_path(output_file);
        debug!(path = %preview_path.display(), "Generating preview");
        if let Err(e) = crate::preview::generate_preview(output_file, &preview_path, 30.0, 480) {
            warn!(error = %e, "Failed to generate preview");
        }
    }

    Ok(())
}

/// Generate styled ASS subtitle file from transcript
fn generate_styled_captions(transcript: &[TranscriptSegment], output_path: &Path) -> Result<()> {
    let mut ass = String::new();
    ass.push_str("[Script Info]\n");
    ass.push_str("Title: Generated Captions\n");
    ass.push_str("ScriptType: v4.00+\n");
    ass.push_str("Collisions: Normal\n");
    ass.push_str("PlayDepth: 0\n\n");

    ass.push_str("[V4+ Styles]\n");
    ass.push_str("Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding\n");
    ass.push_str("Style: Default,Arial,48,&H00FFFFFF,&H000000FF,&H00000000,&H80000000,-1,0,0,0,100,100,0,0,1,2,2,2,10,10,30,1\n\n");

    ass.push_str("[Events]\n");
    ass.push_str(
        "Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\n",
    );

    for seg in transcript {
        let text = seg.text.trim();
        if text.is_empty() || text == "[No speech detected]" {
            continue;
        }
        let start = format_ass_time(seg.start);
        let end = format_ass_time(seg.end);
        // Escape text for ASS format
        // In ASS, \N is a forced newline. Literal backslashes must be escaped as \\.
        // Order matters: escape backslashes first, then newlines.
        let escaped = text
            .replace('\\', "\\\\")
            .replace('\n', "\\N")
            .replace('\r', "");
        ass.push_str(&format!(
            "Dialogue: 0,{},{},Default,,0,0,0,,{}\n",
            start, end, escaped
        ));
    }

    fs::write(output_path, ass)?;
    Ok(())
}

fn format_ass_time(seconds: f32) -> String {
    let seconds = seconds.max(0.0);
    let hours = (seconds / 3600.0) as u32;
    let minutes = ((seconds % 3600.0) / 60.0) as u32;
    let secs = (seconds % 60.0) as u32;
    let centisecs = ((seconds % 1.0) * 100.0) as u32;
    format!("{}:{:02}:{:02}.{:02}", hours, minutes, secs, centisecs)
}

/// Burn subtitle file into video using FFmpeg
fn burn_subtitles_into_video(
    video_path: &Path,
    subtitle_path: &Path,
    output_path: &Path,
) -> Result<()> {
    let status = std::process::Command::new("ffmpeg")
        .args([
            "-i",
            video_path.to_str().context("invalid video path")?,
            "-vf",
            &format!(
                "subtitles='{}'",
                subtitle_path.to_str().context("invalid subtitle path")?
            ),
            "-c:a",
            "copy",
            "-y",
            output_path.to_str().context("invalid output path")?,
        ])
        .status()
        .context("failed to burn subtitles")?;

    if !status.success() {
        anyhow::bail!("ffmpeg subtitle burn failed with status: {}", status);
    }

    // Replace original with captioned version
    if !output_path.exists() {
        anyhow::bail!("ffmpeg subtitle burn did not produce output file");
    }
    fs::rename(output_path, video_path)?;
    Ok(())
}

/// Extract highlight clips based on audio energy peaks in transcript
fn extract_highlight_clips(
    video_path: &Path,
    transcript: &[TranscriptSegment],
    clip_count: u32,
    min_duration: f32,
    max_duration: f32,
    output_dir: &Path,
    clip_pattern: &str,
) -> Result<Vec<PathBuf>> {
    // Analyze audio energy per transcript segment using ffprobe
    let mut segment_energy: Vec<(f32, f32, f32)> = Vec::new(); // (start, end, energy)

    for seg in transcript {
        let text = seg.text.trim();
        if text.is_empty() || text == "[No speech detected]" {
            continue;
        }

        // Estimate energy from segment duration and word count (proxy for speech energy)
        // Longer segments with more words = more content = higher energy
        let duration = seg.end - seg.start;
        let word_count = text.split_whitespace().count() as f32;
        let energy = word_count / duration.max(1.0); // words per second

        segment_energy.push((seg.start, seg.end, energy));
    }

    if segment_energy.is_empty() {
        return Ok(vec![]);
    }

    // Find peaks: sort by energy and take top N segments
    segment_energy.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));

    // Get video duration to clamp clips properly
    let video_duration = crate::ml::FrameExtractor::get_video_duration(video_path)
        .unwrap_or(transcript.last().map(|s| s.end).unwrap_or(60.0));

    let mut clip_times: Vec<(f32, f32)> = Vec::new();
    for &(start, end, _) in segment_energy.iter().take(clip_count as usize) {
        // Expand segment to reasonable clip duration
        let clip_start = (start - 2.0).max(0.0);
        let clip_end = (end + 2.0).min(video_duration);
        let clip_duration = clip_end - clip_start;

        if clip_duration >= min_duration && clip_duration <= max_duration {
            clip_times.push((clip_start, clip_end));
        }
    }

    // Extract clips using FFmpeg
    let mut clip_paths = Vec::new();
    for (i, (clip_start, clip_end)) in clip_times.iter().enumerate() {
        let clip_path = output_dir.join(format!("{}_{}.mp4", clip_pattern, i + 1));

        let duration = clip_end - clip_start;
        let status = std::process::Command::new("ffmpeg")
            .args([
                "-i",
                video_path.to_str().context("invalid path")?,
                "-ss",
                &format!("{}", clip_start),
                "-t",
                &format!("{}", duration),
                "-c",
                "copy",
                "-y",
                clip_path.to_str().context("invalid output path")?,
            ])
            .status()
            .context("failed to extract clip")?;

        if status.success() && clip_path.exists() {
            clip_paths.push(clip_path);
        }
    }

    Ok(clip_paths)
}

pub fn process_batch_dir<A, E, D>(
    input_dir: PathBuf,
    output_dir: PathBuf,
    config: &Config,
    analyzer: &A,
    editor: &E,
    duration_getter: &D,
) -> Result<()>
where
    A: VideoAnalyzer,
    E: VideoEditor,
    D: DurationGetter,
{
    info!(dir = ?input_dir, "Processing directory");
    debug!(output = ?output_dir, mode = ?config.silence.mode, "Batch config");

    fs::create_dir_all(&output_dir).context(format!(
        "Failed to create output directory {:?}",
        output_dir
    ))?;

    let video_files = find_video_files(&input_dir)?;

    if video_files.is_empty() {
        warn!(dir = ?input_dir, "No supported video files found");
        return Ok(());
    }

    // Load or initialize progress tracking
    let progress_path = BatchProgress::default_path(&input_dir);
    let mut progress = BatchProgress::from_file(&progress_path).unwrap_or_default();
    progress.total = video_files.len();

    let total_files = video_files.len();
    let mut successful_files = 0;
    let mut failed_files = 0;
    let mut skipped_files = 0;

    let pb = ProgressBar::new(total_files as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("#>-"),
    );

    let preset_rules = crate::preset_rules::default_preset_rules();

    for (index, input_file) in video_files.iter().enumerate() {
        if progress.is_completed(input_file) {
            info!(file = ?input_file, "Skipping already processed file");
            skipped_files += 1;
            pb.inc(1);
            continue;
        }

        let file_name = input_file
            .file_name()
            .context(format!("Could not get file name for {:?}", input_file))?;
        let output_file = output_dir.join(file_name);

        pb.set_message(format!("{}", input_file.display()));

        // Apply per-file preset based on filename
        let file_preset = crate::preset_rules::preset_for_file(
            input_file,
            &preset_rules,
            crate::config::Preset::Youtube, // Default when no rule matches
        );
        let file_config = if file_preset != crate::config::Preset::Youtube {
            info!(preset = ?file_preset, file = ?file_name, "Applying filename-based preset");
            let mut c = file_preset.to_config();
            // Merge with base config to preserve paths, exports, etc.
            c = c.merge(config.clone());
            c
        } else {
            config.clone()
        };

        match process_single_file(
            input_file.clone(),
            output_file.clone(),
            &file_config,
            analyzer,
            editor,
            duration_getter,
        ) {
            Ok(_) => {
                info!(file = ?input_file, "Successfully processed");
                successful_files += 1;
                progress.mark_completed(input_file);
            }
            Err(e) => {
                warn!(file = ?input_file, error = %e, "Failed to process");
                failed_files += 1;
                progress.mark_failed(input_file);
            }
        }

        if let Err(e) = progress.to_file(&progress_path) {
            warn!("Failed to save progress file: {}", e);
        }
        pb.inc(1);
    }

    pb.finish_with_message("Done");

    println!("\n=== BATCH SUMMARY ===");
    println!("  Total files:     {}", total_files);
    println!("  Successful:      {}", successful_files);
    println!("  Failed:          {}", failed_files);
    println!("  Skipped (done):  {}", skipped_files);
    println!("=====================\n");

    info!(
        total = total_files,
        successful = successful_files,
        failed = failed_files,
        skipped = skipped_files,
        "Batch processing complete"
    );

    Ok(())
}

/// Process a directory of videos in parallel using multiple worker threads.
/// Each worker gets its own analyzer/editor instances since they are stateless.
pub fn process_batch_dir_parallel<A, E, D>(
    input_dir: PathBuf,
    output_dir: PathBuf,
    config: &Config,
    worker_count: usize,
    _analyzer: &A,
    _editor: &E,
    _duration_getter: &D,
) -> Result<()>
where
    A: VideoAnalyzer + Send + Sync,
    E: VideoEditor + Send + Sync,
    D: DurationGetter + Send + Sync,
{
    let worker_count = worker_count.max(1);
    info!(dir = ?input_dir, workers = worker_count, "Processing directory in parallel");

    fs::create_dir_all(&output_dir).context(format!(
        "Failed to create output directory {:?}",
        output_dir
    ))?;

    let mut video_files = find_video_files(&input_dir)?;

    if video_files.is_empty() {
        warn!(dir = ?input_dir, "No supported video files found");
        return Ok(());
    }

    // Load progress and filter out completed files
    let progress_path = BatchProgress::default_path(&input_dir);
    let mut progress = BatchProgress::from_file(&progress_path).unwrap_or_default();
    video_files.retain(|f| !progress.is_completed(f));

    if video_files.is_empty() {
        info!("All files already processed");
        return Ok(());
    }

    progress.total = video_files.len();

    let total_files = video_files.len();
    let successful_files = Arc::new(AtomicUsize::new(0));
    let failed_files = Arc::new(AtomicUsize::new(0));
    let config = Arc::new(config.clone());
    let output_dir = Arc::new(output_dir);
    let progress = Arc::new(std::sync::Mutex::new(progress));
    let progress_path = Arc::new(progress_path);

    // Split files into chunks for each worker
    let chunks: Vec<Vec<PathBuf>> = video_files
        .chunks(total_files.div_ceil(worker_count))
        .map(|c| c.to_vec())
        .collect();

    std::thread::scope(|s| {
        for chunk in chunks {
            let config = Arc::clone(&config);
            let output_dir = Arc::clone(&output_dir);
            let successful = Arc::clone(&successful_files);
            let failed = Arc::clone(&failed_files);
            let progress = Arc::clone(&progress);
            let progress_path = Arc::clone(&progress_path);

            s.spawn(move || {
                for input_file in chunk {
                    let file_name = match input_file.file_name() {
                        Some(name) => name.to_os_string(),
                        None => continue,
                    };
                    let output_file = output_dir.join(&file_name);

                    // Create fresh instances per worker (they're stateless)
                    let analyzer = crate::analyzer::FfmpegAnalyzer;
                    let editor = crate::editor::FfmpegEditor::new(config.video.hw_accel);
                    let duration_getter = FfmpegDurationGetter;

                    match process_single_file(
                        input_file.clone(),
                        output_file,
                        &config,
                        &analyzer,
                        &editor,
                        &duration_getter,
                    ) {
                        Ok(_) => {
                            info!(file = ?input_file, "Successfully processed");
                            successful.fetch_add(1, Ordering::SeqCst);
                            let mut p = progress.lock().unwrap();
                            p.mark_completed(&input_file);
                            let _ = p.to_file(&progress_path);
                        }
                        Err(e) => {
                            warn!(file = ?input_file, error = %e, "Failed to process");
                            failed.fetch_add(1, Ordering::SeqCst);
                            let mut p = progress.lock().unwrap();
                            p.mark_failed(&input_file);
                            let _ = p.to_file(&progress_path);
                        }
                    }
                }
            });
        }
    });

    let successful = successful_files.load(Ordering::SeqCst);
    let failed = failed_files.load(Ordering::SeqCst);

    info!(
        total = total_files,
        successful, failed, "Parallel batch processing complete"
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_format_ass_time() {
        assert_eq!(format_ass_time(0.0), "0:00:00.00");
        assert_eq!(format_ass_time(5.0), "0:00:05.00");
        assert_eq!(format_ass_time(65.5), "0:01:05.50");
        assert_eq!(format_ass_time(3661.25), "1:01:01.25");
        // Negative values should be clamped to 0
        assert_eq!(format_ass_time(-5.0), "0:00:00.00");
    }

    struct MockFfmpegAnalyzer;
    impl crate::analyzer::VideoAnalyzer for MockFfmpegAnalyzer {
        fn detect_silence(
            &self,
            _path: &Path,
            _threshold_db: f32,
            _duration_s: f32,
        ) -> Result<Vec<crate::analyzer::Segment>> {
            Ok(vec![])
        }
    }

    struct MockFfmpegEditor;
    impl VideoEditor for MockFfmpegEditor {
        fn reframe(&self, _input: &Path, _output: &Path) -> Result<()> {
            Ok(())
        }

        fn blur_background(&self, _input: &Path, _output: &Path) -> Result<()> {
            Ok(())
        }

        fn trim_video(
            &self,
            _input: &Path,
            output: &Path,
            _segments: &[crate::analyzer::ProcessedSegment],
        ) -> Result<()> {
            // Simulate successful trimming by creating an empty output file
            fs::File::create(output)?;
            Ok(())
        }

        fn mix_with_music(
            &self,
            _input: &Path,
            _music: &Path,
            _output: &Path,
            _transcript: &[crate::stt_analyzer::TranscriptSegment],
            _duck_volume: f32,
        ) -> Result<()> {
            Ok(())
        }

        fn enhance_audio(&self, _input: &Path, _output: &Path, _target_lufs: f32) -> Result<()> {
            Ok(())
        }

        fn reduce_noise(&self, _input: &Path, _output: &Path) -> Result<()> {
            Ok(())
        }

        fn stabilize(&self, _input: &Path, _output: &Path) -> Result<()> {
            Ok(())
        }

        fn color_correct(&self, _input: &Path, _output: &Path) -> Result<()> {
            Ok(())
        }
    }

    // Mock DurationGetter for testing purposes
    struct MockDurationGetter;
    impl DurationGetter for MockDurationGetter {
        fn get_duration(&self, _path: &Path) -> Result<f32> {
            Ok(60.0) // Return a dummy duration
        }
    }

    #[test]
    fn test_batch_processing_integration() -> Result<()> {
        let input_dir = tempdir()?;
        let output_dir = tempdir()?;

        // Create dummy video files
        fs::File::create(input_dir.path().join("video1.mp4"))?;
        fs::File::create(input_dir.path().join("video2.mov"))?;
        fs::File::create(input_dir.path().join("document.txt"))?; // Should be ignored

        // Use mock implementations
        let mock_analyzer = MockFfmpegAnalyzer;
        let mock_editor = MockFfmpegEditor;
        let mock_duration_getter = MockDurationGetter;

        // Use config with audio enhancement disabled (mock doesn't create files)
        let mut config = Config::default();
        config.audio.enhance = false;

        let result = process_batch_dir(
            input_dir.path().to_path_buf(),
            output_dir.path().to_path_buf(),
            &config,
            &mock_analyzer,
            &mock_editor,
            &mock_duration_getter,
        );

        assert!(result.is_ok());

        // Check if output files were created
        let output_files: Vec<_> = fs::read_dir(output_dir.path())?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .collect();

        assert_eq!(output_files.len(), 2);
        assert!(output_files.iter().any(|p| p.ends_with("video1.mp4")));
        assert!(output_files.iter().any(|p| p.ends_with("video2.mov")));

        Ok(())
    }
}
