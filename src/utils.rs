use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

pub fn check_ffmpeg() -> Result<()> {
    Command::new("ffmpeg")
        .arg("-version")
        .output()
        .context("FFmpeg not found. Please install FFmpeg: https://ffmpeg.org/download.html")?;
    Ok(())
}

pub fn check_ffprobe() -> Result<()> {
    Command::new("ffprobe")
        .arg("-version")
        .output()
        .context(
            "FFprobe not found. Please install FFmpeg (which includes ffprobe): https://ffmpeg.org/download.html",
        )?;
    Ok(())
}

pub fn check_ffmpeg_available() -> bool {
    std::process::Command::new("ffmpeg")
        .arg("-version")
        .output()
        .is_ok()
}

pub fn check_ffprobe_available() -> bool {
    std::process::Command::new("ffprobe")
        .arg("-version")
        .output()
        .is_ok()
}
