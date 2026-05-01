pub mod processing;
mod tabs;
mod theme;

use eframe::egui;
use std::path::{Path, PathBuf};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
    mpsc::Receiver,
};
use std::time::Duration;

use crate::config::{Config, FolderSettings, JoinMode, WatchFolder};

#[derive(Debug, Clone, Copy, PartialEq, Default)]
enum Tab {
    #[default]
    All,
    Folders,
    Queue,
    Settings,
    Activity,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub(crate) enum SettingsCategory {
    #[default]
    Processing,
    Audio,
    Video,
    Exports,
    Advanced,
}

impl SettingsCategory {
    fn label(&self) -> &'static str {
        match self {
            SettingsCategory::Processing => "Processing",
            SettingsCategory::Audio => "Audio",
            SettingsCategory::Video => "Video",
            SettingsCategory::Exports => "Exports",
            SettingsCategory::Advanced => "Advanced",
        }
    }

    fn icon(&self) -> &'static str {
        match self {
            SettingsCategory::Processing => "🎬",
            SettingsCategory::Audio => "🎵",
            SettingsCategory::Video => "📹",
            SettingsCategory::Exports => "📤",
            SettingsCategory::Advanced => "⚙",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum SetupStep {
    Welcome,
    ChooseFolder,
    ProcessingOptions,
    Complete,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ProcessingStatus {
    Idle,
    Watching,
    Processing(String),
    Error(String),
}

#[derive(Debug, Clone)]
struct ActivityEntry {
    timestamp: String,
    filename: String,
    file_size: u64,
    duration: Option<u64>,
    progress: Option<f32>,
    status: EntryStatus,
    message: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum EntryStatus {
    Success,
    Processing,
    Error,
}

impl ActivityEntry {
    #[allow(dead_code)]
    fn success(filename: impl Into<String>, file_size: u64, duration: u64) -> Self {
        let now = chrono::Local::now();
        Self {
            timestamp: now.format("%H:%M:%S").to_string(),
            filename: filename.into(),
            file_size,
            duration: Some(duration),
            progress: None,
            status: EntryStatus::Success,
            message: String::new(),
        }
    }

    #[allow(dead_code)]
    fn processing(filename: impl Into<String>, file_size: u64, progress: f32) -> Self {
        let now = chrono::Local::now();
        Self {
            timestamp: now.format("%H:%M:%S").to_string(),
            filename: filename.into(),
            file_size,
            duration: None,
            progress: Some(progress),
            status: EntryStatus::Processing,
            message: "Queued".to_string(),
        }
    }

    #[allow(dead_code)]
    fn error(filename: impl Into<String>, message: impl Into<String>) -> Self {
        let now = chrono::Local::now();
        Self {
            timestamp: now.format("%H:%M:%S").to_string(),
            filename: filename.into(),
            file_size: 0,
            duration: None,
            progress: None,
            status: EntryStatus::Error,
            message: message.into(),
        }
    }

    fn simple(message: impl Into<String>, success: bool) -> Self {
        let now = chrono::Local::now();
        Self {
            timestamp: now.format("%H:%M:%S").to_string(),
            filename: String::new(),
            file_size: 0,
            duration: None,
            progress: None,
            status: if success {
                EntryStatus::Success
            } else {
                EntryStatus::Error
            },
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct FolderState {
    pub input: PathBuf,
    pub output: PathBuf,
    pub preset: String,
    pub enabled: bool,
    pub settings: FolderSettings,
}

impl From<WatchFolder> for FolderState {
    fn from(folder: WatchFolder) -> Self {
        Self {
            input: folder.input,
            output: folder.output,
            preset: folder.preset,
            enabled: folder.enabled,
            settings: folder.settings,
        }
    }
}

#[derive(Debug)]
pub(crate) enum WatcherEvent {
    Status(ProcessingStatus),
    Log {
        message: String,
        success: bool,
    },
    Processing {
        filename: String,
        file_size: u64,
    },
    Progress {
        filename: String,
        progress: f32,
        message: String,
    },
    Completed {
        filename: String,
        file_size: u64,
        duration_secs: u64,
    },
    Failed {
        filename: String,
        message: String,
    },
}

#[derive(Debug)]
#[allow(dead_code)]
pub(crate) enum QueueEvent {
    Processing {
        filename: String,
        path: PathBuf,
    },
    Progress {
        filename: String,
        path: PathBuf,
        progress: f32,
        message: String,
    },
    Completed {
        filename: String,
        path: PathBuf,
        file_size: u64,
        output_path: PathBuf,
    },
    Failed {
        filename: String,
        path: PathBuf,
        message: String,
    },
    Finished,
}

impl From<FolderState> for WatchFolder {
    fn from(state: FolderState) -> Self {
        Self {
            input: state.input,
            output: state.output,
            preset: state.preset,
            enabled: state.enabled,
            settings: state.settings,
        }
    }
}

impl Default for FolderState {
    fn default() -> Self {
        Self {
            input: PathBuf::from("videos"),
            output: PathBuf::from("videos/output"),
            preset: "youtube".to_string(),
            enabled: true,
            settings: FolderSettings::default(),
        }
    }
}

#[derive(Debug, Clone, Default)]
struct ModalState {
    show: bool,
    editing_idx: Option<usize>,
    input: PathBuf,
    output: PathBuf,
    preset: String,
    enabled: bool,
    delete_confirm_idx: Option<usize>,
}

impl ModalState {
    fn reset_for_add(&mut self) {
        self.show = true;
        self.editing_idx = None;
        self.input = PathBuf::from("videos/youtube");
        self.output = PathBuf::from("videos/youtube/output");
        self.preset = "youtube".to_string();
        self.enabled = true;
    }

    fn set_for_edit(&mut self, idx: usize, folder: &FolderState) {
        self.show = true;
        self.editing_idx = Some(idx);
        self.input = folder.input.clone();
        self.output = folder.output.clone();
        self.preset = folder.preset.clone();
        self.enabled = folder.enabled;
    }

    fn prompt_delete(&mut self, idx: usize) {
        self.delete_confirm_idx = Some(idx);
    }

    fn close(&mut self) {
        self.show = false;
        self.editing_idx = None;
        self.delete_confirm_idx = None;
    }
}

#[derive(Debug, Clone)]
pub(crate) struct QueuedFile {
    path: PathBuf,
    output_dir: PathBuf,
    preset: String,
    settings: FolderSettings,
    status: QueueStatus,
    progress: f32,
    output_path: Option<PathBuf>,
    completed_at: Option<chrono::DateTime<chrono::Local>>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)]
enum QueueStatus {
    Queued,
    Processing,
    Done,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)]
pub(crate) enum ToastKind {
    Success,
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone)]
struct Toast {
    message: String,
    kind: ToastKind,
    created: std::time::Instant,
}

impl Toast {
    fn new(message: impl Into<String>, kind: ToastKind) -> Self {
        Self {
            message: message.into(),
            kind,
            created: std::time::Instant::now(),
        }
    }

    fn expired(&self) -> bool {
        self.created.elapsed().as_secs() > 5
    }

    fn color(&self) -> egui::Color32 {
        match self.kind {
            ToastKind::Success => crate::gui::theme::SUCCESS,
            ToastKind::Error => crate::gui::theme::ERROR,
            ToastKind::Warning => crate::gui::theme::WARNING,
            ToastKind::Info => crate::gui::theme::PROCESSING,
        }
    }

    fn icon(&self) -> &'static str {
        match self.kind {
            ToastKind::Success => "✓",
            ToastKind::Error => "✗",
            ToastKind::Warning => "⚠",
            ToastKind::Info => "ℹ",
        }
    }
}

impl AppState {
    fn add_toast(&mut self, message: impl Into<String>, kind: ToastKind) {
        self.toasts.push(Toast::new(message, kind));
        if self.toasts.len() > 10 {
            self.toasts.remove(0);
        }
    }

    #[allow(dead_code)]
    fn add_warning(&mut self, message: impl Into<String>) {
        self.add_toast(message, ToastKind::Warning);
    }

    #[allow(dead_code)]
    fn add_info(&mut self, message: impl Into<String>) {
        self.add_toast(message, ToastKind::Info);
    }
}

#[derive(Debug)]
pub struct AppState {
    config: Config,
    folders: Vec<FolderState>,
    status: ProcessingStatus,
    activity_log: Vec<ActivityEntry>,
    config_path: Option<PathBuf>,
    current_tab: Tab,
    modal: ModalState,
    selected_folder_idx: usize,
    // First-run setup wizard
    show_setup: bool,
    setup_step: SetupStep,
    setup_folder: PathBuf,
    setup_preset: String,
    setup_enhance: bool,
    setup_remove_silence: bool,
    watcher_rx: Option<Receiver<WatcherEvent>>,
    watcher_stop: Option<Arc<AtomicBool>>,
    // Toast notifications
    toasts: Vec<Toast>,
    // Batch queue
    batch_queue: Vec<QueuedFile>,
    queue_processing: bool,
    queue_rx: Option<Receiver<QueueEvent>>,
    queue_stop: Option<Arc<AtomicBool>>,
    // Activity summary tracking
    last_seen_activity_len: usize,
    // Settings sidebar navigation
    settings_category: SettingsCategory,
    // Debounced config save
    last_save_time: Option<std::time::Instant>,
    // Recent outputs (quick access)
    recent_outputs: Vec<PathBuf>,
}

#[allow(dead_code)]
fn join_mode_display(mode: &JoinMode) -> String {
    match mode {
        JoinMode::Off => "Off".to_string(),
        JoinMode::ByDate => "By Date".to_string(),
        JoinMode::ByName => "By Name".to_string(),
        JoinMode::AfterCount => "After N Files".to_string(),
    }
}

impl AppState {
    fn new() -> Self {
        let config = Config::default();
        let folders: Vec<FolderState> = if config.paths.watch_folders.is_empty() {
            vec![FolderState::default()]
        } else {
            config
                .paths
                .watch_folders
                .iter()
                .map(|f| f.clone().into())
                .collect()
        };

        // Check if this is first run (no config exists)
        let config_exists = Config::default_config_path()
            .map(|p| p.exists())
            .unwrap_or(false);
        let is_first_run = !config_exists;

        let mut state = Self {
            config,
            folders,
            status: ProcessingStatus::Watching,
            activity_log: vec![ActivityEntry::simple("Started watching for videos", true)],
            config_path: None,
            current_tab: Tab::All,
            modal: ModalState::default(),
            selected_folder_idx: 0,
            show_setup: is_first_run,
            setup_step: SetupStep::Welcome,
            setup_folder: std::env::var("HOME")
                .or_else(|_| std::env::var("USERPROFILE"))
                .map(PathBuf::from)
                .unwrap_or_default()
                .join("Videos"),
            setup_preset: "youtube".to_string(),
            setup_enhance: true,
            setup_remove_silence: true,
            watcher_rx: None,
            watcher_stop: None,
            toasts: Vec::new(),
            batch_queue: Vec::new(),
            queue_processing: false,
            queue_rx: None,
            queue_stop: None,
            last_seen_activity_len: 0,
            settings_category: SettingsCategory::default(),
            last_save_time: None,
            recent_outputs: Vec::new(),
        };

        if !is_first_run {
            if let Some(path) = Config::default_config_path() {
                state.load_config(&path);
            }
        } else {
            state.activity_log.push(ActivityEntry::simple(
                "Welcome! Complete setup to get started.",
                true,
            ));
        }

        if !state.show_setup {
            state.restart_watcher();
        }

        state
    }

    fn load_config(&mut self, path: &std::path::Path) {
        match Config::from_file(path) {
            Ok(config) => {
                self.config = config.clone();
                self.folders = if self.config.paths.watch_folders.is_empty() {
                    vec![FolderState::default()]
                } else {
                    self.config
                        .paths
                        .watch_folders
                        .iter()
                        .map(|f| f.clone().into())
                        .collect()
                };
                self.config_path = Some(path.to_path_buf());
                self.activity_log.push(ActivityEntry::simple(
                    format!("Loaded config from {}", path.display()),
                    true,
                ));
                self.restart_watcher();
            }
            Err(e) => {
                self.activity_log.push(ActivityEntry::simple(
                    format!("Failed to load config: {}", e),
                    false,
                ));
            }
        }
    }

    fn auto_save_config(&mut self) {
        self.config.paths.watch_folders = self
            .folders
            .iter()
            .map(|f| {
                let st: FolderState = f.clone();
                WatchFolder::from(st)
            })
            .collect();

        // Debounce: only save to disk and restart watcher if 1 second has passed
        let now = std::time::Instant::now();
        let should_flush = self
            .last_save_time
            .map(|t| now.duration_since(t).as_secs() >= 1)
            .unwrap_or(true);

        if should_flush {
            let path = if let Some(ref p) = self.config_path {
                Some(p.clone())
            } else {
                Config::default_config_path()
            };

            if let Some(path) = path
                && let Err(e) = self.config.to_file(&path)
            {
                self.activity_log.push(ActivityEntry::simple(
                    format!("Failed to auto-save config: {}", e),
                    false,
                ));
            }

            self.restart_watcher();
            self.last_save_time = Some(now);
        }
    }

    fn add_folder_from_modal(&mut self) {
        let folder = FolderState {
            input: self.modal.input.clone(),
            output: self.modal.output.clone(),
            preset: self.modal.preset.clone(),
            enabled: self.modal.enabled,
            settings: FolderSettings::default(),
        };
        self.folders.push(folder);
        self.activity_log
            .push(ActivityEntry::simple("Added new watch folder", true));
        self.auto_save_config();
    }

    fn update_folder_from_modal(&mut self, idx: usize) {
        if let Some(folder) = self.folders.get_mut(idx) {
            folder.input = self.modal.input.clone();
            folder.output = self.modal.output.clone();
            folder.preset = self.modal.preset.clone();
            folder.enabled = self.modal.enabled;
            self.activity_log
                .push(ActivityEntry::simple("Updated watch folder", true));
            self.auto_save_config();
        }
    }

    fn remove_folder(&mut self, index: usize) {
        self.folders.remove(index);
        if self.folders.is_empty() {
            self.selected_folder_idx = 0;
        } else if self.selected_folder_idx >= self.folders.len() {
            self.selected_folder_idx = self.folders.len() - 1;
        }
        self.activity_log
            .push(ActivityEntry::simple("Removed watch folder", true));
        self.auto_save_config();
    }

    fn duplicate_folder(&mut self, index: usize) {
        if let Some(original) = self.folders.get(index).cloned() {
            let new_folder = FolderState {
                input: PathBuf::from(format!("{}_copy", original.input.display())),
                output: PathBuf::from(format!("{}_copy", original.output.display())),
                preset: original.preset,
                enabled: original.enabled,
                settings: original.settings,
            };
            self.folders.push(new_folder);
            self.selected_folder_idx = self.folders.len() - 1;
            self.activity_log
                .push(ActivityEntry::simple("Duplicated watch folder", true));
            self.auto_save_config();
        }
    }

    fn export_config_to(&mut self, path: &Path) -> std::io::Result<()> {
        self.config.paths.watch_folders = self
            .folders
            .iter()
            .map(|f| {
                let st: FolderState = f.clone();
                WatchFolder::from(st)
            })
            .collect();

        let json = serde_json::to_string_pretty(&self.config.paths.watch_folders)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        std::fs::write(path, json)?;
        self.activity_log
            .push(ActivityEntry::simple("Exported config to file", true));
        Ok(())
    }

    fn import_config_from(&mut self, path: &Path) -> std::io::Result<()> {
        let json = std::fs::read_to_string(path)?;
        let watch_folders: Vec<WatchFolder> = serde_json::from_str(&json)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        self.folders = watch_folders.iter().map(|f| f.clone().into()).collect();
        self.selected_folder_idx = 0;
        self.activity_log
            .push(ActivityEntry::simple("Imported config from file", true));
        self.auto_save_config();
        Ok(())
    }

    fn toggle_folder(&mut self, index: usize) {
        if let Some(folder) = self.folders.get_mut(index) {
            folder.enabled = !folder.enabled;
            let status = if folder.enabled {
                "enabled"
            } else {
                "disabled"
            };
            self.activity_log.push(ActivityEntry::simple(
                format!("Folder {} ({})", status, folder.input.display()),
                true,
            ));
            self.auto_save_config();
        }
    }

    fn restart_watcher(&mut self) {
        if let Some(stop) = self.watcher_stop.take() {
            stop.store(true, Ordering::SeqCst);
            // Give the old thread time to finish before starting a new one
            std::thread::sleep(std::time::Duration::from_millis(100));
        }

        let enabled_folders: Vec<FolderState> =
            self.folders.iter().filter(|f| f.enabled).cloned().collect();

        if enabled_folders.is_empty() {
            self.watcher_rx = None;
            self.status = ProcessingStatus::Idle;
            self.activity_log.push(ActivityEntry::simple(
                "No enabled watch folders. Auto-processing is paused.",
                true,
            ));
            return;
        }

        let (rx, stop) = processing::spawn_watcher(self.config.clone(), enabled_folders, true);
        self.watcher_rx = Some(rx);
        self.watcher_stop = Some(stop);
        self.status = ProcessingStatus::Watching;
    }

    fn drain_watcher_events(&mut self) {
        const MAX_ACTIVITY_LOG: usize = 500;
        let Some(rx) = self.watcher_rx.as_ref() else {
            return;
        };

        let mut drained = Vec::new();
        while let Ok(event) = rx.try_recv() {
            drained.push(event);
        }

        for event in drained {
            match event {
                WatcherEvent::Status(status) => self.status = status,
                WatcherEvent::Log { message, success } => {
                    self.activity_log
                        .push(ActivityEntry::simple(message, success));
                }
                WatcherEvent::Processing {
                    filename,
                    file_size,
                } => {
                    self.status = ProcessingStatus::Processing(filename.clone());
                    self.upsert_processing_entry(&filename, file_size, 0.0, "Queued");
                }
                WatcherEvent::Progress {
                    filename,
                    progress,
                    message,
                } => {
                    self.status = ProcessingStatus::Processing(filename.clone());
                    self.upsert_processing_entry(&filename, 0, progress, &message);
                }
                WatcherEvent::Completed {
                    filename,
                    file_size,
                    duration_secs,
                } => {
                    self.status = ProcessingStatus::Watching;
                    self.activity_log.push(ActivityEntry::success(
                        filename.clone(),
                        file_size,
                        duration_secs,
                    ));
                    self.toasts.push(Toast::new(
                        format!("{} processed", filename),
                        ToastKind::Success,
                    ));
                }
                WatcherEvent::Failed { filename, message } => {
                    self.status = ProcessingStatus::Error(message.clone());
                    self.activity_log
                        .push(ActivityEntry::error(filename.clone(), message.clone()));
                    self.toasts.push(Toast::new(
                        format!("{} failed: {}", filename, message),
                        ToastKind::Error,
                    ));
                }
            }
        }

        if self.activity_log.len() > MAX_ACTIVITY_LOG {
            self.activity_log.drain(0..self.activity_log.len() - MAX_ACTIVITY_LOG);
        }
    }

    fn drain_queue_events(&mut self) {
        let Some(rx) = self.queue_rx.as_ref() else {
            return;
        };

        let mut drained = Vec::new();
        while let Ok(event) = rx.try_recv() {
            drained.push(event);
        }

        for event in drained {
            match event {
                QueueEvent::Processing { filename, path } => {
                    for file in &mut self.batch_queue {
                        if file.path == path {
                            file.status = QueueStatus::Processing;
                            file.progress = 0.0;
                            break;
                        }
                    }
                    self.status = ProcessingStatus::Processing(filename);
                }
                QueueEvent::Progress {
                    filename: _,
                    path,
                    progress,
                    message: _,
                } => {
                    for file in &mut self.batch_queue {
                        if file.path == path {
                            file.progress = progress;
                            break;
                        }
                    }
                }
                QueueEvent::Completed {
                    filename,
                    path,
                    file_size: _,
                    output_path,
                } => {
                    for file in &mut self.batch_queue {
                        if file.path == path {
                            file.status = QueueStatus::Done;
                            file.progress = 1.0;
                            file.output_path = Some(output_path.clone());
                            file.completed_at = Some(chrono::Local::now());
                            if output_path.exists() {
                                self.recent_outputs.insert(0, output_path.clone());
                                if self.recent_outputs.len() > 10 {
                                    self.recent_outputs.pop();
                                }
                            }
                            break;
                        }
                    }
                    self.toasts.push(Toast::new(
                        format!("{} processed", filename),
                        ToastKind::Success,
                    ));
                }
                QueueEvent::Failed {
                    filename,
                    path,
                    message,
                } => {
                    for file in &mut self.batch_queue {
                        if file.path == path {
                            file.status = QueueStatus::Error;
                            break;
                        }
                    }
                    self.toasts.push(Toast::new(
                        format!("{} failed: {}", filename, message),
                        ToastKind::Error,
                    ));
                }
                QueueEvent::Finished => {
                    self.queue_processing = false;
                    self.queue_stop = None;
                }
            }
        }

        const MAX_BATCH_QUEUE: usize = 100;
        let now = chrono::Local::now();
        self.batch_queue.retain(|f| {
            if f.status == QueueStatus::Done || f.status == QueueStatus::Error {
                if let Some(completed) = f.completed_at {
                    let elapsed = now.signed_duration_since(completed);
                    if elapsed.num_seconds() > 60 {
                        return false;
                    }
                }
            }
            true
        });
        if self.batch_queue.len() > MAX_BATCH_QUEUE {
            self.batch_queue.drain(0..self.batch_queue.len() - MAX_BATCH_QUEUE);
        }
    }

    fn upsert_processing_entry(
        &mut self,
        filename: &str,
        file_size: u64,
        progress: f32,
        message: &str,
    ) {
        if let Some(entry) = self
            .activity_log
            .iter_mut()
            .rev()
            .find(|entry| entry.status == EntryStatus::Processing && entry.filename == filename)
        {
            entry.timestamp = chrono::Local::now().format("%H:%M:%S").to_string();
            if file_size > 0 {
                entry.file_size = file_size;
            }
            entry.progress = Some(progress.clamp(0.0, 1.0));
            entry.message = message.to_string();
        } else {
            let mut entry = ActivityEntry::processing(filename.to_string(), file_size, progress);
            entry.message = message.to_string();
            self.activity_log.push(entry);
        }
    }
}

impl Drop for AppState {
    fn drop(&mut self) {
        if let Some(stop) = self.watcher_stop.take() {
            stop.store(true, Ordering::SeqCst);
        }
        if let Some(stop) = self.queue_stop.take() {
            stop.store(true, Ordering::SeqCst);
        }
    }
}

pub struct App {
    state: AppState,
}

impl App {
    pub fn new(_start_minimized: bool) -> Self {
        Self {
            state: AppState::new(),
        }
    }

    fn navigate_settings_category(&mut self, delta: i8) {
        let categories = [
            SettingsCategory::Processing,
            SettingsCategory::Audio,
            SettingsCategory::Video,
            SettingsCategory::Exports,
            SettingsCategory::Advanced,
        ];
        let current_idx = categories
            .iter()
            .position(|&c| c == self.state.settings_category)
            .unwrap_or(0);
        let new_idx = ((current_idx as i8 + delta).rem_euclid(categories.len() as i8)) as usize;
        self.state.settings_category = categories[new_idx];
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.state.drain_watcher_events();
        self.state.drain_queue_events();

        // Global keyboard shortcuts (work on all tabs)
        // Skip shortcuts when setup wizard is shown, text input is focused, or file drop is active
        let modifiers = ctx.input(|i| i.modifiers);
        #[cfg(target_os = "macos")]
        let is_ctrl = modifiers.ctrl || modifiers.mac_cmd;
        #[cfg(not(target_os = "macos"))]
        let is_ctrl = modifiers.ctrl;
        let skip_shortcuts = self.state.show_setup
            || ctx.wants_keyboard_input()
            || ctx.input(|i| i.raw.dropped_files.len() > 0);

        // Handle file drops onto the Queue tab
        if self.state.current_tab == Tab::Queue {
            let dropped = ctx.input(|i| {
                i.raw
                    .dropped_files
                    .iter()
                    .filter_map(|f| f.path.clone())
                    .filter(|p| crate::utils::is_video_file(p))
                    .collect::<Vec<_>>()
            });
            for path in dropped {
                let output_dir = PathBuf::from("output");
                let preset = "youtube".to_string();
                let settings = FolderSettings::default();
                self.state.batch_queue.push(QueuedFile {
                    path,
                    output_dir,
                    preset,
                    settings,
                    status: QueueStatus::Queued,
                    progress: 0.0,
                    output_path: None,
                    completed_at: None,
                });
            }
        }

        // Ctrl+1-5 for tab navigation (skip when shift is held, reserved for category access)
        if is_ctrl && !modifiers.shift && !skip_shortcuts {
            let tab_keys = [
                (egui::Key::Num1, Tab::All),
                (egui::Key::Num2, Tab::Folders),
                (egui::Key::Num3, Tab::Queue),
                (egui::Key::Num4, Tab::Settings),
                (egui::Key::Num5, Tab::Activity),
            ];
            for (key, tab) in tab_keys {
                if ctx.input(|i| i.key_pressed(key)) {
                    self.state.current_tab = tab;
                    break;
                }
            }
        }

        // Ctrl+S: Save config (also works in settings tab)
        if is_ctrl && !skip_shortcuts && ctx.input(|i| i.key_pressed(egui::Key::S)) {
            self.state.auto_save_config();
            self.state.add_toast("Config saved", ToastKind::Success);
        }

        // Keyboard shortcuts for settings navigation
        if !skip_shortcuts && (self.state.current_tab == Tab::Settings || self.state.current_tab == Tab::All) {
            if is_ctrl && ctx.input(|i| i.key_pressed(egui::Key::ArrowLeft)) {
                self.navigate_settings_category(-1);
            }
            if is_ctrl && ctx.input(|i| i.key_pressed(egui::Key::ArrowRight)) {
                self.navigate_settings_category(1);
            }

            // Ctrl+Shift+1-5 for category access
            if is_ctrl && modifiers.shift {
                let categories = [
                    SettingsCategory::Processing,
                    SettingsCategory::Audio,
                    SettingsCategory::Video,
                    SettingsCategory::Exports,
                    SettingsCategory::Advanced,
                ];
                let num_keys = [
                    (egui::Key::Num1, 0),
                    (egui::Key::Num2, 1),
                    (egui::Key::Num3, 2),
                    (egui::Key::Num4, 3),
                    (egui::Key::Num5, 4),
                ];
                for (key, idx) in num_keys {
                    if ctx.input(|i| i.key_pressed(key)) {
                        self.state.settings_category = categories[idx];
                        break;
                    }
                }
            }
        }

        // Adaptive repaint: faster when processing (smooth progress), slower when idle (save CPU)
        let is_processing = matches!(self.state.status, ProcessingStatus::Processing(_));
        if is_processing {
            ctx.request_repaint();
        } else {
            ctx.request_repaint_after(Duration::from_millis(250));
        }

        // Show setup wizard for first-run
        if self.state.show_setup {
            self.draw_setup_wizard(ctx);
            return;
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical(|ui| {
                self.draw_header(ui);
                ui.add_space(8.0);

                egui::ScrollArea::vertical().show(ui, |ui| match self.state.current_tab {
                    Tab::All => {
                        self.draw_summary_card(ui);
                        ui.add_space(12.0);
                        self.draw_folders_panel(ui);
                        ui.add_space(12.0);
                        self.draw_settings_panel(ui);
                        ui.add_space(12.0);
                        self.draw_activity_log(ui, false);
                    }
                    Tab::Folders => {
                        self.draw_folders_panel(ui);
                    }
                    Tab::Queue => {
                        self.draw_queue_panel(ui);
                    }
                    Tab::Settings => {
                        self.draw_settings_panel(ui);
                    }
                    Tab::Activity => {
                        self.draw_activity_log(ui, true);
                    }
                });
            });
        });

        if self.state.modal.show {
            self.draw_modal(ctx);
        }

        if self.state.modal.delete_confirm_idx.is_some() {
            self.draw_delete_confirm_modal(ctx);
        }

        // Draw toast notifications
        self.draw_toasts(ctx);

        // Clean up expired toasts
        self.state.toasts.retain(|t| !t.expired());
    }
}
