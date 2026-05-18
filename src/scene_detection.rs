use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;
use tracing::info;

/// Detect scene changes in a video using ffmpeg's scene detection filter.
/// Returns a list of timestamps (in seconds) where scene changes occur.
pub fn detect_scene_changes(video_path: &Path, threshold: f32) -> Result<Vec<f32>> {
    info!("Detecting scene changes...");

    let threshold = threshold.clamp(0.0, 1.0);

    let output = Command::new("ffmpeg")
        .args([
            "-i",
            video_path.to_str().context("invalid video path")?,
            "-vf",
            &format!("select='gt(scene,{:.3})',showinfo", threshold),
            "-f",
            "null",
            "-",
        ])
        .output()
        .context("failed to execute ffmpeg for scene detection")?;

    let stderr = String::from_utf8_lossy(&output.stderr);
    let scenes = parse_scene_changes(&stderr);

    info!(count = scenes.len(), "Scene changes detected");
    Ok(scenes)
}

fn parse_scene_changes(ffmpeg_output: &str) -> Vec<f32> {
    let mut scenes = Vec::new();

    for line in ffmpeg_output.lines() {
        // Look for pts_time in showinfo output
        if line.contains("pts_time:")
            && let Some(pos) = line.find("pts_time:")
        {
            let val_str = &line[pos + "pts_time:".len()..].trim();
            // Extract just the number (may have trailing text)
            let num_str: String = val_str
                .chars()
                .take_while(|c| c.is_ascii_digit() || *c == '.')
                .collect();
            if let Ok(timestamp) = num_str.parse::<f32>() {
                scenes.push(timestamp);
            }
        }
    }

    scenes
}

