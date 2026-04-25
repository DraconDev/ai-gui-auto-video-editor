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
        let content = serde_json::to_string_pretty(self)
            .context("Failed to serialize progress")?;
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
        self.total.saturating_sub(self.completed.len() + self.failed.len())
    }

    /// Get the default progress file path for a given input directory
    pub fn default_path(input_dir: &Path) -> PathBuf {
        let dir_name = input_dir.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("default");
        std::env::temp_dir().join(format!("ai-vid-editor-progress-{}.json", dir_name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
