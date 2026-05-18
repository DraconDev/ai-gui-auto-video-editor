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

#[cfg(feature = "notify-rust")]
pub fn send_notification(summary: &str, body: &str) {
    let _ = notify_rust::Notification::new()
        .summary(summary)
        .body(body)
        .show();
}

#[cfg(not(feature = "notify-rust"))]
pub fn send_notification(_summary: &str, _body: &str) {}

pub fn escape_ffmpeg_filter_path(path: &Path) -> String {
    path.to_string_lossy()
        .replace('\\', "\\\\")
        .replace('\'', "'\\''")
}

#[must_use]
pub struct TempDir {
    path: PathBuf,
    keep: bool,
}

impl TempDir {
    pub fn new(prefix: &str) -> std::io::Result<Self> {
        use std::sync::atomic::{AtomicU64, Ordering};
        use std::time::{SystemTime, UNIX_EPOCH};
        static COUNTER: AtomicU64 = AtomicU64::new(0);

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let count = COUNTER.fetch_add(1, Ordering::SeqCst);
        let path = std::env::temp_dir().join(format!(
            "{}-{}-{}-{}",
            prefix,
            std::process::id(),
            count,
            now
        ));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path)?;
        Ok(Self { path, keep: false })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn into_path(self) -> PathBuf {
        let path = self.path.clone();
        std::mem::forget(self);
        path
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

#[must_use]
pub struct TempFile {
    path: PathBuf,
}

impl TempFile {
    pub fn new(prefix: &str, ext: &str) -> std::io::Result<Self> {
        use std::sync::atomic::{AtomicU64, Ordering};
        use std::time::{SystemTime, UNIX_EPOCH};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let count = COUNTER.fetch_add(1, Ordering::SeqCst);
        let path = std::env::temp_dir().join(format!(
            "{}-{}-{}-{:.0}.{}",
            prefix,
            std::process::id(),
            count,
            now as f64,
            ext
        ));
        Ok(Self { path })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn into_path(self) -> PathBuf {
        let path = self.path.clone();
        std::mem::forget(self);
        path
    }

    pub fn with_tmp<F: FnOnce(&Path) -> std::io::Result<()>>(
        path: &Path,
        f: F,
    ) -> std::io::Result<PathBuf> {
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

    #[test]
    fn test_is_video_file_case_insensitive() {
        let dir = tempdir().unwrap();
        let video_mp4 = dir.path().join("video.MP4");
        let video_mov = dir.path().join("video.mOv");
        let video_avi = dir.path().join("video.AVI");

        fs::write(&video_mp4, "x").unwrap();
        fs::write(&video_mov, "x").unwrap();
        fs::write(&video_avi, "x").unwrap();

        assert!(is_video_file(&video_mp4), "MP4 should be recognized");
        assert!(is_video_file(&video_mov), "mOv should be recognized");
        assert!(is_video_file(&video_avi), "AVI should be recognized");
    }

    #[test]
    fn test_escape_ffmpeg_filter_path_simple() {
        // Simple path without special characters
        let path = Path::new("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf");
        let escaped = escape_ffmpeg_filter_path(path);

        assert_eq!(escaped, "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf");
    }

    #[test]
    fn test_escape_ffmpeg_filter_path_with_single_quotes() {
        // Path with single quotes should be escaped for FFmpeg
        // FFmpeg escaping: ' -> '\''
        let path = Path::new("/path/to/file's/font.ttf");
        let escaped = escape_ffmpeg_filter_path(path);

        // The escaped form should be /path/to/file'\''s/font.ttf
        assert!(
            escaped.contains("'\\''"),
            "Escaped path should contain FFmpeg escaped single quote (\\'\\'\\')"
        );
    }

    #[test]
    fn test_escape_ffmpeg_filter_path_with_backslashes() {
        // Path with backslashes (Windows paths)
        let path = Path::new("C:\\Users\\Test\\font.ttf");
        let escaped = escape_ffmpeg_filter_path(path);

        // Backslashes should be doubled
        assert!(
            escaped.contains("\\\\"),
            "Escaped path should contain doubled backslashes"
        );
    }

    #[test]
    fn test_escape_ffmpeg_filter_path_with_both() {
        // Path with both quotes and backslashes
        let path = Path::new("C:\\Users\\Test\\file's.ttf");
        let escaped = escape_ffmpeg_filter_path(path);

        assert!(escaped.contains("\\\\"), "Should escape backslashes");
        assert!(escaped.contains("\\'"), "Should escape single quotes");
    }

    #[test]
    fn test_temp_dir_keeps_path_when_not_dropped() {
        // Test that TempDir creates the directory
        let temp = TempDir::new("test").unwrap();
        let path = temp.path.clone();

        assert!(path.exists(), "TempDir should create the directory");
        assert!(path.to_string_lossy().contains("test"));
    }

    #[test]
    fn test_temp_dir_cleanup_on_drop() {
        // Test that TempDir removes the directory on drop (unless kept)
        let temp = TempDir::new("cleanup_test").unwrap();
        let path = temp.path.clone();

        assert!(path.exists(), "Directory should exist before drop");

        drop(temp);

        assert!(!path.exists(), "TempDir should remove directory on drop");
    }

    #[test]
    fn test_temp_file_new() {
        // Test that TempFile::new creates a path
        let temp_file = TempFile::new("mytest", "txt").unwrap();
        let path = temp_file.path();

        assert!(path.to_string_lossy().contains("mytest"));
        assert!(path.to_string_lossy().ends_with(".txt"));
    }

    #[test]
    fn test_find_video_files_nested_directories() {
        // Test that find_video_files handles deeply nested directories
        let dir = tempdir().unwrap();

        // Create nested structure
        let nested = dir.path().join("a/b/c/d");
        std::fs::create_dir_all(&nested).unwrap();

        let video = nested.join("deep_video.mp4");
        fs::write(&video, "content").unwrap();

        let found = find_video_files(dir.path()).unwrap();
        assert_eq!(found.len(), 1);
        assert!(found[0].ends_with("deep_video.mp4"));
    }

    #[test]
    fn test_find_video_files_max_depth() {
        // Test that find_video_files respects max_depth (10)
        let dir = tempdir().unwrap();

        // Create a file at depth 15 (beyond max_depth of 10)
        let mut deep_path = dir.path().to_path_buf();
        for i in 0..15 {
            deep_path = deep_path.join(format!("level{}", i));
        }
        std::fs::create_dir_all(&deep_path).unwrap();
        let video = deep_path.join("video.mp4");
        fs::write(&video, "content").unwrap();

        let found = find_video_files(dir.path()).unwrap();

        // File at depth 15 should NOT be found (exceeds max_depth of 10)
        let found_names: Vec<_> = found
            .iter()
            .filter_map(|p| p.file_name())
            .filter_map(|n| n.to_str())
            .collect();

        assert!(
            !found_names.contains(&"video.mp4"),
            "Files beyond max_depth (10) should not be found"
        );
    }

    #[test]
    fn test_video_extensions() {
        // Verify all expected extensions are included
        let extensions: Vec<_> = VIDEO_EXTENSIONS.iter().collect();
        assert!(extensions.contains(&&"mp4"));
        assert!(extensions.contains(&&"mov"));
        assert!(extensions.contains(&&"avi"));
        assert!(extensions.contains(&&"mkv"));
        assert!(extensions.contains(&&"webm"));
        assert_eq!(extensions.len(), 5);
    }

    #[test]
    fn test_escape_ffmpeg_filter_path_empty() {
        let path = Path::new("");
        let escaped = escape_ffmpeg_filter_path(path);
        assert_eq!(escaped, "");
    }

    #[test]
    fn test_escape_ffmpeg_filter_path_unicode() {
        // Unicode characters should pass through unchanged
        let path = Path::new("/path/to/cafe.ttf");
        let escaped = escape_ffmpeg_filter_path(path);
        assert!(escaped.contains("cafe"));
    }

    #[test]
    fn test_find_video_files_empty_dir() -> Result<()> {
        let dir = tempdir()?;
        let found = find_video_files(dir.path())?;
        assert!(found.is_empty());
        Ok(())
    }

    #[test]
    fn test_find_video_files_case_sensitivity() -> Result<()> {
        let dir = tempdir()?;

        // Create files with various case extensions
        let file_mp4 = dir.path().join("video.mp4");
        let file_wmv = dir.path().join("video.wmv"); // Not in VIDEO_EXTENSIONS

        fs::write(&file_mp4, "x")?;
        fs::write(&file_wmv, "x")?;

        let found = find_video_files(dir.path())?;
        assert_eq!(found.len(), 1);
        assert!(found[0].extension().unwrap().to_str() == Some("mp4"));
        Ok(())
    }

    // ── TempDir/TempFile cleanup tests ────────────────────────────────────
    #[test]
    fn test_temp_dir_into_path() {
        let temp = TempDir::new("test").unwrap();
        let path = temp.into_path();
        // into_path takes ownership, dir should still exist (forget prevents cleanup)
        assert!(path.exists());
        // Clean up manually
        let _ = std::fs::remove_dir_all(path);
    }

    #[test]
    fn test_temp_file_new_variant() {
        let temp = TempFile::new("mytest", "mp4").unwrap();
        let path = temp.path();
        assert!(path.extension().unwrap() == "mp4");
        assert!(path.to_string_lossy().contains("mytest"));
    }

    #[test]
    fn test_temp_file_path_preserves_prefix() {
        let temp = TempFile::new("videotest", "mov").unwrap();
        let path_str = temp.path().to_string_lossy();
        assert!(path_str.contains("videotest"));
        assert!(path_str.ends_with(".mov"));
    }

    // ── Path utility tests ──────────────────────────────────────────────
    #[test]
    fn test_escape_ffmpeg_filter_path_multiple_special_chars() {
        let path = Path::new("/path/with spaces/and'quotes/and\\backslash.mp4");
        let escaped = escape_ffmpeg_filter_path(path);
        // Should handle multiple special characters
        assert!(escaped.len() > path.to_string_lossy().len());
    }

    #[test]
    fn test_escape_ffmpeg_filter_path_no_special_chars() {
        let path = Path::new("/simple/path/video.mp4");
        let escaped = escape_ffmpeg_filter_path(path);
        // No special chars to escape
        assert!(escaped.contains("video.mp4"));
    }

    #[test]
    fn test_escape_ffmpeg_filter_path_only_quotes() {
        let path = Path::new("/path/to/file's name.mp4");
        let escaped = escape_ffmpeg_filter_path(path);
        // Single quotes should be escaped
        assert!(escaped.contains("\\'"));
    }

    #[test]
    fn test_escape_ffmpeg_filter_path_handles_various_chars() {
        let path = Path::new("C:\\Users\\test\\video.mp4");
        let escaped = escape_ffmpeg_filter_path(path);
        // Should handle Windows paths with backslashes
        assert!(escaped.len() >= path.to_string_lossy().len());
    }

    #[test]
    fn test_find_video_files_none_found() {
        let dir = tempdir().unwrap();
        // No video files
        let found = find_video_files(dir.path()).unwrap();
        assert!(found.is_empty());
    }

    #[test]
    fn test_find_video_files_multiple_formats() {
        let dir = tempdir().unwrap();
        let formats = ["mp4", "mov", "avi", "mkv"];
        for fmt in formats.iter() {
            let video = dir.path().join(format!("video.{}", fmt));
            std::fs::write(&video, "x").unwrap();
        }
        let found = find_video_files(dir.path()).unwrap();
        assert_eq!(found.len(), 4);
    }

    #[test]
    fn test_is_video_file_symlink() {
        let dir = tempdir().unwrap();
        let video = dir.path().join("video.mp4");
        std::fs::write(&video, "x").unwrap();
        let link = dir.path().join("link.mp4");
        #[cfg(unix)]
        std::os::unix::fs::symlink(&video, &link).unwrap();
        // Symlink to video file should be recognized
        if link.exists() {
            assert!(is_video_file(&link));
        }
    }
}
