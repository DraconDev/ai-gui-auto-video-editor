use eframe::egui;
use egui::RichText;
use rfd::FileDialog;
use std::path::PathBuf;
use std::sync::mpsc;

use super::theme::*;
use super::{ActivityEntry, App, EntryStatus, FolderState, ProcessingStatus, SettingsCategory, SetupStep, Tab, ToastKind};
use crate::config::{FolderSettings, SilenceMode, VideoResolution};
use crate::hwaccel::HwAccel;

impl App {
    pub(crate) fn draw_header(&mut self, ui: &mut egui::Ui) {
        accent_bar().show(ui, |_ui| {});
        ui.add_space(12.0);

        ui.horizontal_wrapped(|ui| {
            ui.label(
                RichText::new("AI Video Processor")
                    .size(22.0)
                    .color(ACCENT_PRIMARY)
                    .strong(),
            );

            let status_text = match &self.state.status {
                ProcessingStatus::Idle => ("Idle", TEXT_MUTED),
                ProcessingStatus::Watching => ("Watching", SUCCESS_DIM),
                ProcessingStatus::Processing(stage) => (stage.as_str(), PROCESSING),
                ProcessingStatus::Error(err) => (err.as_str(), ERROR),
            };
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let (rect, _) = ui.allocate_exact_size(egui::vec2(8.0, 8.0), egui::Sense::hover());
                let dot_color = status_text.1;
                ui.painter().circle_filled(rect.center(), 3.5, dot_color);
                ui.add_space(6.0);
                ui.label(
                    egui::RichText::new(status_text.0)
                        .size(13.0)
                        .color(status_text.1),
                );
            });
        });

        ui.add_space(10.0);

        egui::Frame::NONE
            .fill(PANEL_BG_LIGHT)
            .corner_radius(CORNER_RADIUS_SMALL)
            .inner_margin(egui::vec2(6.0, 4.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    let tabs = [
                        (Tab::All, "All"),
                        (Tab::Folders, "Folders"),
                        (Tab::Queue, "Queue"),
                        (Tab::Settings, "Settings"),
                        (Tab::Activity, "Activity"),
                    ];
                    for (tab, name) in tabs {
                        if ui
                            .add(button_tab(self.state.current_tab == tab, name))
                            .clicked()
                        {
                            self.state.current_tab = tab;
                        }
                    }
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

                    let response = folder_card_compact(enabled).show(ui, |ui| {
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
                                RichText::new(truncate_path(&input.to_string_lossy(), 40))
                                    .color(text_color)
                                    .size(13.0),
                            );
                        });
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Output:").color(muted_color).size(13.0));
                            ui.label(
                                RichText::new(truncate_path(&output.to_string_lossy(), 40))
                                    .color(text_color)
                                    .size(13.0),
                            );
                        });

                        if self.state.folders.len() > 1 {
                            ui.add_space(4.0);
                            ui.horizontal(|ui| {
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        if ui.add(button_small("Remove")).clicked() {
                                            delete_idx = Some(idx);
                                        }
                                    },
                                );
                            });
                        }
                    });

                    if response.response.clicked() {
                        edit_idx = Some(idx);
                    }

                    ui.add_space(8.0);
                }
            }

            if let Some(idx) = toggle_idx {
                self.state.toggle_folder(idx);
            }
            if let Some(idx) = delete_idx {
                self.state.modal.prompt_delete(idx);
            }
            if let Some(idx) = edit_idx {
                let folder = &self.state.folders[idx];
                self.state.modal.set_for_edit(idx, folder);
            }
        });
    }

    pub(crate) fn draw_delete_confirm_modal(&mut self, ctx: &egui::Context) {
        let mut should_delete = false;
        let mut should_close = false;

        let screen_rect = ctx.screen_rect();

        egui::Area::new(egui::Id::new("delete_overlay"))
            .anchor(egui::Align2::LEFT_TOP, egui::vec2(0.0, 0.0))
            .order(egui::Order::Foreground)
            .interactable(true)
            .show(ctx, |ui| {
                let (_rect, response) =
                    ui.allocate_exact_size(screen_rect.size(), egui::Sense::click());
                modal_overlay().show(ui, |ui| {
                    ui.allocate_space(screen_rect.size());
                });
                if response.clicked() {
                    should_close = true;
                }
            });

        egui::Area::new(egui::Id::new("delete_dialog"))
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .order(egui::Order::Foreground)
            .interactable(true)
            .show(ctx, |ui| {
                modal_dialog().show(ui, |ui| {
                    ui.set_min_width(320.0);
                    ui.set_max_width(320.0);

                    ui.horizontal(|ui| {
                        ui.label(RichText::new("⚠").size(24.0).color(WARNING));
                        ui.add_space(8.0);
                        ui.label(
                            RichText::new("Remove Folder")
                                .size(18.0)
                                .color(TEXT_PRIMARY)
                                .strong(),
                        );
                    });

                    ui.add_space(12.0);

                    if let Some(idx) = self.state.modal.delete_confirm_idx
                        && let Some(folder) = self.state.folders.get(idx)
                    {
                        let folder_name = folder
                            .input
                            .file_name()
                            .map(|n| n.to_string_lossy())
                            .unwrap_or_else(|| "this folder".into());

                        ui.label(label_secondary(&format!("Stop watching {}?", folder_name)));
                        ui.add_space(4.0);
                        ui.label(label_muted(
                            "Videos in this folder will no longer be auto-processed.",
                        ));
                    }

                    ui.add_space(20.0);

                    ui.horizontal(|ui| {
                        if ui.add(button_secondary("Cancel")).clicked() {
                            should_close = true;
                        }
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.add(button_danger("Remove")).clicked() {
                                should_delete = true;
                                should_close = true;
                            }
                        });
                    });
                });
            });

        if should_close {
            if should_delete && let Some(idx) = self.state.modal.delete_confirm_idx {
                self.state.remove_folder(idx);
            }
            self.state.modal.delete_confirm_idx = None;
        }
    }

    pub(crate) fn draw_setup_wizard(&mut self, ctx: &egui::Context) {
        let screen_rect = ctx.screen_rect();

        // Background overlay
        egui::Area::new(egui::Id::new("setup_overlay"))
            .anchor(egui::Align2::LEFT_TOP, egui::vec2(0.0, 0.0))
            .order(egui::Order::Background)
            .show(ctx, |ui| {
                ui.allocate_exact_size(screen_rect.size(), egui::Sense::hover());
                ui.painter()
                    .rect_filled(screen_rect, 0.0, egui::Color32::from_rgb(15, 15, 20));
            });

        // Center the wizard
        egui::Area::new(egui::Id::new("setup_wizard"))
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                egui::Frame::NONE
                    .fill(PANEL_BG_LIGHT)
                    .corner_radius(16.0)
                    .inner_margin(egui::vec2(48.0, 40.0))
                    .show(ui, |ui| {
                        ui.set_min_width(520.0);
                        ui.set_max_width(520.0);

                        match self.state.setup_step {
                            SetupStep::Welcome => self.draw_setup_welcome(ui),
                            SetupStep::ChooseFolder => self.draw_setup_folder(ui),
                            SetupStep::ProcessingOptions => self.draw_setup_options(ui),
                            SetupStep::Complete => self.draw_setup_complete(ui),
                        }
                    });
            });
    }

    pub(crate) fn draw_setup_welcome(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.label(
                RichText::new("Welcome to AI Video Editor")
                    .size(28.0)
                    .color(ACCENT_PRIMARY)
                    .strong(),
            );
            ui.add_space(16.0);
            ui.label(
                RichText::new("Let's get you set up in just a few clicks.")
                    .size(16.0)
                    .color(TEXT_SECONDARY),
            );
            ui.add_space(32.0);

            // Feature highlights
            egui::Frame::NONE
                .fill(PANEL_BG)
                .corner_radius(12.0)
                .inner_margin(egui::vec2(24.0, 16.0))
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        self.setup_feature_row(
                            ui,
                            "Auto-remove silence",
                            "Cuts dead air automatically",
                        );
                        ui.add_space(8.0);
                        self.setup_feature_row(
                            ui,
                            "Audio enhancement",
                            "Makes your voice sound professional",
                        );
                        ui.add_space(8.0);
                        self.setup_feature_row(
                            ui,
                            "Auto-reframe",
                            "Convert to vertical video for Shorts/Reels",
                        );
                    });
                });

            ui.add_space(32.0);

            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.add(button_primary("Get Started →")).clicked() {
                        self.state.setup_step = SetupStep::ChooseFolder;
                    }
                });
            });
        });
    }

    pub(crate) fn setup_feature_row(&self, ui: &mut egui::Ui, title: &str, desc: &str) {
        egui::Frame::NONE
            .fill(PANEL_BG)
            .corner_radius(8.0)
            .inner_margin(egui::vec2(12.0, 10.0))
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.set_width(ui.available_width());
                    let dot_size = 8.0;
                    let (rect, _) = ui
                        .allocate_exact_size(egui::vec2(dot_size, dot_size), egui::Sense::hover());
                    ui.painter().circle_filled(rect.center(), 3.5, SUCCESS);
                    ui.add_space(10.0);
                    ui.vertical(|ui| {
                        ui.label(RichText::new(title).size(14.0).color(TEXT_PRIMARY).strong());
                        ui.label(RichText::new(desc).size(12.0).color(TEXT_MUTED));
                    });
                });
            });
    }

    pub(crate) fn draw_setup_folder(&mut self, ui: &mut egui::Ui) {
        ui.label(
            RichText::new("Choose Your Video Folder")
                .size(24.0)
                .color(ACCENT_PRIMARY)
                .strong(),
        );
        ui.add_space(8.0);
        ui.label(
            RichText::new("Select where your raw videos are stored.\nWe'll create an 'output' folder next to it.")
                .size(14.0)
                .color(TEXT_SECONDARY),
        );
        ui.add_space(24.0);

        // Folder path display
        egui::Frame::NONE
            .fill(PANEL_BG)
            .corner_radius(8.0)
            .inner_margin(egui::vec2(16.0, 12.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(self.state.setup_folder.to_string_lossy().as_ref())
                            .size(14.0)
                            .color(TEXT_PRIMARY),
                    );
                });
            });

        ui.add_space(12.0);

        ui.horizontal(|ui| {
            if ui.add(button_secondary("📁 Choose Folder...")).clicked()
                && let Some(path) = FileDialog::new().pick_folder()
            {
                self.state.setup_folder = path;
            }
        });

        ui.add_space(24.0);

        // Preset selection
        ui.label(
            RichText::new("What type of content?")
                .size(16.0)
                .color(TEXT_PRIMARY)
                .strong(),
        );
        ui.add_space(12.0);

        ui.horizontal_wrapped(|ui| {
            for (preset, icon, desc) in [
                ("youtube", "🎬", "YouTube videos (landscape)"),
                ("shorts", "📱", "Shorts/Reels/TikTok (vertical)"),
                ("podcast", "🎙️", "Podcast/audio focus"),
            ] {
                let selected = self.state.setup_preset == preset;
                if self
                    .setup_preset_card(ui, selected, icon, preset, desc)
                    .clicked()
                {
                    self.state.setup_preset = preset.to_string();
                }
                ui.add_space(8.0);
            }
        });

        ui.add_space(32.0);

        ui.horizontal(|ui| {
            if ui.add(button_small("← Back")).clicked() {
                self.state.setup_step = SetupStep::Welcome;
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.add(button_primary("Continue →")).clicked() {
                    self.state.setup_step = SetupStep::ProcessingOptions;
                }
            });
        });
    }

    pub(crate) fn setup_preset_card(
        &self,
        ui: &mut egui::Ui,
        selected: bool,
        icon: &str,
        name: &str,
        desc: &str,
    ) -> egui::Response {
        let bg_color = if selected { ACCENT_PRIMARY } else { PANEL_BG };
        let stroke_color = if selected {
            ACCENT_PRIMARY
        } else {
            PANEL_BG_LIGHT
        };

        egui::Frame::NONE
            .fill(bg_color)
            .corner_radius(10.0)
            .stroke(egui::Stroke::new(2.0, stroke_color))
            .inner_margin(egui::vec2(16.0, 12.0))
            .show(ui, |ui| {
                ui.set_min_width(140.0);
                ui.vertical_centered(|ui| {
                    ui.label(RichText::new(icon).size(28.0));
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new(name)
                            .size(14.0)
                            .color(if selected {
                                egui::Color32::WHITE
                            } else {
                                TEXT_PRIMARY
                            })
                            .strong(),
                    );
                    ui.label(RichText::new(desc).size(11.0).color(if selected {
                        egui::Color32::WHITE
                    } else {
                        TEXT_SECONDARY
                    }));
                });
            })
            .response
    }

    pub(crate) fn draw_setup_options(&mut self, ui: &mut egui::Ui) {
        ui.label(
            RichText::new("Processing Options")
                .size(24.0)
                .color(ACCENT_PRIMARY)
                .strong(),
        );
        ui.add_space(8.0);
        ui.label(
            RichText::new("These can be changed later in Settings.")
                .size(14.0)
                .color(TEXT_SECONDARY),
        );
        ui.add_space(24.0);

        // Toggle options
        self.state.setup_enhance = self.setup_toggle(
            ui,
            "Enhance Audio",
            "Normalize speech & improve clarity",
            self.state.setup_enhance,
        );
        ui.add_space(12.0);
        self.state.setup_remove_silence = self.setup_toggle(
            ui,
            "Remove Silence",
            "Auto-cut dead air & pauses",
            self.state.setup_remove_silence,
        );

        ui.add_space(32.0);

        ui.horizontal(|ui| {
            if ui.add(button_small("← Back")).clicked() {
                self.state.setup_step = SetupStep::ChooseFolder;
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.add(button_primary("Finish Setup ✓")).clicked() {
                    self.complete_setup();
                    self.state.setup_step = SetupStep::Complete;
                }
            });
        });
    }

    pub(crate) fn setup_toggle(
        &self,
        ui: &mut egui::Ui,
        title: &str,
        desc: &str,
        value: bool,
    ) -> bool {
        let mut new_value = value;
        settings_toggle_frame(value).show(ui, |ui| {
            ui.horizontal(|ui| {
                let dot_color = if value { ACCENT_PRIMARY } else { TEXT_MUTED };
                let (dot_rect, _) =
                    ui.allocate_exact_size(egui::vec2(8.0, 8.0), egui::Sense::hover());
                ui.painter()
                    .circle_filled(dot_rect.center(), 3.5, dot_color);
                ui.add_space(8.0);
                ui.vertical(|ui| {
                    ui.label(RichText::new(title).size(15.0).color(TEXT_PRIMARY).strong());
                    ui.label(RichText::new(desc).size(12.0).color(TEXT_SECONDARY));
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .add(button_toggle(value, if value { "ON" } else { "OFF" }))
                        .clicked()
                    {
                        new_value = !value;
                    }
                });
            });
        });
        new_value
    }

    pub(crate) fn draw_setup_complete(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.label(RichText::new("🎉").size(64.0));
            ui.add_space(16.0);
            ui.label(
                RichText::new("You're All Set!")
                    .size(28.0)
                    .color(ACCENT_PRIMARY)
                    .strong(),
            );
            ui.add_space(16.0);
            ui.label(
                RichText::new(
                    "Drop videos into your folder and they'll be processed automatically.",
                )
                .size(14.0)
                .color(TEXT_SECONDARY),
            );
            ui.add_space(24.0);

            // Summary
            egui::Frame::NONE
                .fill(PANEL_BG)
                .corner_radius(10.0)
                .inner_margin(egui::vec2(24.0, 16.0))
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        ui.label(
                            RichText::new("Setup Summary")
                                .size(14.0)
                                .color(TEXT_PRIMARY)
                                .strong(),
                        );
                        ui.add_space(8.0);
                        ui.label(
                            RichText::new(format!(
                                "📁 Folder: {}",
                                self.state.setup_folder.display()
                            ))
                            .size(13.0)
                            .color(TEXT_SECONDARY),
                        );
                        ui.label(
                            RichText::new(format!("🎬 Preset: {}", self.state.setup_preset))
                                .size(13.0)
                                .color(TEXT_SECONDARY),
                        );
                        ui.label(
                            RichText::new(format!(
                                "🔧 Enhance: {}",
                                if self.state.setup_enhance {
                                    "ON"
                                } else {
                                    "OFF"
                                }
                            ))
                            .size(13.0)
                            .color(TEXT_SECONDARY),
                        );
                        ui.label(
                            RichText::new(format!(
                                "✂️ Silence removal: {}",
                                if self.state.setup_remove_silence {
                                    "ON"
                                } else {
                                    "OFF"
                                }
                            ))
                            .size(13.0)
                            .color(TEXT_SECONDARY),
                        );
                    });
                });

            ui.add_space(32.0);

            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.add(button_primary("Start Editing →")).clicked() {
                        self.state.show_setup = false;
                    }
                });
            });
        });
    }

    pub(crate) fn complete_setup(&mut self) {
        // Create output folder
        let output_folder = self.state.setup_folder.join("output");
        let _ = std::fs::create_dir_all(&output_folder);
        let _ = std::fs::create_dir_all(&self.state.setup_folder);

        // Create the folder config
        let folder = FolderState {
            input: self.state.setup_folder.clone(),
            output: output_folder,
            preset: self.state.setup_preset.clone(),
            enabled: true,
            settings: FolderSettings {
                enhance_audio: Some(self.state.setup_enhance),
                remove_silence: Some(self.state.setup_remove_silence),
                ..Default::default()
            },
        };

        self.state.folders = vec![folder];
        self.state.activity_log.push(ActivityEntry::simple(
            format!(
                "Setup complete! Watching: {}",
                self.state.setup_folder.display()
            ),
            true,
        ));

        // Save config
        self.state.auto_save_config();
    }

    pub(crate) fn draw_modal(&mut self, ctx: &egui::Context) {
        let mut should_close = false;
        let mut should_save = false;

        let screen_rect = ctx.screen_rect();

        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            should_close = true;
        }

        egui::Area::new(egui::Id::new("modal_overlay"))
            .anchor(egui::Align2::LEFT_TOP, egui::vec2(0.0, 0.0))
            .order(egui::Order::Foreground)
            .interactable(true)
            .show(ctx, |ui| {
                let (_rect, response) =
                    ui.allocate_exact_size(screen_rect.size(), egui::Sense::click());
                modal_overlay().show(ui, |ui| {
                    ui.allocate_space(screen_rect.size());
                });
                if response.clicked() {
                    should_close = true;
                }
            });

        egui::Area::new(egui::Id::new("modal_dialog"))
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .order(egui::Order::Foreground)
            .interactable(true)
            .show(ctx, |ui| {
                modal_dialog().show(ui, |ui| {
                    ui.set_min_width(320.0);
                    ui.set_max_width(320.0);

                    ui.label(label_secondary("Input Folder"));
                    ui.add_space(3.0);
                    ui.horizontal(|ui| {
                        let mut input_str = self.state.modal.input.to_string_lossy().to_string();
                        ui.add_sized(egui::vec2(240.0, 40.0), text_edit_style(&mut input_str));
                        self.state.modal.input = PathBuf::from(&input_str);
                        if ui.add(button_small("...")).clicked()
                            && let Some(path) = FileDialog::new().pick_folder()
                        {
                            self.state.modal.input = path;
                        }
                    });

                    ui.add_space(12.0);

                    ui.label(label_secondary("Output Folder"));
                    ui.add_space(3.0);
                    ui.horizontal(|ui| {
                        let mut output_str = self.state.modal.output.to_string_lossy().to_string();
                        ui.add_sized(egui::vec2(240.0, 40.0), text_edit_style(&mut output_str));
                        self.state.modal.output = PathBuf::from(&output_str);
                        if ui.add(button_small("...")).clicked()
                            && let Some(path) = FileDialog::new().pick_folder()
                        {
                            self.state.modal.output = path;
                        }
                    });

                    ui.add_space(12.0);

                    ui.label(label_secondary("Preset"));
                    ui.add_space(3.0);
                    ui.horizontal_wrapped(|ui| {
                        for preset in &["youtube", "shorts", "podcast"] {
                            if ui
                                .add(button_pill(self.state.modal.preset == *preset, *preset))
                                .clicked()
                            {
                                let old_preset = self.state.modal.preset.clone();
                                self.state.modal.preset = preset.to_string();

                                if Self::is_default_path(&self.state.modal.input, &old_preset) {
                                    self.state.modal.input =
                                        PathBuf::from(format!("videos/{}", preset));
                                }
                                if Self::is_default_path(&self.state.modal.output, &old_preset) {
                                    self.state.modal.output =
                                        PathBuf::from(format!("videos/{}/output", preset));
                                }
                            }
                        }
                    });

                    ui.add_space(16.0);

                    ui.horizontal(|ui| {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let btn_text = if self.state.modal.editing_idx.is_some() {
                                "Save"
                            } else {
                                "Add"
                            };
                            if ui.add(button_secondary(btn_text)).clicked() {
                                should_save = true;
                                should_close = true;
                            }
                            ui.add_space(8.0);
                            if ui.add(button_small("Cancel")).clicked() {
                                should_close = true;
                            }
                        });
                    });
                });
            });

        if should_close {
            if should_save {
                if let Some(idx) = self.state.modal.editing_idx {
                    self.state.update_folder_from_modal(idx);
                } else {
                    self.state.add_folder_from_modal();
                }
            }
            self.state.modal.close();
        }
    }

    pub(crate) fn is_default_path(path: &std::path::Path, preset: &str) -> bool {
        let default_input = format!("videos/{}", preset);
        let default_output = format!("videos/{}/output", preset);
        path.to_string_lossy() == default_input
            || path.to_string_lossy() == default_output
            || path.to_string_lossy() == "videos"
            || path.to_string_lossy() == "videos/output"
    }

    pub(crate) fn draw_settings_panel(&mut self, ui: &mut egui::Ui) {
        let folder_names: Vec<String> = self
            .state
            .folders
            .iter()
            .map(|f| {
                let name = f
                    .input
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "Folder".to_string());
                truncate_path(&name, 20)
            })
            .collect();

        let preset_name = self
            .state
            .folders
            .get(self.state.selected_folder_idx)
            .map(|f| f.preset.clone())
            .unwrap_or_default();

        settings_panel_frame().show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                for (idx, name) in folder_names.iter().enumerate() {
                    if ui
                        .add(button_pill(idx == self.state.selected_folder_idx, name))
                        .clicked()
                    {
                        self.state.selected_folder_idx = idx;
                    }
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    preset_badge(&preset_name, ui);
                });
            });

            ui.add_space(14.0);

            let mut needs_save = false;
            let folder_idx = self.state.selected_folder_idx;

            if self.state.folders.is_empty() {
                inner_panel().show(ui, |ui| {
                    ui.add_space(20.0);
                    ui.vertical_centered(|ui| {
                        ui.label(
                            RichText::new("📁 No Folders Configured")
                                .size(18.0)
                                .color(TEXT_SECONDARY)
                                .strong(),
                        );
                        ui.add_space(8.0);
                        ui.label(label_muted("Add a folder in the Folders tab to configure settings"));
                    });
                    ui.add_space(20.0);
                });
                return;
            }

            let is_processing = self.state.queue_processing
                || matches!(self.state.status, ProcessingStatus::Processing(_));

            let sidebar_width = 160.0;
            let spacing = 16.0;

            let available = ui.available_width();
            let content_width = available - sidebar_width - spacing;

            ui.horizontal(|ui| {
                // Fixed-width sidebar using allocate_ui
                ui.allocate_ui(egui::vec2(sidebar_width, ui.available_height()), |ui| {
                    ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                        let categories = [
                            SettingsCategory::Processing,
                            SettingsCategory::Audio,
                            SettingsCategory::Video,
                            SettingsCategory::Exports,
                            SettingsCategory::Advanced,
                        ];
                        for cat in categories {
                            let is_active = self.state.settings_category == cat;
                            let label = format!("{} {}", cat.icon(), cat.label());
                            if ui
                                .add(egui::Button::new(
                                    egui::RichText::new(&label).size(14.0).color(if is_active {
                                        TEXT_PRIMARY
                                    } else {
                                        TEXT_SECONDARY
                                    }),
                                )
                                .fill(if is_active { PANEL_BG_LIGHTER } else { PANEL_BG })
                                .stroke(egui::Stroke::new(
                                    if is_active { 1.5 } else { 0.0 },
                                    if is_active { ACCENT_PRIMARY } else { egui::Color32::TRANSPARENT },
                                ))
                                .corner_radius(CORNER_RADIUS_SMALL)
                                .min_size(egui::vec2(sidebar_width - 8.0, 40.0)))
                                .clicked()
                            {
                                self.state.settings_category = cat;
                            }
                            ui.add_space(4.0);
                        }
                    });
                });

                ui.add_space(spacing);

                // Content area - fills remaining width
                ui.allocate_ui(egui::vec2(content_width, ui.available_height()), |ui| {
                    ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                        if is_processing {
                            settings_section_frame(false).show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label(
                                        egui::RichText::new("⚙️")
                                            .size(14.0),
                                    );
                                    ui.add_space(6.0);
                                    ui.label(label_muted("Settings locked during processing"));
                                });
                            });
                        } else {
                            match self.state.settings_category {
                                SettingsCategory::Processing => {
                                    needs_save = self.draw_settings_processing(ui, folder_idx);
                                }
                                SettingsCategory::Audio => {
                                    needs_save = self.draw_settings_audio(ui, folder_idx);
                                }
                                SettingsCategory::Video => {
                                    needs_save = self.draw_settings_video(ui, folder_idx);
                                }
                                SettingsCategory::Exports => {
                                    needs_save = self.draw_settings_exports(ui, folder_idx);
                                }
                                SettingsCategory::Advanced => {
                                    needs_save = self.draw_settings_advanced(ui, folder_idx);
                                }
                            }
                        }
                    });
                });
            });

            if needs_save {
                self.state.auto_save_config();
            }
        });
    }

    fn draw_settings_processing(&mut self, ui: &mut egui::Ui, folder_idx: usize) -> bool {
        let mut needs_save = false;
        let folder = self.state.folders.get(folder_idx);

        let stabilize = folder.and_then(|f| f.settings.stabilize).unwrap_or(false);
        let color_correct = folder.and_then(|f| f.settings.color_correct).unwrap_or(false);
        let reframe = folder.and_then(|f| f.settings.reframe).unwrap_or(false);
        let blur = folder.and_then(|f| f.settings.blur_background).unwrap_or(false);
        let scene_detect = folder.and_then(|f| f.settings.scene_detect).unwrap_or(false);
        let threshold = folder.and_then(|f| f.settings.silence_threshold_db).unwrap_or(-30.0);
        let silence_mode = folder.and_then(|f| f.settings.silence_mode).unwrap_or(SilenceMode::Cut);
        let silence_padding = folder.and_then(|f| f.settings.silence_padding).unwrap_or(0.1);
        let silence_speedup_factor = folder.and_then(|f| f.settings.silence_speedup_factor).unwrap_or(4.0);
        let silence_min_duration = folder.and_then(|f| f.settings.silence_min_duration).unwrap_or(0.5);
        let silence_min_for_speedup = folder.and_then(|f| f.settings.silence_min_silence_for_speedup).unwrap_or(0.5);
        let silence_scene_threshold = folder.and_then(|f| f.settings.silence_scene_threshold).unwrap_or(0.3);

        ui.label(section_title("Processing"));
        ui.add_space(4.0);
        ui.add(egui::Label::new(egui::RichText::new("Core video editing: silence removal, stabilization, color correction").size(12.0).color(TEXT_SECONDARY)));
        ui.add_space(12.0);

        let mut stabilize = stabilize;
        if Self::draw_settings_toggle(
            ui,
            "Stabilize Video",
            "Reduce camera shake in moving footage",
            &mut stabilize,
        ) && let Some(f) = self.state.folders.get_mut(folder_idx)
        {
            f.settings.stabilize = Some(stabilize);
            needs_save = true;
        }
        ui.add_space(6.0);

        let mut color_correct = color_correct;
        if Self::draw_settings_toggle(
            ui,
            "Color Correct",
            "Auto-balance contrast and white levels",
            &mut color_correct,
        ) && let Some(f) = self.state.folders.get_mut(folder_idx)
        {
            f.settings.color_correct = Some(color_correct);
            needs_save = true;
        }
        ui.add_space(6.0);

        let mut reframe = reframe;
        if Self::draw_settings_toggle(
            ui,
            "Auto-Reframe (9:16)",
            "Uses AI face detection to track subjects and crop to vertical 9:16",
            &mut reframe,
        ) && let Some(f) = self.state.folders.get_mut(folder_idx)
        {
            f.settings.reframe = Some(reframe);
            needs_save = true;
        }
        ui.add_space(6.0);

        if reframe {
            let mut blur = blur;
            if Self::draw_settings_toggle(
                ui,
                "Blur Background",
                "Only works with Auto-Reframe above. Blurs edges outside the vertical crop",
                &mut blur,
            ) && let Some(f) = self.state.folders.get_mut(folder_idx)
            {
                f.settings.blur_background = Some(blur);
                needs_save = true;
            }
            ui.add_space(6.0);
        }

        let mut scene_detect = scene_detect;
        if Self::draw_settings_toggle(
            ui,
            "Scene Detection",
            "Uses visual scene changes (not just audio silence) to find better cut points",
            &mut scene_detect,
        ) && let Some(f) = self.state.folders.get_mut(folder_idx)
        {
            f.settings.scene_detect = Some(scene_detect);
            needs_save = true;
        }
        ui.add_space(6.0);

        let mut threshold = threshold;
        let threshold_label = format!("{threshold:.0} dB");
        if Self::draw_advanced_slider(
            ui,
            "Silence Threshold",
            "Lower values keep more ambient speech",
            &mut threshold,
            -60.0..=-10.0,
            threshold_label,
            1.0,
        ) && let Some(f) = self.state.folders.get_mut(folder_idx)
        {
            f.settings.silence_threshold_db = Some(threshold);
            needs_save = true;
        }
        ui.add_space(6.0);

        ui.label(label_secondary("Silence Mode"));
        ui.add_space(4.0);
        let mode_options: [(String, SilenceMode); 3] = [
            (String::from("Keep All"), SilenceMode::Keep),
            (String::from("Cut"), SilenceMode::Cut),
            (String::from("Speed Up"), SilenceMode::Speedup),
        ];
        let mut selected_mode = silence_mode;
        let mode_label = match selected_mode {
            SilenceMode::Keep => "Keep All",
            SilenceMode::Cut => "Cut",
            SilenceMode::Speedup => "Speed Up",
        };
        dropdown_selector(ui, &format!("silence_mode_{}", folder_idx), &mut selected_mode, &mode_options, mode_label);
        ui.add(egui::Label::new(egui::RichText::new("Keep All = no changes. Cut = remove silence. Speed Up = keep but play faster").size(11.0).color(TEXT_SECONDARY)));
        if selected_mode != silence_mode && let Some(f) = self.state.folders.get_mut(folder_idx) {
            f.settings.silence_mode = Some(selected_mode);
            needs_save = true;
        }

        ui.add_space(8.0);

        if selected_mode != SilenceMode::Keep {
            let mut silence_padding = silence_padding;
            let padding_label = format!("{:.2}s", silence_padding);
            if Self::draw_advanced_slider(
                ui,
                "Silence Padding",
                "Keep this much audio before/after cuts",
                &mut silence_padding,
                0.0..=0.5,
                padding_label,
                0.01,
            ) && let Some(f) = self.state.folders.get_mut(folder_idx)
            {
                f.settings.silence_padding = Some(silence_padding);
                needs_save = true;
            }
            ui.add_space(6.0);

            let mut silence_min_duration = silence_min_duration;
            let min_dur_label = format!("{:.1}s", silence_min_duration);
            if Self::draw_advanced_slider(
                ui,
                "Min Silence Duration",
                "Ignore silences shorter than this",
                &mut silence_min_duration,
                0.1..=2.0,
                min_dur_label,
                0.1,
            ) && let Some(f) = self.state.folders.get_mut(folder_idx)
            {
                f.settings.silence_min_duration = Some(silence_min_duration);
                needs_save = true;
            }
            ui.add_space(6.0);
        }

        if selected_mode == SilenceMode::Speedup {
            let mut silence_speedup_factor = silence_speedup_factor;
            let speedup_label = format!("{:.1}x", silence_speedup_factor);
            if Self::draw_advanced_slider(
                ui,
                "Speed Up Factor",
                "How much to speed up silent sections",
                &mut silence_speedup_factor,
                1.5..=8.0,
                speedup_label,
                0.1,
            ) && let Some(f) = self.state.folders.get_mut(folder_idx)
            {
                f.settings.silence_speedup_factor = Some(silence_speedup_factor);
                needs_save = true;
            }
            ui.add_space(6.0);

            let mut silence_min_for_speedup = silence_min_for_speedup;
            let min_speedup_label = format!("{:.1}s", silence_min_for_speedup);
            if Self::draw_advanced_slider(
                ui,
                "Min Speedup Duration",
                "Only speed up silences longer than this",
                &mut silence_min_for_speedup,
                0.1..=2.0,
                min_speedup_label,
                0.1,
            ) && let Some(f) = self.state.folders.get_mut(folder_idx)
            {
                f.settings.silence_min_silence_for_speedup = Some(silence_min_for_speedup);
                needs_save = true;
            }
            ui.add_space(6.0);
        }

        if scene_detect {
            let mut silence_scene_threshold = silence_scene_threshold;
            let scene_label = format!("{:.2}", silence_scene_threshold);
            if Self::draw_advanced_slider(
                ui,
                "Scene Threshold",
                "Higher = fewer scene changes detected",
                &mut silence_scene_threshold,
                0.1..=0.9,
                scene_label,
                0.01,
            ) && let Some(f) = self.state.folders.get_mut(folder_idx)
            {
                f.settings.silence_scene_threshold = Some(silence_scene_threshold);
                needs_save = true;
            }
        }

        needs_save
    }

    fn draw_settings_audio(&mut self, ui: &mut egui::Ui, folder_idx: usize) -> bool {
        let mut needs_save = false;
        let folder = self.state.folders.get(folder_idx);

        let enhance = folder.and_then(|f| f.settings.enhance_audio).unwrap_or(true);
        let noise_reduction = folder.and_then(|f| f.settings.noise_reduction).unwrap_or(false);
        let lufs = folder.and_then(|f| f.settings.target_lufs).unwrap_or(-14.0);
        let duck_volume = folder.and_then(|f| f.settings.duck_volume).unwrap_or(0.2);
        let music_path = folder.and_then(|f| f.settings.music_path.clone());

        ui.label(section_title("Audio"));
        ui.add_space(4.0);
        ui.add(egui::Label::new(egui::RichText::new("Audio enhancement, noise reduction, background music mixing").size(12.0).color(TEXT_SECONDARY)));
        ui.add_space(12.0);

        let mut enhance = enhance;
        if Self::draw_settings_toggle(
            ui,
            "Enhance Audio",
            "Normalize loudness and improve voice clarity",
            &mut enhance,
        ) && let Some(f) = self.state.folders.get_mut(folder_idx)
        {
            f.settings.enhance_audio = Some(enhance);
            needs_save = true;
        }
        ui.add_space(6.0);

        let mut noise_reduction = noise_reduction;
        if Self::draw_settings_toggle(
            ui,
            "Noise Reduction",
            "Remove background hum, hiss, and noise",
            &mut noise_reduction,
        ) && let Some(f) = self.state.folders.get_mut(folder_idx)
        {
            f.settings.noise_reduction = Some(noise_reduction);
            needs_save = true;
        }
        ui.add_space(6.0);

        let mut lufs = lufs;
        let lufs_label = format!("{lufs:.0} LUFS");
        if Self::draw_advanced_slider(
            ui,
            "Target Loudness",
            "Final audio loudness (YouTube = -14 LUFS)",
            &mut lufs,
            -24.0..=-6.0,
            lufs_label,
            1.0,
        ) && let Some(f) = self.state.folders.get_mut(folder_idx)
        {
            f.settings.target_lufs = Some(lufs);
            needs_save = true;
        }
        ui.add_space(6.0);

        ui.label(label_secondary("Background Music"));
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            let music_label = music_path
                .as_ref()
                .and_then(|p| p.file_name())
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "None".to_string());
            ui.label(label_muted(&music_label));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.add(button_small("Choose...")).clicked()
                    && let Some(path) = FileDialog::new()
                        .add_filter("Audio", &["mp3", "wav", "ogg", "flac", "m4a"])
                        .pick_file()
                    && let Some(f) = self.state.folders.get_mut(folder_idx)
                {
                    f.settings.music_path = Some(path);
                    needs_save = true;
                }
                if music_path.is_some()
                    && ui.add(button_small("✕")).clicked()
                    && let Some(f) = self.state.folders.get_mut(folder_idx)
                {
                    f.settings.music_path = None;
                    needs_save = true;
                }
            });
        });

        if music_path.is_some() {
            ui.add_space(6.0);
            let mut duck_volume = duck_volume;
            let duck_label = format!("{:.0}%", duck_volume * 100.0);
            if Self::draw_advanced_slider(
                ui,
                "Duck Volume",
                "Only when Background Music is set. Lowers music volume when someone speaks",
                &mut duck_volume,
                0.0..=1.0,
                duck_label,
                0.01,
            ) && let Some(f) = self.state.folders.get_mut(folder_idx)
            {
                f.settings.duck_volume = Some(duck_volume);
                needs_save = true;
            }
        }
        ui.add_space(8.0);

        needs_save
    }

    fn draw_settings_video(&mut self, ui: &mut egui::Ui, folder_idx: usize) -> bool {
        let mut needs_save = false;
        let folder = self.state.folders.get(folder_idx);

        let hw_accel = folder.and_then(|f| f.settings.hw_accel).unwrap_or(HwAccel::None);
        let target_res = folder
            .and_then(|f| f.settings.target_resolution)
            .unwrap_or(VideoResolution::Fhd1080p);
        let watermark_path = folder.and_then(|f| f.settings.watermark_path.clone());
        let watermark_position = folder
            .and_then(|f| f.settings.watermark_position.clone())
            .unwrap_or_else(|| "bottom-right".to_string());
        let watermark_scale = folder.and_then(|f| f.settings.watermark_scale).unwrap_or(1.0);

        ui.label(section_title("Video Output"));
        ui.add_space(4.0);
        ui.add(egui::Label::new(egui::RichText::new("Output resolution, GPU encoding, watermark overlay").size(12.0).color(TEXT_SECONDARY)));
        ui.add_space(12.0);

        ui.label(label_secondary("GPU Encoding"));
        ui.add_space(4.0);
        let hw_accel_options: [(String, HwAccel); 5] = [
            (String::from("None (CPU)"), HwAccel::None),
            (String::from("NVIDIA NVENC"), HwAccel::Nvenc),
            (String::from("AMD AMF"), HwAccel::Amf),
            (String::from("VAAPI"), HwAccel::Vaapi),
            (String::from("VideoToolbox (macOS)"), HwAccel::VideoToolbox),
        ];
        let mut selected_hw = hw_accel;
        let hw_label = selected_hw.display_name();
        dropdown_selector(
            ui,
            &format!("hw_accel_{}", folder_idx),
            &mut selected_hw,
            &hw_accel_options,
            hw_label,
        );
        if selected_hw != hw_accel
            && let Some(f) = self.state.folders.get_mut(folder_idx)
        {
            f.settings.hw_accel = Some(selected_hw);
            needs_save = true;
        }

        ui.add_space(8.0);
        ui.label(label_secondary("Target Resolution"));
        ui.add_space(4.0);
        let resolution_options: [(String, VideoResolution); 6] = [
            (String::from("720p HD"), VideoResolution::Hd720p),
            (String::from("1080p Full HD"), VideoResolution::Fhd1080p),
            (String::from("1440p QHD"), VideoResolution::Qhd1440p),
            (String::from("4K UHD"), VideoResolution::Uhd4k),
            (String::from("1080p Vertical"), VideoResolution::Vertical1080p),
            (String::from("720p Vertical"), VideoResolution::Vertical720p),
        ];
        let mut selected_res = target_res;
        let res_label = selected_res.display_name();
        dropdown_selector(
            ui,
            &format!("resolution_{}", folder_idx),
            &mut selected_res,
            &resolution_options,
            res_label,
        );
        if selected_res != target_res
            && let Some(f) = self.state.folders.get_mut(folder_idx)
        {
            f.settings.target_resolution = Some(selected_res);
            needs_save = true;
        }

        ui.add_space(12.0);
        ui.label(section_title("Watermark"));
        ui.add_space(8.0);

        ui.label(label_secondary("Watermark Image"));
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            let watermark_label = watermark_path
                .as_ref()
                .and_then(|p| p.file_name())
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "None".to_string());
            ui.label(label_muted(&watermark_label));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.add(button_small("Choose PNG...")).clicked()
                    && let Some(path) = FileDialog::new()
                        .add_filter("Image", &["png"])
                        .pick_file()
                    && let Some(f) = self.state.folders.get_mut(folder_idx)
                {
                    f.settings.watermark_path = Some(path);
                    needs_save = true;
                }
                if watermark_path.is_some()
                    && ui.add(button_small("✕")).clicked()
                    && let Some(f) = self.state.folders.get_mut(folder_idx)
                {
                    f.settings.watermark_path = None;
                    needs_save = true;
                }
            });
        });

        if watermark_path.is_some() {
            ui.add_space(6.0);

            ui.label(label_secondary("Position"));
            ui.add_space(4.0);
            ui.horizontal_wrapped(|ui| {
                let positions = ["bottom-right", "bottom-left", "top-right", "top-left"];
                let labels = ["Bottom R", "Bottom L", "Top R", "Top L"];
                for (i, pos) in positions.iter().enumerate() {
                    let is_selected = watermark_position == *pos;
                    if ui.add(button_pill(is_selected, labels[i])).clicked()
                        && let Some(f) = self.state.folders.get_mut(folder_idx)
                    {
                        f.settings.watermark_position = Some(pos.to_string());
                        needs_save = true;
                    }
                    ui.add_space(4.0);
                }
            });

            ui.add_space(6.0);

            let mut watermark_scale = watermark_scale;
            let scale_label = format!("{:.1}x", watermark_scale);
            if Self::draw_advanced_slider(
                ui,
                "Scale",
                "Size of watermark relative to video",
                &mut watermark_scale,
                0.1..=3.0,
                scale_label,
                0.1,
            ) && let Some(f) = self.state.folders.get_mut(folder_idx)
            {
                f.settings.watermark_scale = Some(watermark_scale);
                needs_save = true;
            }
        }

        needs_save
    }

    fn draw_settings_exports(&mut self, ui: &mut egui::Ui, folder_idx: usize) -> bool {
        let mut needs_save = false;
        let folder = self.state.folders.get(folder_idx);

        let subtitles = folder.and_then(|f| f.settings.subtitles).unwrap_or(false);
        let chapters = folder.and_then(|f| f.settings.chapters).unwrap_or(false);
        let captions = folder.and_then(|f| f.settings.captions).unwrap_or(false);
        let clips = folder.and_then(|f| f.settings.clips).unwrap_or(false);
        let preview = folder.and_then(|f| f.settings.preview).unwrap_or(false);
        let multi_format = folder.and_then(|f| f.settings.multi_format).unwrap_or(false);
        let extra_resolutions = folder.and_then(|f| f.settings.extra_resolutions.clone()).unwrap_or_default();
        let fcpxml = folder.and_then(|f| f.settings.fcpxml).unwrap_or(false);
        let edl = folder.and_then(|f| f.settings.edl).unwrap_or(false);
        let thumbnail = folder.and_then(|f| f.settings.thumbnail).unwrap_or(false);
        let filler_words = folder.and_then(|f| f.settings.filler_words).unwrap_or(false);
        let clip_count = folder.and_then(|f| f.settings.clip_count).unwrap_or(3);
        let clip_min_duration = folder.and_then(|f| f.settings.clip_min_duration).unwrap_or(15.0);
        let clip_max_duration = folder.and_then(|f| f.settings.clip_max_duration).unwrap_or(60.0);

        ui.label(section_title("Exports"));
        ui.add_space(4.0);
        ui.add(egui::Label::new(egui::RichText::new("Generate subtitles, chapters, clips, and additional formats").size(12.0).color(TEXT_SECONDARY)));
        ui.add_space(12.0);

        let mut subtitles = subtitles;
        if Self::draw_settings_toggle(
            ui,
            "SRT Subtitles",
            "Generate .srt subtitle file from transcript",
            &mut subtitles,
        ) && let Some(f) = self.state.folders.get_mut(folder_idx)
        {
            f.settings.subtitles = Some(subtitles);
            needs_save = true;
        }
        ui.add_space(6.0);

        let mut chapters = chapters;
        if Self::draw_settings_toggle(
            ui,
            "YouTube Chapters",
            "Generate timestamped chapters from transcript",
            &mut chapters,
        ) && let Some(f) = self.state.folders.get_mut(folder_idx)
        {
            f.settings.chapters = Some(chapters);
            needs_save = true;
        }
        ui.add_space(6.0);

        let mut captions = captions;
        if Self::draw_settings_toggle(
            ui,
            "Burn Captions",
            "Embed styled subtitles directly in video",
            &mut captions,
        ) && let Some(f) = self.state.folders.get_mut(folder_idx)
        {
            f.settings.captions = Some(captions);
            needs_save = true;
        }
        ui.add_space(6.0);

        let mut fcpxml = fcpxml;
        if Self::draw_settings_toggle(
            ui,
            "FCPXML",
            "Final Cut Pro XML for DaVinci/Premiere",
            &mut fcpxml,
        ) && let Some(f) = self.state.folders.get_mut(folder_idx)
        {
            f.settings.fcpxml = Some(fcpxml);
            needs_save = true;
        }
        ui.add_space(6.0);

        let mut edl = edl;
        if Self::draw_settings_toggle(
            ui,
            "EDL",
            "Edit Decision List for Avid/Premiere",
            &mut edl,
        ) && let Some(f) = self.state.folders.get_mut(folder_idx)
        {
            f.settings.edl = Some(edl);
            needs_save = true;
        }
        ui.add_space(6.0);

        let mut thumbnail = thumbnail;
        if Self::draw_settings_toggle(
            ui,
            "Thumbnail",
            "Extract a thumbnail image from the video",
            &mut thumbnail,
        ) && let Some(f) = self.state.folders.get_mut(folder_idx)
        {
            f.settings.thumbnail = Some(thumbnail);
            needs_save = true;
        }
        ui.add_space(6.0);

        let mut filler_words = filler_words;
        if Self::draw_settings_toggle(
            ui,
            "Remove Filler Words",
            "Requires speech transcription. Cuts common filler words (um, uh, ah, er)",
            &mut filler_words,
        ) && let Some(f) = self.state.folders.get_mut(folder_idx)
        {
            f.settings.filler_words = Some(filler_words);
            needs_save = true;
        }
        ui.add_space(6.0);

        let mut preview = preview;
        if Self::draw_settings_toggle(
            ui,
            "Preview File",
            "Generate a short low-res preview alongside output",
            &mut preview,
        ) && let Some(f) = self.state.folders.get_mut(folder_idx)
        {
            f.settings.preview = Some(preview);
            needs_save = true;
        }
        ui.add_space(6.0);

        let mut multi_format = multi_format;
        if Self::draw_settings_toggle(
            ui,
            "Multi-Format Export",
            "Generate outputs at multiple resolutions",
            &mut multi_format,
        ) && let Some(f) = self.state.folders.get_mut(folder_idx)
        {
            f.settings.multi_format = Some(multi_format);
            needs_save = true;
        }

        if multi_format {
            ui.add_space(6.0);
            ui.label(label_secondary("Extra Resolutions"));
            ui.add_space(4.0);
            let mut current_resolutions = extra_resolutions;
            let available_resolutions: [(VideoResolution, &str); 4] = [
                (VideoResolution::Hd720p, "720p"),
                (VideoResolution::Qhd1440p, "1440p"),
                (VideoResolution::Uhd4k, "4K"),
                (VideoResolution::Vertical720p, "Vertical 720p"),
            ];
            ui.horizontal_wrapped(|ui| {
                for (res, label) in available_resolutions {
                    let is_selected = current_resolutions.contains(&res);
                    if ui.add(button_pill(is_selected, label)).clicked() {
                        if is_selected {
                            current_resolutions.retain(|r| *r != res);
                        } else {
                            current_resolutions.push(res);
                        }
                        if let Some(f) = self.state.folders.get_mut(folder_idx) {
                            f.settings.extra_resolutions = Some(current_resolutions.clone());
                            needs_save = true;
                        }
                    }
                    ui.add_space(4.0);
                }
            });
        }
        ui.add_space(12.0);

        let mut clips = clips;
        if Self::draw_settings_toggle(
            ui,
            "Extract Clips",
            "Pull highlight clips for Shorts, Reels, TikTok",
            &mut clips,
        ) && let Some(f) = self.state.folders.get_mut(folder_idx)
        {
            f.settings.clips = Some(clips);
            needs_save = true;
        }

        if clips {
            ui.add_space(6.0);

            let mut clip_count_f = clip_count as f32;
            let clip_count_label = format!("{} clips", clip_count);
            if Self::draw_advanced_slider(
                ui,
                "Clip Count",
                "Number of highlight clips to extract",
                &mut clip_count_f,
                1.0..=10.0,
                clip_count_label,
                1.0,
            ) && let Some(f) = self.state.folders.get_mut(folder_idx)
            {
                f.settings.clip_count = Some(clip_count_f as u32);
                needs_save = true;
            }
            ui.add_space(6.0);

            let mut new_clip_min = clip_min_duration;
            let min_label = format!("{:.0}s min", clip_min_duration);
            if Self::draw_advanced_slider(
                ui,
                "Min Clip Duration",
                "Clips shorter than this won't be extracted",
                &mut new_clip_min,
                5.0..=60.0,
                min_label,
                1.0,
            ) && let Some(f) = self.state.folders.get_mut(folder_idx)
            {
                f.settings.clip_min_duration = Some(new_clip_min);
                needs_save = true;
            }
            ui.add_space(6.0);

            let mut new_clip_max = clip_max_duration;
            let max_label = format!("{:.0}s max", clip_max_duration);
            if Self::draw_advanced_slider(
                ui,
                "Max Clip Duration",
                "Longest clip length to extract",
                &mut new_clip_max,
                new_clip_min.max(30.0)..=300.0,
                max_label,
                1.0,
            ) && let Some(f) = self.state.folders.get_mut(folder_idx)
            {
                f.settings.clip_max_duration = Some(new_clip_max);
                needs_save = true;
            }
        }

        needs_save
    }

    fn draw_settings_advanced(&mut self, ui: &mut egui::Ui, folder_idx: usize) -> bool {
        let mut needs_save = false;
        let folder = self.state.folders.get(folder_idx);

        let intro_path = folder.and_then(|f| f.settings.intro_path.clone());
        let outro_path = folder.and_then(|f| f.settings.outro_path.clone());

        ui.label(section_title("Advanced"));
        ui.add_space(4.0);
        ui.add(egui::Label::new(egui::RichText::new("Fine-tune silence detection, add intro/outro videos").size(12.0).color(TEXT_SECONDARY)));
        ui.add_space(12.0);

        ui.label(section_title("Intro / Outro"));
        ui.add_space(8.0);

        ui.label(label_secondary("Intro Video"));
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            let intro_label = intro_path
                .as_ref()
                .and_then(|p| p.file_name())
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "None".to_string());
            ui.label(label_muted(&intro_label));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.add(button_small("Choose...")).clicked()
                    && let Some(path) = FileDialog::new()
                        .add_filter("Video", &["mp4", "mov", "mkv", "avi"])
                        .pick_file()
                    && let Some(f) = self.state.folders.get_mut(folder_idx)
                {
                    f.settings.intro_path = Some(path);
                    needs_save = true;
                }
                if intro_path.is_some()
                    && ui.add(button_small("✕")).clicked()
                    && let Some(f) = self.state.folders.get_mut(folder_idx)
                {
                    f.settings.intro_path = None;
                    needs_save = true;
                }
            });
        });

        ui.add_space(8.0);

        ui.label(label_secondary("Outro Video"));
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            let outro_label = outro_path
                .as_ref()
                .and_then(|p| p.file_name())
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "None".to_string());
            ui.label(label_muted(&outro_label));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.add(button_small("Choose...")).clicked()
                    && let Some(path) = FileDialog::new()
                        .add_filter("Video", &["mp4", "mov", "mkv", "avi"])
                        .pick_file()
                    && let Some(f) = self.state.folders.get_mut(folder_idx)
                {
                    f.settings.outro_path = Some(path);
                    needs_save = true;
                }
                if outro_path.is_some()
                    && ui.add(button_small("✕")).clicked()
                    && let Some(f) = self.state.folders.get_mut(folder_idx)
                {
                    f.settings.outro_path = None;
                    needs_save = true;
                }
            });
        });

        ui.add_space(12.0);

        ui.horizontal(|ui| {
            ui.label(label_muted("Restore this folder's settings to defaults."));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.add(button_small("Reset to Defaults")).clicked()
                    && let Some(f) = self.state.folders.get_mut(folder_idx)
                {
                    f.settings = FolderSettings::default();
                    needs_save = true;
                    self.state.activity_log.push(ActivityEntry::simple(
                        format!("Reset folder {} to defaults", folder_idx + 1),
                        true,
                    ));
                }
            });
        });

        needs_save
    }
    pub(crate) fn draw_settings_toggle(
        ui: &mut egui::Ui,
        label: &str,
        help_text: &str,
        value: &mut bool,
    ) -> bool {
        let mut changed = false;
        settings_toggle_frame(*value).show(ui, |ui| {
            ui.horizontal(|ui| {
                let dot_color = if *value { ACCENT_PRIMARY } else { TEXT_MUTED };
                let (dot_rect, _) =
                    ui.allocate_exact_size(egui::vec2(8.0, 8.0), egui::Sense::hover());
                ui.painter()
                    .circle_filled(dot_rect.center(), 3.5, dot_color);
                ui.add_space(6.0);
                ui.label(RichText::new(label).color(TEXT_PRIMARY).size(15.0).strong());
                ui.add_space(8.0);
                ui.label(label_muted(help_text));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let switch_text = if *value { "ON" } else { "OFF" };
                    if ui.add(button_toggle(*value, switch_text)).clicked() {
                        *value = !*value;
                        changed = true;
                    }
                });
            });
        });
        changed
    }

    pub(crate) fn draw_advanced_slider(
        ui: &mut egui::Ui,
        title: &str,
        help_text: &str,
        value: &mut f32,
        range: std::ops::RangeInclusive<f32>,
        value_label: String,
        step: f32,
    ) -> bool {
        let mut changed = false;
        settings_toggle_frame(true).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(label_secondary(title));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    settings_value_badge(ui, &value_label);
                });
            });
            ui.add_space(8.0);
            if slider_glow(value, range, ui, step).changed() {
                changed = true;
            }
            ui.add_space(4.0);
            ui.label(label_muted(help_text));
        });
        changed
    }

    pub(crate) fn draw_validation_warning(ui: &mut egui::Ui, message: &str) {
        egui::Frame::NONE
            .fill(WARNING_BG)
            .corner_radius(6.0)
            .inner_margin(egui::vec2(10.0, 8.0))
            .stroke(egui::Stroke::new(1.0, WARNING))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("⚠")
                            .size(14.0)
                            .color(WARNING),
                    );
                    ui.add_space(6.0);
                    ui.label(
                        egui::RichText::new(message)
                            .size(13.0)
                            .color(WARNING),
                    );
                });
            });
    }

    pub(crate) fn draw_summary_card(&mut self, ui: &mut egui::Ui) {
        let new_entries = self
            .state
            .activity_log
            .len()
            .saturating_sub(self.state.last_seen_activity_len);
        if new_entries == 0 {
            return;
        }

        let success_count = self
            .state
            .activity_log
            .iter()
            .rev()
            .take(new_entries)
            .filter(|e| e.status == EntryStatus::Success)
            .count();
        let error_count = new_entries.saturating_sub(success_count);
        let has_errors = error_count > 0;

        let bg = if has_errors { WARNING_BG } else { SUCCESS_BG };
        let accent = if has_errors { WARNING } else { SUCCESS };
        let dim = if has_errors { WARNING } else { SUCCESS_DIM };

        egui::Frame::NONE
            .fill(bg)
            .corner_radius(10.0)
            .stroke(egui::Stroke::new(1.0, accent))
            .inner_margin(14.0)
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.set_width(ui.available_width());
                    let dot_size = 10.0;
                    let (rect, _) = ui
                        .allocate_exact_size(egui::vec2(dot_size, dot_size), egui::Sense::hover());
                    ui.painter().circle_filled(rect.center(), 4.5, accent);

                    ui.add_space(10.0);

                    let label = if has_errors {
                        format!(
                            "{} new — {} done, {} failed",
                            new_entries, success_count, error_count
                        )
                    } else {
                        format!("{} new — {} completed", new_entries, success_count)
                    };
                    ui.label(
                        egui::RichText::new(label)
                            .size(14.0)
                            .color(TEXT_PRIMARY)
                            .strong(),
                    );

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if let Some(last) = self
                            .state
                            .activity_log
                            .iter()
                            .rev()
                            .find(|e| !e.filename.is_empty())
                        {
                            ui.label(
                                egui::RichText::new(truncate_path(&last.filename, 36))
                                    .size(12.0)
                                    .color(TEXT_MUTED),
                            );
                        }
                    });
                });

                ui.add_space(4.0);

                ui.horizontal_wrapped(|ui| {
                    ui.set_width(ui.available_width());
                    ui.add_space(14.0);
                    if has_errors {
                        ui.label(
                            egui::RichText::new(
                                "Some files failed — check the Activity tab for details",
                            )
                            .size(12.0)
                            .color(WARNING),
                        );
                    } else {
                        ui.label(
                            egui::RichText::new("All files processed successfully")
                                .size(12.0)
                                .color(dim),
                        );
                    }
                });
            });

        self.state.last_seen_activity_len = self.state.activity_log.len();
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
            };

            let toast_id = egui::Id::new(format!("toast_{}", i));
            let response = egui::Area::new(toast_id)
                .anchor(egui::Align2::RIGHT_BOTTOM, egui::vec2(-20.0, -20.0 - stack_y))
                .interactable(true)
                .show(ctx, |ui| {
                    egui::Frame::NONE
                        .fill(bg_color)
                        .corner_radius(8.0)
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
                                ui.label(
                                    egui::RichText::new(icon)
                                        .size(16.0)
                                        .color(color),
                                );
                                ui.add_space(8.0);
                                ui.label(
                                    egui::RichText::new(&toast.message)
                                        .size(13.0)
                                        .color(TEXT_PRIMARY),
                                );
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if ui.add(egui::Button::new("×").small().frame(false)).clicked() {
                                        dismiss_indices.push(i);
                                    }
                                });
                            });

                            let remaining = 5.0 - elapsed;
                            if remaining > 0.0 && remaining < 2.0 {
                                ui.add_space(4.0);
                                let progress = remaining / 2.0;
                                ui.add(
                                    egui::ProgressBar::new(1.0 - progress)
                                        .fill(color)
                                        .corner_radius(2.0)
                                        .desired_width(ui.available_width()),
                                );
                            }
                        });
                });
            // Check if toast was clicked
            if response.response.interact(egui::Sense::click()).is_pointer_button_down_on() {
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

                egui::ScrollArea::vertical()
                    .max_height(if full_height { 800.0 } else { 200.0 })
                    .show(ui, |ui| {
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
                                    log_entry_error(
                                        ui,
                                        &entry.timestamp,
                                        &entry.filename,
                                        &entry.message,
                                    );
                                }
                            }
                        }
                    });
            }
        });
    }

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
                                let settings = folder
                                    .map(|f| f.settings.clone())
                                    .unwrap_or_default();
                                self.state.batch_queue.push(super::QueuedFile {
                                    path,
                                    output_dir,
                                    preset,
                                    settings,
                                    status: super::QueueStatus::Queued,
                                    progress: 0.0,
                                    output_path: None,
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
                        super::QueueStatus::Queued => (TEXT_MUTED, "Queued"),
                        super::QueueStatus::Processing => (PROCESSING, "Processing..."),
                        super::QueueStatus::Done => (SUCCESS, "Done"),
                        super::QueueStatus::Error => (ERROR, "Error"),
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
                                    if file.status == super::QueueStatus::Queued
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

                        if file.status == super::QueueStatus::Processing {
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
                            .retain(|f| f.status != super::QueueStatus::Done);
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let done = self
                            .state
                            .batch_queue
                            .iter()
                            .filter(|f| f.status == super::QueueStatus::Done)
                            .count();
                        let total = self.state.batch_queue.len();
                        ui.label(label_muted(&format!("{} / {} done", done, total)));
                    });
                });
            }
        });
    }

    fn start_queue_processing(&mut self) {
        let queue: Vec<super::QueuedFile> = self
            .state
            .batch_queue
            .iter()
            .filter(|f| matches!(f.status, super::QueueStatus::Queued))
            .cloned()
            .collect();

        if queue.is_empty() {
            self.state.queue_processing = false;
            return;
        }

        let (tx, rx) = mpsc::channel();
        let stop =
            super::processing::spawn_queue_worker(self.state.config.clone(), queue, tx, true);

        self.state.queue_rx = Some(rx);
        self.state.queue_stop = Some(stop);
        self.state.queue_processing = true;
    }
}
