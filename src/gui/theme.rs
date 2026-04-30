#![cfg(feature = "gui")]

use eframe::egui;

pub const PANEL_BG: egui::Color32 = egui::Color32::from_rgb(14, 14, 16);
pub const PANEL_BG_LIGHT: egui::Color32 = egui::Color32::from_rgb(22, 22, 26);
pub const PANEL_BG_LIGHTER: egui::Color32 = egui::Color32::from_rgb(32, 32, 38);

pub const ACCENT_PRIMARY: egui::Color32 = egui::Color32::from_rgb(230, 57, 70);
pub const ACCENT_DARK: egui::Color32 = egui::Color32::from_rgb(180, 45, 55);
#[allow(dead_code)]
pub fn accent_glow() -> egui::Color32 {
    egui::Color32::from_rgba_premultiplied(230, 57, 70, 40)
}

pub const TEXT_PRIMARY: egui::Color32 = egui::Color32::from_rgb(250, 250, 252);
pub const TEXT_SECONDARY: egui::Color32 = egui::Color32::from_rgb(165, 165, 172);
pub const TEXT_MUTED: egui::Color32 = egui::Color32::from_rgb(90, 90, 100);

pub const BORDER: egui::Color32 = egui::Color32::from_rgb(32, 32, 38);
pub const BORDER_LIGHT: egui::Color32 = egui::Color32::from_rgb(48, 48, 56);

pub const SUCCESS: egui::Color32 = egui::Color32::from_rgb(72, 200, 120);
pub const SUCCESS_BG: egui::Color32 = egui::Color32::from_rgb(18, 40, 26);
pub const SUCCESS_DIM: egui::Color32 = egui::Color32::from_rgb(40, 140, 80);
pub const ERROR: egui::Color32 = egui::Color32::from_rgb(255, 75, 75);
pub const ERROR_BG: egui::Color32 = egui::Color32::from_rgb(45, 16, 16);
pub const WARNING: egui::Color32 = egui::Color32::from_rgb(255, 193, 7);
pub const WARNING_BG: egui::Color32 = egui::Color32::from_rgb(50, 40, 10);
pub const PROCESSING: egui::Color32 = egui::Color32::from_rgb(86, 160, 255);
pub const PROCESSING_BG: egui::Color32 = egui::Color32::from_rgb(20, 36, 60);
pub const PROCESSING_DIM: egui::Color32 = egui::Color32::from_rgb(60, 120, 200);
pub const SETTINGS_PANEL_BG: egui::Color32 = egui::Color32::from_rgb(12, 14, 18);
pub const SETTINGS_SECTION_BG: egui::Color32 = egui::Color32::from_rgb(18, 21, 28);
pub const SETTINGS_SECTION_BG_HIGHLIGHT: egui::Color32 = egui::Color32::from_rgb(30, 20, 26);
pub const SETTINGS_SECTION_BORDER_HIGHLIGHT: egui::Color32 = egui::Color32::from_rgb(100, 50, 60);

pub const CORNER_RADIUS: f32 = 14.0;
pub const CORNER_RADIUS_SMALL: f32 = 8.0;
#[allow(dead_code)]
pub const CORNER_RADIUS_PILL: f32 = 24.0;

#[allow(dead_code)]
pub fn glow_color() -> egui::Color32 {
    egui::Color32::from_rgba_unmultiplied(230, 57, 70, 80)
}

pub fn accent_bar() -> egui::Frame {
    egui::Frame::NONE
        .fill(ACCENT_PRIMARY)
        .inner_margin(egui::vec2(0.0, 3.0))
}

pub fn panel_frame() -> egui::Frame {
    egui::Frame::NONE
        .fill(PANEL_BG)
        .corner_radius(CORNER_RADIUS)
        .inner_margin(20.0)
        .stroke(egui::Stroke::new(1.0, BORDER))
}

pub fn settings_panel_frame() -> egui::Frame {
    egui::Frame::NONE
        .fill(SETTINGS_PANEL_BG)
        .corner_radius(CORNER_RADIUS)
        .inner_margin(22.0)
        .stroke(egui::Stroke::new(1.0, BORDER_LIGHT))
        .shadow(egui::epaint::Shadow {
            offset: [0, 6],
            blur: 20,
            spread: 0,
            color: egui::Color32::from_black_alpha(100),
        })
}

