//! UI component for performing textual searches within the book.
use crate::app::ReaderApp;
use eframe::egui;

impl ReaderApp {
    /// Render the search side-panel.
    pub fn render_search_panel(&mut self, ctx: &egui::Context) {
        if !self.show_search {
            return;
        }

        egui::SidePanel::right("search_panel")
            .default_width(320.0)
            .min_width(260.0)
            .show(ctx, |ui| {
                ui.heading(self.i18n.t("search.title"));
                ui.add_space(4.0);

                let mut run_search = false;
                ui.horizontal(|ui| {
                    let resp = ui.add(
                        egui::TextEdit::singleline(&mut self.search_query)
                            .hint_text(self.i18n.t("search.placeholder"))
                            .desired_width(ui.available_width() - 60.0),
                    );
                    if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        run_search = true;
                    }
                    if ui.button(self.i18n.t("search.go")).clicked() {
                        run_search = true;
                    }
                });

                if run_search && !self.search_query.is_empty() {
                    if let Some(book) = &self.book {
                        self.search_results =
                            reader_core::search::search_book(book, &self.search_query, false);
                        self.search_selected = None;
                        self.search_target_block = None;
                        crate::ui::reader_state::SEARCH_HIGHLIGHT_BLOCK.set(None);
                    }
                }

                ui.add_space(2.0);
                ui.label(
                    egui::RichText::new(self.i18n.t("search.hint"))
                        .size(11.0)
                        .color(egui::Color32::GRAY),
                );
                ui.add_space(4.0);
                if !self.search_results.is_empty() {
                    ui.label(self.i18n.tf1(
                        "search.results_count",
                        &self.search_results.len().to_string(),
                    ));
                    ui.add_space(4.0);

                    egui::ScrollArea::vertical()
                        .auto_shrink([false; 2])
                        .show(ui, |ui| {
                            let results = self.search_results.clone();
                            for (idx, result) in results.iter().enumerate() {
                                let selected = self.search_selected == Some(idx);
                                let resp = ui.selectable_label(
                                    selected,
                                    format!("[{}] {}", result.chapter_title, result.context),
                                );
                                if resp.clicked() {
                                    self.search_selected = Some(idx);
                                    // Jump to the chapter
                                    if self.current_chapter != result.chapter_index {
                                    self.current_chapter = result.chapter_index;
                                    self.current_page = 0;
                                }
                                self.pages_dirty = true;
                                self.search_target_block = Some(result.block_index);
                                }
                            }
                        });
                } else if !self.search_query.is_empty() {
                    ui.label(self.i18n.t("search.no_results"));
                }
            });
    }
}
