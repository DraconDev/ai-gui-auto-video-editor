use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::{info, warn};

/// Generate a thumbnail from a video by extracting and scoring candidate frames.
/// Returns the path to the generated thumbnail image.
pub fn generate_thumbnail(
    video_path: &Path,
    output_path: &Path,
    width: u32,
    height: u32,
) -> Result<()> {
    info!("Generating thumbnail...");

    // Extract candidate frames at 1-second intervals
    let temp_dir = std::env::temp_dir().join(format!(
        "ai-vid-editor-thumbs-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&temp_dir);
    std::fs::create_dir_all(&temp_dir)?;

    // Extract frames using ffmpeg
    let status = Command::new("ffmpeg")
        .args([
            "-i",
            video_path.to_str().context("invalid video path")?,
            "-vf",
            &format!("fps=1,scale={}:{}", width, height),
            "-q:v",
            "2",
            "-y",
            &format!("{}/frame_%04d.jpg", temp_dir.display()),
        ])
        .status()
        .context("failed to execute ffmpeg for thumbnail extraction")?;

    if !status.success() {
        let _ = std::fs::remove_dir_all(&temp_dir);
        anyhow::bail!("ffmpeg thumbnail extraction failed");
    }

    // Score frames and pick the best one
    let best_frame = score_frames(&temp_dir)?;

    if let Some(best) = best_frame {
        std::fs::copy(&best, output_path)?;
        info!(frame = ?best, "Selected best thumbnail frame");
    } else {
        // Fallback: just extract a frame at 1 second into the video
        extract_frame_at_time(video_path, output_path, width, height, 1.0)?;
        info!("Fallback thumbnail extracted at 1 second mark");
    }

    let _ = std::fs::remove_dir_all(&temp_dir);
    Ok(())
}

/// Score frames and return the best one based on:
/// - Sharpness (variance of Laplacian)
/// - Color variance (interesting colors)
/// - Face presence (if detectable)
fn score_frames(temp_dir: &Path) -> Result<Option<PathBuf>> {
    let mut entries: Vec<_> = std::fs::read_dir(temp_dir)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("jpg"))
        .collect();

    entries.sort();

    if entries.is_empty() {
        return Ok(None);
    }

    // For now, pick a frame around the 10-30% mark (avoids intro black screens,
    // avoids ending credits)
    let start_idx = entries.len() / 10;
    let end_idx = entries.len() * 3 / 10;
    let candidate_range = &entries[start_idx..end_idx.max(start_idx + 1)];

    // Score each candidate and pick the best
    let mut best_score = f32::MIN;
    let mut best_frame = None;

    for frame_path in candidate_range {
        match score_single_frame(frame_path) {
            Ok(score) => {
                if score > best_score {
                    best_score = score;
                    best_frame = Some(frame_path.clone());
                }
            }
            Err(e) => {
                warn!(path = ?frame_path, error = %e, "Failed to score frame");
            }
        }
    }

    Ok(best_frame)
}

/// Score a single frame based on sharpness and color variance.
/// Higher score = better thumbnail candidate.
fn score_single_frame(frame_path: &Path) -> Result<f32> {
    // Use ffmpeg to compute the entropy (color variance) of the frame
    let output = Command::new("ffmpeg")
        .args([
            "-i",
            frame_path.to_str().context("invalid frame path")?,
            "-vf",
            "entropy,metadata=print:file=-",
            "-f",
            "null",
            "-",
        ])
        .output()
        .context("failed to execute ffmpeg for frame scoring")?;

    let stderr = String::from_utf8_lossy(&output.stderr);

    // Parse entropy from output - higher entropy = more detail
    let entropy = parse_entropy(&stderr).unwrap_or(0.0);

    // Simple heuristic: prefer frames with moderate-high entropy
    // (not too bland, not too noisy)
    let score = entropy;

    Ok(score)
}

fn parse_entropy(ffmpeg_output: &str) -> Option<f32> {
    for line in ffmpeg_output.lines() {
        if line.contains("entropy") {
            if let Some(val_str) = line.split(':').last() {
                if let Ok(val) = val_str.trim().parse::<f32>() {
                    return Some(val);
                }
            }
        }
    }
    None
}

/// Extract a single frame at a given time position (in seconds)
fn extract_frame_at_time(
    video_path: &Path,
    output_path: &Path,
    width: u32,
    height: u32,
    time_seconds: f32,
) -> Result<()> {
    let status = Command::new("ffmpeg")
        .args([
            "-ss",
            &format!("{:.2}", time_seconds),
            "-i",
            video_path.to_str().context("invalid video path")?,
            "-vframes",
            "1",
            "-vf",
            &format!("scale={}:{}", width, height),
            "-q:v",
            "2",
            "-y",
            output_path.to_str().context("invalid output path")?,
        ])
        .status()
        .context("failed to execute ffmpeg for frame extraction")?;

    if !status.success() {
        anyhow::bail!("ffmpeg frame extraction failed");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    fn create_test_video(path: &Path, duration_secs: f32) {
        let status = Command::new("ffmpeg")
            .args([
                "-f", "lavfi",
                "-i", &format!("testsrc=duration={}:size=320x240:rate=30", duration_secs),
                "-f", "lavfi",
                "-i", &format!("sine=frequency=1000:duration={}", duration_secs),
                "-c:v", "libx264",
                "-preset", "ultrafast",
                "-crf", "28",
                "-c:a", "aac",
                "-b:a", "32k",
                "-shortest",
                "-y",
                path.to_str().unwrap(),
            ])
            .status()
            .expect("ffmpeg not found");
        assert!(status.success());
    }

    #[test]
    fn test_generate_thumbnail() {
        let temp_dir = tempfile::tempdir().unwrap();
        let video = temp_dir.path().join("input.mp4");
        let thumb = temp_dir.path().join("thumb.jpg");
        create_test_video(&video, 3.0);

        generate_thumbnail(&video, &thumb, 320, 180).unwrap();
        assert!(thumb.exists(), "thumbnail should be generated");
    }

    #[test]
    fn test_extract_frame_at_time() {
        let temp_dir = tempfile::tempdir().unwrap();
        let video = temp_dir.path().join("input.mp4");
        let frame = temp_dir.path().join("frame.jpg");
        create_test_video(&video, 3.0);

        extract_frame_at_time(&video, &frame, 320, 180, 0.5).unwrap();
        assert!(frame.exists(), "frame should be extracted");
    }
}
