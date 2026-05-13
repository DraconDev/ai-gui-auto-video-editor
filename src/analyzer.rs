use anyhow::{Context, Result};
use std::path::Path;

#[derive(Debug, PartialEq, Clone)]
pub struct Segment {
    pub start: f32,
    pub end: f32,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ProcessedSegment {
    pub start: f32,
    pub end: f32,
    pub speed: f32,
}

pub trait VideoAnalyzer: Send + Sync {
    fn detect_silence(
        &self,
        path: &Path,
        threshold_db: f32,
        duration_s: f32,
    ) -> Result<Vec<Segment>>;
}

pub struct FfmpegAnalyzer;

impl VideoAnalyzer for FfmpegAnalyzer {
    fn detect_silence(
        &self,
        path: &Path,
        threshold_db: f32,
        duration_s: f32,
    ) -> Result<Vec<Segment>> {
        // Get video duration first to handle unclosed silence segments at EOF
        let duration = get_video_duration(path).unwrap_or(f32::MAX);

        let output = std::process::Command::new("ffmpeg")
            .args([
                "-i",
                path.to_str().context("invalid path")?,
                "-af",
                &format!("silencedetect=noise={}dB:d={}", threshold_db, duration_s),
                "-f",
                "null",
                "-",
            ])
            .output()
            .context("failed to execute ffmpeg")?;

        let stderr = String::from_utf8_lossy(&output.stderr);
        Ok(parse_ffmpeg_silence(&stderr, duration))
    }
}

/// Get video duration via ffprobe.
fn get_video_duration(path: &Path) -> Result<f32> {
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

pub fn parse_ffmpeg_silence(output: &str, video_duration: f32) -> Vec<Segment> {
    let mut segments = Vec::new();
    let mut current_start: Option<f32> = None;

    for line in output.lines() {
        if line.contains("silence_start:") {
            if let Some(pos) = line.find("silence_start:") {
                let val_str = &line[pos + "silence_start:".len()..].trim();
                if let Ok(start) = val_str.parse::<f32>() {
                    current_start = Some(start);
                }
            }
        } else if line.contains("silence_end:")
            && let Some(start) = current_start.take()
            && let Some(pos) = line.find("silence_end:")
        {
            let part = &line[pos + "silence_end:".len()..];
            let val_str = if let Some(pipe_pos) = part.find('|') {
                part[..pipe_pos].trim()
            } else {
                part.trim()
            };
            if let Ok(end) = val_str.parse::<f32>() {
                // Validate: end must be strictly greater than start
                if end > start {
                    segments.push(Segment { start, end });
                }
            }
        }
    }

    // Handle unclosed silence segment at EOF
    if let Some(start) = current_start
        && video_duration > start
    {
        segments.push(Segment {
            start,
            end: video_duration,
        });
    }

    segments
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_silence_output() {
        let output = r#"[silencedetect @ 0x559e1c2c4840] silence_start: 1.234
[silencedetect @ 0x559e1c2c4840] silence_end: 4.567 | silence_duration: 3.333
[silencedetect @ 0x559e1c2c4840] silence_start: 10.0
[silencedetect @ 0x559e1c2c4840] silence_end: 12.5 | silence_duration: 2.5"#;

        let segments = parse_ffmpeg_silence(output, 20.0);
        assert_eq!(segments.len(), 2);
        assert_eq!(
            segments[0],
            Segment {
                start: 1.234,
                end: 4.567
            }
        );
        assert_eq!(
            segments[1],
            Segment {
                start: 10.0,
                end: 12.5
            }
        );
    }

    #[test]
    fn test_parse_silence_negative_duration_filtered() {
        // end <= start should be filtered out
        let output = r#"[silencedetect] silence_start: 5.0
[silencedetect] silence_end: 5.0 | silence_duration: 0.0
[silencedetect] silence_start: 10.0
[silencedetect] silence_end: 9.0 | silence_duration: -1.0"#;

        let segments = parse_ffmpeg_silence(output, 20.0);
        assert_eq!(segments.len(), 0);
    }

    #[test]
    fn test_parse_silence_missing_start() {
        // silence_end without matching silence_start should be ignored
        let output = r#"[silencedetect] silence_end: 4.0 | silence_duration: 3.0"#;

        let segments = parse_ffmpeg_silence(output, 20.0);
        assert_eq!(segments.len(), 0);
    }

    #[test]
    fn test_parse_silence_unmatched_start() {
        // silence_start without matching silence_end should be extended to EOF
        let output = r#"[silencedetect] silence_start: 1.0"#;

        let segments = parse_ffmpeg_silence(output, 10.0);
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0], Segment { start: 1.0, end: 10.0 });
    }