pub fn settings_section_frame(highlight: bool) -> egui::Frame {
    let (bg, border) = if highlight {
        (
            SETTINGS_SECTION_BG_HIGHLIGHT,
            SETTINGS_SECTION_BORDER_HIGHLIGHT,
        )
    } else {
        (SETTINGS_SECTION_BG, BORDER_LIGHT)
    };
    egui::Frame::NONE
        .fill(bg)
        .corner_radius(CORNER_RADIUS_SMALL)
        .inner_margin(14.0)
        .stroke(egui::Stroke::new(1.0, border))
}

pub fn settings_toggle_frame(enabled: bool) -> egui::Frame {
    let (border, bg) = if enabled {
        (ACCENT_DARK, egui::Color32::from_rgb(40, 26, 32))
    } else {
        (BORDER_LIGHT, egui::Color32::from_rgb(22, 26, 36))
    };
    egui::Frame::NONE
        .fill(bg)
        .corner_radius(CORNER_RADIUS_SMALL)
        .inner_margin(egui::vec2(12.0, 10.0))
        .stroke(egui::Stroke::new(1.0, border))
}

pub fn inner_panel() -> egui::Frame {
    egui::Frame::NONE
        .fill(PANEL_BG_LIGHT)
        .corner_radius(CORNER_RADIUS_SMALL)
        .inner_margin(14.0)
        .stroke(egui::Stroke::new(1.0, BORDER))
}

pub fn card_frame(bg: egui::Color32) -> egui::Frame {
    egui::Frame::NONE
        .fill(bg)
        .corner_radius(CORNER_RADIUS_SMALL)
        .inner_margin(egui::vec2(10.0, 6.0))
        .stroke(egui::Stroke::new(1.0, BORDER))
}

pub fn folder_card_compact(enabled: bool) -> egui::Frame {
    let bg = if enabled {
        PANEL_BG_LIGHTER
    } else {
        PANEL_BG_LIGHT
    };
    let border = if enabled { BORDER_LIGHT } else { BORDER };
    egui::Frame::NONE
        .fill(bg)
        .corner_radius(CORNER_RADIUS_SMALL)
        .inner_margin(egui::vec2(14.0, 10.0))
        .stroke(egui::Stroke::new(1.0, border))
}

#[allow(dead_code)]
pub fn section_header(text: &str) -> egui::RichText {
    egui::RichText::new(text)
        .size(13.0)
        .color(TEXT_SECONDARY)
        .strong()
}

#[allow(dead_code)]
pub fn section_title(text: &str) -> egui::RichText {
    egui::RichText::new(text)
        .size(16.0)
        .color(TEXT_PRIMARY)
        .strong()
}

pub fn label_primary(text: &str) -> egui::RichText {
    egui::RichText::new(text).color(TEXT_PRIMARY).size(16.0)
}

pub fn label_secondary(text: &str) -> egui::RichText {
    egui::RichText::new(text).color(TEXT_SECONDARY).size(15.0)
}

pub fn label_muted(text: &str) -> egui::RichText {
    egui::RichText::new(text).color(TEXT_MUTED).size(14.0)
}

pub fn text_edit_style(text: &mut String) -> egui::TextEdit<'_> {
    egui::TextEdit::singleline(text)
        .text_color(TEXT_PRIMARY)
        .background_color(PANEL_BG_LIGHTER)
        .desired_width(f32::INFINITY)
        .cursor_at_end(true)
        .min_size(egui::vec2(0.0, 40.0))
        .vertical_align(egui::Align::Center)
}

pub fn button_secondary(text: impl Into<String>) -> egui::Button<'static> {
    egui::Button::new(egui::RichText::new(text).color(TEXT_PRIMARY).size(15.0))
        .fill(PANEL_BG_LIGHTER)
        .stroke(egui::Stroke::new(1.0, BORDER_LIGHT))
        .corner_radius(CORNER_RADIUS_SMALL)
        .min_size(egui::vec2(80.0, 38.0))
}

