mod theme;
mod processing;
mod tabs;

use eframe::egui;
use std::path::PathBuf;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
    mpsc::Receiver,
};
use std::time::Duration;

use ai_vid_editor::{Config, FolderSettings, JoinMode, WatchFolder};

#[derive(Debug, Clone, Copy, PartialEq, Default)]
enum Tab {
    #[default]
    All,
    Folders,
    Queue,
    Settings,
    Activity,
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
enum ProcessingStatus {
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
struct FolderState {
    input: PathBuf,
    output: PathBuf,
    preset: String,
    enabled: bool,
    settings: FolderSettings,
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
enum WatcherEvent {
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
struct QueuedFile {
    path: PathBuf,
    output_dir: PathBuf,
    preset: String,
    status: QueueStatus,
    progress: f32,
    output_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)]
enum QueueStatus {
    Queued,
    Processing,
    Done,
    Error,
}

#[derive(Debug, Clone)]
struct Toast {
    message: String,
    success: bool,
    created: std::time::Instant,
}

impl Toast {
    fn new(message: impl Into<String>, success: bool) -> Self {
        Self {
            message: message.into(),
            success,
            created: std::time::Instant::now(),
        }
    }

    fn expired(&self) -> bool {
        self.created.elapsed().as_secs() > 5
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

#[allow(dead_code)]
fn notify_complete(filename: &str) {
    let _ = notify_rust::Notification::new()
        .summary("Processing Complete")
        .body(&format!("{} has been processed", filename))
        .show();
}

#[allow(dead_code)]
fn notify_error(filename: &str, error: &str) {
    let _ = notify_rust::Notification::new()
        .summary("Processing Error")
        .body(&format!("Failed to process {}: {}", filename, error))
        .show();
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
        self.config.paths.watch_folders = self.folders.iter().map(|f| f.clone().into()).collect();

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
        if self.folders.len() > 1 {
            self.folders.remove(index);
            self.activity_log
                .push(ActivityEntry::simple("Removed watch folder", true));
            self.auto_save_config();
        }
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

        let (rx, stop) = processing::spawn_watcher(self.config.clone(), enabled_folders);
        self.watcher_rx = Some(rx);
        self.watcher_stop = Some(stop);
        self.status = ProcessingStatus::Watching;
    }

    fn drain_watcher_events(&mut self) {
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
                        true,
                    ));
                }
                WatcherEvent::Failed { filename, message } => {
                    self.status = ProcessingStatus::Error(message.clone());
                    self.activity_log
                        .push(ActivityEntry::error(filename.clone(), message.clone()));
                    self.toasts.push(Toast::new(
                        format!("{} failed: {}", filename, message),
                        false,
                    ));
                }
            }
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
                QueueEvent::Progress { filename: _, path, progress, message: _ } => {
                    for file in &mut self.batch_queue {
                        if file.path == path {
                            file.progress = progress;
                            break;
                        }
                    }
                }
                QueueEvent::Completed { filename, path, file_size: _, output_path } => {
                    for file in &mut self.batch_queue {
                        if file.path == path {
                            file.status = QueueStatus::Done;
                            file.progress = 1.0;
                            file.output_path = Some(output_path);
                            break;
                        }
                    }
                    self.toasts.push(Toast::new(format!("{} processed", filename), true));
                }
                QueueEvent::Failed { filename, path, message } => {
                    for file in &mut self.batch_queue {
                        if file.path == path {
                            file.status = QueueStatus::Error;
                            break;
                        }
                    }
                    self.toasts.push(Toast::new(
                        format!("{} failed: {}", filename, message),
                        false,
                    ));
                }
                QueueEvent::Finished => {
                    self.queue_processing = false;
                    self.queue_stop = None;
                }
            }
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
    pub fn new() -> Self {
        Self {
            state: AppState::new(),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.state.drain_watcher_events();
        self.state.drain_queue_events();
        ctx.request_repaint_after(Duration::from_millis(250));

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

