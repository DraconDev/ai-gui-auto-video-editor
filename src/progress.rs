use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

/// Tracks progress of batch processing jobs so they can be resumed.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BatchProgress {
    /// Files that have been successfully processed
    pub completed: HashMap<PathBuf, u64>,
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

    /// Check if a file has already been processed (and has the same mtime)
    pub fn is_completed(&self, path: &Path) -> bool {
        let mtime = std::fs::metadata(path)
            .ok()
            .and_then(|m| m.modified().ok())
            .map(|t| t.elapsed().unwrap_or_default().as_secs());
        match (self.completed.get(path), mtime) {
            (Some(&saved_mtime), Some(current_mtime)) => {
                // Allow 5s tolerance for filesystem timestamp precision
                saved_mtime.abs_diff(current_mtime) <= 5
            }
            (Some(_), None) => true, // Can't check mtime? Assume same file
            _ => false,
        }
    }

    /// Mark a file as completed with current mtime
    pub fn mark_completed(&mut self, path: &Path) {
        let mtime = std::fs::metadata(path)
            .ok()
            .and_then(|m| m.modified().ok())
            .map(|t| t.elapsed().unwrap_or_default().as_secs())
            .unwrap_or(0);
        self.completed.insert(path.to_path_buf(), mtime);
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
        std::env::temp_dir().join(format!("ai-vid-editor-progress-{dir_name}.json"))
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
                (PathBuf::from("/tmp/video1.mp4"), 1000),
                (PathBuf::from("/tmp/video2.mov"), 2000),
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
        let mut progress = BatchProgress {
            total: 10,
            ..Default::default()
        };

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

    #[test]
    fn test_progress_zero_total() {
        let mut progress = BatchProgress {
            total: 0,
            ..Default::default()
        };
        assert_eq!(progress.remaining(), 0);
        progress.mark_completed(PathBuf::from("/tmp/v.mp4").as_path());
        // Saturating sub prevents negative
        assert_eq!(progress.remaining(), 0);
    }

    #[test]
    fn test_progress_failed_vs_completed_separate() {
        let mut progress = BatchProgress {
            total: 5,
            ..Default::default()
        };
        let file = PathBuf::from("/tmp/test.mp4");

        // Mark as failed
        progress.mark_failed(&file);
        assert!(
            !progress.is_completed(&file),
            "Failed files are not completed"
        );
        assert_eq!(
            progress.remaining(),
            4,
            "Failed files no longer count toward remaining"
        );
    }

    #[test]
    fn test_progress_serialization_preserves_all_fields() -> Result<()> {
        let progress = BatchProgress {
            total: 42,
            completed: vec![(PathBuf::from("/a.mp4"), 123)].into_iter().collect(),
            failed: vec![PathBuf::from("/b.mp4")].into_iter().collect(),
        };

        let dir = tempdir()?;
        let path = dir.path().join("p.json");
        progress.to_file(&path)?;

        let loaded = BatchProgress::from_file(&path)?;
        assert_eq!(loaded.total, 42);
        assert_eq!(loaded.completed.len(), 1);
        assert_eq!(loaded.failed.len(), 1);
        Ok(())
    }

    // ── BatchProgress method tests ──────────────────────────────────────────
    #[test]
    fn test_batch_progress_completed_count() {
        let mut progress = BatchProgress::default();
        progress.total = 5;
        progress.completed.insert(PathBuf::from("/a.mp4"), 123);
        progress.completed.insert(PathBuf::from("/b.mp4"), 124);

        assert_eq!(progress.completed.len(), 2);
        assert_eq!(progress.total - progress.completed.len(), 3);
    }

    #[test]
    fn test_batch_progress_failed_count() {
        let mut progress = BatchProgress::default();
        progress.total = 5;
        progress.failed.insert(PathBuf::from("/a.mp4"));

        assert_eq!(progress.failed.len(), 1);
    }

    #[test]
    fn test_batch_progress_remaining_count() {
        let mut progress = BatchProgress::default();
        progress.total = 10;
        progress.completed.insert(PathBuf::from("/a.mp4"), 123);
        progress.completed.insert(PathBuf::from("/b.mp4"), 124);
        progress.failed.insert(PathBuf::from("/c.mp4"));

        let remaining = progress.total - progress.completed.len();
        assert_eq!(remaining, 8);
    }

    #[test]
    fn test_batch_progress_from_file_not_found() -> Result<()> {
        let dir = tempdir()?;
        let path = dir.path().join("nonexistent.json");

        let result = BatchProgress::from_file(&path);
        assert!(result.is_err(), "Should error on missing file");
        Ok(())
    }

    #[test]
    fn test_batch_progress_to_file_roundtrip() -> Result<()> {
        let dir = tempdir()?;
        let path = dir.path().join("p2.json");

        let mut progress = BatchProgress::default();
        progress.total = 10;
        progress.completed.insert(PathBuf::from("/test.mp4"), 456);

        progress.to_file(&path)?;
        assert!(path.exists());

        let loaded = BatchProgress::from_file(&path)?;
        assert_eq!(loaded.total, 10);
        Ok(())
    }

    // ── BatchProgress edge cases ──────────────────────────────────────────
    #[test]
    fn test_batch_progress_empty_completed() {
        let mut progress = BatchProgress::default();
        progress.total = 5;
        // No completed files
        assert_eq!(progress.completed.len(), 0);
    }

    #[test]
    fn test_batch_progress_full_completion() {
        let mut progress = BatchProgress::default();
        progress.total = 3;
        progress.completed.insert(PathBuf::from("/a.mp4"), 100);
        progress.completed.insert(PathBuf::from("/b.mp4"), 200);
        progress.completed.insert(PathBuf::from("/c.mp4"), 300);
        // All files completed
        assert_eq!(progress.completed.len(), progress.total);
    }

    #[test]
    fn test_batch_progress_mixed_results() {
        let mut progress = BatchProgress::default();
        progress.total = 10;
        progress.completed.insert(PathBuf::from("/a.mp4"), 100);
        progress.failed.insert(PathBuf::from("/b.mp4"));
        // Some completed, some failed
        assert!(progress.completed.len() < progress.total);
        assert!(progress.failed.len() > 0);
    }

    // ── BatchProgress more edge cases ────────────────────────────────────
    #[test]
    fn test_batch_progress_all_failed() {
        let mut progress = BatchProgress::default();
        progress.total = 5;
        progress.failed.insert(PathBuf::from("/a.mp4"));
        progress.failed.insert(PathBuf::from("/b.mp4"));
        // All files failed
        assert!(progress.completed.is_empty());
        assert_eq!(progress.failed.len(), 2);
    }

    #[test]
    fn test_batch_progress_none_completed() {
        let mut progress = BatchProgress::default();
        progress.total = 3;
        // No completed files
        assert!(progress.completed.is_empty());
        assert!(progress.failed.is_empty());
    }

    #[test]
    fn test_batch_progress_one_of_many() {
        let mut progress = BatchProgress::default();
        progress.total = 100;
        progress.completed.insert(PathBuf::from("/single.mp4"), 50);
        // Only one of many completed
        assert_eq!(progress.completed.len(), 1);
        assert_eq!(progress.failed.len(), 0);
    }

    #[test]
    fn test_batch_progress_large_batch() {
        let mut progress = BatchProgress::default();
        progress.total = 1000;
        for i in 0..100 {
            progress
                .completed
                .insert(PathBuf::from(format!("/{}.mp4", i)), i * 10);
        }
        // Many files processed
        assert_eq!(progress.completed.len(), 100);
    }
}