pub fn button_small(text: impl Into<String>) -> egui::Button<'static> {
    egui::Button::new(egui::RichText::new(text).color(TEXT_SECONDARY).size(14.0))
        .fill(PANEL_BG_LIGHT)
        .stroke(egui::Stroke::new(1.0, BORDER))
        .corner_radius(CORNER_RADIUS_SMALL)
        .min_size(egui::vec2(60.0, 34.0))
}

#[allow(dead_code)]
pub fn button_icon(icon: &str, _tooltip: &str) -> egui::Button<'static> {
    egui::Button::new(egui::RichText::new(icon).size(16.0))
        .fill(PANEL_BG_LIGHTER)
        .stroke(egui::Stroke::new(1.0, BORDER_LIGHT))
        .corner_radius(CORNER_RADIUS_SMALL)
        .min_size(egui::vec2(36.0, 36.0))
        .sense(egui::Sense::click())
}

#[allow(dead_code)]
pub fn button_primary(text: impl Into<String>) -> egui::Button<'static> {
    egui::Button::new(
        egui::RichText::new(text)
            .color(egui::Color32::WHITE)
            .size(17.0)
            .strong(),
    )
    .fill(ACCENT_PRIMARY)
    .stroke(egui::Stroke::new(2.0, ACCENT_DARK))
    .corner_radius(CORNER_RADIUS_PILL)
    .min_size(egui::vec2(180.0, 52.0))
}

#[allow(dead_code)]
pub fn button_danger(text: impl Into<String>) -> egui::Button<'static> {
    egui::Button::new(egui::RichText::new(text).color(ERROR).size(15.0).strong())
        .fill(ERROR_BG)
        .stroke(egui::Stroke::new(1.0, ERROR))
        .corner_radius(CORNER_RADIUS_SMALL)
        .min_size(egui::vec2(80.0, 38.0))
}

pub fn button_toggle(is_active: bool, text: impl Into<String>) -> egui::Button<'static> {
    let btn = if is_active {
        egui::Button::new(
            egui::RichText::new(text)
                .color(TEXT_PRIMARY)
                .size(13.0)
                .strong(),
        )
        .fill(ACCENT_PRIMARY)
        .stroke(egui::Stroke::new(1.0, ACCENT_PRIMARY))
    } else {
        egui::Button::new(egui::RichText::new(text).color(TEXT_SECONDARY).size(13.0))
            .fill(PANEL_BG_LIGHT)
            .stroke(egui::Stroke::new(1.0, BORDER))
    };
    btn.corner_radius(CORNER_RADIUS_SMALL)
        .min_size(egui::vec2(55.0, 30.0))
}

pub fn button_tab(is_active: bool, text: impl Into<String>) -> egui::Button<'static> {
    let btn = if is_active {
        egui::Button::new(
            egui::RichText::new(text)
                .color(TEXT_PRIMARY)
                .size(15.0)
                .strong(),
        )
        .fill(PANEL_BG_LIGHTER)
        .stroke(egui::Stroke::new(0.0, egui::Color32::TRANSPARENT))
    } else {
        egui::Button::new(egui::RichText::new(text).color(TEXT_SECONDARY).size(15.0))
            .fill(PANEL_BG)
            .stroke(egui::Stroke::new(0.0, egui::Color32::TRANSPARENT))
    };
    btn.corner_radius(CORNER_RADIUS_SMALL)
        .min_size(egui::vec2(85.0, 36.0))
}

pub fn button_pill(is_active: bool, text: impl Into<String>) -> egui::Button<'static> {
    let btn = if is_active {
        egui::Button::new(
            egui::RichText::new(text)
                .color(TEXT_PRIMARY)
                .size(14.0)
                .strong(),
        )
        .fill(ACCENT_PRIMARY)
        .stroke(egui::Stroke::new(1.0, ACCENT_DARK))
    } else {
        egui::Button::new(egui::RichText::new(text).color(TEXT_SECONDARY).size(14.0))
            .fill(PANEL_BG_LIGHT)
            .stroke(egui::Stroke::new(1.0, BORDER))
    };
    btn.corner_radius(16.0).min_size(egui::vec2(60.0, 32.0))
}

