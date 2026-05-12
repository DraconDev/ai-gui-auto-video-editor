use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
    mpsc::{self, Receiver},
};
use std::time::{Duration, Instant};

use super::{FolderState, ProcessingStatus, QueueEvent, WatcherEvent};
use crate::Config;
use crate::FfmpegAnalyzer;
use crate::FfmpegDurationGetter;
use crate::FfmpegEditor;
use crate::batch_processor::ProcessingProgress;
use crate::batch_processor::process_single_file_with_intro_outro_progress;
use crate::config::FolderSettings;

#[cfg(feature = "notify-rust")]
fn send_desktop_notification(title: &str, body: &str) {
    let _ = notify_rust::Notification::new()
        .summary(title)
        .body(body)
        .show();
}

#[cfg(not(feature = "notify-rust"))]
fn send_desktop_notification(_title: &str, _body: &str) {}

pub(crate) fn spawn_watcher(
    config: Config,
    folders: Vec<FolderState>,
    notify: bool,
) -> (Receiver<WatcherEvent>, Arc<AtomicBool>, Arc<AtomicBool>) {
    let (tx, rx) = mpsc::channel();
    let stop = Arc::new(AtomicBool::new(false));
    let thread_stop = Arc::clone(&stop);
    let shutdown_complete = Arc::new(AtomicBool::new(false));
    let shutdown_complete_thread = Arc::clone(&shutdown_complete);

    std::thread::spawn(move || {
        watch_folders_loop(config, folders, tx, thread_stop, notify);
        shutdown_complete_thread.store(true, Ordering::SeqCst);
    });

    (rx, stop, shutdown_complete)
}

fn watch_folders_loop(
    config: Config,
    folders: Vec<FolderState>,
    tx: mpsc::Sender<WatcherEvent>,
    stop: Arc<AtomicBool>,
    notify: bool,
) {
    const MAX_ATTEMPTED: usize = 10_000;
    let poll_interval = Duration::from_secs(config.watch.interval.max(1));
    let mut attempted = VecDeque::new();
    let intro = config.paths.intro.clone();
    let outro = config.paths.outro.clone();
    let analyzer = FfmpegAnalyzer;
    let editor = FfmpegEditor::new(config.video.hw_accel);
    let duration_getter = FfmpegDurationGetter;

    if tx
        .send(WatcherEvent::Log {
            message: format!("Watching {} folder(s) for new videos", folders.len()),
            success: true,
        })
        .is_err()
    {
        return;
    }
    if tx
        .send(WatcherEvent::Status(ProcessingStatus::Watching))
        .is_err()
    {
        return;
    }

    while !stop.load(Ordering::SeqCst) {
        for folder in &folders {
            if stop.load(Ordering::SeqCst) {
                return;
            }

            if let Err(err) = std::fs::create_dir_all(&folder.input) {
                if tx
                    .send(WatcherEvent::Log {
                        message: format!(
                            "Failed to create input folder {}: {}",
                            folder.input.display(),
                            err
                        ),
                        success: false,
                    })
                    .is_err()
                {
                    return;
                }
                continue;
            }

            if let Err(err) = std::fs::create_dir_all(&folder.output) {
                if tx
                    .send(WatcherEvent::Log {
                        message: format!(
                            "Failed to create output folder {}: {}",
                            folder.output.display(),
                            err
                        ),
                        success: false,
                    })
                    .is_err()
                {
                    return;
                }
                continue;
            }

            let entries = match std::fs::read_dir(&folder.input) {
                Ok(entries) => entries,
                Err(err) => {
                    if tx
                        .send(WatcherEvent::Log {
                            message: format!(
                                "Failed to read watch folder {}: {}",
                                folder.input.display(),
                                err
                            ),
                            success: false,
                        })
                        .is_err()
                    {
                        return;
                    }
                    continue;
                }
            };

            for entry in entries.flatten() {
                if stop.load(Ordering::SeqCst) {
                    return;
                }

                let path = entry.path();
                if !is_video_file(&path) || attempted.contains(&path) {
                    continue;
                }

                let Some(file_name) = path.file_name().map(|name| name.to_os_string()) else {
                    continue;
                };

                let output_path = folder.output.join(&file_name);
                if output_path.exists() {
                    attempted.push_back(path);
                    if attempted.len() > MAX_ATTEMPTED {
                        attempted.pop_front();
                    }
                    continue;
                }

                let metadata = entry.metadata().ok();
                let file_size = metadata.as_ref().map_or(0, |m| m.len());
                let file_label = PathBuf::from(&file_name).display().to_string();

                if tx
                    .send(WatcherEvent::Processing {
                        filename: file_label.clone(),
                        file_size,
                    })
                    .is_err()
                {
                    return;
                }

                let started = Instant::now();
                let folder_config =
                    config.with_folder_settings(&folder.preset, &folder.settings);
                let result = process_single_file_with_intro_outro_progress(
                    path.clone(),
                    output_path,
                    &folder_config,
                    &analyzer,
                    &editor,
                    &duration_getter,
                    intro.clone(),
                    outro.clone(),
                    |progress: ProcessingProgress| {
                        let _ = tx.send(WatcherEvent::Progress {
                            filename: file_label.clone(),
                            progress: progress.fraction,
                            message: progress.stage,
                        });
                    },
                );

                attempted.push_back(path);
                if attempted.len() > MAX_ATTEMPTED {
                    attempted.pop_front();
                }

                match result {
                    Ok(()) => {
                        if notify {
                            send_desktop_notification(
                                "Processing Complete",
                                &format!("{} is ready", file_label),
                            );
                        }
                        if tx
                            .send(WatcherEvent::Completed {
                                filename: file_label,
                                file_size,
                                duration_secs: started.elapsed().as_secs().max(1),
                            })
                            .is_err()
                        {
                            return;
                        }
                    }
                    Err(err) => {
                        if notify {
                            send_desktop_notification(
                                "Processing Failed",
                                &format!("{}: {}", file_label, err),
                            );
                        }
                        if tx
                            .send(WatcherEvent::Failed {
                                filename: file_label,
                                message: err.to_string(),
                            })
                            .is_err()
                        {
                            return;
                        }
                    }
                }
            }
        }

        if tx
            .send(WatcherEvent::Status(ProcessingStatus::Watching))
            .is_err()
        {
            return;
        }

        for _ in 0..poll_interval.as_millis().div_ceil(250) {
            if stop.load(Ordering::SeqCst) {
                return;
            }
            std::thread::sleep(Duration::from_millis(250));
        }
    }
}