    #[test]
    fn test_parse_silence_malformed_lines_mixed() {
        // Mix of valid and malformed lines
        let output = r#"random noise here
[silencedetect] silence_start: 1.0
more noise
[silencedetect] silence_end: 4.0 | silence_duration: 3.0
[silencedetect] invalid_line: xyz
[silencedetect] silence_start: 10.0
[silencedetect] silence_end: 12.0"#;

        let segments = parse_ffmpeg_silence(output, 20.0);
        assert_eq!(segments.len(), 2);
        assert_eq!(
            segments[0],
            Segment {
                start: 1.0,
                end: 4.0
            }
        );
        assert_eq!(
            segments[1],
            Segment {
                start: 10.0,
                end: 12.0
            }
        );
    }

    #[test]
    fn test_parse_silence_empty_output() {
        let segments = parse_ffmpeg_silence("", 10.0);
        assert_eq!(segments.len(), 0);
    }

    #[test]
    fn test_parse_silence_extra_noise_after_pipe() {
        // Extra text after silence_duration should not break parsing
        let output = r#"[silencedetect] silence_start: 5.0
[silencedetect] silence_end: 8.0 | silence_duration: 3.0 | extra: stuff"#;

        let segments = parse_ffmpeg_silence(output, 20.0);
        assert_eq!(segments.len(), 1);
        assert_eq!(
            segments[0],
            Segment {
                start: 5.0,
                end: 8.0
            }
        );
    }

    #[test]
    fn test_parse_silence_large_float_values() {
        // Very long timestamps (e.g., hour-long videos)
        let output = r#"[silencedetect] silence_start: 3600.5
[silencedetect] silence_end: 3605.75 | silence_duration: 5.25"#;

        let segments = parse_ffmpeg_silence(output, 100.0);
        assert_eq!(segments.len(), 1);
        assert_eq!(
            segments[0],
            Segment {
                start: 3600.5,
                end: 3605.75
            }
        );
    }

    #[test]
    fn test_parse_silence_multiple_unmatched_starts() {
        // Multiple starts without ends - only the last unmatched start should be ignored
        let output = r#"[silencedetect] silence_start: 1.0
[silencedetect] silence_start: 2.0
[silencedetect] silence_end: 3.0 | silence_duration: 1.0"#;

        let segments = parse_ffmpeg_silence(output, 100.0);
        assert_eq!(segments.len(), 1);
        // Second start overwrites the first, so segment is 2.0-3.0
        assert_eq!(
            segments[0],
            Segment {
                start: 2.0,
                end: 3.0
            }
        );
    }

    #[test]
    fn test_parse_silence_end_without_start_then_valid() {
        // Orphan silence_end followed by valid pair
        let output = r#"[silencedetect] silence_end: 4.0 | silence_duration: 3.0
[silencedetect] silence_start: 10.0
[silencedetect] silence_end: 12.0 | silence_duration: 2.0"#;

        let segments = parse_ffmpeg_silence(output, 100.0);
        assert_eq!(segments.len(), 1);
        assert_eq!(
            segments[0],
            Segment {
                start: 10.0,
                end: 12.0
            }
        );
    }

    #[test]
    fn test_parse_silence_no_decimal_points() {
        // Integer timestamps without decimals
        let output = r#"[silencedetect] silence_start: 5
[silencedetect] silence_end: 10 | silence_duration: 5"#;

        let segments = parse_ffmpeg_silence(output, 100.0);
        assert_eq!(segments.len(), 1);
        assert_eq!(
            segments[0],
            Segment {
                start: 5.0,
                end: 10.0
            }
        );
    }

