use eframe::egui;
use egui::RichText;

use super::super::{App, EntryStatus, ProcessingStatus, QueueStatus, Tab, ToastKind};
use crate::config::SilenceMode;
use crate::gui::theme::*;

impl App {
    pub(crate) fn draw_dashboard(&mut self, ui: &mut egui::Ui) {
        // Stats row
        let folder_count = self.state.folders.len();
        let enabled_count = self.state.folders.iter().filter(|f| f.enabled).count();
        let queued_count = self
            .state
            .batch_queue
            .iter()
            .filter(|f| f.status == QueueStatus::Queued)
            .count();
        let processing_count = self
            .state
            .batch_queue
            .iter()
            .filter(|f| matches!(f.status, QueueStatus::Processing | QueueStatus::Queued))
            .count();

        panel_frame().show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.set_width(ui.available_width());
                let stat_items = [
                    ("Folders", format!("{}/{}", enabled_count, folder_count)),
                    ("Queue", format!("{}", queued_count)),
                    ("Processing", format!("{}", processing_count)),
                ];
                for (label, value) in stat_items {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(label).size(12.0).color(TEXT_MUTED));
                        ui.add_space(4.0);
                        ui.label(
                            egui::RichText::new(&value)
                                .size(16.0)
                                .color(ACCENT_PRIMARY)
                                .strong(),
                        );
                    });
                    ui.add_space(24.0);
                }
            });
        });

        ui.add_space(12.0);

        // Quick actions
        panel_frame().show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.set_width(ui.available_width());
                ui.label(
                    RichText::new("Quick Actions")
                        .size(14.0)
                        .color(TEXT_PRIMARY)
                        .strong(),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Toggle watcher on/off
                    let is_watching = self.state.watcher_rx.is_some();
                    let watch_label = if is_watching { "■ Stop" } else { "▶ Watch" };
                    let watch_btn = if is_watching {
                        button_danger(watch_label)
                    } else {
                        button_primary(watch_label)
                    };
                    if ui.add(watch_btn).clicked() {
                        if is_watching {
                            // Stop watcher
                            if let Some(stop) = self.state.watcher_stop.take() {
                                stop.store(true, std::sync::atomic::Ordering::SeqCst);
                            }
                            self.state.watcher_rx = None;
                            self.state.status = ProcessingStatus::Idle;
                        } else {
                            // Start watcher
                            self.state.restart_watcher();
                            self.state.add_toast("Started watching for videos", ToastKind::Success);
                        }
                    }

                    ui.add_space(8.0);

                    if ui.add(button_secondary("+ Add Folder")).clicked() {
                        self.state.modal.reset_for_add();
                    }
                });
            });
        });

        ui.add_space(12.0);

        // Two-column row: Recent Activity (left) + Watch Folders & Settings (right)
        let column_spacing = 12.0;
        let available_width = ui.available_width();
        // On narrow screens (<600px), stack vertically; otherwise two columns
        let use_two_columns = available_width >= 600.0;

        if use_two_columns {
            let left_width = (available_width - column_spacing) * 0.58;
            let right_width = available_width - left_width - column_spacing;

            ui.horizontal_top(|ui| {
                // Left column: Recent Activity
                ui.push_id("dash_col_left", |ui| {
                    ui.set_min_width(left_width);
                    ui.set_max_width(left_width);
                    self.draw_dashboard_recent_activity(ui);
                });

                ui.add_space(column_spacing);

                // Right column: Watch Folders + Settings stacked
                ui.push_id("dash_col_right", |ui| {
                    ui.set_min_width(right_width);
                    ui.set_max_width(right_width);
                    ui.vertical(|ui| {
                        self.draw_dashboard_folders_summary(ui);
                        ui.add_space(12.0);
                        self.draw_dashboard_settings_summary(ui);
                    });
                });
            });
        } else {
            // Narrow layout: stack everything vertically
            self.draw_dashboard_recent_activity(ui);
            ui.add_space(12.0);
            self.draw_dashboard_folders_summary(ui);
            ui.add_space(12.0);
            self.draw_dashboard_settings_summary(ui);
        }
    }

    fn draw_dashboard_recent_activity(&mut self, ui: &mut egui::Ui) {
        panel_frame().show(ui, |ui| {
            ui.vertical(|ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.set_width(ui.available_width());
                    ui.label(
                        RichText::new("Recent Activity")
                            .size(14.0)
                            .color(TEXT_PRIMARY)
                            .strong(),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.add(button_small("View All")).clicked() {
                            self.state.current_tab = Tab::Activity;
                        }
                    });
                });

                ui.add_space(8.0);

                let recent: Vec<_> = self.state.activity_log.iter().rev().take(6).collect();

                if recent.is_empty() || recent.iter().all(|e| e.filename.is_empty()) {
                    ui.add_space(8.0);
                    ui.label(label_muted("No recent activity"));
                } else {
                    for entry in recent {
                        ui.add_space(4.0);
                        ui.horizontal_wrapped(|ui| {
                            ui.set_width(ui.available_width());
                            let dot = match entry.status {
                                EntryStatus::Success => "●",
                                EntryStatus::Processing => "◐",
                                EntryStatus::Error => "✕",
                            };
                            let dot_color = match entry.status {
                                EntryStatus::Success => SUCCESS,
                                EntryStatus::Processing => PROCESSING,
                                EntryStatus::Error => ERROR,
                            };
                            ui.label(egui::RichText::new(dot).size(10.0).color(dot_color));
                            ui.add_space(6.0);
                            ui.label(
                                egui::RichText::new(&entry.timestamp)
                                    .size(11.0)
                                    .color(TEXT_MUTED),
                            );
                            if !entry.filename.is_empty() {
                                ui.add_space(8.0);
                                ui.label(
                                    egui::RichText::new(truncate_path(&entry.filename, 32))
                                        .size(12.0)
                                        .color(TEXT_PRIMARY),
                                );
                            } else if !entry.message.is_empty() {
                                ui.add_space(8.0);
                                ui.label(
                                    egui::RichText::new(&entry.message)
                                        .size(12.0)
                                        .color(TEXT_SECONDARY),
                                );
                            }
                        });
                    }
                }
            });
        });
    }

    fn draw_dashboard_folders_summary(&mut self, ui: &mut egui::Ui) {
        panel_frame().show(ui, |ui| {
            ui.vertical(|ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.set_width(ui.available_width());
                    ui.label(
                        RichText::new("Watch Folders")
                            .size(14.0)
                            .color(TEXT_PRIMARY)
                            .strong(),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.add(button_small("Manage →")).clicked() {
                            self.state.current_tab = Tab::Folders;
                        }
                    });
                });

                ui.add_space(8.0);

                if self.state.folders.is_empty() {
                    ui.label(label_muted("No folders configured"));
                } else {
                    for folder in self.state.folders.iter().take(3) {
                        ui.add_space(4.0);
                        ui.horizontal_wrapped(|ui| {
                            ui.set_width(ui.available_width());
                            let status_icon = if folder.enabled { "●" } else { "○" };
                            let status_color = if folder.enabled { SUCCESS } else { TEXT_MUTED };
                            ui.label(
                                egui::RichText::new(status_icon)
                                    .size(10.0)
                                    .color(status_color),
                            );
                            ui.add_space(4.0);
                            ui.label(
                                egui::RichText::new(truncate_path(
                                    &folder.input.to_string_lossy(),
                                    36,
                                ))
                                .size(12.0)
                                .color(if folder.enabled {
                                    TEXT_PRIMARY
                                } else {
                                    TEXT_MUTED
                                }),
                            );
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    ui.label(
                                        egui::RichText::new(&folder.preset)
                                            .size(10.0)
                                            .color(TEXT_MUTED),
                                    );
                                },
                            );
                        });
                    }
                    if self.state.folders.len() > 3 {
                        ui.add_space(4.0);
                        ui.label(
                            egui::RichText::new(format!("+{} more", self.state.folders.len() - 3))
                                .size(11.0)
                                .color(TEXT_MUTED),
                        );
                    }
                }
            });
        });
    }

    fn draw_dashboard_settings_summary(&mut self, ui: &mut egui::Ui) {
        panel_frame().show(ui, |ui| {
            ui.vertical(|ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.set_width(ui.available_width());
                    ui.label(
                        RichText::new("Settings")
                            .size(14.0)
                            .color(TEXT_PRIMARY)
                            .strong(),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.add(button_small("Edit →")).clicked() {
                            self.state.current_tab = Tab::Settings;
                        }
                    });
                });

                ui.add_space(8.0);

                if let Some(folder) = self.state.folders.get(self.state.selected_folder_idx) {
                    let summary_items = [
                        (
                            "Silence",
                            folder
                                .settings
                                .silence_mode
                                .map(Self::silence_mode_name)
                                .unwrap_or("—"),
                        ),
                        (
                            "Resolution",
                            folder
                                .settings
                                .target_resolution
                                .map(|r| r.display_name())
                                .unwrap_or_default(),
                        ),
                        ("Stabilize", Self::yes_no(folder.settings.stabilize)),
                        ("Color", Self::yes_no(folder.settings.color_correct)),
                        ("Reframe", Self::yes_no(folder.settings.reframe)),
                    ];
                    for (label, value) in summary_items {
                        ui.add_space(2.0);
                        ui.horizontal_wrapped(|ui| {
                            ui.set_width(ui.available_width());
                            ui.add_space(4.0);
                            ui.label(egui::RichText::new(label).size(12.0).color(TEXT_MUTED));
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    ui.label(
                                        egui::RichText::new(value).size(12.0).color(TEXT_PRIMARY),
                                    );
                                },
                            );
                        });
                    }
                } else {
                    ui.label(label_muted("No folder selected"));
                }
            });
        });
    }

    fn silence_mode_name(mode: SilenceMode) -> &'static str {
        match mode {
            SilenceMode::Keep => "Keep All",
            SilenceMode::Cut => "Cut",
            SilenceMode::Speedup => "Speed Up",
        }
    }

    fn yes_no(val: Option<bool>) -> &'static str {
        match val {
            Some(true) => "On",
            Some(false) => "Off",
            None => "—",
        }
    }

    pub(crate) fn draw_toasts(&mut self, ctx: &egui::Context) {
        if self.state.toasts.is_empty() {
            return;
        }

        // Take ownership so we can modify while processing
        let mut toasts = std::mem::take(&mut self.state.toasts);
        let stack_offset = 70.0;
        let mut dismiss_indices = Vec::new();

        for (i, toast) in toasts.iter().enumerate() {
            let elapsed = toast.created.elapsed().as_secs() as f32;
            let alpha = 1.0 - (elapsed / 5.0).min(1.0);
            let stack_y = i as f32 * stack_offset;
            let color = toast.color();
            let icon = toast.icon();

            let bg_alpha = ((alpha * 220.0) as u8).min(220);
            let bg_color = match toast.kind {
                ToastKind::Success => egui::Color32::from_rgba_unmultiplied(18, 40, 26, bg_alpha),
                ToastKind::Error => egui::Color32::from_rgba_unmultiplied(45, 16, 16, bg_alpha),
                ToastKind::Warning => egui::Color32::from_rgba_unmultiplied(50, 40, 10, bg_alpha),
                ToastKind::Info => egui::Color32::from_rgba_unmultiplied(20, 36, 60, bg_alpha),
            };

            let toast_id = egui::Id::new(format!("toast_{}", i));
            let response = egui::Area::new(toast_id)
                .anchor(
                    egui::Align2::RIGHT_BOTTOM,
                    egui::vec2(-20.0, -20.0 - stack_y),
                )
                .interactable(true)
                .show(ctx, |ui| {
                    egui::Frame::NONE
                        .fill(bg_color)
                        .corner_radius(0.0)
                        .inner_margin(egui::vec2(14.0, 10.0))
                        .stroke(egui::Stroke::new(
                            1.0,
                            egui::Color32::from_rgba_unmultiplied(
                                color.r(),
                                color.g(),
                                color.b(),
                                ((alpha * 180.0) as u32).min(180) as u8,
                            ),
                        ))
                        .show(ui, |ui| {
                            ui.set_width(300.0);
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new(icon).size(16.0).color(color));
                                ui.add_space(8.0);
                                ui.label(
                                    egui::RichText::new(&toast.message)
                                        .size(13.0)
                                        .color(TEXT_PRIMARY),
                                );
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        if ui
                                            .add(egui::Button::new("×").small().frame(false))
                                            .clicked()
                                        {
                                            dismiss_indices.push(i);
                                        }
                                    },
                                );
                            });

                            let remaining = 5.0 - elapsed;
                            if remaining > 0.0 && remaining < 2.0 {
                                ui.add_space(4.0);
                                let progress = remaining / 2.0;
                                ui.add(
                                    egui::ProgressBar::new(1.0 - progress)
                                        .fill(color)
                                        .corner_radius(0.0)
                                        .desired_width(ui.available_width()),
                                );
                            }
                        });
                });
            // Check if toast was clicked (only primary/left button)
            let clicked = response.response.interact(egui::Sense::click()).clicked();
            if clicked {
                dismiss_indices.push(i);
            }
        }

        // Remove dismissed toasts (in reverse order to maintain indices)
        for &idx in dismiss_indices.iter().rev() {
            toasts.remove(idx);
        }

        // Put remaining toasts back
        self.state.toasts = toasts;
    }

    pub(crate) fn draw_activity_log(&mut self, ui: &mut egui::Ui, full_height: bool) {
        panel_frame().show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("Activity")
                        .size(18.0)
                        .color(ACCENT_PRIMARY)
                        .strong(),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.add(button_small("Import")).clicked()
                        && let Some(path) = rfd::FileDialog::new()
                            .add_filter("JSON", &["json"])
                            .pick_file()
                    {
                        if let Err(e) = self.state.import_config_from(&path) {
                            self.state
                                .add_toast(format!("Import failed: {}", e), ToastKind::Error);
                        } else {
                            self.state
                                .add_toast("Config imported successfully", ToastKind::Success);
                        }
                    }
                    if ui.add(button_small("Export")).clicked()
                        && let Some(path) = rfd::FileDialog::new()
                            .add_filter("JSON", &["json"])
                            .save_file()
                    {
                        if let Err(e) = self.state.export_config_to(&path) {
                            self.state
                                .add_toast(format!("Export failed: {}", e), ToastKind::Error);
                        } else {
                            self.state
                                .add_toast("Config exported successfully", ToastKind::Success);
                        }
                    }
                    if ui.add(button_small("Clear")).clicked() {
                        self.state.activity_log.clear();
                    }
                });
            });

            ui.add_space(12.0);

            if self.state.activity_log.is_empty() {
                inner_panel().show(ui, |ui| {
                    ui.add_space(12.0);
                    ui.label(label_muted("No activity yet"));
                    ui.add_space(12.0);
                });
            } else {
                let entries: Vec<_> = if full_height {
                    self.state.activity_log.iter().rev().collect()
                } else {
                    self.state.activity_log.iter().rev().take(50).collect()
                };

                for entry in entries {
                    ui.add_space(4.0);
                    match entry.status {
                        EntryStatus::Success => {
                            log_entry_success(
                                ui,
                                &entry.timestamp,
                                &entry.filename,
                                &format_file_size(entry.file_size),
                                &format_duration(entry.duration.unwrap_or(0)),
                            );
                        }
                        EntryStatus::Processing => {
                            log_entry_processing(
                                ui,
                                &entry.timestamp,
                                &entry.filename,
                                &entry.message,
                                entry.progress.unwrap_or(0.0),
                            );
                        }
                        EntryStatus::Error => {
                            log_entry_error(ui, &entry.timestamp, &entry.filename, &entry.message);
                        }
                    }
                }
            }
        });
    }
}
