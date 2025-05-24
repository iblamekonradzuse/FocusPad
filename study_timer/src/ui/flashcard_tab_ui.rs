use crate::StudyTimerApp;
use eframe::egui;

pub fn display(ui: &mut egui::Ui, ctx: &egui::Context, app: &mut StudyTimerApp) {
    egui::TopBottomPanel::top("flashcard_tab_header").show_inside(ui, |ui| {
        ui.horizontal(|ui| {
            ui.heading("Flashcards");
            ui.label(format!(
                "Due cards: {}",
                app.study_data.get_due_cards_count()
            ));
        });
    });

    egui::CentralPanel::default().show_inside(ui, |ui| {
        if app.tab_manager.is_split_active() {
            // Handle split view if needed
        } else {
            ui.horizontal(|ui| {
                // Review panel
                if let Some(deck_id) = app.deck_manager_ui.selected_deck_id {
                    if let Some(deck) = app.study_data.decks.iter_mut().find(|d| d.id == deck_id) {
                        egui::SidePanel::left("review_panel")
                            .resizable(false)
                            .default_width(300.0)
                            .show_inside(ui, |ui| {
                                app.flashcard_reviewer.display(ui, deck);
                            });
                    }
                }

                // Deck management panel
                let needs_save = app.deck_manager_ui.display(ui, &mut app.study_data.decks);
                if needs_save {
                    if let Err(err) = app.study_data.save() {
                        app.status.show(&format!("Error saving: {}", err));
                    }
                }
            });
        }
    });
}
