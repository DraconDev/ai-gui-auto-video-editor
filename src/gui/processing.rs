use std::collections::HashSet;
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
use crate::Preset;
use crate::batch_processor::ProcessingProgress;
use crate::batch_processor::process_single_file_with_intro_outro_progress;
use crate::config::FolderSettings;
use crate::config::SilenceMode;

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
) -> (Receiver<WatcherEvent>, Arc<AtomicBool>) {
    let (tx, rx) = mpsc::channel();
    let stop = Arc::new(AtomicBool::new(false));
    let thread_stop = Arc::clone(&stop);

    std::thread::spawn(move || {
        watch_folders_loop(config, folders, tx, thread_stop, notify);
    });

    (rx, stop)
}

fn watch_folders_loop(
    config: Config,
    folders: Vec<FolderState>,
    tx: mpsc::Sender<WatcherEvent>,
    stop: Arc<AtomicBool>,
    notify: bool,
) {
    let poll_interval = Duration::from_secs(config.watch.interval.max(1));
    let mut attempted = HashSet::new();
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
                    attempted.insert(path);
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
                let folder_config = build_folder_config(&config, folder);
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

                attempted.insert(path);

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

pub fn build_folder_config(config: &Config, folder: &FolderState) -> Config {
    let mut merged = if let Some(preset) = Preset::parse_name(&folder.preset) {
        config.clone().merge(preset.to_config())
    } else {
        config.clone()
    };

    // Silence settings
    if let Some(silence_mode) = folder.settings.silence_mode {
        merged.silence.mode = silence_mode;
    }
    if let Some(threshold) = folder.settings.silence_threshold_db {
        merged.silence.threshold_db = threshold;
    }
    if let Some(min_duration) = folder.settings.silence_min_duration {
        merged.silence.min_duration = min_duration;
    }
    if let Some(padding) = folder.settings.silence_padding {
        merged.silence.padding = padding;
    }
    if let Some(speedup_factor) = folder.settings.silence_speedup_factor {
        merged.silence.speedup_factor = speedup_factor;
    }
    if let Some(min_silence_for_speedup) = folder.settings.silence_min_silence_for_speedup {
        merged.silence.min_silence_for_speedup = min_silence_for_speedup;
    }
    if let Some(scene_threshold) = folder.settings.silence_scene_threshold {
        merged.silence.scene_threshold = scene_threshold;
    }
    if let Some(scene_detect) = folder.settings.scene_detect {
        merged.silence.scene_detect = scene_detect;
    }

    // Audio settings
    if let Some(enhance_audio) = folder.settings.enhance_audio {
        merged.audio.enhance = enhance_audio;
    }
    if let Some(target_lufs) = folder.settings.target_lufs {
        merged.audio.target_lufs = target_lufs;
    }
    if let Some(noise_reduction) = folder.settings.noise_reduction {
        merged.audio.noise_reduction = noise_reduction;
    }
    if let Some(music_path) = folder.settings.music_path.clone() {
        merged.paths.music = Some(music_path);
    }
    if let Some(duck_volume) = folder.settings.duck_volume {
        merged.audio.duck_volume = duck_volume;
    }
    if let Some(filler_words) = folder.settings.filler_words {
        merged.filler_words.enabled = filler_words;
    }

    // Video settings
    if let Some(stabilize) = folder.settings.stabilize {
        merged.video.stabilize = stabilize;
    }
    if let Some(color_correct) = folder.settings.color_correct {
        merged.video.color_correct = color_correct;
    }
    if let Some(reframe) = folder.settings.reframe {
        merged.video.reframe = reframe;
    }
    if let Some(blur_background) = folder.settings.blur_background {
        merged.video.blur_background = blur_background;
    }
    if let Some(hw_accel) = folder.settings.hw_accel {
        merged.video.hw_accel = hw_accel;
    }
    if let Some(target_resolution) = folder.settings.target_resolution {
        merged.video.target_resolution = target_resolution;
    }
    if let Some(watermark_path) = folder.settings.watermark_path.clone() {
        merged.video.watermark = Some(watermark_path);
    }
    if let Some(watermark_position) = folder.settings.watermark_position.clone() {
        merged.video.watermark_position = watermark_position;
    }
    if let Some(watermark_scale) = folder.settings.watermark_scale {
        merged.video.watermark_scale = watermark_scale;
    }

    // Path settings
    if let Some(intro_path) = folder.settings.intro_path.clone() {
        merged.paths.intro = Some(intro_path);
    }
    if let Some(outro_path) = folder.settings.outro_path.clone() {
        merged.paths.outro = Some(outro_path);
    }

    // Export settings
    if let Some(preview) = folder.settings.preview {
        merged.export.preview = preview;
    }
    if let Some(multi_format) = folder.settings.multi_format {
        merged.export.multi_format = multi_format;
    }
    if let Some(subtitles) = folder.settings.subtitles {
        merged.export.subtitles = subtitles;
    }
    if let Some(chapters) = folder.settings.chapters {
        merged.export.chapters = chapters;
    }
    if let Some(captions) = folder.settings.captions {
        merged.export.captions = captions;
    }
    if let Some(clips) = folder.settings.clips {
        merged.export.clips = clips;
    }
    if let Some(clip_count) = folder.settings.clip_count {
        merged.export.clip_count = clip_count;
    }
    if let Some(clip_min_duration) = folder.settings.clip_min_duration {
        merged.export.clip_min_duration = clip_min_duration;
    }
    if let Some(clip_max_duration) = folder.settings.clip_max_duration {
        merged.export.clip_max_duration = clip_max_duration;
    }
    if let Some(fcpxml) = folder.settings.fcpxml {
        merged.export.fcpxml = fcpxml;
    }
    if let Some(edl) = folder.settings.edl {
        merged.export.edl = edl;
    }
    if let Some(thumbnail) = folder.settings.thumbnail {
        merged.export.thumbnail = thumbnail;
    }
    if let Some(ref extra_resolutions) = folder.settings.extra_resolutions {
        merged.export.extra_resolutions = extra_resolutions.clone();
    }

    merged
}

fn is_video_file(path: &Path) -> bool {
    path.is_file()
        && path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| {
                matches!(
                    ext.to_ascii_lowercase().as_str(),
                    "mp4" | "mov" | "avi" | "mkv" | "webm"
                )
            })
            .unwrap_or(false)
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

        let output_ext = file.path
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
