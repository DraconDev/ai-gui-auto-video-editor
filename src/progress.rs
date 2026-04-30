use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Tracks progress of batch processing jobs so they can be resumed.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BatchProgress {
    /// Files that have been successfully processed
    pub completed: HashSet<PathBuf>,
    /// Files that failed processing
    pub failed: HashSet<PathBuf>,
    /// Total number of files in the batch
    pub total: usize,
}

impl BatchProgress {
    /// Load progress from a state file
    pub fn from_file(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read progress file: {:?}", path))?;
        let progress: BatchProgress = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse progress file: {:?}", path))?;
        Ok(progress)
    }

    /// Save progress to a state file
    pub fn to_file(&self, path: &Path) -> Result<()> {
        let content = serde_json::to_string_pretty(self).context("Failed to serialize progress")?;
        std::fs::write(path, content)
            .with_context(|| format!("Failed to write progress file: {:?}", path))?;
        Ok(())
    }

    /// Check if a file has already been processed
    pub fn is_completed(&self, path: &Path) -> bool {
        self.completed.contains(path)
    }

    /// Mark a file as completed
    pub fn mark_completed(&mut self, path: &Path) {
        self.completed.insert(path.to_path_buf());
    }

    /// Mark a file as failed
    pub fn mark_failed(&mut self, path: &Path) {
        self.failed.insert(path.to_path_buf());
    }

    /// Get the number of remaining files
    pub fn remaining(&self) -> usize {
        self.total
            .saturating_sub(self.completed.len() + self.failed.len())
    }

    /// Get the default progress file path for a given input directory
    pub fn default_path(input_dir: &Path) -> PathBuf {
        let dir_name = input_dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("default");
        std::env::temp_dir().join(format!("ai-vid-editor-progress-{}.json", dir_name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_progress_tracking() {
        let mut progress = BatchProgress {
            total: 5,
            ..Default::default()
        };

        let file1 = PathBuf::from("/tmp/video1.mp4");
        let file2 = PathBuf::from("/tmp/video2.mp4");

        assert!(!progress.is_completed(&file1));
        progress.mark_completed(&file1);
        assert!(progress.is_completed(&file1));

        progress.mark_failed(&file2);
        assert_eq!(progress.remaining(), 3);
    }

    #[test]
    fn test_progress_serialization_roundtrip() -> Result<()> {
        let progress = BatchProgress {
            total: 3,
            completed: vec![
                PathBuf::from("/tmp/video1.mp4"),
                PathBuf::from("/tmp/video2.mov"),
            ]
            .into_iter()
            .collect(),
            failed: vec![PathBuf::from("/tmp/video3.avi")].into_iter().collect(),
        };

        let dir = tempdir()?;
        let path = dir.path().join("progress.json");
        progress.to_file(&path)?;

        let loaded = BatchProgress::from_file(&path)?;
        assert_eq!(loaded.total, 3);
        assert_eq!(loaded.completed.len(), 2);
        assert_eq!(loaded.failed.len(), 1);
        assert!(loaded.is_completed(PathBuf::from("/tmp/video1.mp4").as_path()));
        Ok(())
    }

    #[test]
    fn test_progress_remaining_calculation() {
        let mut progress = BatchProgress::default();
        progress.total = 10;

        assert_eq!(progress.remaining(), 10);

        progress.mark_completed(PathBuf::from("/tmp/v1.mp4").as_path());
        assert_eq!(progress.remaining(), 9);

        progress.mark_failed(PathBuf::from("/tmp/v2.mp4").as_path());
        assert_eq!(progress.remaining(), 8);

        // Saturating at 0 when total is exceeded
        progress.total = 3;
        progress.mark_completed(PathBuf::from("/tmp/a.mp4").as_path());
        progress.mark_completed(PathBuf::from("/tmp/b.mp4").as_path());
        progress.mark_completed(PathBuf::from("/tmp/c.mp4").as_path());
        progress.mark_completed(PathBuf::from("/tmp/d.mp4").as_path());
        assert_eq!(progress.remaining(), 0);
    }

    #[test]
    fn test_progress_default_path() {
        let input_dir = PathBuf::from("/videos/holiday");
        let path = BatchProgress::default_path(&input_dir);
        let path_str = path.to_string_lossy();
        assert!(path_str.contains("ai-vid-editor-progress"));
        assert!(path_str.contains("holiday"));
    }

    #[test]
    fn test_progress_default_path_fallback() {
        let input_dir = PathBuf::from("/");
        let path = BatchProgress::default_path(&input_dir);
        let path_str = path.to_string_lossy();
        assert!(path_str.contains("default"));
    }
}
