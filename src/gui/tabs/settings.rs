use eframe::egui;
use egui::RichText;
use rfd::FileDialog;

use super::super::{ActivityEntry, App, ProcessingStatus, SettingsCategory};
use crate::config::{FolderSettings, JoinMode, SilenceMode, VideoResolution};
use crate::gui::theme::*;
use crate::hwaccel::HwAccel;

impl App {
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
                        ui.label(label_muted(
                            "Add a folder in the Folders tab to configure settings",
                        ));
                    });
                    ui.add_space(20.0);
                });
                return;
            }

            // Category title — shows which settings section is active
            let category_title = match self.state.settings_category {
                SettingsCategory::Processing => "Processing Settings",
                SettingsCategory::Audio => "Audio Settings",
                SettingsCategory::Video => "Video Settings",
                SettingsCategory::Exports => "Export Settings",
                SettingsCategory::Advanced => "Advanced Settings",
            };
            ui.label(
                RichText::new(category_title)
                    .size(18.0)
                    .color(ACCENT_PRIMARY)
                    .strong(),
            );
            ui.add_space(8.0);

            let is_processing = self.state.queue_processing
                || matches!(self.state.status, ProcessingStatus::Processing(_));

            // Content area — full width (sidebar is in the main nav now)
            if is_processing {
                settings_section_frame(false).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("⚙").size(14.0));
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

            ui.add_space(20.0);
            ui.separator();
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                // Two-click confirmation: first click shows confirm text, second click runs
                let confirm_id = ui.id().with("setup_confirm");
                let mut confirm_shown = ui.data(|d| d.get_temp::<bool>(confirm_id).unwrap_or(false));
                if confirm_shown {
                    ui.label(egui::RichText::new("This will add a new folder (existing folders kept). Continue?")
                        .size(12.0)
                        .color(WARNING));
                    ui.add_space(8.0);
                    if ui.add(button_primary("Yes, Run Setup")).clicked() {
                        self.state.show_setup = true;
                        self.state.setup_step = super::SetupStep::Welcome;
                        confirm_shown = false;
                    }
                    if ui.add(button_small("Cancel")).clicked() {
                        confirm_shown = false;
                    }
                } else {
                    if ui.add(button_small("⚙ Re-run Setup Wizard")).clicked() {
                        confirm_shown = true;
                    }
                }
                ui.data_mut(|d| d.insert_temp(confirm_id, confirm_shown));
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
        let color_correct = folder
            .and_then(|f| f.settings.color_correct)
            .unwrap_or(false);
        let reframe = folder.and_then(|f| f.settings.reframe).unwrap_or(false);
        let blur = folder
            .and_then(|f| f.settings.blur_background)
            .unwrap_or(false);
        let scene_detect = folder
            .and_then(|f| f.settings.scene_detect)
            .unwrap_or(false);
        let threshold = folder
            .and_then(|f| f.settings.silence_threshold_db)
            .unwrap_or(-30.0);
        let silence_mode = folder
            .and_then(|f| f.settings.silence_mode)
            .unwrap_or(SilenceMode::Cut);
        let silence_padding = folder
            .and_then(|f| f.settings.silence_padding)
            .unwrap_or(0.1);
        let silence_speedup_factor = folder
            .and_then(|f| f.settings.silence_speedup_factor)
            .unwrap_or(4.0);
        let silence_min_duration = folder
            .and_then(|f| f.settings.silence_min_duration)
            .unwrap_or(0.5);
        let silence_min_for_speedup = folder
            .and_then(|f| f.settings.silence_min_silence_for_speedup)
            .unwrap_or(0.5);
        let silence_scene_threshold = folder
            .and_then(|f| f.settings.silence_scene_threshold)
            .unwrap_or(0.3);

        ui.label(section_title("Processing"));
        ui.add_space(4.0);
        ui.add(egui::Label::new(
            egui::RichText::new(
                "Core video editing: silence removal, stabilization, color correction",
            )
            .size(12.0)
            .color(TEXT_SECONDARY),
        ));
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
        dropdown_selector(
            ui,
            &format!("silence_mode_{}", folder_idx),
            &mut selected_mode,
            &mode_options,
            mode_label,
        );
        ui.add(egui::Label::new(
            egui::RichText::new(
                "Keep All = no changes. Cut = remove silence. Speed Up = keep but play faster",
            )
            .size(11.0)
            .color(TEXT_SECONDARY),
        ));
        if selected_mode != silence_mode
            && let Some(f) = self.state.folders.get_mut(folder_idx)
        {
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

        let enhance = folder
            .and_then(|f| f.settings.enhance_audio)
            .unwrap_or(true);
        let noise_reduction = folder
            .and_then(|f| f.settings.noise_reduction)
            .unwrap_or(false);
        let lufs = folder.and_then(|f| f.settings.target_lufs).unwrap_or(-14.0);
        let duck_volume = folder.and_then(|f| f.settings.duck_volume).unwrap_or(0.2);
        let music_path = folder.and_then(|f| f.settings.music_path.clone());

        ui.label(section_title("Audio"));
        ui.add_space(4.0);
        ui.add(egui::Label::new(
            egui::RichText::new("Audio enhancement, noise reduction, background music mixing")
                .size(12.0)
                .color(TEXT_SECONDARY),
        ));
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
            "Loudness in decibels relative to full scale - YouTube = -14 LUFS",
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

        let hw_accel = folder
            .and_then(|f| f.settings.hw_accel)
            .unwrap_or(HwAccel::None);
        let target_res = folder
            .and_then(|f| f.settings.target_resolution)
            .unwrap_or(VideoResolution::Fhd1080p);
        let watermark_path = folder.and_then(|f| f.settings.watermark_path.clone());
        let watermark_position = folder
            .and_then(|f| f.settings.watermark_position.clone())
            .unwrap_or_else(|| "bottom-right".to_string());
        let watermark_scale = folder
            .and_then(|f| f.settings.watermark_scale)
            .unwrap_or(1.0);

        ui.label(section_title("Video Output"));
        ui.add_space(4.0);
        ui.add(egui::Label::new(
            egui::RichText::new("Output resolution, GPU encoding, watermark overlay")
                .size(12.0)
                .color(TEXT_SECONDARY),
        ));
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
            (
                String::from("1080p Vertical"),
                VideoResolution::Vertical1080p,
            ),
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
                    && let Some(path) = FileDialog::new().add_filter("Image", &["png"]).pick_file()
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
        let multi_format = folder
            .and_then(|f| f.settings.multi_format)
            .unwrap_or(false);
        let extra_resolutions = folder
            .and_then(|f| f.settings.extra_resolutions.clone())
            .unwrap_or_default();
        let fcpxml = folder.and_then(|f| f.settings.fcpxml).unwrap_or(false);
        let edl = folder.and_then(|f| f.settings.edl).unwrap_or(false);
        let thumbnail = folder.and_then(|f| f.settings.thumbnail).unwrap_or(false);
        let filler_words = folder
            .and_then(|f| f.settings.filler_words)
            .unwrap_or(false);
        let clip_count = folder.and_then(|f| f.settings.clip_count).unwrap_or(3);
        let clip_min_duration = folder
            .and_then(|f| f.settings.clip_min_duration)
            .unwrap_or(15.0);
        let clip_max_duration = folder
            .and_then(|f| f.settings.clip_max_duration)
            .unwrap_or(60.0);

        ui.label(section_title("Exports"));
        ui.add_space(4.0);
        ui.add(egui::Label::new(
            egui::RichText::new("Generate subtitles, chapters, clips, and additional formats")
                .size(12.0)
                .color(TEXT_SECONDARY),
        ));
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
            "XML file for importing into Final Cut Pro, DaVinci Resolve, or Premiere Pro",
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
            "Edit list for importing into Avid Media Composer or other NLEs",
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
            let available_resolutions: [(VideoResolution, &str); 6] = [
                (VideoResolution::Hd720p, "720p"),
                (VideoResolution::Fhd1080p, "1080p"),
                (VideoResolution::Qhd1440p, "1440p"),
                (VideoResolution::Uhd4k, "4K"),
                (VideoResolution::Vertical720p, "Vertical 720p"),
                (VideoResolution::Vertical1080p, "Vertical 1080p"),
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
        let join_mode = folder
            .and_then(|f| f.settings.join_mode)
            .unwrap_or(JoinMode::Off);
        let join_after_count = folder
            .and_then(|f| f.settings.join_after_count)
            .unwrap_or(5);

        ui.label(section_title("Advanced"));
        ui.add_space(4.0);
        ui.add(egui::Label::new(
            egui::RichText::new("Fine-tune silence detection, add intro/outro videos")
                .size(12.0)
                .color(TEXT_SECONDARY),
        ));
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

        ui.label(section_title("Video Joining"));
        ui.add_space(8.0);

        ui.label(label_secondary("Join Mode"));
        ui.add_space(4.0);
        let mode_options: [(String, JoinMode); 4] = [
            (String::from("Off"), JoinMode::Off),
            (String::from("By Date"), JoinMode::ByDate),
            (String::from("By Name"), JoinMode::ByName),
            (String::from("After Count"), JoinMode::AfterCount),
        ];
        let mut selected_mode = join_mode;
        let mode_label = match selected_mode {
            JoinMode::Off => "Off",
            JoinMode::ByDate => "By Date",
            JoinMode::ByName => "By Name",
            JoinMode::AfterCount => "After Count",
        };
        dropdown_selector(
            ui,
            &format!("join_mode_{}", folder_idx),
            &mut selected_mode,
            &mode_options,
            mode_label,
        );
        ui.add(egui::Label::new(
            egui::RichText::new("Off = no joining. After Count = join every N videos.")
                .size(11.0)
                .color(TEXT_SECONDARY),
        ));
        if selected_mode != join_mode
            && let Some(f) = self.state.folders.get_mut(folder_idx)
        {
            f.settings.join_mode = Some(selected_mode);
            needs_save = true;
        }

        if selected_mode == JoinMode::AfterCount {
            ui.add_space(6.0);
            let mut count = join_after_count as f32;
            let count_label = format!("{} videos", join_after_count);
            if Self::draw_advanced_slider(
                ui,
                "Join After Count",
                "Join videos after this many are processed",
                &mut count,
                2.0..=20.0,
                count_label,
                1.0,
            ) && let Some(f) = self.state.folders.get_mut(folder_idx)
            {
                f.settings.join_after_count = Some(count as u32);
                needs_save = true;
            }
        }

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
                let label_widget =
                    ui.label(RichText::new(label).color(TEXT_PRIMARY).size(15.0).strong());
                label_widget.on_hover_text(help_text);
                ui.add_space(8.0);
                let help_widget = ui.label(label_muted(help_text));
                help_widget.on_hover_text(help_text);
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
}
