//! Reading statistics and progress UI.
use crate::app::ReaderApp;
use eframe::egui;

impl ReaderApp {
    /// Render the reading statistics window.
    pub fn render_stats_window(&mut self, ctx: &egui::Context) {
        if !self.show_stats {
            return;
        }

        let mut open = true;
        egui::Window::new(self.i18n.t("stats.title"))
            .open(&mut open)
            .collapsible(false)
            .resizable(true)
            .default_width(400.0)
            .show(ctx, |ui| {
                let config = self.book_config.as_ref();
                let stats = config.and_then(|c| c.reading_stats.as_ref());

                if let Some(stats) = stats {
                    let hours = stats.total_seconds / 3600;
                    let mins = (stats.total_seconds % 3600) / 60;
                    ui.heading(
                        self.i18n
                            .tf1("stats.total_time", &format!("{}h {}m", hours, mins)),
                    );
                    ui.add_space(8.0);

                    if !stats.sessions.is_empty() {
                        ui.label(self.i18n.t("stats.recent_sessions"));
                        ui.add_space(4.0);

                        egui::Grid::new("stats_grid").striped(true).show(ui, |ui| {
                            ui.label(egui::RichText::new(self.i18n.t("stats.date")).strong());
                            ui.label(egui::RichText::new(self.i18n.t("stats.duration")).strong());
                            ui.end_row();

                            // Show last 20 sessions, newest first
                            let sessions: Vec<_> = stats.sessions.iter().rev().take(20).collect();
                            for session in sessions {
                                ui.label(&session.date);
                                let m = session.seconds / 60;
                                let s = session.seconds % 60;
                                ui.label(format!("{m}m {s}s"));
                                ui.end_row();
                            }
                        });
                    }
                } else {
                    ui.label(self.i18n.t("stats.no_data"));
                }
            });

        if !open {
            self.show_stats = false;
        }
    }
}
