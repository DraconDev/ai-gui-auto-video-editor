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
