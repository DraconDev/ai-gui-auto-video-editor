use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::info;

/// Generate a quick low-resolution preview of a video.
/// This is useful for reviewing edits before committing to a full render.
///
/// # Arguments
/// * `input` - Input video path
/// * `output` - Output preview path
/// * `max_duration` - Maximum preview duration in seconds (e.g., 30.0 for 30s preview)
/// * `scale_width` - Width of preview in pixels (e.g., 480 for fast preview)
pub fn generate_preview(
    input: &Path,
    output: &Path,
    max_duration: f32,
    scale_width: u32,
) -> Result<()> {
    info!(max_duration, scale_width, "Generating preview");

    let status = Command::new("ffmpeg")
        .args([
            "-i",
            input.to_str().context("invalid input path")?,
            "-t",
            &format!("{}", max_duration),
            "-vf",
            &format!("scale={}:-2:flags=fast_bilinear,fps=15", scale_width),
            "-c:v",
            "libx264",
            "-preset",
            "ultrafast",
            "-crf",
            "32",
            "-c:a",
            "aac",
            "-b:a",
            "48k",
            "-movflags",
            "+faststart",
            "-y",
            output.to_str().context("invalid output path")?,
        ])
        .status()
        .context("failed to execute ffmpeg for preview generation")?;

    if !status.success() {
        anyhow::bail!("ffmpeg preview generation failed with status: {}", status);
    }

    info!("Preview generated successfully");
    Ok(())
}

/// Generate a preview path based on the output path
pub fn preview_path(output: &Path) -> PathBuf {
    let stem = output.file_stem().unwrap_or_default();
    let ext = output.extension().unwrap_or_default();
    output.with_file_name(format!(
        "{}_preview.{}",
        stem.to_string_lossy(),
        ext.to_string_lossy()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

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
            .map_err(|_| "ffmpeg not found".to_string())?;
        if status.success() {
            Ok(())
        } else {
            Err("ffmpeg test video creation failed".to_string())
        }
    }

    #[test]
    fn test_generate_preview() {
        let temp_dir = tempfile::tempdir().unwrap();
        let video = temp_dir.path().join("input.mp4");
        let preview = temp_dir.path().join("preview.mp4");
        create_test_video(&video, 5.0).expect("ffmpeg not found");

        generate_preview(&video, &preview, 3.0, 240).unwrap();
        assert!(preview.exists(), "preview should be generated");
    }

    #[test]
    fn test_preview_path() {
        let output = Path::new("/tmp/video.mp4");
        let preview = preview_path(output);
        assert_eq!(preview, PathBuf::from("/tmp/video_preview.mp4"));
    }
}
