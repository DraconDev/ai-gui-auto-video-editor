use eframe::egui;
use egui::RichText;
use rfd::FileDialog;
use std::path::PathBuf;

use super::super::{ActivityEntry, App, FolderState, SetupStep};
use crate::config::{FolderSettings, SilenceMode};
use crate::gui::theme::*;

impl App {
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
                    .rect_filled(screen_rect, 0.0, PANEL_BG);
            });

        // Center the wizard with border and shadow
        egui::Area::new(egui::Id::new("setup_wizard"))
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                egui::Frame::NONE
                    .fill(PANEL_BG)
                    .corner_radius(0.0)
                    .inner_margin(0.0)
                    .stroke(egui::Stroke::new(1.0, BORDER_LIGHT))
                    .shadow(egui::epaint::Shadow {
                        offset: [0, 12],
                        blur: 48,
                        spread: 0,
                        color: egui::Color32::from_black_alpha(120),
                    })
                    .show(ui, |ui| {
                        ui.set_min_width(540.0);
                        ui.set_max_width(540.0);

                        // Accent bar at top of wizard
                        accent_bar().show(ui, |_ui| {});

                        // Step indicator
                        let steps = [
                            (SetupStep::Welcome, "Welcome"),
                            (SetupStep::ChooseFolder, "Folder"),
                            (SetupStep::ProcessingOptions, "Options"),
                            (SetupStep::Complete, "Done"),
                        ];
                        let current_idx = match self.state.setup_step {
                            SetupStep::Welcome => 0,
                            SetupStep::ChooseFolder => 1,
                            SetupStep::ProcessingOptions => 2,
                            SetupStep::Complete => 3,
                        };

                        egui::Frame::NONE
                            .fill(PANEL_BG_LIGHT)
                            .inner_margin(egui::vec2(32.0, 20.0))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    for (i, (_, label)) in steps.iter().enumerate() {
                                        if i > 0 {
                                            // Connector line
                                            let (rect, _) = ui.allocate_exact_size(
                                                egui::vec2(40.0, 2.0),
                                                egui::Sense::hover(),
                                            );
                                            let line_color = if i <= current_idx {
                                                ACCENT_PRIMARY
                                            } else {
                                                BORDER
                                            };
                                            ui.painter().rect_filled(
                                                rect,
                                                0.0,
                                                line_color,
                                            );
                                        }

                                        let is_done = i < current_idx;
                                        let is_current = i == current_idx;
                                        let dot_color = if is_done || is_current {
                                            ACCENT_PRIMARY
                                        } else {
                                            TEXT_MUTED
                                        };
                                        let text_color = if is_current {
                                            TEXT_PRIMARY
                                        } else if is_done {
                                            ACCENT_PRIMARY
                                        } else {
                                            TEXT_MUTED
                                        };

                                        ui.vertical(|ui| {
                                            ui.horizontal_centered(|ui| {
                                                let (rect, _) = ui.allocate_exact_size(
                                                    egui::vec2(12.0, 12.0),
                                                    egui::Sense::hover(),
                                                );
                                                ui.painter().circle_filled(
                                                    rect.center(),
                                                    if is_done || is_current { 5.0 } else { 3.5 },
                                                    dot_color,
                                                );
                                                ui.add_space(8.0);
                                                ui.label(
                                                    egui::RichText::new(*label)
                                                        .size(12.0)
                                                        .color(text_color)
                                                        .strong(),
                                                );
                                            });
                                        });
                                    }
                                });
                            });

                        // Content area — no outer padding; each step manages its own
                        egui::Frame::NONE
                            .fill(PANEL_BG)
                            .inner_margin(0.0)
                            .show(ui, |ui| {
                                match self.state.setup_step {
                                    SetupStep::Welcome => self.draw_setup_welcome(ui),
                                    SetupStep::ChooseFolder => self.draw_setup_folder(ui),
                                    SetupStep::ProcessingOptions => self.draw_setup_options(ui),
                                    SetupStep::Complete => self.draw_setup_complete(ui),
                                }
                            });
                    });
            });
    }

    pub(crate) fn draw_setup_welcome(&mut self, ui: &mut egui::Ui) {
        // Full-width hero section
        egui::Frame::NONE
            .fill(PANEL_BG_LIGHT)
            .inner_margin(egui::vec2(40.0, 32.0))
            .show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.label(
                        RichText::new("Welcome to AI Video Editor")
                            .size(28.0)
                            .color(ACCENT_PRIMARY)
                            .strong(),
                    );
                    ui.add_space(12.0);
                    ui.label(
                        RichText::new("Let's get you set up in just a few clicks.")
                            .size(16.0)
                            .color(TEXT_SECONDARY),
                    );
                });
            });

        // Padded content below
        egui::Frame::NONE
            .fill(PANEL_BG)
            .inner_margin(egui::vec2(40.0, 24.0))
            .show(ui, |ui| {
                // Feature highlights
                ui.vertical(|ui| {
                    self.setup_feature_row(
                        ui,
                        "Auto-remove silence",
                        "Cuts dead air automatically",
                    );
                    ui.add_space(10.0);
                    self.setup_feature_row(
                        ui,
                        "Audio enhancement",
                        "Makes your voice sound professional",
                    );
                    ui.add_space(10.0);
                    self.setup_feature_row(
                        ui,
                        "Auto-reframe",
                        "Convert to vertical video for Shorts/Reels",
                    );
                });

                ui.add_space(24.0);

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
            .corner_radius(0.0)
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
        egui::Frame::NONE
            .fill(PANEL_BG)
            .inner_margin(egui::vec2(40.0, 32.0))
            .show(ui, |ui| {
                self.draw_setup_folder_inner(ui);
            });
    }

    fn draw_setup_folder_inner(&mut self, ui: &mut egui::Ui) {
        ui.label(
            RichText::new("Choose Your Video Folder")
                .size(20.0)
                .color(ACCENT_PRIMARY)
                .strong(),
        );
        ui.add_space(4.0);
        ui.label(
            RichText::new("Select where your raw videos are stored. We'll create an 'output' folder next to it.")
                .size(13.0)
                .color(TEXT_SECONDARY),
        );
        ui.add_space(16.0);

        // Folder path display
        egui::Frame::NONE
            .fill(PANEL_BG_LIGHT)
            .corner_radius(0.0)
            .inner_margin(egui::vec2(12.0, 10.0))
            .stroke(egui::Stroke::new(1.0, BORDER))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("📁")
                            .size(14.0)
                            .color(TEXT_MUTED),
                    );
                    ui.add_space(8.0);
                    ui.label(
                        RichText::new(self.state.setup_folder.to_string_lossy().as_ref())
                            .size(13.0)
                            .color(TEXT_PRIMARY),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.add(button_small("Browse...")).clicked()
                            && let Some(path) = FileDialog::new().pick_folder()
                        {
                            self.state.setup_folder = path;
                        }
                    });
                });
            });

        ui.add_space(16.0);

        // Preset selection — YouTube full-width, then 2x3 grid
        ui.label(
            RichText::new("Content Type")
                .size(14.0)
                .color(TEXT_PRIMARY)
                .strong(),
        );
        ui.add_space(8.0);

        // YouTube — full-width top row
        {
            let selected = self.state.setup_preset == "youtube";
            let btn = button_pill(selected, "🎬 YouTube")
                .min_size(egui::vec2(ui.available_width(), 40.0));
            if ui.add(btn).clicked() {
                let old_preset = self.state.setup_preset.clone();
                if Self::is_default_setup_path(&self.state.setup_folder, &old_preset) {
                    self.state.setup_folder = self.state.setup_folder
                        .parent()
                        .map(|p| p.join("youtube"))
                        .unwrap_or_else(|| PathBuf::from("videos/youtube"));
                }
                self.state.setup_preset = "youtube".to_string();
            }
        }

        ui.add_space(8.0);

        // 2x3 grid for other presets
        let grid_presets: &[&[(&str, &str)]] = &[
            &[("shorts", "📱 Shorts"), ("podcast", "🎵 Podcast"), ("tiktok", "📱 TikTok")],
            &[("reels", "📸 Reels"), ("twitter", "🐦 Twitter"), ("minimal", "⚙ Minimal")],
        ];

        for row in grid_presets {
            let cols = row.len();
            let spacing = 8.0;
            let total_spacing = spacing * (cols as f32 - 1.0);
            let pill_width = (ui.available_width() - total_spacing) / cols as f32;

            ui.horizontal(|ui| {
                for &(preset, label) in *row {
                    let selected = self.state.setup_preset == preset;
                    let btn = button_pill(selected, label)
                        .min_size(egui::vec2(pill_width, 36.0));
                    if ui.add(btn).clicked() {
                        let old_preset = self.state.setup_preset.clone();
                        if Self::is_default_setup_path(&self.state.setup_folder, &old_preset) {
                            self.state.setup_folder = self.state.setup_folder
                                .parent()
                                .map(|p| p.join(preset))
                                .unwrap_or_else(|| PathBuf::from(format!("videos/{}", preset)));
                        }
                        self.state.setup_preset = preset.to_string();
                    }
                }
            });
            ui.add_space(6.0);
        }

        // Show description of selected preset
        let selected_desc = match self.state.setup_preset.as_str() {
            "youtube" => "Landscape video with silence removal & audio enhancement",
            "shorts" => "Vertical 9:16 with auto-reframe and face tracking",
            "podcast" => "Audio-focused with noise reduction and loudness normalization",
            "tiktok" => "Vertical format optimized for TikTok",
            "reels" => "Vertical format optimized for Instagram Reels",
            "twitter" => "Compressed landscape for Twitter/X video posts",
            "minimal" => "No processing — just copy/mux the input",
            _ => "",
        };
        ui.add_space(6.0);
        ui.label(
            egui::RichText::new(selected_desc)
                .size(12.0)
                .color(TEXT_SECONDARY),
        );

        ui.add_space(24.0);

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

    pub(crate) fn draw_setup_options(&mut self, ui: &mut egui::Ui) {
        egui::Frame::NONE
            .fill(PANEL_BG)
            .inner_margin(egui::vec2(40.0, 32.0))
            .show(ui, |ui| {
                self.draw_setup_options_inner(ui);
            });
    }

    fn draw_setup_options_inner(&mut self, ui: &mut egui::Ui) {
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
        egui::Frame::NONE
            .fill(PANEL_BG)
            .inner_margin(egui::vec2(40.0, 32.0))
            .show(ui, |ui| {
                self.draw_setup_complete_inner(ui);
            });
    }

    fn draw_setup_complete_inner(&mut self, ui: &mut egui::Ui) {
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
                .corner_radius(0.0)
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
                                "✂ Silence removal: {}",
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
                silence_mode: Some(if self.state.setup_remove_silence {
                    SilenceMode::Cut
                } else {
                    SilenceMode::Keep
                }),
                ..Default::default()
            },
        };

        self.state.folders.push(folder);
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
                    ui.set_min_width(460.0);
                    ui.set_max_width(460.0);

                    ui.label(label_secondary("Input Folder"));
                    ui.add_space(3.0);
                    ui.horizontal(|ui| {
                        let mut input_str = self.state.modal.input.to_string_lossy().to_string();
                        ui.add_sized(egui::vec2(360.0, 40.0), text_edit_style(&mut input_str));
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
                        ui.add_sized(egui::vec2(360.0, 40.0), text_edit_style(&mut output_str));
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
                        for preset in &[
                            "youtube", "shorts", "podcast", "tiktok", "reels", "twitter", "minimal",
                        ] {
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

    /// Check if the setup folder path looks like a default (so we update it on preset switch)
    pub(crate) fn is_default_setup_path(path: &std::path::Path, preset: &str) -> bool {
        let filename = path.file_name().map(|n| n.to_string_lossy().to_string());
        // It's a default path if the folder name matches the current preset
        // or is "Videos" (the initial parent default)
        // or the path itself is the Videos root (user hasn't customized)
        filename.as_deref() == Some(preset)
            || filename.as_deref() == Some("Videos")
            || Self::is_default_path(path, preset)
            || path.ends_with("Videos")
    }
}
