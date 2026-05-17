use eframe::egui;
use egui::RichText;
use std::path::PathBuf;
use std::sync::mpsc;

use super::super::App;
use crate::gui::theme::*;

impl App {
    pub(crate) fn draw_queue_panel(&mut self, ui: &mut egui::Ui) {
        panel_frame().show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("Batch Queue")
                        .size(18.0)
                        .color(ACCENT_PRIMARY)
                        .strong(),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    #[allow(clippy::collapsible_if)]
                    if ui.add(button_add("+ Add Files")).clicked() {
                        if let Some(paths) = rfd::FileDialog::new().pick_files() {
                            for path in paths {
                                let folder = self.state.folders.get(self.state.selected_folder_idx);
                                let output_dir = folder
                                    .map(|f| f.output.clone())
                                    .unwrap_or_else(|| PathBuf::from("output"));
                                let preset = folder
                                    .map(|f| f.preset.clone())
                                    .unwrap_or_else(|| "youtube".to_string());
                                let settings =
                                    folder.map(|f| f.settings.clone()).unwrap_or_default();
                                self.state.batch_queue.push(crate::gui::QueuedFile {
                                    path,
                                    output_dir,
                                    preset,
                                    settings,
                                    status: crate::gui::QueueStatus::Queued,
                                    progress: 0.0,
                                    output_path: None,
                                    completed_at: None,
                                });
                            }
                        }
                    }
                    ui.add_space(8.0);
                    if self.state.queue_processing {
                        if ui.add(button_small("Stop")).clicked() {
                            if let Some(stop) = self.state.queue_stop.take() {
                                stop.store(true, std::sync::atomic::Ordering::SeqCst);
                            }
                            self.state.queue_processing = false;
                        }
                    } else if ui.add(button_primary("Process All")).clicked()
                        && !self.state.batch_queue.is_empty()
                    {
                        self.state.queue_processing = true;
                        self.start_queue_processing();
                    }
                });
            });

            ui.add_space(12.0);

            if self.state.batch_queue.is_empty() {
                inner_panel().show(ui, |ui| {
                    ui.add_space(12.0);
                    ui.vertical_centered(|ui| {
                        ui.label(label_muted("No files in queue"));
                        ui.add_space(8.0);
                        ui.label(label_muted("Click + Add Files to get started"));
                    });
                    ui.add_space(12.0);
                });
            } else {
                let mut remove_idx: Option<usize> = None;
                for (idx, file) in self.state.batch_queue.iter_mut().enumerate() {
                    let (status_color, status_text) = match file.status {
                        crate::gui::QueueStatus::Queued => (TEXT_MUTED, "Queued"),
                        crate::gui::QueueStatus::Processing => (PROCESSING, "Processing..."),
                        crate::gui::QueueStatus::Done => (SUCCESS, "Done"),
                        crate::gui::QueueStatus::Error => (ERROR, "Error"),
                    };

                    folder_card_compact(true).show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(
                                    file.path
                                        .file_name()
                                        .map(|n| n.to_string_lossy().to_string())
                                        .unwrap_or_default(),
                                )
                                .color(TEXT_PRIMARY)
                                .size(14.0)
                                .strong(),
                            );
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if file.status == crate::gui::QueueStatus::Error
                                        && ui.add(button_small("Retry")).clicked()
                                    {
                                        file.status = crate::gui::QueueStatus::Queued;
                                        file.progress = 0.0;
                                        file.completed_at = None;
                                    }
                                    if file.status == crate::gui::QueueStatus::Queued
                                        && ui.add(button_small("Remove")).clicked()
                                    {
                                        remove_idx = Some(idx);
                                    }
                                    ui.add_space(8.0);
                                    ui.label(
                                        RichText::new(status_text)
                                            .color(status_color)
                                            .size(12.0)
                                            .strong(),
                                    );
                                },
                            );
                        });

                        if file.status == crate::gui::QueueStatus::Processing {
                            ui.add_space(4.0);
                            ui.horizontal(|ui| {
                                ui.add(
                                    egui::ProgressBar::new(file.progress)
                                        .fill(PROCESSING)
                                        .corner_radius(4.0)
                                        .desired_width(ui.available_width() - 50.0),
                                );
                                ui.add_space(8.0);
                                ui.label(
                                    RichText::new(format!("{}%", (file.progress * 100.0) as u32))
                                        .color(PROCESSING)
                                        .size(12.0)
                                        .strong(),
                                );
                            });
                        }

                        ui.horizontal(|ui| {
                            ui.label(label_muted("Preset:"));
                            ui.label(label_secondary(&file.preset));
                            ui.add_space(8.0);
                            ui.label(label_muted("Output:"));
                            ui.label(label_secondary(&truncate_path(
                                &file.output_dir.to_string_lossy(),
                                30,
                            )));
                        });
                    });
                    ui.add_space(6.0);
                }

                if let Some(idx) = remove_idx {
                    self.state.batch_queue.remove(idx);
                }

                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui.add(button_small("Clear Completed")).clicked() {
                        self.state
                            .batch_queue
                            .retain(|f| f.status != crate::gui::QueueStatus::Done);
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let done = self
                            .state
                            .batch_queue
                            .iter()
                            .filter(|f| f.status == crate::gui::QueueStatus::Done)
                            .count();
                        let total = self.state.batch_queue.len();
                        ui.label(label_muted(&format!("{} / {} done", done, total)));
                    });
                });
            }
        });
    }

    fn start_queue_processing(&mut self) {
        let queue: Vec<crate::gui::QueuedFile> = self
            .state
            .batch_queue
            .iter()
            .filter(|f| matches!(f.status, crate::gui::QueueStatus::Queued))
            .cloned()
            .collect();

        if queue.is_empty() {
            self.state.queue_processing = false;
            return;
        }

        let (tx, rx) = mpsc::channel();
        let stop =
            crate::gui::processing::spawn_queue_worker(self.state.config.clone(), queue, tx, true);

        self.state.queue_rx = Some(rx);
        self.state.queue_stop = Some(stop);
        self.state.queue_processing = true;
    }
}