    #[test]
    fn test_parse_silence_whitespace_variations() {
        // Extra whitespace around values
        let output = r#"[silencedetect] silence_start:   1.5  
[silencedetect] silence_end:   4.5   | silence_duration: 3.0"#;

        let segments = parse_ffmpeg_silence(output, 100.0);
        assert_eq!(segments.len(), 1);
        assert_eq!(
            segments[0],
            Segment {
                start: 1.5,
                end: 4.5
            }
        );
    }
    mod proptests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            /// Property: parse_ffmpeg_silence should never panic regardless of input
            #[test]
            fn never_panics(output: String, duration: f32) {
                let duration = if duration.is_nan() || duration <= 0.0 { 1.0 } else { duration };
                let segments = parse_ffmpeg_silence(&output, duration);
                for seg in &segments {
                    prop_assert!(seg.start >= 0.0, "start must be non-negative");
                    prop_assert!(seg.end > seg.start, "end must be > start");
                }
            }
        }

        proptest! {
            /// Property: isolated silence_end without matching start should be ignored
            #[test]
            fn orphan_end_ignored(end_val: f32, dur_val: f32) {
                let duration = dur_val.abs().max(1.0);
                let end = end_val.abs().min(duration);
                let output = format!("[sd] silence_end: {} | silence_duration: {}", end, dur_val.abs());
                let segments = parse_ffmpeg_silence(&output, duration);
                prop_assert_eq!(segments.len(), 0);
            }
        }

        proptest! {
            /// Property: paired start/end should always produce a valid segment
            #[test]
            fn paired_start_end_produces_segment(start: f32, end: f32, duration: f32) {
                let duration = duration.abs().max(1.0);
                let start = start.abs().min(duration - 0.001);
                let end = end.abs().max(start + 0.001).min(duration);
                let output = format!(
                    "[sd] silence_start: {}\n[sd] silence_end: {} | silence_duration: {}",
                    start, end, end - start
                );
                let segments = parse_ffmpeg_silence(&output, duration);
                prop_assert_eq!(segments.len(), 1);
                prop_assert!((segments[0].start - start).abs() < 0.001);
                prop_assert!((segments[0].end - end).abs() < 0.001);
            }
        }
    }

}

    mod proptests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            /// Property: parse_ffmpeg_silence should never panic regardless of input
            #[test]
            fn never_panics(output: String, duration: f32) {
                let duration = if duration.is_nan() || duration <= 0.0 { 1.0 } else { duration };
                let segments = parse_ffmpeg_silence(&output, duration);
                for seg in &segments {
                    prop_assert!(seg.start >= 0.0, "start must be non-negative");
                    prop_assert!(seg.end > seg.start, "end must be > start");
                }
            }
        }

        proptest! {
            /// Property: isolated silence_end without matching start should be ignored
            #[test]
            fn orphan_end_ignored(end_val: f32, dur_val: f32) {
                let duration = dur_val.abs().max(1.0);
                let end = end_val.abs().min(duration);
                let output = format!("[sd] silence_end: {} | silence_duration: {}", end, dur_val.abs());
                let segments = parse_ffmpeg_silence(&output, duration);
                prop_assert_eq!(segments.len(), 0);
            }
        }

        proptest! {
            /// Property: paired start/end should always produce a valid segment
            #[test]
            fn paired_start_end_produces_segment(start: f32, end: f32, duration: f32) {
                let duration = duration.abs().max(1.0);
                let start = start.abs().min(duration - 0.001);
                let end = end.abs().max(start + 0.001).min(duration);
                let output = format!(
                    "[sd] silence_start: {}\n[sd] silence_end: {} | silence_duration: {}",
                    start, end, end - start
                );
                let segments = parse_ffmpeg_silence(&output, duration);
                prop_assert_eq!(segments.len(), 1);
                prop_assert!((segments[0].start - start).abs() < 0.001);
                prop_assert!((segments[0].end - end).abs() < 0.001);
            }
        }
    }

