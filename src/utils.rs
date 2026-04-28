use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub const VIDEO_EXTENSIONS: &[&str] = &["mp4", "mov", "avi", "mkv", "webm"];

pub fn find_video_files(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut video_files = Vec::new();

    for entry in WalkDir::new(dir)
        .max_depth(10)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file()
            && let Some(extension) = path.extension().and_then(|s| s.to_str())
            && VIDEO_EXTENSIONS.contains(&extension.to_lowercase().as_str())
        {
            video_files.push(path.to_path_buf());
        }
    }
    Ok(video_files)
}

pub fn is_video_file(path: &Path) -> bool {
    path.is_file()
        && path
            .extension()
            .and_then(|s| s.to_str())
            .map(|s| VIDEO_EXTENSIONS.contains(&s.to_lowercase().as_str()))
            .unwrap_or(false)
}

pub fn check_ffmpeg() -> Result<()> {
    use std::process::Command;
    Command::new("ffmpeg")
        .arg("-version")
        .output()
        .context("FFmpeg not found. Please install FFmpeg: https://ffmpeg.org/download.html")?;
    Ok(())
}

pub fn check_ffprobe() -> Result<()> {
    use std::process::Command;
    Command::new("ffprobe")
        .arg("-version")
        .output()
        .context("FFprobe not found. Please install FFmpeg (which includes ffprobe): https://ffmpeg.org/download.html")?;
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

pub fn escape_ffmpeg_filter_path(path: &Path) -> String {
    path.to_string_lossy()
        .replace('\\', "\\\\")
        .replace('\'', "'\\''")
}

pub struct TempDir {
    path: PathBuf,
    keep: bool,
}

impl TempDir {
    pub fn new(prefix: &str) -> std::io::Result<Self> {
        let path = std::env::temp_dir().join(format!("{}-{}", prefix, std::process::id()));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path)?;
        Ok(Self { path, keep: false })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn into_path(self) -> PathBuf {
        self.path
    }

    pub fn keep(mut self) {
        self.keep = true;
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        if !self.keep {
            let _ = std::fs::remove_dir_all(&self.path);
        }
    }
}

pub struct TempFile {
    path: PathBuf,
}

impl TempFile {
    pub fn new(prefix: &str, ext: &str) -> std::io::Result<Self> {
        let path = std::env::temp_dir().join(format!("{}-{}.{}", prefix, std::process::id(), ext));
        Ok(Self { path })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn into_path(self) -> PathBuf {
        self.path
    }

    pub fn with_tmp<F: FnOnce(&Path) -> std::io::Result<()>>(path: &Path, f: F) -> std::io::Result<PathBuf> {
        let tmp = path.with_extension("tmp");
        f(&tmp)?;
        std::fs::rename(&tmp, path)?;
        Ok(path.to_path_buf())
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_find_video_files() -> Result<()> {
        let dir = tempdir()?;
        let video1 = dir.path().join("video1.mp4");
        let video2 = dir.path().join("subdir/video2.mov");
        let text_file = dir.path().join("text.txt");
        let unsupported_video = dir.path().join("unsupported.ogg");

        fs::write(&video1, "dummy video content")?;
        fs::create_dir(dir.path().join("subdir"))?;
        fs::write(&video2, "dummy video content")?;
        fs::write(&text_file, "dummy text content")?;
        fs::write(&unsupported_video, "dummy video content")?;

        let video3 = dir.path().join("video3.mkv");
        fs::write(&video3, "dummy video content")?;

        let found_files = find_video_files(dir.path())?;
        assert_eq!(found_files.len(), 3);
        assert!(found_files.contains(&video1));
        assert!(found_files.contains(&video2));
        assert!(found_files.contains(&video3));
        assert!(!found_files.contains(&text_file));
        assert!(!found_files.contains(&unsupported_video));

        Ok(())
    }

    #[test]
    fn test_is_video_file() {
        let dir = tempdir().unwrap();
        let video = dir.path().join("test.mp4");
        let txt = dir.path().join("test.txt");
        fs::write(&video, "x").unwrap();
        fs::write(&txt, "x").unwrap();

        assert!(is_video_file(&video));
        assert!(!is_video_file(&txt));
        assert!(!is_video_file(&dir.path().join("nonexistent.mp4")));
    }
}
