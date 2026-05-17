//! Shared test helpers for library unit tests.
//! Also re-exported via `pub mod tests_common` in lib.rs so integration tests
//! (`tests/`) can access them as `crate_name::tests_common::`.

use std::path::Path;
use std::process::Command;

/// Helper: create a small test video using ffmpeg (video+audio, 320x240, libx264/aac).
/// Mirrors the helper historically duplicated across preview.rs, scene_detection.rs,
/// and editor.rs.
pub fn create_test_video(output_path: &Path, duration_secs: f32) -> Result<(), String> {
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
            output_path
                .to_str()
                .ok_or_else(|| "non-UTF-8 path".to_string())?,
        ])
        .status()
        .map_err(|_| "ffmpeg not found".to_string())?;
    if status.success() {
        Ok(())
    } else {
        Err("ffmpeg test video creation failed".to_string())
    }
}

/// Check whether ffmpeg is available on this system.
pub fn has_ffmpeg() -> bool {
    Command::new("ffmpeg").arg("-version").status().is_ok()
}

/// Helper: create a small test image (50x50 red square PNG) using ffmpeg.
pub fn create_test_image(output_path: &Path) -> Result<(), String> {
    let status = Command::new("ffmpeg")
        .args([
            "-f",
            "lavfi",
            "-i",
            "color=c=red:size=50x50",
            "-frames:v",
            "1",
            "-y",
            output_path
                .to_str()
                .ok_or_else(|| "non-UTF-8 path".to_string())?,
        ])
        .status()
        .map_err(|_| "ffmpeg not found".to_string())?;
    if status.success() {
        Ok(())
    } else {
        Err("ffmpeg test image creation failed".to_string())
    }
}
