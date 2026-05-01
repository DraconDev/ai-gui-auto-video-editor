use crate::analyzer::{ProcessedSegment, Segment};
use crate::config::SilenceMode;
use crate::stt_analyzer::TranscriptSegment;
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::{info, warn};

const TRIM_SEGMENTS_PER_CHUNK: usize = 48;

struct ScopedTempFile {
    path: PathBuf,
}

impl ScopedTempFile {
    fn new(prefix: &str, ext: &str) -> Self {
        let path = std::env::temp_dir().join(format!("{}-{}.{}", prefix, std::process::id(), ext));
        Self { path }
    }
    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for ScopedTempFile {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

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
    let mut current_pos = 0.0;
    let mut prev_is_filler = false;

for seg in transcript {
        let is_filler = filler_words
            .iter()
            .any(|&f| seg.text.to_lowercase().contains(f));

        if is_filler {
            let keep_end = (seg.start + padding).min(total_duration);
            let cut_end = (seg.end - padding).max(0.0);

            if keep_end > current_pos {
                if prev_is_filler {
                    if let Some(prev) = processed.last_mut() {
                        prev.end = keep_end;
                    }
                } else {
                    processed.push(ProcessedSegment {
                        start: current_pos,
                        end: keep_end,
                        speed: 1.0,
                    });
                }
            }
            current_pos = cut_end;
            prev_is_filler = true;
        } else {
            if current_pos < seg.start {
                let gap = seg.start - current_pos;
                if prev_is_filler && (gap - padding).abs() < 0.001 {
                    if let Some(prev) = processed.last_mut() {
                        prev.end = current_pos + padding;
                    }
                    current_pos = seg.end - padding;
                    prev_is_filler = false;
                }
            }
            if current_pos < seg.end {
                processed.push(ProcessedSegment {
                    start: current_pos,
                    end: seg.end,
                    speed: 1.0,
                });
            }
            current_pos = seg.end;
            prev_is_filler = false;
        }
    }
                } else {
                    eprintln!("  pushing [{:.1}, {:.1})", current_pos, keep_end);
                    processed.push(ProcessedSegment {
                        start: current_pos,
                        end: keep_end,
                        speed: 1.0,
                    });
                }
            }
            current_pos = cut_end;
            prev_is_filler = true;
} else {
            if current_pos < seg.start {
                let gap = seg.start - current_pos;
                if prev_is_filler && (gap - padding).abs() < 0.001 {
                    if let Some(prev) = processed.last_mut() {
                        prev.end = current_pos + padding;
                    }
                    current_pos = seg.end - padding;
                    prev_is_filler = false;
                }
            }
            if current_pos < seg.end {
                processed.push(ProcessedSegment {
                    start: current_pos,
                    end: seg.end,
                    speed: 1.0,
                });
            }
            current_pos = seg.end;
            prev_is_filler = false;
        }
    }
                    current_pos = seg.end - padding;
                    prev_is_filler = false;
                }
            }
            if current_pos < seg.end {
                processed.push(ProcessedSegment {
                    start: current_pos,
                    end: seg.end,
                    speed: 1.0,
                });
            }
            current_pos = seg.end;
            prev_is_filler = false;
        }
                    current_pos = seg.end;
                    prev_is_filler = false;
                    continue;
                }
            }
            if current_pos < seg.end {
                eprintln!("  pushing [{:.1}, {:.1})", current_pos, seg.end);
                processed.push(ProcessedSegment {
                    start: current_pos,
                    end: seg.end,
                    speed: 1.0,
                });
            }
            current_pos = seg.end;
            prev_is_filler = false;
        }
        eprintln!("  after: current_pos={:.1} prev_is_filler={} processed_len={}", current_pos, prev_is_filler, processed.len());
    }

    if current_pos < total_duration {
        processed.push(ProcessedSegment {
            start: current_pos,
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

        // Pass 1: Measure audio loudness
        let measure_filter = format!(
            "highpass=f=80,lowpass=f=12000,equalizer=f=1500:t=q:w=3:g=1.5,loudnorm=I={target_lufs}:TP=-1.5:LRA=11:print_format=json"
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

        // Pass 2: Apply measured normalization
        let filter = if let Some(s) = stats {
            format!(
                "highpass=f=80,lowpass=f=12000,equalizer=f=1500:t=q:w=3:g=1.5,loudnorm=I={}:TP=-1.5:LRA=11:measured_I={}:measured_TP={}:measured_LRA={}:measured_thresh={}:offset={}:linear=true",
                target_lufs, s.i, s.tp, s.lra, s.thresh, s.offset
            )
        } else {
            // Fallback to single-pass if measurement failed
            format!(
                "highpass=f=80,lowpass=f=12000,equalizer=f=1500:t=q:w=3:g=1.5,loudnorm=I={target_lufs}:TP=-1.5:LRA=11"
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
        // Apply FFT-based noise reduction
        // afftdn removes steady background noise (fans, AC, hiss)
        let filter = "afftdn=nf=-25:tn=1";

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
        let trf_file = ScopedTempFile::new("ai-vid-editor-vidstab", "trf");
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

                    processor.generate_crop_filter(&crop_regions, w, h, target_resolution)
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

    fn blur_background(&self, input: &Path, output: &Path) -> Result<()> {
        info!("Background blur: Processing video...");

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
}

fn run_trim_filter_job(
    input: &Path,
    output: &Path,
    segments: &[ProcessedSegment],
    codec: &str,
) -> Result<()> {
    let (v_filter, a_filter) = generate_trim_filters(segments);

    let status = Command::new("ffmpeg")
        .args([
            "-i",
            input.to_str().context("invalid input path")?,
            "-filter_complex",
            &format!("{}{}", v_filter, a_filter),
            "-map",
            "[outv]",
            "-map",
            "[outa]",
            "-c:v",
            codec,
            "-preset",
            "veryfast",
            "-crf",
            "20",
            "-c:a",
            "aac",
            "-b:a",
            "192k",
            "-movflags",
            "+faststart",
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

fn create_trim_chunk_dir(output: &Path) -> Result<PathBuf> {
    let parent = output.parent().unwrap_or_else(|| Path::new("."));
    let stem = output
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("trim");
    let chunk_dir = parent.join(format!(".ai-vid-editor-{}-{}", stem, std::process::id()));

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

    let status = Command::new("ffmpeg")
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
        .status()
        .context("failed to execute ffmpeg concat")?;

    let _ = fs::remove_file(&concat_list);
    for chunk_file in chunk_files {
        let _ = fs::remove_file(chunk_file);
    }

    if !status.success() {
        anyhow::bail!("ffmpeg concat failed with status: {}", status);
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
    // Find the JSON block in ffmpeg stderr output
    let json_start = stderr.find('{')?;
    let json_str = &stderr[json_start..];
    let json_end = json_str.find('}')? + 1;
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
fn chain_atempo_filters(speed: f32) -> String {
    const ATEMPO_MIN: f32 = 0.5;
    const ATEMPO_MAX: f32 = 2.0;

    if (ATEMPO_MIN..=ATEMPO_MAX).contains(&speed) {
        return format!("atempo={}", speed);
    }

    let mut filters = Vec::new();
    let mut remaining = speed;

    if speed > ATEMPO_MAX {
        // Speed up: chain multiple 2.0x filters
        while remaining > ATEMPO_MAX {
            filters.push("atempo=2.0".to_string());
            remaining /= 2.0;
        }
        if remaining > 1.0 {
            filters.push(format!("atempo={}", remaining));
        }
    } else if speed < ATEMPO_MIN {
        // Slow down: chain multiple 0.5x filters
        while remaining < ATEMPO_MIN {
            filters.push("atempo=0.5".to_string());
            remaining /= 0.5;
        }
        if remaining < 1.0 {
            filters.push(format!("atempo={}", remaining));
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
    use std::process::Command;

    /// Helper: create a small test video using ffmpeg
    fn create_test_video(path: &Path, duration_secs: f32) -> Result<(), String> {
        let status = Command::new("ffmpeg")
            .args([
                "-f",
                "lavfi",
                "-i",
                &format!("testsrc=duration={}:size=320x240:rate=30", duration_secs),
                "-f",
                "lavfi",
                "-i",
                "sine=frequency=1000:duration=0.1",
                "-c:v",
                "libx264",
                "-preset",
                "ultrafast",
                "-crf",
                "28",
                "-c:a",
                "aac",
                "-b:a",
                "32k",
                "-shortest",
                "-y",
                path.to_str().unwrap(),
            ])
            .status()
            .map_err(|_| "ffmpeg not found".to_string())?;
        if status.success() {
            Ok(())
        } else {
            Err("ffmpeg test video creation failed".to_string())
        }
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
        assert_eq!(processed[1].start, 2.0);
        assert_eq!(processed[1].end, 2.1);
    }

    #[test]
    fn test_chain_atempo_filters() {
        // Within range: single filter
        assert_eq!(chain_atempo_filters(1.5), "atempo=1.5");
        assert_eq!(chain_atempo_filters(0.75), "atempo=0.75");

        // Speed up beyond 2.0: chain multiple
        assert_eq!(chain_atempo_filters(4.0), "atempo=2.0,atempo=2");
        assert_eq!(chain_atempo_filters(3.0), "atempo=2.0,atempo=1.5");

        // Slow down below 0.5: chain multiple
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
}
