use std::path::PathBuf;
use std::sync::Arc;

use crate::config::Config;

/// Represents a single file in the processing queue
#[derive(Clone, Debug)]
pub struct QueuedFile {
    pub path: PathBuf,
    pub output_dir: PathBuf,
    pub preset: String,
    pub config: Arc<Config>,
}

/// Events for the queue channel
#[derive(Debug, Clone)]
pub enum QueueEvent {
    /// Add files to the queue
    AddFiles(Vec<QueuedFile>),
    /// Clear all queued files
    Clear,
}
