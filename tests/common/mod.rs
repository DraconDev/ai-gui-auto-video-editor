use anyhow::Context;
use std::path::PathBuf;
use std::process::Command;

#[allow(dead_code)]
pub fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

#[allow(dead_code)]
pub fn test_video_path() -> PathBuf {
    let path = fixtures_dir().join("test_video_temp.mp4");
    if !path.exists() {
        create_test_video_with_silence(&path, 6);
    }
    path
}

pub fn create_test_video_with_silence(output_path: &std::path::Path, duration_secs: u32) -> bool {
    let status = Command::new("ffmpeg")
        .args([
            "-f",
            "lavfi",
            "-i",
            &format!("sine=frequency=1000:duration={}", duration_secs),
            "-f",
            "lavfi",
            "-i",
            &format!("color=c=black:s=320x240:d={}", duration_secs),
            "-af",
            "volume=0:enable='between(t,1,2)+between(t,4,5)'",
            "-c:v",
            "libx264",
            "-c:a",
            "aac",
            "-shortest",
            "-y",
            output_path.to_str().unwrap(),
        ])
        .status()
        .is_ok();

    status && output_path.exists()
}

#[allow(dead_code)]
pub fn create_test_audio_file(output_path: &std::path::Path, duration_secs: u32) -> bool {
    let status = Command::new("ffmpeg")
        .args([
            "-f",
            "lavfi",
            "-i",
            &format!("sine=frequency=440:duration={}", duration_secs),
            "-c:a",
            "aac",
            "-y",
            output_path.to_str().unwrap(),
        ])
        .status()
        .is_ok();

    status && output_path.exists()
}

#[allow(dead_code)]
pub fn create_test_watermark_png(output_path: &std::path::Path, size: u32) -> bool {
    use std::process::Command;
    let status = Command::new("python3")
        .args([
            "-c",
            &format!(
                "from PIL import Image; img = Image.new('RGBA', ({}, {}), (255, 0, 0, 200)); img.save('{}')",
                size,
                size,
                output_path.to_str().unwrap()
            ),
        ])
        .status()
        .is_ok();

    status && output_path.exists()
}

#[allow(dead_code)]
pub fn has_ffmpeg() -> bool {
    Command::new("ffmpeg").arg("-version").status().is_ok()
}

#[allow(dead_code)]
pub fn has_ffprobe() -> bool {
    Command::new("ffprobe").arg("-version").status().is_ok()
}

/// Helper: create a small test video using ffmpeg (video+audio, 320x240, libx264/aac).
/// Delegates to the shared crate helper so integration and unit tests use the same code.
pub fn create_test_video(output_path: &std::path::Path, duration_secs: f32) -> Result<(), String> {
    ai_vid_editor::tests_common::create_test_video(output_path, duration_secs)
}

#[allow(dead_code)]
pub fn test_speech_video_path() -> std::path::PathBuf {
    let path = fixtures_dir().join("test_speech_video.mp4");
    if !path.exists() {
        create_speech_video(
            &path,
            "Hello world. This is a test. Um, let me think. Ah yes, okay.",
            8,
        );
    }
    path
}

#[allow(dead_code)]
pub fn create_speech_video(output_path: &std::path::Path, text: &str, duration_secs: u32) -> bool {
    use std::fs;
    use std::io::Write;
    use std::process::Command;

    let temp_dir = output_path
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    let wav_path = temp_dir.join("temp_speech.wav");
    let mp3_path = temp_dir.join("temp_speech.mp3");

    let espeak_ok = Command::new("espeak")
        .args(["-w", wav_path.to_str().unwrap(), text])
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    if !espeak_ok || !wav_path.exists() {
        let _ = fs::remove_file(&wav_path);
        let _ = fs::remove_file(&mp3_path);
        return false;
    }

    let ffmpeg_ok = Command::new("ffmpeg")
        .args([
            "-f",
            "lavfi",
            "-i",
            &format!("color=c=black:s=320x240:d={}", duration_secs),
            "-i",
            wav_path.to_str().unwrap(),
            "-c:v",
            "libx264",
            "-c:a",
            "aac",
            "-shortest",
            "-y",
            output_path.to_str().unwrap(),
        ])
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    let _ = fs::remove_file(&wav_path);
    let _ = fs::remove_file(&mp3_path);

    ffmpeg_ok && output_path.exists()
}
