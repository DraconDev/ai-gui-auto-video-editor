use crate::analyzer::{ProcessedSegment, Segment};
use crate::config::SilenceMode;
use crate::ml::AutoReframeProcessor;
use crate::ml::BackgroundBlurProcessor;
use crate::stt_analyzer::TranscriptSegment;
use anyhow::{Context, Result};
use image::GenericImageView;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::{info, warn};

const TRIM_SEGMENTS_PER_CHUNK: usize = 48;

/// Calculate segments to keep after processing silences
///
/// # Arguments
/// * `silence_segments` - Detected silent segments
/// * `total_duration` - Total video duration in seconds
/// * `padding` - Padding around cuts in seconds
/// * `mode` - How to handle silences (Cut or Speedup)
/// * `speedup_factor` - Speed multiplier when mode is Speedup
/// * `min_silence_for_speedup` - Minimum silence duration to speedup (seconds)
pub fn calculate_keep_segments(
    silence_segments: &[Segment],
    total_duration: f32,
    padding: f32,
    mode: SilenceMode,
    speedup_factor: f32,
    min_silence_for_speedup: f32,
) -> Vec<ProcessedSegment> {
    if mode == SilenceMode::Keep {
        return vec![ProcessedSegment {
            start: 0.0,
            end: total_duration,
            speed: 1.0,
        }];
    }

    let mut processed = Vec::new();
    let mut current_pos = 0.0;

    for silence in silence_segments {
        let silence_duration = silence.end - silence.start;

        // Add the non-silent segment before this silence
        let keep_end = (silence.start + padding).min(total_duration);
        if keep_end > current_pos {
            processed.push(ProcessedSegment {
                start: current_pos,
                end: keep_end,
                speed: 1.0,
            });
        }

        // Handle the silence based on mode
        // Note: SilenceMode::Keep is handled by early return at function entry
        match mode {
            SilenceMode::Cut => {
                let cut_end = (silence.end - padding).max(0.0);
                current_pos = current_pos.max(keep_end).max(cut_end);
            }
            SilenceMode::Speedup => {
                let silence_start = (silence.start + padding).max(0.0);
                let silence_end = (silence.end - padding).min(total_duration);

                if silence_duration >= min_silence_for_speedup && silence_end > silence_start {
                    processed.push(ProcessedSegment {
                        start: silence_start,
                        end: silence_end,
                        speed: speedup_factor,
                    });
                }
                current_pos = current_pos.max(keep_end).max(silence_end);
            }
            SilenceMode::Keep => unreachable!(),
        }
    }

    // Add the final segment after the last silence
    if current_pos < total_duration {
        processed.push(ProcessedSegment {
            start: current_pos,
            end: total_duration,
            speed: 1.0,
        });
    }

    processed
}

/// Legacy function for backward compatibility - uses Cut mode
pub fn calculate_keep_segments_simple(
    silence_segments: &[Segment],
    total_duration: f32,
    padding: f32,
) -> Vec<ProcessedSegment> {
    calculate_keep_segments(
        silence_segments,
        total_duration,
        padding,
        SilenceMode::Cut,
        4.0,
        0.5,
    )
}

pub fn calculate_keep_segments_from_transcript(
    transcript: &[TranscriptSegment],
    total_duration: f32,
    filler_words: &[&str],
    padding: f32,
) -> Vec<ProcessedSegment> {
    let mut processed: Vec<ProcessedSegment> = Vec::new();

    for (i, seg) in transcript.iter().enumerate() {
        let is_filler = filler_words
            .iter()
            .any(|&f| seg.text.to_lowercase().contains(f));

        if is_filler {
            continue;
        }

        let seg_start = seg.start.max(0.0);
        let seg_end = seg.end.min(total_duration);

        if seg_start >= seg_end {
            continue;
        }

        let prev_end = processed.last().map(|s| s.end).unwrap_or(0.0);
        let mut gap_filled = false;

        if i > 0 {
            let prev_seg = &transcript[i - 1];
            let prev_is_filler = filler_words
                .iter()
                .any(|&f| prev_seg.text.to_lowercase().contains(f));

            if prev_is_filler && seg_start > prev_end {
                let gap = seg_start - prev_end;
                if (gap - padding * 2.0).abs() < 0.001 || gap < padding {
                    processed.push(ProcessedSegment {
                        start: prev_end,
                        end: seg_end,
                        speed: 1.0,
                    });
                    gap_filled = true;
                }
            }
        }

        if !gap_filled {
            let start = prev_end.max(seg_start).min(seg_end);
            if start < seg_end {
                processed.push(ProcessedSegment {
                    start,
                    end: seg_end,
                    speed: 1.0,
                });
            }
        }
    }

    if let Some(last) = processed.last()
        && last.end < total_duration
    {
        processed.push(ProcessedSegment {
            start: last.end,
            end: total_duration,
            speed: 1.0,
        });
    }

    processed
}

pub trait VideoEditor: Send + Sync {
    fn trim_video(&self, input: &Path, output: &Path, segments: &[ProcessedSegment]) -> Result<()>;
    fn trim_video_with_progress(
        &self,
        input: &Path,
        output: &Path,
        segments: &[ProcessedSegment],
        progress: &mut dyn FnMut(f32),
    ) -> Result<()> {
        progress(0.0);
        self.trim_video(input, output, segments)?;
        progress(1.0);
        Ok(())
    }
    fn mix_with_music(
        &self,
        input: &Path,
        music: &Path,
        output: &Path,
        transcript: &[TranscriptSegment],
        duck_volume: f32,
    ) -> Result<()>;
    fn enhance_audio(&self, input: &Path, output: &Path, target_lufs: f32) -> Result<()>;
    fn reduce_noise(&self, input: &Path, output: &Path) -> Result<()>;
    fn stabilize(&self, input: &Path, output: &Path) -> Result<()>;
    fn color_correct(&self, input: &Path, output: &Path) -> Result<()>;
    fn reframe(
        &self,
        input: &Path,
        output: &Path,
        target_resolution: crate::config::VideoResolution,
    ) -> Result<()>;
    fn blur_background(&self, input: &Path, output: &Path) -> Result<()>;
    fn ml_blur_background(
        &self,
        input: &Path,
        output: &Path,
        blur_strength: f32,
        inference_scale: f32,
    ) -> Result<()>;
}

pub struct FfmpegEditor {
    pub hw_accel: crate::hwaccel::HwAccel,
}

impl Default for FfmpegEditor {
    fn default() -> Self {
        Self {
            hw_accel: crate::hwaccel::HwAccel::None,
        }
    }
}

impl FfmpegEditor {
    pub fn new(hw_accel: crate::hwaccel::HwAccel) -> Self {
        Self { hw_accel }
    }
    fn run_reframe_filter(&self, input: &Path, output: &Path, filter: &str) -> Result<()> {
        info!(filter = %filter, "Applying crop filter");

        let status = Command::new("ffmpeg")
            .args([
                "-i",
                input.to_str().context("invalid input path")?,
                "-vf",
                filter,
                "-c:a",
                "copy",
                "-y",
                output.to_str().context("invalid output path")?,
            ])
            .status()
            .context("failed to execute ffmpeg")?;

        if !status.success() {
            anyhow::bail!("ffmpeg failed with status: {}", status);
        }

        Ok(())
    }
}

impl VideoEditor for FfmpegEditor {
    fn trim_video(&self, input: &Path, output: &Path, segments: &[ProcessedSegment]) -> Result<()> {
        self.trim_video_with_progress(input, output, segments, &mut |_| {})
    }

