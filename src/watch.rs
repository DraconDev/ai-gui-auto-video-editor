use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use anyhow::Result;
use tracing::{error, info};

use crate::analyzer::FfmpegAnalyzer;
use crate::batch_processor::{
    CachingDurationGetter, process_single_file_with_intro_outro_progress,
};
use crate::config::Config;
use crate::editor::FfmpegEditor;

/// Shared configuration for a single watch folder
pub struct WatchFolderConfig<'a> {
    pub watch_dir: &'a PathBuf,
    pub output_dir: &'a PathBuf,
    pub config: &'a Config,
    pub intro: Option<PathBuf>,
    pub outro: Option<PathBuf>,
    pub notify: bool,
    pub dry_run: bool,
    pub folder_label: &'a str,
}

/// Track config file mtime for hot-reload detection.
#[allow(dead_code)]
pub struct ConfigWatcher {
    path: Option<PathBuf>,
    last_mtime: Option<std::time::SystemTime>,
}

#[allow(dead_code)]
impl ConfigWatcher {
    pub fn new() -> Self {
        Self {
            path: None,
            last_mtime: None,
        }
    }

    /// Check if the config file has changed since last check.
    /// If it changed, return true and update the tracked mtime.
    pub fn check_for_reload(&mut self, config_path: Option<&Path>) -> bool {
        let path = match config_path {
            Some(p) => p,
            None => return false,
        };
        let mtime = match std::fs::metadata(path).and_then(|m| m.modified()) {
            Ok(t) => t,
            Err(_) => return false,
        };
        if self.last_mtime.is_some_and(|t| t == mtime) {
            return false;
        }
        self.last_mtime = Some(mtime);
        self.path = Some(path.to_path_buf());
        true
    }

    pub fn was_ever_loaded(&self) -> bool {
        self.last_mtime.is_some()
    }
}

/// Run the watch loop for a single folder.
/// Extracted to avoid duplication between single-watch and multi-watch modes.
pub fn run_watch_loop(params: WatchFolderConfig) -> Result<()> {
    let mut processed: HashSet<PathBuf> = HashSet::new();
    let analyzer = FfmpegAnalyzer;
    let editor = FfmpegEditor::new(params.config.video.hw_accel);
    let duration_getter = CachingDurationGetter::new();

    let mut heartbeat = 0u32;
    let mut last_processed: Option<String> = None;
    let label = params.folder_label;

    loop {
        std::thread::sleep(Duration::from_secs(params.config.watch.interval));

        heartbeat += 1;
        if heartbeat.is_multiple_of(6) {
            info!(
                "[{}] Watching {} {label} for new files...",
                timestamp(),
                params.watch_dir.display()
            );
        }

        if let Ok(entries) = std::fs::read_dir(params.watch_dir) {
            for entry in entries.flatten() {
                let path = entry.path();

                if !is_new_video(&path, &processed) {
                    continue;
                }

                let now = timestamp();
                info!(
                    "
[{}] [NEW FILE] {label} {:?}",
                    now, path
                );

                let file_name = path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "output.mp4".to_string());
                let output_path = params.output_dir.join(&file_name);

                info!("[{}] [START] {label} Processing {}...", now, file_name);

                if params.notify {
                    notify_processing(&path);
                }

                let start_time = Instant::now();
                let file_name_for_progress = file_name.clone();

                let result = process_single_file_with_intro_outro_progress(
                    path.clone(),
                    output_path.clone(),
                    params.config,
                    &analyzer,
                    &editor,
                    &duration_getter,
                    params.intro.clone(),
                    params.outro.clone(),
                    move |p| {
                        let now = timestamp();
                        info!(
                            "[{}] [{:>6.1}%] {label} {} - {}",
                            now,
                            p.fraction * 100.0,
                            file_name_for_progress,
                            p.stage
                        );
                    },
                );

                if params.dry_run {
                    let elapsed = start_time.elapsed().as_secs_f32();
                    info!(
                        "[{}] [DRY-RUN] {label} {} -> {} ({:.1}s)",
                        timestamp(),
                        file_name,
                        output_path.display(),
                        elapsed
                    );
                    last_processed = Some(file_name.clone());
                    processed.insert(path);
                    continue;
                }

                match &result {
                    Ok(_) => {
                        let elapsed = start_time.elapsed().as_secs_f32();
                        info!(
                            "[{}] [{:>7}] {label} {} -> {} ({:.1}s)",
                            timestamp(),
                            "DONE",
                            file_name,
                            output_path.display(),
                            elapsed
                        );
                        if params.notify {
                            notify_complete(&path, &output_path);
                        }
                        last_processed = Some(file_name.clone());
                        processed.insert(path);
                    }
                    Err(e) => {
                        let elapsed = start_time.elapsed().as_secs_f32();
                        last_processed = Some(format!("{} (error)", file_name));
                        error!(
                            "[{}] [ERROR] {label} {} failed after {:.1}s: {}",
                            timestamp(),
                            file_name,
                            elapsed,
                            e
                        );
                        if params.notify {
                            notify_error(&path, &e.to_string());
                        }
                        processed.insert(path);
                    }
                }
            }
        }

        // Show status line when nothing processed recently
        if last_processed.is_none() || heartbeat.is_multiple_of(12) {
            let status = match &last_processed {
                Some(name) => format!("last: {name}"),
                None => "waiting for files...".to_string(),
            };
            info!("[{}] [{status}] {label}", timestamp());
        }
    }
}

fn is_new_video(path: &Path, processed: &HashSet<PathBuf>) -> bool {
    if processed.contains(path) {
        return false;
    }
    let ext = match path.extension().and_then(|e| e.to_str()) {
        Some(e) => e.to_lowercase(),
        None => return false,
    };
    crate::utils::VIDEO_EXTENSIONS.contains(&ext.as_str())
}

pub(crate) fn timestamp() -> String {
    use std::time::SystemTime;
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = now.as_secs();
    let hours = (secs / 3600) % 24;
    let minutes = (secs / 60) % 60;
    let seconds = secs % 60;
    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}

fn notify_processing(path: &Path) {
    crate::utils::send_notification("Processing Started", &format!("{}", path.display()));
}

fn notify_complete(path: &Path, output: &Path) {
    crate::utils::send_notification(
        "Processing Complete",
        &format!("{} -> {}", path.display(), output.display()),
    );
}

fn notify_error(path: &Path, error: &str) {
    crate::utils::send_notification(
        "Processing Error",
        &format!("{} failed: {}", path.display(), error),
    );
}
