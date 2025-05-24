use crate::StudyTimerApp;
use eframe::egui;

pub fn display(ui: &mut egui::Ui, _ctx: &egui::Context, app: &mut StudyTimerApp) {
    // Check if we're in fullscreen mode first
    if app.flashcard_reviewer.is_fullscreen {
        // Display only the fullscreen flashcard reviewer
        if let Some(deck_id) = app.deck_manager_ui.selected_deck_id {
            if let Some(deck) = app.study_data.decks.iter_mut().find(|d| d.id == deck_id) {
                app.flashcard_reviewer.display(ui, deck);
            }
        }
        return;
    }

    egui::TopBottomPanel::top("flashcard_tab_header").show_inside(ui, |ui| {
        ui.horizontal(|ui| {
            ui.heading("🎯 Flashcards");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(format!(
                    "📅 Due cards: {}",
                    app.study_data.get_due_cards_count()
                ));
            });
        });
    });

    egui::CentralPanel::default().show_inside(ui, |ui| {
        if app.tab_manager.is_split_active() {
            // Handle split view if needed
            display_split_view(ui, app);
        } else {
            display_single_view(ui, app);
        }
    });
}

fn display_single_view(ui: &mut egui::Ui, app: &mut StudyTimerApp) {
    match app.deck_manager_ui.view_mode {
        crate::ui::flashcard_ui::ViewMode::DeckList => {
            // Show deck management only
            let needs_save = app.deck_manager_ui.display(ui, &mut app.study_data.decks);
            if needs_save {
                if let Err(err) = app.study_data.save() {
                    app.status.show(&format!("Error saving: {}", err));
                }
            }
        }
        crate::ui::flashcard_ui::ViewMode::DeckView => {
            // Show deck view with review panel
            ui.horizontal(|ui| {
                // Review panel (left side)
                if let Some(deck_id) = app.deck_manager_ui.selected_deck_id {
                    if let Some(deck) = app.study_data.decks.iter_mut().find(|d| d.id == deck_id) {
                        egui::SidePanel::left("review_panel")
                            .resizable(true)
                            .default_width(350.0)
                            .min_width(300.0)
                            .show_inside(ui, |ui| {
                                ui.with_layout(egui::Layout::top_down_justified(egui::Align::Center), |ui| {
                                    ui.heading("📖 Study Mode");
                                    ui.separator();
                                    app.flashcard_reviewer.display(ui, deck);
                                });
                            });
                    }
                }

                // Deck management panel (right side)
                ui.vertical(|ui| {
                    let needs_save = app.deck_manager_ui.display(ui, &mut app.study_data.decks);
                    if needs_save {
                        if let Err(err) = app.study_data.save() {
                            app.status.show(&format!("Error saving: {}", err));
                        }
                    }
                });
            });
        }
    }
}

fn display_split_view(ui: &mut egui::Ui, app: &mut StudyTimerApp) {
    // Handle split view functionality if needed
    // For now, just display the single view
    display_single_view(ui, app);
}