pub fn button_add(text: impl Into<String>) -> egui::Button<'static> {
    egui::Button::new(egui::RichText::new(text).color(ACCENT_PRIMARY).size(14.0))
        .fill(PANEL_BG)
        .stroke(egui::Stroke::new(1.0, ACCENT_PRIMARY))
        .corner_radius(CORNER_RADIUS_SMALL)
        .min_size(egui::vec2(65.0, 32.0))
}

pub fn dropdown_selector<T: PartialEq + Copy>(
    ui: &mut egui::Ui,
    id: &str,
    selected: &mut T,
    options: &[(String, T)],
    current_label: &str,
) {
    let popup_id = egui::Id::new(format!("{}_popup", id));
    let is_popup_open = ui.data(|d| d.get_temp::<bool>(popup_id).unwrap_or(false));

    let chevron = if is_popup_open { "▲" } else { "▼" };
    let desired_width = 220.0_f32;

    let button = egui::Button::new(
        egui::RichText::new(format!("{}  {}", current_label, chevron))
            .size(14.0)
            .color(if is_popup_open {
                TEXT_PRIMARY
            } else {
                TEXT_SECONDARY
            }),
    )
    .fill(if is_popup_open {
        PANEL_BG_LIGHTER
    } else {
        PANEL_BG_LIGHT
    })
    .corner_radius(CORNER_RADIUS_SMALL)
    .stroke(egui::Stroke::new(
        1.0,
        if is_popup_open {
            ACCENT_PRIMARY
        } else {
            BORDER_LIGHT
        },
    ))
    .min_size(egui::vec2(desired_width, 36.0));

    let response = ui.add(button);

    if response.clicked() {
        let new_state = !is_popup_open;
        ui.data_mut(|d| d.insert_temp(popup_id, new_state));
    }

    if is_popup_open {
        let button_bottom = response.rect.max.y + 4.0;
        let popup_height = (options.len() as f32 * 36.0).min(200.0);
        let popup_width = desired_width;

        egui::Area::new(popup_id)
            .order(egui::Order::Foreground)
            .fixed_pos(egui::pos2(response.rect.min.x, button_bottom))
            .show(ui.ctx(), |ui| {
                let popup_rect = egui::Rect::from_min_size(
                    egui::pos2(response.rect.min.x, button_bottom),
                    egui::vec2(popup_width, popup_height),
                );
                ui.set_clip_rect(popup_rect);

                egui::Frame::NONE
                    .fill(PANEL_BG_LIGHTER)
                    .corner_radius(CORNER_RADIUS_SMALL)
                    .stroke(egui::Stroke::new(1.0, BORDER_LIGHT))
                    .inner_margin(egui::vec2(4.0, 4.0))
                    .show(ui, |ui| {
                        ui.set_min_size(popup_rect.size());
                        for (label, value) in options {
                            let is_selected = *value == *selected;
                            let item_bg = if is_selected {
                                egui::Color32::from_rgb(42, 18, 26)
                            } else if ui.input(|i| i.pointer.any_down()) {
                                egui::Color32::from_rgb(38, 38, 46)
                            } else {
                                PANEL_BG_LIGHTER
                            };

                            let item_response = egui::Frame::NONE
                                .fill(item_bg)
                                .corner_radius(CORNER_RADIUS_SMALL)
                                .inner_margin(egui::vec2(12.0, 10.0))
                                .show(ui, |ui| {
                                    ui.set_width(ui.available_width());
                                    ui.horizontal_wrapped(|ui| {
                                        ui.set_width(ui.available_width());
                                        ui.label(
                                            egui::RichText::new(label.as_str()).size(14.0).color(
                                                if is_selected {
                                                    ACCENT_PRIMARY
                                                } else {
                                                    TEXT_PRIMARY
                                                },
                                            ),
                                        );
                                        if is_selected {
                                            ui.with_layout(
                                                egui::Layout::right_to_left(egui::Align::Center),
                                                |ui| {
                                                    ui.label(
                                                        egui::RichText::new("✓")
                                                            .size(13.0)
                                                            .color(ACCENT_PRIMARY),
                                                    );
                                                },
                                            );
                                        }
                                    });
                                    ui.response()
                                });

                            if item_response.inner.clicked() {
                                *selected = *value;
                                ui.data_mut(|d| d.insert_temp(popup_id, false));
                            }
                        }
                    });
            });

        if ui.input(|i| i.pointer.any_pressed())
            && let Some(pos) = ui.input(|i| i.pointer.interact_pos())
            && !response.rect.contains(pos)
        {
            ui.data_mut(|d| d.insert_temp(popup_id, false));
        }
    }
}