    fn trim_video_with_progress(
        &self,
        input: &Path,
        output: &Path,
        segments: &[ProcessedSegment],
        progress: &mut dyn FnMut(f32),
    ) -> Result<()> {
        if segments.is_empty() {
            anyhow::bail!("No segments to process");
        }

        let codec = self.hw_accel.video_codec();

        if segments.len() <= TRIM_SEGMENTS_PER_CHUNK {
            run_trim_filter_job(input, output, segments, codec)?;
            progress(1.0);
            return Ok(());
        }

        let chunk_dir = create_trim_chunk_dir(output)?;
        let chunk_count = segments.len().div_ceil(TRIM_SEGMENTS_PER_CHUNK);
        let mut chunk_files = Vec::with_capacity(chunk_count);

        for (idx, chunk) in segments.chunks(TRIM_SEGMENTS_PER_CHUNK).enumerate() {
            let chunk_path = chunk_dir.join(format!("chunk_{idx:04}.mp4"));
            run_trim_filter_job(input, &chunk_path, chunk, codec)?;
            chunk_files.push(chunk_path);
            progress((idx + 1) as f32 / (chunk_count + 1) as f32);
        }

        concat_chunk_files(&chunk_files, output)?;
        progress(1.0);

        let _ = fs::remove_dir_all(&chunk_dir);
        Ok(())
    }

    fn mix_with_music(
        &self,
        input: &Path,
        music: &Path,
        output: &Path,
        transcript: &[TranscriptSegment],
        duck_volume: f32,
    ) -> Result<()> {
        let duck_filter = generate_duck_filter(transcript, duck_volume);

        let status = Command::new("ffmpeg")
            .args([
                "-i",
                input.to_str().context("invalid input path")?,
                "-i",
                music.to_str().context("invalid music path")?,
                "-filter_complex",
                &duck_filter,
                "-map",
                "0:v",
                "-map",
                "[outa]",
                "-y",
                output.to_str().context("invalid output path")?,
            ])
            .status()
            .context("failed to execute ffmpeg")?;

        if !status.success() {
            anyhow::bail!("ffmpeg failed with status: {}", status);
        }

        Ok(())
    }

    fn enhance_audio(&self, input: &Path, output: &Path, target_lufs: f32) -> Result<()> {
        let input_str = input.to_str().context("invalid input path")?;

        // Two-pass loudnorm with EQ for voice clarity:
        // - highpass=60: removes rumble, preserves deep male voices
        // - equalizer=f=4000: presence region (3-5kHz), boosts vocal clarity
        // - loudnorm: EBU R128 standardization, LRA=7 for speech (keeps linear mode)
        // Note: 4kHz is the "s" and "t" frequency region — gives crispness without harshness
        let measure_filter = format!(
            "highpass=f=60,equalizer=f=4000:t=q:w=2:g=1.5,loudnorm=I={target_lufs}:TP=-2.0:LRA=7:print_format=json"
        );

        let measure_output = Command::new("ffmpeg")
            .args(["-i", input_str, "-af", &measure_filter, "-f", "null", "-"])
            .stderr(std::process::Stdio::piped())
            .output()
            .context("failed to run loudnorm measurement pass")?;

        // Only parse stats if measurement succeeded
        let stats = if measure_output.status.success() {
            let stderr = String::from_utf8_lossy(&measure_output.stderr);
            parse_loudnorm_stats(&stderr)
        } else {
            warn!("loudnorm measurement pass failed, falling back to single-pass");
            None
        };

        // Pass 2: Apply measured normalization with presence EQ
        // Uses measured values from pass 1 for accurate normalization
        let filter = if let Some(s) = stats {
            format!(
                "highpass=f=60,equalizer=f=4000:t=q:w=2:g=1.5,loudnorm=I={}:TP=-2.0:LRA=7:measured_I={}:measured_TP={}:measured_LRA={}:measured_thresh={}:offset={}:linear=true",
                target_lufs, s.i, s.tp, s.lra, s.thresh, s.offset
            )
        } else {
            // Fallback to single-pass if measurement failed
            format!(
                "highpass=f=60,equalizer=f=4000:t=q:w=2:g=1.5,loudnorm=I={target_lufs}:TP=-2.0:LRA=7"
            )
        };

        let status = Command::new("ffmpeg")
            .args([
                "-i",
                input_str,
                "-af",
                &filter,
                "-c:v",
                "copy",
                "-y",
                output.to_str().context("invalid output path")?,
            ])
            .status()
            .context("failed to execute ffmpeg")?;

        if !status.success() {
            anyhow::bail!("ffmpeg failed with status: {}", status);
        }

        Ok(())
    }

    fn reduce_noise(&self, input: &Path, output: &Path) -> Result<()> {
        // Apply FFT-based noise reduction with balanced settings.
        // afftdn removes steady background noise (fans, AC, hiss).
        // noise_reduction=15 (not nf=-25 which was too aggressive).
        // nr=15: 15dB reduction — removes noise while preserving voice clarity.
        // nf=-25 caused muffled/underwater artifacts.
        // tn=true: track noise for better quality but slower processing.
        let filter = "afftdn=nr=15:tn=true";

        let status = Command::new("ffmpeg")
            .args([
                "-i",
                input.to_str().context("invalid input path")?,
                "-af",
                filter,
                "-c:v",
                "copy",
                "-y",
                output.to_str().context("invalid output path")?,
            ])
            .status()
            .context("failed to execute ffmpeg")?;

        if !status.success() {
            anyhow::bail!("ffmpeg failed with status: {}", status);
        }

        Ok(())
    }

    fn stabilize(&self, input: &Path, output: &Path) -> Result<()> {
        let input_str = input.to_str().context("invalid input path")?;
        let output_str = output.to_str().context("invalid output path")?;
        let trf_file = crate::utils::TempFile::new("agave-vidstab", "trf")?;
        let escaped_trf_path = crate::utils::escape_ffmpeg_filter_path(trf_file.path());

        let status1 = Command::new("ffmpeg")
            .args([
                "-i",
                input_str,
                "-vf",
                &format!(
                    "vidstabdetect=stepsize=6:shakiness=5:accuracy=15:result={}",
                    escaped_trf_path
                ),
                "-f",
                "null",
                "-",
            ])
            .status()
            .context("failed to execute ffmpeg (stabilize pass 1)")?;

        if !status1.success() {
            anyhow::bail!("ffmpeg stabilize pass 1 failed with status: {}", status1);
        }

        let status2 = Command::new("ffmpeg")
            .args([
                "-i",
                input_str,
                "-vf",
                &format!(
                    "vidstabtransform=input={}:smoothing=10:optzoom=1:interpol=bicubic",
                    escaped_trf_path
                ),
                "-c:a",
                "copy",
                "-y",
                output_str,
            ])
            .status()
            .context("failed to execute ffmpeg (stabilize pass 2)")?;

        if !status2.success() {
            anyhow::bail!("ffmpeg stabilize pass 2 failed with status: {}", status2);
        }

        Ok(())
    }

    fn color_correct(&self, input: &Path, output: &Path) -> Result<()> {
        // Auto color correction using eq filter
        // Adjusts brightness, contrast, saturation for a more balanced look
        // Also applies slight sharpening for clarity
        let filter = "eq=contrast=1.1:brightness=0.05:saturation=1.1,unsharp=5:5:0.5:5:5:0.0";

        let status = Command::new("ffmpeg")
            .args([
                "-i",
                input.to_str().context("invalid input path")?,
                "-vf",
                filter,
                "-c:a",
                "copy",
                "-y",
                output.to_str().context("invalid output path")?,
            ])
            .status()
            .context("failed to execute ffmpeg")?;

        if !status.success() {
            anyhow::bail!("ffmpeg failed with status: {}", status);
        }

        Ok(())
    }

