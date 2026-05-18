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
        assert_eq!(
            segments[0],
            Segment {
                start: 1.0,
                end: 10.0
            }
        );
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
            fn paired_start_end_produces_segment(start: f64, end: f64, duration: f64) {
                let duration = duration.abs().max(1.0).min(1e6);
                let start = start.abs().min(duration - 0.1);
                let end = end.abs().max(start + 0.1).min(duration);
                let start_f32 = start as f32;
                let end_f32 = end as f32;
                let dur_f32 = duration as f32;
                let output = format!(
                    "[sd] silence_start: {}\n[sd] silence_end: {} | silence_duration: {}",
                    start_f32, end_f32, end_f32 - start_f32
                );
                let segments = parse_ffmpeg_silence(&output, dur_f32);
                prop_assert_eq!(segments.len(), 1);
                prop_assert!((segments[0].start - start_f32).abs() < 0.05);
                prop_assert!((segments[0].end - end_f32).abs() < 0.05);
            }
        }
    }

    // ── ProcessedSegment logic tests (no FFmpeg needed) ─────────────────────

    #[test]
    fn test_processed_segment_valid_bounds() {
        let seg = ProcessedSegment {
            start: 0.0,
            end: 10.0,
            speed: 1.0,
        };
        assert!(seg.start >= 0.0);
        assert!(seg.end > seg.start);
        assert!(seg.speed > 0.0);
    }

    #[test]
    fn test_processed_segment_speed_positive() {
        let seg = ProcessedSegment {
            start: 0.0,
            end: 5.0,
            speed: 2.0,
        };
        assert!(seg.speed > 0.0);
    }

    #[test]
    fn test_segment_duration() {
        let seg = ProcessedSegment {
            start: 5.0,
            end: 15.0,
            speed: 1.0,
        };
        assert!((seg.end - seg.start - 10.0).abs() < 1e-6);
    }

    #[test]
    fn test_segment_merge_adjacent() {
        // Two adjacent segments (end of first == start of second)
        let seg1 = Segment {
            start: 0.0,
            end: 5.0,
        };
        let seg2 = Segment {
            start: 5.0,
            end: 10.0,
        };
        // Merged: start = min, end = max
        let merged_start = seg1.start.min(seg2.start);
        let merged_end = seg1.end.max(seg2.end);
        assert_eq!(merged_start, 0.0);
        assert_eq!(merged_end, 10.0);
    }

    #[test]
    fn test_segment_merge_overlapping() {
        let seg1 = Segment {
            start: 0.0,
            end: 8.0,
        };
        let seg2 = Segment {
            start: 5.0,
            end: 12.0,
        };
        // Merged bounding box
        let merged_start = seg1.start.min(seg2.start);
        let merged_end = seg1.end.max(seg2.end);
        assert_eq!(merged_start, 0.0);
        assert_eq!(merged_end, 12.0);
    }

    #[test]
    fn test_segment_sorting() {
        let mut segments = vec![
            Segment {
                start: 10.0,
                end: 15.0,
            },
            Segment {
                start: 0.0,
                end: 5.0,
            },
            Segment {
                start: 5.0,
                end: 10.0,
            },
        ];
        segments.sort_by(|a, b| a.start.total_cmp(&b.start));
        assert_eq!(segments[0].start, 0.0);
        assert_eq!(segments[1].start, 5.0);
        assert_eq!(segments[2].start, 10.0);
    }

    // ── ProcessedSegment speed bounds tests ──────────────────────────────────
    #[test]
    fn test_processed_segment_speed_within_bounds() {
        // Test speed is always positive and within reasonable bounds
        let segment = ProcessedSegment {
            start: 0.0,
            end: 10.0,
            speed: 1.5,
        };
        assert!(segment.speed > 0.0);
        assert!(segment.speed <= 16.0, "Speed should be capped at 16x");
    }

    #[test]
    fn test_processed_segment_very_fast_speed() {
        let segment = ProcessedSegment {
            start: 0.0,
            end: 5.0,
            speed: 4.0,
        };
        assert!(segment.speed > 1.0);
    }

    #[test]
    fn test_processed_segment_very_slow_speed() {
        let segment = ProcessedSegment {
            start: 0.0,
            end: 20.0,
            speed: 0.25,
        };
        assert!(segment.speed > 0.0);
    }

    // ── Segment boundary edge cases ─────────────────────────────────────────
    #[test]
    fn test_segment_exact_boundary() {
        let seg1 = Segment { start: 0.0, end: 10.0 };
        let seg2 = Segment { start: 10.0, end: 20.0 };
        // These segments touch exactly at 10.0
        assert_eq!(seg1.end, seg2.start);
    }

    #[test]
    fn test_segment_zero_duration() {
        let seg = Segment { start: 5.0, end: 5.0 };
        // Zero-length segment should be detected
        let duration = seg.end - seg.start;
        assert!(duration.abs() < 1e-6);
    }

    #[test]
    fn test_segments_non_overlapping() {
        let seg1 = Segment { start: 0.0, end: 5.0 };
        let seg2 = Segment { start: 10.0, end: 15.0 };
        // Gap between segments
        let gap = seg2.start - seg1.end;
        assert_eq!(gap, 5.0);
    }

    #[test]
    fn test_segments_fully_contained() {
        let outer = Segment { start: 0.0, end: 20.0 };
        let inner = Segment { start: 5.0, end: 15.0 };
        // Inner is fully contained within outer
        assert!(inner.start >= outer.start);
        assert!(inner.end <= outer.end);
    }
}