pub fn slider_glow(
    value: &mut f32,
    range: std::ops::RangeInclusive<f32>,
    ui: &mut egui::Ui,
    step: f32,
) -> egui::Response {
    let spacing = ui.spacing();
    let slider_width = spacing.slider_width;
    let rail_height = 4.0;
    let handle_radius = 7.0;

    let available_width = ui.available_width();
    let width = slider_width.min(available_width).max(100.0);

    let (rect, mut response) = ui.allocate_exact_size(
        egui::vec2(width, handle_radius * 2.0 + 4.0),
        egui::Sense::click_and_drag(),
    );

    let range_size = *range.end() - *range.start();
    let fraction = (*value - *range.start()) / range_size;

    let handle_x = egui::lerp(rect.left()..=rect.right(), fraction);
    let handle_center = egui::pos2(handle_x, rect.center().y);

    let track_rect = egui::Rect::from_min_size(
        egui::pos2(rect.left(), rect.center().y - rail_height / 2.0),
        egui::vec2(rect.width(), rail_height),
    );

    let painter = ui.painter();

    painter.rect_filled(track_rect, 2.0, egui::Color32::from_rgb(50, 50, 60));

    if fraction > 0.0 {
        let filled_width = (handle_x - rect.left()).max(0.0);
        let filled_rect =
            egui::Rect::from_min_size(track_rect.left_top(), egui::vec2(filled_width, rail_height));
        painter.rect_filled(filled_rect, 2.0, ACCENT_PRIMARY);
    }

    painter.circle_filled(handle_center, handle_radius, ACCENT_PRIMARY);
    painter.circle_filled(
        handle_center,
        handle_radius - 2.5,
        egui::Color32::from_rgb(255, 110, 125),
    );

    if response.clicked() || response.dragged() {
        let pointer_pos = ui.input(|i| i.pointer.interact_pos());
        if let Some(pos) = pointer_pos {
            let new_fraction = ((pos.x - rect.left()) / rect.width()).clamp(0.0, 1.0);
            let new_value = *range.start() + new_fraction * range_size;
            let stepped = (new_value / step).round() * step;
            *value = stepped.clamp(*range.start(), *range.end());
            response.mark_changed();
        }
    }

    let value_text = if step >= 1.0 {
        format!("{}", *value as i32)
    } else if step >= 0.1 {
        format!("{:.1}", *value)
    } else {
        format!("{:.2}", *value)
    };
    let text_color = TEXT_SECONDARY;
    let font_id = egui::FontId::proportional(13.0);
    let text_galley = painter.layout_no_wrap(value_text, font_id, text_color);
    let text_pos = egui::pos2(
        rect.right() + 12.0,
        rect.center().y - text_galley.size().y / 2.0,
    );
    painter.galley(text_pos, text_galley, text_color);

    response
}

pub fn modal_overlay() -> egui::Frame {
    egui::Frame::NONE.fill(egui::Color32::from_rgba_unmultiplied(0, 0, 0, 200))
}

pub fn modal_dialog() -> egui::Frame {
    egui::Frame::NONE
        .fill(PANEL_BG)
        .corner_radius(CORNER_RADIUS)
        .inner_margin(24.0)
        .stroke(egui::Stroke::new(1.0, BORDER_LIGHT))
        .shadow(egui::epaint::Shadow {
            offset: [0, 8],
            blur: 32,
            spread: 0,
            color: egui::Color32::from_black_alpha(150),
        })
}