#[allow(dead_code)]
pub fn make_test_folder_state() -> FolderState {
    FolderState {
        input: std::path::PathBuf::from("/input"),
        output: std::path::PathBuf::from("/output"),
        preset: String::new(),
        enabled: true,
        settings: FolderSettings::default(),
    }
}

pub(crate) fn spawn_queue_worker(
    config: Config,
    queue: Vec<super::QueuedFile>,
    tx: mpsc::Sender<QueueEvent>,
    notify: bool,
) -> Arc<AtomicBool> {
    let stop = Arc::new(AtomicBool::new(false));
    let thread_stop = Arc::clone(&stop);

    std::thread::spawn(move || {
        queue_worker_loop(config, queue, tx, thread_stop, notify);
    });

    stop
}

fn queue_worker_loop(
    config: Config,
    queue: Vec<super::QueuedFile>,
    tx: mpsc::Sender<QueueEvent>,
    stop: Arc<AtomicBool>,
    notify: bool,
) {
    let mut successful = 0;
    let mut failed = 0;
    let queue_len = queue.len();

    for file in queue {
        if stop.load(Ordering::SeqCst) {
            return;
        }

        let filename = file
            .path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        let _ = tx.send(QueueEvent::Processing {
            filename: filename.clone(),
            path: file.path.clone(),
        });

        let output_ext = file
            .path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("mp4");
        let output_file = file.output_dir.join(format!(
            "{}.{}",
            file.path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("output"),
            output_ext
        ));

        let folder_state = FolderState {
            input: file.path.clone(),
            output: file.output_dir.clone(),
            preset: file.preset.clone(),
            enabled: true,
            settings: file.settings.clone(),
        };
        let file_config = build_folder_config(&config, &folder_state);

        let analyzer = FfmpegAnalyzer;
        let editor = FfmpegEditor::new(file_config.video.hw_accel);
        let duration_getter = FfmpegDurationGetter;

        let result = process_single_file_with_intro_outro_progress(
            file.path.clone(),
            output_file.clone(),
            &file_config,
            &analyzer,
            &editor,
            &duration_getter,
            file_config.paths.intro.clone(),
            file_config.paths.outro.clone(),
            |progress| {
                let _ = tx.send(QueueEvent::Progress {
                    filename: filename.clone(),
                    path: file.path.clone(),
                    progress: progress.fraction,
                    message: progress.stage.clone(),
                });
            },
        );

        match result {
            Ok(_) => {
                let file_size = output_file.metadata().map(|m| m.len()).unwrap_or(0);
                let _ = tx.send(QueueEvent::Completed {
                    filename,
                    path: file.path.clone(),
                    file_size,
                    output_path: output_file,
                });
                successful += 1;
            }
            Err(e) => {
                let _ = tx.send(QueueEvent::Failed {
                    filename,
                    path: file.path.clone(),
                    message: e.to_string(),
                });
                failed += 1;
            }
        }
    }

    if notify && queue_len > 0 {
        let total = successful + failed;
        let body = if failed == 0 {
            format!(
                "Processed {} file{} successfully",
                total,
                if total == 1 { "" } else { "s" }
            )
        } else {
            format!(
                "Processed {} file{} ({} failed)",
                total,
                if total == 1 { "" } else { "s" },
                failed
            )
        };
        send_desktop_notification("Batch Complete", &body);
    }

    let _ = tx.send(QueueEvent::Finished);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{FolderSettings, SilenceMode};
    use crate::gui::FolderState;

    fn make_test_folder_state() -> FolderState {
        FolderState {
            input: std::path::PathBuf::from("/input"),
            output: std::path::PathBuf::from("/output"),
            preset: String::new(),
            enabled: true,
            settings: FolderSettings::default(),
        }
    }

    #[test]
    fn test_build_folder_config_defaults_preserve_base() {
        let base_config = Config::default();
        let folder = make_test_folder_state();

        let merged = build_folder_config(&base_config, &folder);

        // With no folder overrides, merged should equal base
        assert_eq!(
            merged.silence.threshold_db,
            base_config.silence.threshold_db
        );
        assert_eq!(merged.silence.mode, base_config.silence.mode);
        assert_eq!(merged.audio.enhance, base_config.audio.enhance);
    }

    #[test]
    fn test_build_folder_config_silence_mode_override() {
        let base_config = Config::default();
        let mut folder = make_test_folder_state();
        folder.settings.silence_mode = Some(SilenceMode::Speedup);

        let merged = build_folder_config(&base_config, &folder);
        assert_eq!(merged.silence.mode, SilenceMode::Speedup);
    }

    #[test]
    fn test_build_folder_config_legacy_remove_silence_true() {
        let base_config = Config::default();
        let mut folder = make_test_folder_state();
        folder.settings.remove_silence = Some(true);
        folder.settings.silence_mode = None;

        let merged = build_folder_config(&base_config, &folder);
        assert_eq!(merged.silence.mode, SilenceMode::Cut);
    }

    #[test]
    fn test_build_folder_config_legacy_remove_silence_false() {
        let base_config = Config::default();
        let mut folder = make_test_folder_state();
        folder.settings.remove_silence = Some(false);
        folder.settings.silence_mode = None;

        let merged = build_folder_config(&base_config, &folder);
        assert_eq!(merged.silence.mode, SilenceMode::Keep);
    }

    #[test]
    fn test_build_folder_config_silence_mode_wins_over_legacy() {
        let base_config = Config::default();
        let mut folder = make_test_folder_state();
        folder.settings.silence_mode = Some(SilenceMode::Speedup);
        folder.settings.remove_silence = Some(true); // Legacy value should be ignored

        let merged = build_folder_config(&base_config, &folder);
        assert_eq!(merged.silence.mode, SilenceMode::Speedup);
    }

    #[test]
    fn test_build_folder_config_watermark_path() {
        let base_config = Config::default();
        let mut folder = make_test_folder_state();
        folder.settings.watermark_path = Some(std::path::PathBuf::from("/path/to/watermark.png"));

        let merged = build_folder_config(&base_config, &folder);
        assert_eq!(
            merged.video.watermark,
            Some(std::path::PathBuf::from("/path/to/watermark.png"))
        );
    }

    #[test]
    fn test_build_folder_config_music_path() {
        let base_config = Config::default();
        let mut folder = make_test_folder_state();
        folder.settings.music_path = Some(std::path::PathBuf::from("/path/to/music.mp3"));

        let merged = build_folder_config(&base_config, &folder);
        assert_eq!(
            merged.paths.music,
            Some(std::path::PathBuf::from("/path/to/music.mp3"))
        );
    }

    #[test]
    fn test_build_folder_config_threshold_override() {
        let base_config = Config::default();
        let mut folder = make_test_folder_state();
        folder.settings.silence_threshold_db = Some(-50.0);

        let merged = build_folder_config(&base_config, &folder);
        assert_eq!(merged.silence.threshold_db, -50.0);
    }

    #[test]
    fn test_build_folder_config_preset_merge() {
        let base_config = Config::default();
        let mut folder = make_test_folder_state();
        folder.preset = "shorts".to_string();

        let merged = build_folder_config(&base_config, &folder);
        // Shorts preset should change target resolution to vertical
        assert_eq!(
            merged.video.target_resolution,
            crate::config::VideoResolution::Vertical1080p
        );
        // Shorts preset should enable reframe
        assert!(merged.video.reframe);
    }

    #[test]
    fn test_build_folder_config_invalid_preset_ignored() {
        let base_config = Config::default();
        let mut folder = make_test_folder_state();
        folder.preset = "nonexistent_preset".to_string();

        let merged = build_folder_config(&base_config, &folder);
        // Invalid preset should be ignored, config stays the same
        assert_eq!(
            merged.video.target_resolution,
            base_config.video.target_resolution
        );
    }
}