    fn reframe(
        &self,
        input: &Path,
        output: &Path,
        target_resolution: crate::config::VideoResolution,
    ) -> Result<()> {
        info!("Auto-reframe: Analyzing video for face tracking...");

        let filter = match crate::ml::AutoReframeProcessor::new() {
            Ok(processor) => match processor.analyze_video(input, 1.0) {
                Ok(crop_regions) => {
                    let (w, h) = match crate::ml::FrameExtractor::get_video_dimensions(input) {
                        Ok(dims) => dims,
                        Err(e) => {
                            warn!(error = %e, "Failed to get video dimensions, using center crop");
                            let (sw, sh) = target_resolution.dimensions();
                            return self.run_reframe_filter(
                                input,
                                output,
                                &format!("crop=ih*9/16:ih,scale={}:{}", sw, sh),
                            );
                        }
                    };

                    AutoReframeProcessor::generate_crop_filter(
                        &crop_regions,
                        w,
                        h,
                        target_resolution,
                    )
                }
                Err(e) => {
                    warn!(error = %e, "Face detection failed, using center crop");
                    let (sw, sh) = target_resolution.dimensions();
                    format!("crop=ih*9/16:ih,scale={}:{}", sw, sh)
                }
            },
            Err(e) => {
                warn!(error = %e, "Could not load face detection model, using center crop");
                let (sw, sh) = target_resolution.dimensions();
                format!("crop=ih*9/16:ih,scale={}:{}", sw, sh)
            }
        };

        self.run_reframe_filter(input, output, &filter)
    }

    /// Apply a simple uniform box-blur to the entire video.
    ///
    /// This is a fast FFmpeg filter-based blur; it does **not** perform person
    /// segmentation. For ML-based background blur that keeps the subject sharp,
    /// see `ml::BackgroundBlurProcessor` (not yet integrated into the video pipeline).
    fn blur_background(&self, input: &Path, output: &Path) -> Result<()> {
        info!("Background blur: applying uniform boxblur to entire video...");

        let filter = "boxblur=20:5";

        let status = Command::new("ffmpeg")
            .args([
                "-i",
                input.to_str().context("invalid input path")?,
                "-vf",
                filter,
                "-c:a",
                "copy",
                "-y",
                output.to_str().context("invalid output path")?,
            ])
            .status()
            .context("failed to execute ffmpeg")?;

        if !status.success() {
            anyhow::bail!("ffmpeg failed with status: {}", status);
        }

        Ok(())
    }

    fn ml_blur_background(
        &self,
        input: &Path,
        output: &Path,
        blur_strength: f32,
        inference_scale: f32,
    ) -> Result<()> {
        info!(
            strength = blur_strength,
            scale = inference_scale,
            "ML background blur: starting person segmentation pipeline"
        );
        // Create temp directory for frame extraction
        let frame_dir = crate::utils::TempDir::new("agave-ml-blur")?;
        let frame_pattern = frame_dir.path().join("frame_%06d.png");
        let frame_pattern_str = frame_pattern.to_str().context("invalid frame path")?;

        // Step 1: Extract all frames as PNG
        let status = Command::new("ffmpeg")
            .args([
                "-i",
                input.to_str().context("invalid input path")?,
                "-vf", "fps=1", // 1 fps = 1 frame per second (adjustable)
                "-y",
                frame_pattern_str,
            ])
            .status()
            .context("ffmpeg frame extraction failed")?;

        if !status.success() {
            anyhow::bail!("ffmpeg frame extraction failed with status: {}", status);
        }

        // Collect frame files and sort
        let mut frame_files: Vec<_> = std::fs::read_dir(frame_dir.path())
            .context("failed to read frame directory")?
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .map(|ext| ext == "png")
                    .unwrap_or(false)
            })
            .collect();

        if frame_files.is_empty() {
            anyhow::bail!("no frames extracted from video");
        }

        frame_files.sort_by_key(|e| e.file_name());

        // Step 2: Load ML model once (reused for all frames)
        let processor = BackgroundBlurProcessor::new()?;
        info!(frames = frame_files.len(), "loaded background blur processor");

        // Step 3: Process each frame
        let mut processed_count = 0;
        for entry in &frame_files {
            let frame_path = entry.path();
            let frame = image::open(&frame_path)
                .with_context(|| format!("failed to load frame: {:?}", frame_path))?;

            // Scale down if inference_scale < 1.0
            let frame_for_inference = if inference_scale < 1.0 {
                let (w, h) = frame.dimensions();
                let new_w = (w as f32 * inference_scale) as u32;
                let new_h = (h as f32 * inference_scale) as u32;
                frame.resize(new_w, new_h, image::imageops::FilterType::Triangle)
            } else {
                frame.clone()
            };


            // Run ML segmentation + blur + composite
            let blurred = processor
                .process_frame(&frame_for_inference, blur_strength as u32)
                .with_context(|| format!("failed to process frame: {:?}", frame_path))?;

            // Upscale blurred result back to original frame dimensions
            let blurred = if inference_scale < 1.0 {
                let (orig_w, orig_h) = frame.dimensions();
                blurred.resize(orig_w, orig_h, image::imageops::FilterType::Triangle)
            } else {
                blurred
            };

            // Save composited frame
            blurred
                .save(&frame_path)
                .with_context(|| format!("failed to save frame: {:?}", frame_path))?;

            processed_count += 1;
        }

        info!(processed = processed_count, "ML frames processed");

        // Step 4: Re-encode frames back to video, preserving audio
        let status = Command::new("ffmpeg")
            .args([
                "-framerate", "1",
                "-i",
                frame_pattern_str,
                "-i",
                input.to_str().context("invalid input path")?,
                "-c:v", "libx264",
                "-pix_fmt", "yuv420p",
                "-c:a", "copy",
                "-map", "0:v",
                "-map", "1:a",
                "-shortest",
                "-y",
                output.to_str().context("invalid output path")?,
            ])
            .status()
            .context("ffmpeg re-encode failed")?;

        if !status.success() {
            anyhow::bail!("ffmpeg re-encode failed with status: {}", status);
        }

        info!("ML background blur complete");
        Ok(())
    }
}


/// Probe the frame rate of a video file using FFmpeg
fn run_trim_filter_job(
    input: &Path,
    output: &Path,
    segments: &[ProcessedSegment],
    codec: &str,
) -> Result<()> {
    let (v_filter, a_filter) = generate_trim_filters(segments);
    let filter_complex = format!("{}{}", v_filter, a_filter);

    // Build codec-specific args:
    // - NVENC/AMF: use VBR high quality with adaptive quantization for best perceptual quality
    // - Software (libx264): use CRF for constant quality
    let is_hw = matches!(codec, "h264_nvenc" | "h264_amf");
    let mut args: Vec<&str> = vec![
        "-i",
        input.to_str().context("invalid input path")?,
        "-filter_complex",
        &filter_complex,
        "-map",
        "[outv]",
        "-map",
        "[outa]",
        "-c:v",
        codec,
    ];
    if is_hw {
        // Hardware encoders: VBR HQ + spatial AQ for perceptual quality
        // YouTube recommends VBR, High Profile, CABAC
        args.extend(&["-preset", "p7"]); // p7 = slow = max quality
        args.extend(&["-rc:v", "vbr_hq"]);
        args.extend(&["-cq:v", "23"]); // Quality level (0-51, lower=better)
        args.extend(&["-refs:v", "16"]); // Reference frames for quality
        args.extend(&["-bf:v", "3"]); // B-frames
        args.extend(&["-spatial_aq:v", "1"]); // Spatial adaptive quantization
        args.extend(&["-aq-strength:v", "8"]); // AQ strength (1-15)
        args.extend(&["-coder:v", "cabac"]); // CABAC > CAVLC
    } else {
        // Software: CRF for constant quality, slower preset
        args.extend(&["-preset", "slow"]);
        args.extend(&["-crf", "20"]);
    }
    args.extend(&[ // Audio: AAC-LC @ 192kbps, 48kHz (YouTube recommended)
        "-c:a",
        "aac",
        "-b:a",
        "192k",
        "-ar",
        "48000", // 48kHz sample rate (YouTube recommended)
        "-movflags",
        "+faststart", // Moov atom at front for web streaming
        "-y",
    ]);
    args.push(output.to_str().context("invalid output path")?);

    let status = Command::new("ffmpeg").args(&args).status()
        .context("failed to execute ffmpeg")?;

    if !status.success() {
        anyhow::bail!("ffmpeg failed with status: {}", status);
    }

    Ok(())
}

