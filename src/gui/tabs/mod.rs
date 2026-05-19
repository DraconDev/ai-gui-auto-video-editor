use eframe::egui;
use egui::RichText;

use super::theme::*;
use super::{App, ProcessingStatus, SetupStep, Tab};

impl App {
    pub(crate) fn draw_header(&mut self, ui: &mut egui::Ui) {
        accent_bar().show(ui, |_ui| {});
        ui.add_space(12.0);

        ui.horizontal_wrapped(|ui| {
            let tab_label = match self.state.current_tab {
                Tab::All => "Dashboard",
                Tab::Folders => "Watch Folders",
                Tab::Queue => "Batch Queue",
                Tab::Settings => "Settings",
                Tab::Activity => "Activity Log",
            };
            ui.label(
                RichText::new(tab_label)
                    .size(20.0)
                    .color(ACCENT_PRIMARY)
                    .strong(),
            );

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Watch/Stop toggle in header for prominence
                let is_watching = self.state.watcher_rx.is_some();
                if is_watching {
                    if ui.add(button_danger("■ Stop")).clicked() {
                        if let Some(stop) = self.state.watcher_stop.take() {
                            stop.store(true, std::sync::atomic::Ordering::SeqCst);
                        }
                        self.state.watcher_rx = None;
                        self.state.status = ProcessingStatus::Idle;
                    }
                } else {
                    if ui.add(button_primary("▶ Watch")).clicked() {
                        self.start_watcher();
                    }
                }
                ui.add_space(12.0);
                ui.add_space(8.0);
                let (rect, _) = ui.allocate_exact_size(egui::vec2(8.0, 8.0), egui::Sense::hover());
                let dot_color = match &self.state.status {
                    ProcessingStatus::Idle => TEXT_MUTED,
                    ProcessingStatus::Watching => SUCCESS_DIM,
                    ProcessingStatus::Processing(_) => PROCESSING,
                    ProcessingStatus::Error(_) => ERROR,
                };
                ui.painter().circle_filled(rect.center(), 3.5, dot_color);
                ui.add_space(6.0);
                let status_text = match &self.state.status {
                    ProcessingStatus::Idle => "Idle",
                    ProcessingStatus::Watching => "Watching",
                    ProcessingStatus::Processing(stage) => stage.as_str(),
                    ProcessingStatus::Error(err) => err.as_str(),
                };
                ui.label(egui::RichText::new(status_text).size(13.0).color(dot_color));
            });
        });
    }

    pub(crate) fn draw_folders_panel(&mut self, ui: &mut egui::Ui) {
        panel_frame().show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("Watch Folders")
                        .size(18.0)
                        .color(ACCENT_PRIMARY)
                        .strong(),
                );

                let (status_text, status_color, bg_color) = match &self.state.status {
                    ProcessingStatus::Idle => ("Paused", TEXT_SECONDARY, PANEL_BG_LIGHT),
                    ProcessingStatus::Watching => ("Watching", SUCCESS, SUCCESS_BG),
                    ProcessingStatus::Processing(_) => ("Processing", WARNING, PANEL_BG_LIGHT),
                    ProcessingStatus::Error(_) => ("Error", ERROR, ERROR_BG),
                };

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.add(button_add("+ Add")).clicked() {
                        self.state.modal.reset_for_add();
                    }
                    ui.add_space(12.0);
                    status_badge_with_bg(ui, status_text, status_color, bg_color);
                });
            });

            ui.add_space(12.0);

            let mut toggle_idx: Option<usize> = None;
            let mut edit_idx: Option<usize> = None;
            let mut delete_idx: Option<usize> = None;

            if self.state.folders.is_empty() {
                inner_panel().show(ui, |ui| {
                    ui.add_space(12.0);
                    ui.label(label_muted("No folders configured"));
                    ui.add_space(8.0);
                    if ui.add(button_secondary("+ Add Folder")).clicked() {
                        self.state.modal.reset_for_add();
                    }
                    ui.add_space(12.0);
                });
            } else {
                for (idx, folder) in self.state.folders.iter().enumerate() {
                    let enabled = folder.enabled;
                    let input = folder.input.clone();
                    let output = folder.output.clone();
                    let preset = folder.preset.clone();
                    let muted_color = if enabled { TEXT_SECONDARY } else { TEXT_MUTED };
                    let text_color = if enabled { TEXT_PRIMARY } else { TEXT_MUTED };

                    folder_card_compact(enabled).show(ui, |ui| {
                        ui.horizontal(|ui| {
                            if ui
                                .add(button_toggle(enabled, if enabled { "ON" } else { "OFF" }))
                                .clicked()
                            {
                                toggle_idx = Some(idx);
                            }

                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    preset_badge(&preset, ui);
                                },
                            );
                        });

                        ui.add_space(6.0);

                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Input:").color(muted_color).size(13.0));
                            ui.label(
                                RichText::new(truncate_path(&input.to_string_lossy(), 30))
                                    .color(text_color)
                                    .size(13.0),
                            );
                        });
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Output:").color(muted_color).size(13.0));
                            ui.label(
                                RichText::new(truncate_path(&output.to_string_lossy(), 30))
                                    .color(text_color)
                                    .size(13.0),
                            );
                        });

                        ui.add_space(4.0);
                        ui.horizontal(|ui| {
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if ui.add(button_small("Edit")).clicked() {
                                        edit_idx = Some(idx);
                                    }
                                    if ui.add(button_small("Remove")).clicked() {
                                        delete_idx = Some(idx);
                                    }
                                },
                            );
                        });
                    });

                    // Removed click-to-edit on card body — explicit Edit button above

                    ui.add_space(8.0);
                }
            }

            if let Some(idx) = toggle_idx {
                self.state.toggle_folder(idx);
            }
            if let Some(idx) = delete_idx {
                self.state.modal.prompt_delete(idx);
            }
            if let Some(idx) = edit_idx
                && let Some(folder) = self.state.folders.get(idx)
            {
                self.state.modal.set_for_edit(idx, folder);
            }
        });

        // Recent outputs section
        if !self.state.recent_outputs.is_empty() {
            ui.add_space(16.0);
            ui.separator();
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("Recent Outputs")
                        .size(12.0)
                        .color(TEXT_SECONDARY)
                        .strong(),
                );
            });
            ui.add_space(4.0);
            for (idx, path) in self.state.recent_outputs.iter().enumerate().take(5) {
                if idx > 0 {
                    ui.add_space(4.0);
                }
                let filename = path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| path.display().to_string());
                let dir = path
                    .parent()
                    .map(|p| p.display().to_string())
                    .unwrap_or_default();
                ui.horizontal_wrapped(|ui| {
                    let label = egui::Label::new(format!("📄 {}", filename));
                    let resp = ui.add(label);
                    resp.on_hover_text(dir.clone());
                });
            }
        }
    }
}

mod dashboard;
mod modals;
mod queue;
mod settings;
