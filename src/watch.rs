use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use anyhow::Result;

use crate::analyzer::FfmpegAnalyzer;
use crate::batch_processor::{
    process_single_file_with_intro_outro_progress, CachingDurationGetter, FfmpegEditor,
};
use crate::config::Config;

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
            println!(
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
                println!("\n[{}] [NEW FILE] {label} {:?}", now, path);

                let file_name = path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "output.mp4".to_string());
                let output_path = params.output_dir.join(&file_name);

                println!("[{}] [START] {label} Processing {}...", now, file_name);

                if params.notify {
                    notify_processing(&path);
                }

                let start_time = Instant::now();

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
                        println!(
                            "[{}] [{:>6.1}%] {label} {} - {}",
                            now,
                            p.fraction * 100.0,
                            file_name,
                            p.stage
                        );
                    },
                );

                if params.dry_run {
                    let elapsed = start_time.elapsed().as_secs_f32();
                    println!(
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
                        println!(
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
                        eprintln!(
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
        if last_processed.is_none() || heartbeat % 12 == 0 {
            let status = match &last_processed {
                Some(name) => format!("last: {name}"),
                None => "waiting for files...".to_string(),
            };
            println!("[{}] [{status}] {label}", timestamp());
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

fn timestamp() -> String {
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
