//! General application settings UI.
use crate::app::ReaderApp;
use eframe::egui;
use egui::{Color32, CornerRadius, Vec2};

impl ReaderApp {
    pub fn render_api_settings_panel(&mut self, ctx: &egui::Context) {
        let dark = self.dark_mode;
        let panel_bg = if dark {
            Color32::from_rgb(32, 32, 36)
        } else {
            Color32::from_rgb(248, 248, 252)
        };
        let heading_color = if dark {
            Color32::from_gray(220)
        } else {
            Color32::from_gray(30)
        };
        let subtitle_color = if dark {
            Color32::from_gray(140)
        } else {
            Color32::from_gray(100)
        };
        let accent = Color32::from_rgb(56, 132, 255);

        egui::SidePanel::right("api_settings_panel")
            .default_width(300.0)
            .max_width(360.0)
            .show(ctx, |ui| {
                egui::Frame::new()
                    .fill(panel_bg)
                    .inner_margin(16.0)
                    .show(ui, |ui| {
                        // Header
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(self.i18n.t("settings.api_title"))
                                    .size(18.0)
                                    .strong()
                                    .color(heading_color),
                            );
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.small_button("✕").clicked() {
                                    self.show_api_settings = false;
                                }
                            });
                        });
                        ui.separator();
                        ui.add_space(12.0);

                        // ── Translation API ──
                        ui.label(
                            egui::RichText::new(self.i18n.t("settings.translate_section"))
                                .size(15.0)
                                .strong()
                                .color(heading_color),
                        );
                        ui.add_space(4.0);

                        ui.label(
                            egui::RichText::new(self.i18n.t("settings.api_url"))
                                .size(12.0)
                                .color(subtitle_color),
                        );
                        ui.add(
                            egui::TextEdit::singleline(&mut self.translate_api_url)
                                .hint_text("https://api.example.com/translate")
                                .desired_width(f32::INFINITY),
                        );
                        ui.add_space(4.0);

                        ui.label(
                            egui::RichText::new(self.i18n.t("settings.api_key"))
                                .size(12.0)
                                .color(subtitle_color),
                        );
                        ui.add(
                            egui::TextEdit::singleline(&mut self.translate_api_key)
                                .password(true)
                                .hint_text("sk-...")
                                .desired_width(f32::INFINITY),
                        );
                        ui.add_space(12.0);

                        // ── Dictionary API ──
                        ui.label(
                            egui::RichText::new(self.i18n.t("settings.dictionary_section"))
                                .size(15.0)
                                .strong()
                                .color(heading_color),
                        );
                        ui.add_space(4.0);

                        ui.label(
                            egui::RichText::new(self.i18n.t("settings.api_url"))
                                .size(12.0)
                                .color(subtitle_color),
                        );
                        ui.add(
                            egui::TextEdit::singleline(&mut self.dictionary_api_url)
                                .hint_text("https://api.example.com/dict")
                                .desired_width(f32::INFINITY),
                        );
                        ui.add_space(4.0);

                        ui.label(
                            egui::RichText::new(self.i18n.t("settings.api_key"))
                                .size(12.0)
                                .color(subtitle_color),
                        );
                        ui.add(
                            egui::TextEdit::singleline(&mut self.dictionary_api_key)
                                .password(true)
                                .hint_text("key-...")
                                .desired_width(f32::INFINITY),
                        );
                        ui.add_space(16.0);

                        // ── CSC Settings ──
                        ui.separator();
                        ui.add_space(8.0);
                        self.render_csc_settings(ui);
                        ui.add_space(16.0);

                        // Save button
                        let save_btn = egui::Button::new(
                            egui::RichText::new(self.i18n.t("settings.save"))
                                .size(14.0)
                                .color(Color32::WHITE),
                        )
                        .fill(accent)
                        .corner_radius(CornerRadius::same(5))
                        .min_size(Vec2::new(80.0, 30.0));
                        if ui.add(save_btn).clicked() {
                            self.show_api_settings = false;
                        }
                    });
            });
    }
}