pub fn preset_badge(preset: &str, ui: &mut egui::Ui) {
    let color = match preset {
        "youtube" => egui::Color32::from_rgb(230, 57, 70),
        "shorts" => egui::Color32::from_rgb(255, 140, 0),
        "podcast" => egui::Color32::from_rgb(100, 149, 237),
        "tiktok" => egui::Color32::from_rgb(0, 242, 234),
        "reels" => egui::Color32::from_rgb(225, 48, 108),
        "twitter" => egui::Color32::from_rgb(29, 161, 242),
        "minimal" => egui::Color32::from_rgb(150, 150, 150),
        _ => ACCENT_PRIMARY,
    };
    egui::Frame::NONE
        .fill(color)
        .corner_radius(4.0)
        .inner_margin(egui::vec2(12.0, 6.0))
        .show(ui, |ui| {
            ui.label(
                egui::RichText::new(preset)
                    .color(TEXT_PRIMARY)
                    .size(12.0)
                    .strong(),
            );
        });
}

pub fn settings_value_badge(ui: &mut egui::Ui, value: &str) {
    egui::Frame::NONE
        .fill(egui::Color32::from_rgb(42, 22, 28))
        .corner_radius(4.0)
        .inner_margin(egui::vec2(10.0, 5.0))
        .stroke(egui::Stroke::new(1.0, ACCENT_DARK))
        .show(ui, |ui| {
            ui.label(
                egui::RichText::new(value)
                    .color(TEXT_PRIMARY)
                    .size(12.0)
                    .strong(),
            );
        });
}

pub fn truncate_path(path: &str, max_len: usize) -> String {
    if path.len() <= max_len {
        path.to_string()
    } else if max_len < 7 {
        // Too short to show anything meaningful, just truncate
        path.chars().take(max_len).collect()
    } else {
        let start = &path[..max_len / 2 - 2];
        let end = &path[path.len() - max_len / 2 + 2..];
        format!("{}...{}", start, end)
    }
}

pub fn status_badge_with_bg(
    ui: &mut egui::Ui,
    status: &str,
    dot_color: egui::Color32,
    bg: egui::Color32,
) {
    egui::Frame::NONE
        .fill(bg)
        .corner_radius(CORNER_RADIUS_SMALL)
        .inner_margin(egui::vec2(14.0, 10.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                let (rect, _) =
                    ui.allocate_exact_size(egui::vec2(12.0, 12.0), egui::Sense::hover());
                ui.painter().circle_filled(rect.center(), 5.0, dot_color);
                ui.add_space(8.0);
                ui.label(
                    egui::RichText::new(status)
                        .color(dot_color)
                        .size(15.0)
                        .strong(),
                );
            });
        });
}

pub fn log_entry_success(
    ui: &mut egui::Ui,
    timestamp: &str,
    filename: &str,
    size: &str,
    duration: &str,
) {
    card_frame(SUCCESS_BG).show(ui, |ui| {
        ui.horizontal_wrapped(|ui| {
            ui.set_width(ui.available_width());
            let (rect, _) = ui.allocate_exact_size(egui::vec2(10.0, 10.0), egui::Sense::hover());
            ui.painter().circle_filled(rect.center(), 4.5, SUCCESS);
            ui.add_space(8.0);
            ui.label(label_muted(timestamp));
            ui.add_space(8.0);
            ui.label(label_primary(filename));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(label_muted(&format!("{} · {}", size, duration)));
            });
        });
    });
}

pub fn log_entry_processing(
    ui: &mut egui::Ui,
    timestamp: &str,
    filename: &str,
    message: &str,
    progress: f32,
) {
    card_frame(PROCESSING_BG).show(ui, |ui| {
        ui.horizontal_wrapped(|ui| {
            ui.set_width(ui.available_width());
            let (rect, _) = ui.allocate_exact_size(egui::vec2(10.0, 10.0), egui::Sense::hover());
            ui.painter().circle_filled(rect.center(), 4.5, PROCESSING);
            ui.add_space(8.0);
            ui.label(label_muted(timestamp));
            ui.add_space(8.0);
            ui.label(label_primary(filename));
        });
        ui.add_space(2.0);
        ui.horizontal_wrapped(|ui| {
            ui.set_width(ui.available_width());
            ui.add_space(18.0);
            ui.label(label_muted(message));
        });
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.add_space(18.0);
            ui.add(
                egui::ProgressBar::new(progress)
                    .text(format!("{:.0}%", progress * 100.0))
                    .fill(PROCESSING_DIM)
                    .corner_radius(4.0)
                    .desired_width(200.0),
            );
        });
    });
}