/// Convert scene changes into segments that can be used for cutting.
/// Each segment represents a "scene" between two detected changes.
pub fn scenes_to_segments(
    scene_changes: &[f32],
    total_duration: f32,
) -> Vec<crate::analyzer::Segment> {
    if scene_changes.is_empty() {
        return vec![crate::analyzer::Segment {
            start: 0.0,
            end: total_duration,
        }];
    }

    let mut segments = Vec::new();
    let mut current_start = 0.0;

    for &scene_time in scene_changes {
        if scene_time > current_start {
            segments.push(crate::analyzer::Segment {
                start: current_start,
                end: scene_time,
            });
        }
        current_start = scene_time;
    }

    // Add final segment
    if current_start < total_duration {
        segments.push(crate::analyzer::Segment {
            start: current_start,
            end: total_duration,
        });
    }

    segments
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_video(path: &Path, duration_secs: f32) -> Result<(), String> {
        crate::tests_common::create_test_video(path, duration_secs)
    }

    #[test]
    fn test_detect_scene_changes() {
        let temp_dir = tempfile::tempdir().unwrap();
        let video = temp_dir.path().join("input.mp4");
        create_test_video(&video, 3.0).expect("ffmpeg not found");

        // Test video with testsrc may not have scene changes, but the function should not panic
        let scenes = detect_scene_changes(&video, 0.3).unwrap();
        // testsrc doesn't have scene changes, so we expect 0 or very few
        assert!(
            scenes.len() <= 2,
            "test video should have at most 2 scene changes"
        );
    }

    #[test]
    fn test_scenes_to_segments() {
        let scenes = vec![1.0, 3.0, 5.0];
        let segments = scenes_to_segments(&scenes, 6.0);

        assert_eq!(segments.len(), 4);
        assert_eq!(segments[0].start, 0.0);
        assert_eq!(segments[0].end, 1.0);
        assert_eq!(segments[1].start, 1.0);
        assert_eq!(segments[1].end, 3.0);
        assert_eq!(segments[2].start, 3.0);
        assert_eq!(segments[2].end, 5.0);
        assert_eq!(segments[3].start, 5.0);
        assert_eq!(segments[3].end, 6.0);
    }

    #[test]
    fn test_scenes_to_segments_empty() {
        let segments = scenes_to_segments(&[], 10.0);
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].start, 0.0);
        assert_eq!(segments[0].end, 10.0);
    }

    #[test]
    fn test_scenes_to_segments_single_change() {
        // Single scene change in the middle
        let scenes = vec![2.0];
        let segments = scenes_to_segments(&scenes, 10.0);

        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0].start, 0.0);
        assert_eq!(segments[0].end, 2.0);
        assert_eq!(segments[1].start, 2.0);
        assert_eq!(segments[1].end, 10.0);
    }

    #[test]
    fn test_scenes_to_segments_at_start() {
        // Scene change at time 0 - should be skipped (zero-length segment)
        let scenes = vec![0.0, 5.0];
        let segments = scenes_to_segments(&scenes, 10.0);

        // Scene at 0.0 creates no segment, so we get segments from 0-5 and 5-10
        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0].start, 0.0);
        assert_eq!(segments[0].end, 5.0);
        assert_eq!(segments[1].start, 5.0);
        assert_eq!(segments[1].end, 10.0);
    }

    #[test]
    fn test_scenes_to_segments_at_end() {
        // Scene change at the very end - should be skipped (zero-length segment)
        let scenes = vec![5.0, 10.0];
        let segments = scenes_to_segments(&scenes, 10.0);

        // Scene at 10.0 creates no final segment, so we get 0-5 and 5-10
        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0].start, 0.0);
        assert_eq!(segments[0].end, 5.0);
        assert_eq!(segments[1].start, 5.0);
        assert_eq!(segments[1].end, 10.0);
    }

    #[test]
    fn test_parse_scene_changes_from_ffmpeg_output() {
        // Simulate FFmpeg showinfo output
        let ffmpeg_output = r#"
[Parsed_showinfo_0 @ 0x123] n:0 pts:0 pts_time:0.0
[Parsed_showinfo_0 @ 0x123] n:1 pts:25 pts_time:1.0
[Parsed_showinfo_0 @ 0x123] n:2 pts:75 pts_time:3.0
[Parsed_showinfo_0 @ 0x123] n:3 pts:125 pts_time:5.0
"#;
        let scenes = parse_scene_changes(ffmpeg_output);

        assert_eq!(scenes.len(), 4);
        assert_eq!(scenes[0], 0.0);
        assert_eq!(scenes[1], 1.0);
        assert_eq!(scenes[2], 3.0);
        assert_eq!(scenes[3], 5.0);
    }

    #[test]
    fn test_parse_scene_changes_with_malformed_output() {
        // Test that malformed lines are handled gracefully
        // Each line has pts_time: followed by a number
        let ffmpeg_output = "n:0 pts:0 pts_time:1.5\nsome random text\nn:1 pts:25 pts_time:2.5\nno pts_time here\nn:2 pts:50 pts_time:4.0\n";
        let scenes = parse_scene_changes(ffmpeg_output);

        // Should get 3 valid timestamps: 1.5, 2.5, 4.0
        // Malformed lines without pts_time are skipped
        assert_eq!(scenes.len(), 3, "Should parse all 3 valid timestamps");
        assert_eq!(scenes[0], 1.5);
        assert_eq!(scenes[1], 2.5);
        assert_eq!(scenes[2], 4.0);
    }

    #[test]
    fn test_scenes_to_segments_single_scene() {
        let scenes = vec![5.0];
        let segments = scenes_to_segments(&scenes, 10.0);
        // Single scene marker should split into 2 segments
        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0].start, 0.0);
        assert_eq!(segments[0].end, 5.0);
        assert_eq!(segments[1].start, 5.0);
        assert_eq!(segments[1].end, 10.0);
    }

    #[test]
    fn test_scenes_to_segments_adjacent_scenes() {
        let scenes = vec![3.0, 6.0, 9.0];
        let segments = scenes_to_segments(&scenes, 10.0);
        // 3 scene markers = 4 segments
        assert_eq!(segments.len(), 4);
    }

    #[test]
    fn test_scenes_to_segments_empty_scenes() {
        let scenes: Vec<f32> = vec![];
        let segments = scenes_to_segments(&scenes, 30.0);
        // No scene markers = single segment covering entire video
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].start, 0.0);
        assert_eq!(segments[0].end, 30.0);
    }
}
