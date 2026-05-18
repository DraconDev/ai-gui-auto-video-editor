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

    fn create_test_video(path: &Path, duration_secs: f32) -> Result<(), String> {
        crate::tests_common::create_test_video(path, duration_secs)
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

    #[test]
    fn test_preview_path_different_extensions() {
        // Test with different video extensions - preview keeps original extension
        assert_eq!(
            preview_path(Path::new("/tmp/video.mov")),
            PathBuf::from("/tmp/video_preview.mov")
        );
        assert_eq!(
            preview_path(Path::new("/tmp/video.avi")),
            PathBuf::from("/tmp/video_preview.avi")
        );
        assert_eq!(
            preview_path(Path::new("/tmp/video.mkv")),
            PathBuf::from("/tmp/video_preview.mkv")
        );
    }

    #[test]
    fn test_preview_path_nested() {
        // Test with nested path
        let output = Path::new("/home/user/videos/project/video.mp4");
        let preview = preview_path(output);
        assert_eq!(
            preview,
            PathBuf::from("/home/user/videos/project/video_preview.mp4")
        );
    }

    #[test]
    fn test_generate_preview_short_video() {
        // Test generating preview from a short video (shorter than max_duration)
        let temp_dir = tempfile::tempdir().unwrap();
        let video = temp_dir.path().join("input.mp4");
        let preview = temp_dir.path().join("preview.mp4");
        create_test_video(&video, 2.0).expect("ffmpeg not found");

        // Preview of 2s video with max_duration=5s should work
        generate_preview(&video, &preview, 5.0, 240).unwrap();
        assert!(
            preview.exists(),
            "preview should be generated for short video"
        );
    }

    // ── preview_path pure logic tests ────────────────────────────────────

    // --- preview_path pure logic tests ---
    #[test]
    fn test_preview_path_adds_preview_suffix() {
        let output = Path::new("/a/b/c/video.mp4");
        let preview = preview_path(output);
        assert!(preview.to_string_lossy().contains("_preview"));
    }

    #[test]
    fn test_preview_path_preserves_extension() {
        let output = Path::new("video.avi");
        let preview = preview_path(output);
        assert_eq!(preview.extension().unwrap(), "avi");
    }

    #[test]
    fn test_preview_path_webm_preserved() {
        let output = Path::new("video.webm");
        let preview = preview_path(output);
        assert_eq!(preview.extension().unwrap(), "webm");
    }

    #[test]
    fn test_preview_path_mkv_preserved() {
        let output = Path::new("video.mkv");
        let preview = preview_path(output);
        assert_eq!(preview.extension().unwrap(), "mkv");
    }
}