pub fn log_entry_error(ui: &mut egui::Ui, timestamp: &str, filename: &str, message: &str) {
    card_frame(ERROR_BG).show(ui, |ui| {
        ui.horizontal_wrapped(|ui| {
            ui.set_width(ui.available_width());
            let (rect, _) = ui.allocate_exact_size(egui::vec2(10.0, 10.0), egui::Sense::hover());
            ui.painter().circle_filled(rect.center(), 4.5, ERROR);
            ui.add_space(8.0);
            ui.label(label_muted(timestamp));
            ui.add_space(8.0);
            ui.label(label_primary(filename));
        });
        ui.add_space(2.0);
        ui.horizontal_wrapped(|ui| {
            ui.set_width(ui.available_width());
            ui.add_space(18.0);
            ui.label(egui::RichText::new(message).color(ERROR).size(13.0));
        });
    });
}

pub fn format_file_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.0} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.0} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

pub fn format_duration(seconds: u64) -> String {
    let mins = seconds / 60;
    let secs = seconds % 60;
    if mins > 0 {
        format!("{}m {}s", mins, secs)
    } else {
        format!("{}s", secs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_path_short() {
        let path = "/foo/bar.txt";
        assert_eq!(truncate_path(path, 50), path);
        assert_eq!(truncate_path(path, 100), path);
    }

    #[test]
    fn test_truncate_path_exact_length() {
        let path = "/foo/bar.txt";
        assert_eq!(truncate_path(path, path.len()), path);
    }

    #[test]
    fn test_truncate_path_long() {
        let path =
            "/very/long/path/that/needs/truncating/especially/when/it/is/really/very/long/file.txt";
        let result = truncate_path(path, 20);
        assert!(result.len() <= 20);
        assert!(result.contains("..."));
    }

    #[test]
    fn test_truncate_path_very_short() {
        let path = "/foo/bar.txt";
        let result = truncate_path(path, 3);
        assert!(result.len() <= 3);
    }

    #[test]
    fn test_truncate_path_empty() {
        assert_eq!(truncate_path("", 10), "");
        assert_eq!(truncate_path("", 0), "");
    }

    #[test]
    fn test_format_file_size_bytes() {
        assert_eq!(format_file_size(0), "0 B");
        assert_eq!(format_file_size(100), "100 B");
        assert_eq!(format_file_size(1023), "1023 B");
    }

    #[test]
    fn test_format_file_size_kb() {
        assert_eq!(format_file_size(1024), "1 KB");
        assert_eq!(format_file_size(1500), "1 KB");
        assert_eq!(format_file_size(1024 * 100), "100 KB");
    }

    #[test]
    fn test_format_file_size_mb() {
        assert_eq!(format_file_size(1024 * 1024), "1 MB");
        assert_eq!(format_file_size(1024 * 1024 * 50), "50 MB");
        assert_eq!(format_file_size(1024 * 1024 * 100), "100 MB");
    }

    #[test]
    fn test_format_file_size_gb() {
        assert_eq!(format_file_size(1024 * 1024 * 1024), "1.0 GB");
        assert_eq!(format_file_size(1024 * 1024 * 1024 * 2), "2.0 GB");
        assert_eq!(format_file_size(1024 * 1024 * 1024 * 1500), "1500.0 GB");
    }

    #[test]
    fn test_format_duration_seconds() {
        assert_eq!(format_duration(0), "0s");
        assert_eq!(format_duration(30), "30s");
        assert_eq!(format_duration(59), "59s");
    }

    #[test]
    fn test_format_duration_minutes() {
        assert_eq!(format_duration(60), "1m 0s");
        assert_eq!(format_duration(90), "1m 30s");
        assert_eq!(format_duration(125), "2m 5s");
    }

    #[test]
    fn test_format_duration_large() {
        assert_eq!(format_duration(3600), "60m 0s");
        assert_eq!(format_duration(3661), "61m 1s");
    }
}