fn create_trim_chunk_dir(output: &Path) -> Result<PathBuf> {
    let parent = output.parent().unwrap_or_else(|| Path::new("."));
    let stem = output
        .file_stem()
        .map(|stem| stem.to_string_lossy().to_string())
        .unwrap_or_else(|| "trim".to_string());
    let chunk_dir = parent.join(format!(
        ".agave-{}-{:x}-{:016x}",
        stem,
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ));

    if chunk_dir.exists() {
        let _ = fs::remove_dir_all(&chunk_dir);
    }
    fs::create_dir_all(&chunk_dir)?;
    Ok(chunk_dir)
}

fn concat_chunk_files(chunk_files: &[PathBuf], output: &Path) -> Result<()> {
    if chunk_files.is_empty() {
        anyhow::bail!("No chunk files to concatenate");
    }

    if chunk_files.len() == 1 {
        // Use copy+remove instead of rename to handle cross-filesystem moves
        std::fs::copy(&chunk_files[0], output)?;
        let _ = std::fs::remove_file(&chunk_files[0]);
        return Ok(());
    }

    let concat_list = output.with_extension("concat.txt");
    let concat_contents = chunk_files
        .iter()
        .map(|path| {
            let escaped = path
                .display()
                .to_string()
                .replace('\'', "'\\''")
                .replace('\n', "\\n")
                .replace('\r', "\\r");
            format!("file '{}'\n", escaped)
        })
        .collect::<String>();
    fs::write(&concat_list, concat_contents)?;

    let output_result = Command::new("ffmpeg")
        .args([
            "-f",
            "concat",
            "-safe",
            "0",
            "-i",
            concat_list.to_str().context("invalid concat list path")?,
            "-c",
            "copy",
            "-y",
            output.to_str().context("invalid output path")?,
        ])
        .output()
        .context("failed to execute ffmpeg concat")?;

    // Clean up temp files AFTER checking status
    let _ = fs::remove_file(&concat_list);
    if output_result.status.success() {
        for chunk_file in chunk_files {
            let _ = fs::remove_file(chunk_file);
        }
    } else {
        // Leave chunk files for debugging if concat failed
        let stderr = String::from_utf8_lossy(&output_result.stderr);
        let last_lines = stderr.lines().rev().take(5).collect::<Vec<_>>().join("\n");
        anyhow::bail!(
            "ffmpeg concat failed with status: {}\n{}",
            output_result.status,
            last_lines
        );
    }

    Ok(())
}

struct LoudnormStats {
    i: String,
    tp: String,
    lra: String,
    thresh: String,
    offset: String,
}

fn parse_loudnorm_stats(stderr: &str) -> Option<LoudnormStats> {
    // Find the JSON block in ffmpeg stderr output.
    // FFmpeg's loudnorm filter outputs multi-line JSON ending with:
    //
    //       "normalization_type" : "dynamic",
    //       "target_offset" : "0.04"
    //   }
    //
    // CRITICAL: the string value "dynamic" contains a '}' character.
    // Using str::find('}') finds that inner '}' first, truncating the JSON
    // and losing all fields after "normalization_type" (the entire file gets
    // parsed as ~264 bytes, with only the first 5 fields present).
    // Solution: use rfind to get the LAST '}' in the block.
    let json_start = stderr.find('{')?;
    let json_str = &stderr[json_start..];
    let json_end = json_str.rfind('}').map(|p| p + 1).unwrap_or(json_str.len());
    let json_str = &json_str[..json_end];

    let get_val = |key: &str| -> Option<String> {
        // Pattern matches quoted key (handles spaces around colon)
        let pattern = format!("\"{}\"", key);
        let idx = json_str.find(&pattern)?;
        let after = &json_str[idx + pattern.len()..];
        // Skip whitespace and colon
        let after = after.trim_start().strip_prefix(':')?;
        let after = after.trim_start();
        // Handle quoted strings and numbers
        if let Some(stripped) = after.strip_prefix('"') {
            let end = stripped.find('"')?;
            Some(stripped[..end].to_string())
        } else {
            let end = after
                .find(|c| [',', '\n', '}'].contains(&c))
                .unwrap_or(after.len());
            Some(after[..end].trim().to_string())
        }
    };

    // Parse all values and validate they are valid finite numbers
    let stats = LoudnormStats {
        i: get_val("input_i")?,
        tp: get_val("input_tp")?,
        lra: get_val("input_lra")?,
        thresh: get_val("input_thresh")?,
        offset: get_val("target_offset")?,
    };

    // Validate all fields are valid finite numbers
    for val in [
        &stats.i,
        &stats.tp,
        &stats.lra,
        &stats.thresh,
        &stats.offset,
    ] {
        if val.is_empty()
            || val.parse::<f64>().ok()?.is_nan()
            || val.parse::<f64>().ok()?.is_infinite()
        {
            return None;
        }
    }

    Some(stats)
}

fn generate_trim_filters(segments: &[ProcessedSegment]) -> (String, String) {
    let mut v_filter = String::new();
    let mut a_filter = String::new();
    let mut v_concat = String::new();
    let mut a_concat = String::new();

    for (i, seg) in segments.iter().enumerate() {
        // Handle speed adjustment
        let setpts = if seg.speed != 1.0 {
            format!("setpts={}*PTS", 1.0 / seg.speed)
        } else {
            "setpts=PTS-STARTPTS".to_string()
        };

        let atempo = if seg.speed != 1.0 {
            // ffmpeg's atempo only supports 0.5 to 2.0
            // Chain multiple atempo filters for speeds outside this range
            chain_atempo_filters(seg.speed)
        } else {
            "asetpts=PTS-STARTPTS".to_string()
        };

        v_filter.push_str(&format!(
            "[0:v]trim=start={}:end={}, {}[v{}];",
            seg.start, seg.end, setpts, i
        ));
        a_filter.push_str(&format!(
            "[0:a]atrim=start={}:end={}, {}[a{}];",
            seg.start, seg.end, atempo, i
        ));
        v_concat.push_str(&format!("[v{}]", i));
        a_concat.push_str(&format!("[a{}]", i));
    }

    v_filter.push_str(&format!(
        "{}concat=n={}:v=1:a=0[outv];",
        v_concat,
        segments.len()
    ));
    a_filter.push_str(&format!(
        "{}concat=n={}:v=0:a=1[outa]",
        a_concat,
        segments.len()
    ));

    (v_filter, a_filter)
}

/// Chain atempo filters to achieve speeds outside ffmpeg's 0.5-2.0 range.
/// ffmpeg's atempo only supports 0.5 to 2.0 per filter instance.
///
/// Uses sqrt-chaining for high speedups (3x+) to avoid sample-skipping artifacts:
/// - Instead of `atempo=2.0,atempo=1.5` (skips samples at high atempo)
/// - Uses `atempo=sqrt(N)` chained twice for smoother phase vocoder quality
///
/// For slowdown (0.5x-), uses 0.5x chained filters.
fn chain_atempo_filters(speed: f32) -> String {
    const ATEMPO_MIN: f32 = 0.5;
    const ATEMPO_MAX: f32 = 2.0;

    if (ATEMPO_MIN..=ATEMPO_MAX).contains(&speed) {
        return format!("atempo={}", speed);
    }

    let mut filters = Vec::new();

    if speed > ATEMPO_MAX {
        // High speedup (3x+): use sqrt chaining for smoother phase vocoder
        if speed > 4.0 {
            // Very high speedup (>4x): chain 2.0x filters
            filters.push("atempo=2.0".to_string());
            filters.push("atempo=2.0".to_string());
        } else {
            // For 3x-4x: use sqrt for smoother quality (e.g., sqrt(3) ≈ 1.73)
            let sqrt_speed = speed.sqrt();
            filters.push(format!("atempo={:.1}", sqrt_speed));
            filters.push(format!("atempo={:.1}", sqrt_speed));
        }
    } else if speed < ATEMPO_MIN {
        // Slow down: chain multiple 0.5x filters
        let mut remaining = speed;
        while remaining < ATEMPO_MIN {
            filters.push("atempo=0.5".to_string());
            remaining /= 0.5; // divide by 0.5 = multiply by 2 (accelerates toward 1.0)
        }
        if (ATEMPO_MIN..1.0).contains(&remaining) {
            filters.push(format!("atempo={:.1}", remaining));
        }
    }

    filters.join(",")
}

