use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;
use tracing::info;

/// Detect scene changes in a video using ffmpeg's scene detection filter.
/// Returns a list of timestamps (in seconds) where scene changes occur.
pub fn detect_scene_changes(video_path: &Path, threshold: f32) -> Result<Vec<f32>> {
    info!("Detecting scene changes...");

    let output = Command::new("ffmpeg")
        .args([
            "-i",
            video_path.to_str().context("invalid video path")?,
            "-vf",
            &format!("select='gt(scene,{})',showinfo", threshold),
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
    use std::process::Command;

    fn create_test_video(path: &std::path::Path, duration_secs: f32) {
        let status = Command::new("ffmpeg")
            .args([
                "-f",
                "lavfi",
                "-i",
                &format!("testsrc=duration={}:size=320x240:rate=30", duration_secs),
                "-f",
                "lavfi",
                "-i",
                &format!("sine=frequency=1000:duration={}", duration_secs),
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
            .expect("ffmpeg not found");
        assert!(status.success());
    }

    #[test]
    fn test_detect_scene_changes() {
        let temp_dir = tempfile::tempdir().unwrap();
        let video = temp_dir.path().join("input.mp4");
        create_test_video(&video, 3.0);

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
}