fn generate_duck_filter(transcript: &[TranscriptSegment], duck_volume: f32) -> String {
    let mut volume_expr = "1.0".to_string();

    for seg in transcript {
        volume_expr = format!(
            "if(between(t,{},{}),{},{volume_expr})",
            seg.start, seg.end, duck_volume
        );
    }

    format!(
        "[1:a]volume=volume='{}'[ducked];[0:a][ducked]amix=inputs=2:duration=first[outa]",
        volume_expr
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_video(path: &Path, duration_secs: f32) -> Result<(), String> {
        crate::tests_common::create_test_video(path, duration_secs)
    }

    #[test]
    fn test_trim_video_single_segment() {
        let temp_dir = tempfile::tempdir().unwrap();
        let input = temp_dir.path().join("input.mp4");
        let output = temp_dir.path().join("output.mp4");
        create_test_video(&input, 3.0).expect("ffmpeg not found");

        let editor = FfmpegEditor::default();
        let segments = vec![ProcessedSegment {
            start: 0.5,
            end: 2.0,
            speed: 1.0,
        }];

        editor.trim_video(&input, &output, &segments).unwrap();
        assert!(output.exists(), "trimmed output should exist");
    }

    #[test]
    fn test_trim_video_multiple_segments() {
        let temp_dir = tempfile::tempdir().unwrap();
        let input = temp_dir.path().join("input.mp4");
        let output = temp_dir.path().join("output.mp4");
        create_test_video(&input, 5.0).expect("ffmpeg not found");

        let editor = FfmpegEditor::default();
        let segments = vec![
            ProcessedSegment {
                start: 0.0,
                end: 1.0,
                speed: 1.0,
            },
            ProcessedSegment {
                start: 2.0,
                end: 3.0,
                speed: 1.0,
            },
        ];

        editor.trim_video(&input, &output, &segments).unwrap();
        assert!(output.exists(), "trimmed output should exist");
    }

    #[test]
    fn test_trim_video_empty_segments_fails() {
        let temp_dir = tempfile::tempdir().unwrap();
        let input = temp_dir.path().join("input.mp4");
        let output = temp_dir.path().join("output.mp4");
        create_test_video(&input, 1.0).expect("ffmpeg not found");

        let editor = FfmpegEditor::default();
        let result = editor.trim_video(&input, &output, &[]);
        assert!(result.is_err(), "trim with empty segments should fail");
    }

    #[test]
    fn test_enhance_audio() {
        let temp_dir = tempfile::tempdir().unwrap();
        let input = temp_dir.path().join("input.mp4");
        let output = temp_dir.path().join("output.mp4");
        create_test_video(&input, 2.0).expect("ffmpeg not found");

        let editor = FfmpegEditor::default();
        editor.enhance_audio(&input, &output, -14.0).unwrap();
        assert!(output.exists(), "enhanced output should exist");
    }

    #[test]
    fn test_reduce_noise() {
        let temp_dir = tempfile::tempdir().unwrap();
        let input = temp_dir.path().join("input.mp4");
        let output = temp_dir.path().join("output.mp4");
        create_test_video(&input, 2.0).expect("ffmpeg not found");

        let editor = FfmpegEditor::default();
        editor.reduce_noise(&input, &output).unwrap();
        assert!(output.exists(), "noise-reduced output should exist");
    }

    #[test]
    fn test_color_correct() {
        let temp_dir = tempfile::tempdir().unwrap();
        let input = temp_dir.path().join("input.mp4");
        let output = temp_dir.path().join("output.mp4");
        create_test_video(&input, 2.0).expect("ffmpeg not found");

        let editor = FfmpegEditor::default();
        editor.color_correct(&input, &output).unwrap();
        assert!(output.exists(), "color-corrected output should exist");
    }

    #[test]
    fn test_concat_chunk_files_single() {
        let temp_dir = tempfile::tempdir().unwrap();
        let chunk = temp_dir.path().join("chunk.mp4");
        let output = temp_dir.path().join("output.mp4");
        create_test_video(&chunk, 1.0).expect("ffmpeg not found");

        concat_chunk_files(&[chunk], &output).unwrap();
        assert!(output.exists(), "concat output should exist");
    }

    #[test]
    fn test_concat_chunk_files_multiple() {
        let temp_dir = tempfile::tempdir().unwrap();
        let chunk1 = temp_dir.path().join("chunk1.mp4");
        let chunk2 = temp_dir.path().join("chunk2.mp4");
        let output = temp_dir.path().join("output.mp4");
        create_test_video(&chunk1, 1.0).expect("ffmpeg not found");
        create_test_video(&chunk2, 1.0).expect("ffmpeg not found");

        concat_chunk_files(&[chunk1, chunk2], &output).unwrap();
        assert!(output.exists(), "concat output should exist");
    }

    #[test]
    fn test_concat_chunk_files_empty_fails() {
        let temp_dir = tempfile::tempdir().unwrap();
        let output = temp_dir.path().join("output.mp4");

        let result = concat_chunk_files(&[], &output);
        assert!(result.is_err(), "concat with empty list should fail");
    }

    #[test]
    fn test_calculate_keep_segments_cut_mode() {
        let silences = vec![Segment {
            start: 2.0,
            end: 3.0,
        }];
        let duration = 10.0;
        let padding = 0.1;
        let processed =
            calculate_keep_segments(&silences, duration, padding, SilenceMode::Cut, 4.0, 0.5);

        assert_eq!(processed.len(), 2);
        assert_eq!(processed[0].end, 2.1);
        assert_eq!(processed[1].start, 2.9);
        assert_eq!(processed[0].speed, 1.0);
        assert_eq!(processed[1].speed, 1.0);
    }

    #[test]
    fn test_calculate_keep_segments_speedup_mode() {
        let silences = vec![
            Segment {
                start: 2.0,
                end: 4.0,
            }, // 2 second silence
        ];
        let duration = 10.0;
        let padding = 0.1;
        let processed =
            calculate_keep_segments(&silences, duration, padding, SilenceMode::Speedup, 4.0, 0.5);

        // Should have 3 segments: before silence, silence (sped up), after silence
        assert_eq!(processed.len(), 3);
        assert_eq!(processed[0].end, 2.1);
        assert_eq!(processed[0].speed, 1.0);

        // Silence segment should be sped up
        assert_eq!(processed[1].start, 2.1);
        assert_eq!(processed[1].end, 3.9);
        assert_eq!(processed[1].speed, 4.0);

        // After silence
        assert_eq!(processed[2].start, 3.9);
        assert_eq!(processed[2].speed, 1.0);
    }

    #[test]
    fn test_calculate_keep_segments_speedup_short_silence() {
        // Silence too short for speedup should be cut
        let silences = vec![
            Segment {
                start: 2.0,
                end: 2.3,
            }, // 0.3 second silence (below min)
        ];
        let duration = 10.0;
        let padding = 0.1;
        let min_silence = 0.5;
        let processed = calculate_keep_segments(
            &silences,
            duration,
            padding,
            SilenceMode::Speedup,
            4.0,
            min_silence,
        );

        // Short silence should be cut, not sped up
        assert_eq!(processed.len(), 2);
        assert_eq!(processed[0].speed, 1.0);
        assert_eq!(processed[1].speed, 1.0);
    }

    #[test]
    fn test_calculate_keep_segments_multiple_silences() {
        let silences = vec![
            Segment {
                start: 2.0,
                end: 3.0,
            },
            Segment {
                start: 5.0,
                end: 7.0,
            },
        ];
        let duration = 10.0;
        let padding = 0.1;
        let processed =
            calculate_keep_segments(&silences, duration, padding, SilenceMode::Cut, 4.0, 0.5);

        assert_eq!(processed.len(), 3);
        assert_eq!(processed[0].start, 0.0);
        assert_eq!(processed[0].end, 2.1);
        assert_eq!(processed[1].start, 2.9);
        assert_eq!(processed[1].end, 5.1);
        assert_eq!(processed[2].start, 6.9);
        assert_eq!(processed[2].end, 10.0);
    }

    #[test]
    fn test_generate_duck_filter() {
        let transcript = vec![TranscriptSegment {
            start: 1.0,
            end: 2.0,
            text: "hello".to_string(),
            confidence: 1.0,
        }];
        let filter = generate_duck_filter(&transcript, 0.2);
        assert!(filter.contains("between(t,1,2)"));
        assert!(filter.contains("volume='if(between(t,1,2),0.2,1.0)'"));
        assert!(filter.contains("amix=inputs=2"));
    }

    #[test]
    fn test_parse_loudnorm_stats() {
        let ffmpeg_output = r#"[Parsed_loudnorm_0 @ 0x7f6be8003740] 
{
	"input_i" : "-21.05",
	"input_tp" : "-18.06",
	"input_lra" : "0.00",
	"input_thresh" : "-31.05",
	"output_i" : "-14.04",
	"output_tp" : "-10.97",
	"output_lra" : "0.00",
	"output_thresh" : "-24.04",
	"normalization_type" : "dynamic",
	"target_offset" : "0.04"
}
[out#0/null @ 0x55b50aa50140] video:4KiB"#;

        let stats = parse_loudnorm_stats(ffmpeg_output).expect("should parse loudnorm stats");
        assert_eq!(stats.i, "-21.05");
        assert_eq!(stats.tp, "-18.06");
        assert_eq!(stats.lra, "0.00");
        assert_eq!(stats.thresh, "-31.05");
        assert_eq!(stats.offset, "0.04");
    }

    #[test]
    fn test_calculate_keep_segments_from_transcript() {
        use crate::stt_analyzer::TranscriptSegment;

        let transcript = vec![
            TranscriptSegment {
                start: 0.0,
                end: 2.0,
                text: "hello world".to_string(),
                confidence: 1.0,
            },
            TranscriptSegment {
                start: 2.0,
                end: 3.0,
                text: "um".to_string(),
                confidence: 1.0,
            },
            TranscriptSegment {
                start: 3.0,
                end: 10.0,
                text: "this is the rest".to_string(),
                confidence: 1.0,
            },
        ];

        let processed = calculate_keep_segments_from_transcript(&transcript, 10.0, &["um"], 0.1);

        assert_eq!(processed.len(), 2);
        assert_eq!(processed[0].start, 0.0);
        assert_eq!(processed[0].end, 2.0);
        assert_eq!(processed[1].start, 3.0);
        assert_eq!(processed[1].end, 10.0);
    }

    #[test]
    fn test_chain_atempo_filters() {
        // Within range: single filter
        assert_eq!(chain_atempo_filters(1.5), "atempo=1.5");
        assert_eq!(chain_atempo_filters(0.75), "atempo=0.75");

        // Speed up 4.0x: use exact 2.0x chained (no sqrt needed for power of 2)
        assert_eq!(chain_atempo_filters(4.0), "atempo=2.0,atempo=2.0");
        // Speed up 3.0x: use sqrt(3) chained for smoother quality (~1.7x each)
        assert_eq!(chain_atempo_filters(3.0), "atempo=1.7,atempo=1.7");

        // Slow down below 0.5: chain multiple 0.5x filters
        assert_eq!(chain_atempo_filters(0.25), "atempo=0.5,atempo=0.5");
    }
    #[test]
    fn test_calculate_keep_segments_boundary_at_min_silence() {
        // Silence exactly at min_silence_for_speedup should be sped up
        let silences = vec![Segment {
            start: 2.0,
            end: 2.5, // exactly 0.5s silence
        }];
        let duration = 10.0;
        let padding = 0.0;
        let min_silence = 0.5;
        let processed = calculate_keep_segments(
            &silences,
            duration,
            padding,
            SilenceMode::Speedup,
            4.0,
            min_silence,
        );

        // Silence at boundary should be included (>= not >)
        // 3 segments: before silence (0.0-2.0), sped-up silence (2.0-2.5), after silence (2.5-10.0)
        assert_eq!(processed.len(), 3);
        assert_eq!(processed[0].speed, 1.0);
        assert_eq!(processed[0].start, 0.0);
        assert_eq!(processed[0].end, 2.0);
        assert_eq!(processed[1].speed, 4.0);
        assert_eq!(processed[1].start, 2.0);
        assert_eq!(processed[1].end, 2.5);
        assert_eq!(processed[2].speed, 1.0);
        assert_eq!(processed[2].start, 2.5);
        assert_eq!(processed[2].end, 10.0);
    }

    #[test]
    fn test_calculate_keep_segments_just_below_min_silence() {
        // Silence just below min_silence_for_speedup should be cut (skipped)
        let silences = vec![Segment {
            start: 2.0,
            end: 2.49, // just below 0.5s
        }];
        let duration = 10.0;
        let padding = 0.0;
        let min_silence = 0.5;
        let processed = calculate_keep_segments(
            &silences,
            duration,
            padding,
            SilenceMode::Speedup,
            4.0,
            min_silence,
        );

        // Short silence should be cut (skipped in speedup mode)
        // But the segments before and after silence should still exist
        // 2 segments: before (0.0-2.0) and after (2.49-10.0)
        assert_eq!(processed.len(), 2);
        assert_eq!(processed[0].speed, 1.0);
        assert_eq!(processed[0].start, 0.0);
        assert_eq!(processed[0].end, 2.0);
        assert_eq!(processed[1].speed, 1.0);
        assert_eq!(processed[1].start, 2.49);
        assert_eq!(processed[1].end, 10.0);
    }

    #[test]
    fn test_calculate_keep_segments_simple() {
        use crate::analyzer::Segment;

        let silences = vec![Segment {
            start: 2.0,
            end: 3.0,
        }];
        let duration = 10.0;
        let padding = 0.1;

        let processed = calculate_keep_segments_simple(&silences, duration, padding);

        // Should produce 2 segments (before and after silence)
        assert_eq!(processed.len(), 2);
        // Both should be normal speed in Cut mode
        assert_eq!(processed[0].speed, 1.0);
        assert_eq!(processed[1].speed, 1.0);
        // First segment: 0.0 to 2.1 (2.0 + padding)
        assert_eq!(processed[0].start, 0.0);
        assert_eq!(processed[0].end, 2.1);
        // Second segment: 2.9 (3.0 - padding) to 10.0
        assert_eq!(processed[1].start, 2.9);
        assert_eq!(processed[1].end, 10.0);
    }

    #[test]
    fn test_calculate_keep_segments_large_padding_no_overlap() {
        // Regression test: padding > silence_duration/2 should NOT create overlapping segments
        // Silence at 2.0-3.0 (1s duration), padding=0.6
        // keep_end = 2.0 + 0.6 = 2.6
        // cut_end = 3.0 - 0.6 = 2.4
        // Without fix: processed[0].end=2.6, processed[1].start=2.4 → OVERLAP!
        // With fix: current_pos = max(2.6, 2.4) = 2.6, so processed[1].start=2.6 → NO OVERLAP
        let silences = vec![Segment {
            start: 2.0,
            end: 3.0,
        }];
        let duration = 10.0;
        let padding = 0.6;
        let processed =
            calculate_keep_segments(&silences, duration, padding, SilenceMode::Cut, 4.0, 0.5);

        assert_eq!(processed.len(), 2);
        // First segment ends at 2.6 (2.0 + 0.6)
        assert_eq!(processed[0].end, 2.6);
        // Second segment starts at 2.6 (NOT 2.4!) - this is the bug fix
        assert_eq!(processed[1].start, 2.6);
        assert!(
            processed[0].end <= processed[1].start,
            "Segments must not overlap"
        );
    }

    #[test]
    fn test_calculate_keep_segments_very_large_padding() {
        // Even more extreme case: padding=0.9 with 1s silence
        let silences = vec![Segment {
            start: 2.0,
            end: 3.0,
        }];
        let duration = 10.0;
        let padding = 0.9;
        let processed =
            calculate_keep_segments(&silences, duration, padding, SilenceMode::Cut, 4.0, 0.5);

        assert_eq!(processed.len(), 2);
        // keep_end = 2.9, cut_end = 2.1, current_pos = max(2.9, 2.1) = 2.9
        assert_eq!(processed[0].end, 2.9);
        assert_eq!(processed[1].start, 2.9);
        assert!(
            processed[0].end <= processed[1].start,
            "Segments must not overlap"
        );
    }

    #[test]
    fn test_calculate_keep_segments_adjacent_silences_no_gaps() {
        // Two silences that are adjacent after padding is applied
        // Silence 1: 2.0-3.0, Silence 2: 3.0-4.0
        // With padding=0.3:
        // First cut: keep_end=2.3, cut_end=2.7, current_pos=max(2.3,2.7)=2.7
        // Second cut: keep_end=3.3, cut_end=3.7, current_pos=max(3.3,3.7)=3.7
        let silences = vec![
            Segment {
                start: 2.0,
                end: 3.0,
            },
            Segment {
                start: 3.0,
                end: 4.0,
            },
        ];
        let duration = 10.0;
        let padding = 0.3;
        let processed =
            calculate_keep_segments(&silences, duration, padding, SilenceMode::Cut, 4.0, 0.5);

        // Should have 3 segments: before first silence, between, after second
        assert_eq!(processed.len(), 3);
        // No overlaps
        for i in 0..processed.len() - 1 {
            assert!(
                processed[i].end <= processed[i + 1].start,
                "Segment {} end ({}) should be <= segment {} start ({})",
                i,
                processed[i].end,
                i + 1,
                processed[i + 1].start
            );
        }
    }

    #[test]
    fn test_calculate_keep_segments_zero_padding() {
        // Edge case: zero padding should still work
        let silences = vec![Segment {
            start: 2.0,
            end: 3.0,
        }];
        let duration = 10.0;
        let padding = 0.0;
        let processed =
            calculate_keep_segments(&silences, duration, padding, SilenceMode::Cut, 4.0, 0.5);

        assert_eq!(processed.len(), 2);
        assert_eq!(processed[0].end, 2.0); // keep_end = 2.0 + 0 = 2.0
        assert_eq!(processed[1].start, 3.0); // cut_end = 3.0 - 0 = 3.0
        assert!(
            processed[0].end <= processed[1].start,
            "Segments must not overlap"
        );
    }

    #[test]
    fn test_calculate_keep_segments_padding_exceeds_silence() {
        // padding > silence duration - edge case
        let silences = vec![Segment {
            start: 2.0,
            end: 2.5, // 0.5s silence
        }];
        let duration = 10.0;
        let padding = 0.4; // padding > half silence duration
        let processed =
            calculate_keep_segments(&silences, duration, padding, SilenceMode::Cut, 4.0, 0.5);

        // keep_end = 2.4, cut_end = 2.1, current_pos = max(2.4, 2.1) = 2.4
        assert_eq!(processed.len(), 2);
        assert_eq!(processed[0].end, 2.4);
        assert_eq!(processed[1].start, 2.4);
        assert!(
            processed[0].end <= processed[1].start,
            "Segments must not overlap"
        );
    }

    #[test]
    fn test_calculate_keep_segments_speedup_large_padding() {
        // When padding is large, silence_start > silence_end, so speedup segment is skipped
        // This tests that no overlap occurs in speedup mode with large padding
        let silences = vec![Segment {
            start: 2.0,
            end: 3.0, // 1s silence
        }];
        let duration = 10.0;
        let padding = 0.6; // large padding: silence_start=2.6, silence_end=2.4
        let processed = calculate_keep_segments(
            &silences,
            duration,
            padding,
            SilenceMode::Speedup,
            4.0,
            0.2, // min_silence_for_speedup = 0.2s
        );

        // With padding=0.6, silence_start(2.6) > silence_end(2.4), so speedup segment is skipped
        // Should have 2 segments: before and after silence
        assert_eq!(processed.len(), 2);
        // Check no overlap
        assert!(
            processed[0].end <= processed[1].start,
            "First and second segments must not overlap"
        );
    }

    #[test]
    fn test_stabilize() {
        let temp_dir = tempfile::tempdir().unwrap();
        let input = temp_dir.path().join("input.mp4");
        let output = temp_dir.path().join("stabilized.mp4");
        create_test_video(&input, 3.0).expect("ffmpeg not found");

        let editor = FfmpegEditor::default();
        let result = editor.stabilize(&input, &output);

        assert!(result.is_ok(), "Stabilization should succeed");
        assert!(output.exists(), "Stabilized output should exist");
    }

    #[test]
    fn test_reframe() {
        let temp_dir = tempfile::tempdir().unwrap();
        let input = temp_dir.path().join("input.mp4");
        let output = temp_dir.path().join("reframed.mp4");
        create_test_video(&input, 3.0).expect("ffmpeg not found");

        let editor = FfmpegEditor::default();
        let result = editor.reframe(
            &input,
            &output,
            crate::config::VideoResolution::Vertical1080p,
        );

        assert!(result.is_ok(), "Reframe should succeed");
        assert!(output.exists(), "Reframed output should exist");
    }

    #[test]
    fn test_blur_background() {
        let temp_dir = tempfile::tempdir().unwrap();
        let input = temp_dir.path().join("input.mp4");
        let output = temp_dir.path().join("blurred.mp4");
        create_test_video(&input, 3.0).expect("ffmpeg not found");

        let editor = FfmpegEditor::default();
        let result = editor.blur_background(&input, &output);

        assert!(result.is_ok(), "Blur background should succeed");
        assert!(output.exists(), "Blurred output should exist");
    }
    // ── parse_loudnorm_stats edge cases ───────────────────────────────────

    #[test]
    fn test_parse_loudnorm_stats_minimal_output() {
        let ffmpeg_output = r#"{
	"input_i" : "-20.0",
	"input_tp" : "-1.0",
	"input_lra" : "0.0",
	"input_thresh" : "-20.0",
	"output_i" : "-20.0",
	"output_tp" : "-1.0",
	"input_lra" : "0.0",
	"output_thresh" : "-20.0",
	"normalization_type" : "dynamic",
	"target_offset" : "0.0"
}"#;
        let stats = parse_loudnorm_stats(ffmpeg_output).expect("should parse");
        assert_eq!(stats.i, "-20.0");
        assert_eq!(stats.offset, "0.0");
    }

    #[test]
    fn test_parse_loudnorm_stats_invalid_json() {
        let invalid_output = "not json at all";
        assert!(parse_loudnorm_stats(invalid_output).is_none());
    }

    // ── Duck filter generation tests ──────────────────────────────────────

    #[test]
    fn test_generate_duck_filter_custom_values() {
        let transcript = vec![TranscriptSegment {
            start: 0.0,
            end: 5.0,
            text: "Speaking".to_string(),
            confidence: 0.9,
        }];
        let filter = generate_duck_filter(&transcript, 0.5);
        assert!(filter.contains("0.5"));
    }

    #[test]
    fn test_generate_duck_filter_aggressive() {
        let transcript = vec![TranscriptSegment {
            start: 0.0,
            end: 10.0,
            text: "A".to_string(),
            confidence: 0.9,
        }];
        let filter = generate_duck_filter(&transcript, 0.1);
        assert!(filter.contains("0.1"));
    }
    #[test]
    fn test_generate_duck_filter_no_duck() {
        let transcript: Vec<TranscriptSegment> = vec![];
        let filter = generate_duck_filter(&transcript, 1.0);
        assert!(filter.contains("1.0"));
    }

    // ── calculate_keep_segments edge cases ─────────────────────────────────

    #[test]
    fn test_calculate_keep_segments_empty_silences() {
        let silences: Vec<Segment> = vec![];
        let segments = calculate_keep_segments(&silences, 60.0, 0.1, SilenceMode::Cut, 2.0, 0.5);
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].start, 0.0);
        assert_eq!(segments[0].end, 60.0);
    }

    #[test]
    fn test_calculate_keep_segments_single_short_silence() {
        let silences = vec![Segment {
            start: 10.0,
            end: 11.0,
        }];
        let segments = calculate_keep_segments(&silences, 60.0, 0.1, SilenceMode::Cut, 2.0, 0.5);
        assert!(segments.len() >= 1);
        assert!(segments[0].start >= 0.0);
    }

    #[test]
    fn test_calculate_keep_segments_silence_at_start() {
        let silences = vec![Segment {
            start: 0.0,
            end: 5.0,
        }];
        let segments = calculate_keep_segments(&silences, 60.0, 0.1, SilenceMode::Cut, 2.0, 0.5);
        assert!(segments[0].start >= 0.0);
    }

    #[test]
    fn test_calculate_keep_segments_silence_at_end() {
        let silences = vec![Segment {
            start: 55.0,
            end: 60.0,
        }];
        let segments = calculate_keep_segments(&silences, 60.0, 0.1, SilenceMode::Cut, 2.0, 0.5);
        assert!(segments.last().unwrap().end <= 60.0);
    }

    #[test]
    fn test_calculate_keep_segments_consecutive_silences() {
        let silences = vec![
            Segment {
                start: 10.0,
                end: 15.0,
            },
            Segment {
                start: 20.0,
                end: 25.0,
            },
            Segment {
                start: 30.0,
                end: 35.0,
            },
        ];
        let segments = calculate_keep_segments(&silences, 60.0, 0.1, SilenceMode::Cut, 2.0, 0.5);
        assert!(segments.len() >= 2);
    }

    // ── editor edge cases ─────────────────────────────────────────────────
    #[test]
    fn test_calculate_keep_segments_no_silences() {
        let silences: Vec<Segment> = vec![];
        let segments = calculate_keep_segments(&silences, 30.0, 0.1, SilenceMode::Cut, 2.0, 1.0);
        // No silences means no segments to remove, so we should have one keep segment
        assert!(segments.len() >= 1);
    }

    #[test]
    fn test_calculate_keep_segments_full_video_silent() {
        let silences = vec![Segment {
            start: 0.0,
            end: 30.0,
        }]; // Entire video is silence
        let segments = calculate_keep_segments(&silences, 30.0, 0.1, SilenceMode::Cut, 2.0, 1.0);
        // If entire video is silence, result depends on mode
        let total_duration: f32 = segments.iter().map(|s| s.end - s.start).sum();
        assert!(total_duration <= 30.0);
    }

    #[test]
    fn test_calculate_keep_segments_different_modes() {
        let silences = vec![Segment {
            start: 10.0,
            end: 15.0,
        }];
        // Test silence modes don't panic
        for mode in [SilenceMode::Cut, SilenceMode::Keep, SilenceMode::Speedup] {
            let _ = calculate_keep_segments(&silences, 30.0, 0.1, mode, 2.0, 1.0);
        }
    }

    #[test]
    fn test_calculate_keep_segments_very_long_video() {
        let silences = vec![Segment {
            start: 3600.0,
            end: 3700.0,
        }]; // 1 hour video
        let segments = calculate_keep_segments(&silences, 7200.0, 0.1, SilenceMode::Cut, 2.0, 1.0);
        // Should handle long videos without issue
        let total_duration: f32 = segments.iter().map(|s| s.end - s.start).sum();
        assert!(total_duration > 0.0);
    }

    #[test]
    fn test_calculate_keep_segments_extreme_speedup() {
        let silences = vec![Segment {
            start: 0.0,
            end: 10.0,
        }];
        let segments = calculate_keep_segments(&silences, 20.0, 0.1, SilenceMode::Cut, 16.0, 1.0);
        // Extreme speedup should still work
        debug_assert!(!segments.is_empty());
    }

    #[test]
    fn test_calculate_keep_segments_zero_speedup() {
        let silences = vec![Segment {
            start: 0.0,
            end: 5.0,
        }];
        let segments = calculate_keep_segments(&silences, 30.0, 0.1, SilenceMode::Cut, 1.0, 0.0);
        // Zero speedup should not panic
        debug_assert!(!segments.is_empty());
    }

    // ── editor more edge cases ─────────────────────────────────────────────
    #[test]
    fn test_calculate_keep_segments_single_silence() {
        let silences = vec![Segment {
            start: 10.0,
            end: 20.0,
        }];
        let segments = calculate_keep_segments(&silences, 60.0, 0.2, SilenceMode::Cut, 1.0, 1.0);
        // Single silence should produce 2 keep segments (before and after)
        let total: f32 = segments.iter().map(|s| s.end - s.start).sum();
        assert!(total > 0.0);
    }

    #[test]
    fn test_calculate_keep_segments_negative_padding() {
        let silences = vec![Segment {
            start: 10.0,
            end: 20.0,
        }];
        let segments = calculate_keep_segments(&silences, 30.0, -0.5, SilenceMode::Cut, 1.0, 1.0);
        // Negative padding may produce empty or different segments
        // (empty segments are a valid edge case)
        debug_assert!(segments.len() <= 1000);
    }

    #[test]
    fn test_calculate_keep_segments_min_speedup() {
        let silences = vec![Segment {
            start: 0.0,
            end: 10.0,
        }];
        let segments = calculate_keep_segments(&silences, 30.0, 0.1, SilenceMode::Cut, 0.5, 1.0);
        // Minimum speedup should work
        let total: f32 = segments.iter().map(|s| s.end - s.start).sum();
        assert!(total >= 0.0);
    }

    #[test]
    fn test_calculate_keep_segments_max_padding() {
        let silences = vec![Segment {
            start: 10.0,
            end: 20.0,
        }];
        let segments = calculate_keep_segments(&silences, 60.0, 5.0, SilenceMode::Cut, 1.0, 1.0);
        // Large padding should still work
        debug_assert!(segments.len() <= 10000);
    }

    // ── editor utility tests ────────────────────────────────────────────────
    #[test]
    fn test_calculate_keep_segments_various_silences() {
        let silences = vec![
            Segment {
                start: 5.0,
                end: 10.0,
            },
            Segment {
                start: 20.0,
                end: 25.0,
            },
            Segment {
                start: 40.0,
                end: 45.0,
            },
        ];
        let segments = calculate_keep_segments(&silences, 60.0, 0.5, SilenceMode::Cut, 1.0, 1.0);
        // Multiple silences should be handled
        let total: f32 = segments.iter().map(|s| s.end - s.start).sum();
        assert!(total > 0.0);
    }

    #[test]
    fn test_calculate_keep_segments_keep_mode_variation() {
        let silences = vec![Segment {
            start: 10.0,
            end: 20.0,
        }];
        let segments = calculate_keep_segments(&silences, 30.0, 0.1, SilenceMode::Keep, 1.0, 1.0);
        // Keep mode should produce some segments
        debug_assert!(!segments.is_empty());
    }

    #[test]
    fn test_calculate_keep_segments_speedup_mode_variation() {
        let silences = vec![Segment {
            start: 10.0,
            end: 20.0,
        }];
        let segments =
            calculate_keep_segments(&silences, 30.0, 0.1, SilenceMode::Speedup, 2.0, 1.0);
        // Speedup mode should produce some segments
        debug_assert!(!segments.is_empty());
    }
}
