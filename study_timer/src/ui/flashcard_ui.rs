use crate::ui::flashcard::{Card, Deck, Grade};
use eframe::egui;

pub struct FlashcardReviewer {
    current_card_index: usize,
    show_answer: bool,
    selected_deck_id: Option<u64>,
    new_deck_name: String,
    new_deck_description: String,
    new_card_front: String,
    new_card_back: String,
}

impl FlashcardReviewer {
    pub fn new() -> Self {
        Self {
            current_card_index: 0,
            show_answer: false,
            selected_deck_id: None,
            new_deck_name: String::new(),
            new_deck_description: String::new(),
            new_card_front: String::new(),
            new_card_back: String::new(),
        }
    }

    pub fn display(&mut self, ui: &mut egui::Ui, deck: &mut Deck) {
        ui.vertical(|ui| {
            ui.heading(&deck.name);
            if let Some(desc) = &deck.description {
                ui.label(desc);
            }

            ui.separator();
            
            if let Some(card) = deck.cards.get(self.current_card_index) {
                ui.label(egui::RichText::new("Question:").strong());
                ui.label(&card.front);
                
                if self.show_answer {
                    ui.label(egui::RichText::new("Answer:").strong());
                    ui.label(&card.back);
                    
                    ui.horizontal(|ui| {
                        if ui.button("Again").clicked() {
                            deck.cards[self.current_card_index].add_review(Grade::Again);
                            self.next_card(deck.cards.len());
                        }
                        if ui.button("Hard").clicked() {
                            deck.cards[self.current_card_index].add_review(Grade::Hard);
                            self.next_card(deck.cards.len());
                        }
                        if ui.button("Good").clicked() {
                            deck.cards[self.current_card_index].add_review(Grade::Good);
                            self.next_card(deck.cards.len());
                        }
                        if ui.button("Easy").clicked() {
                            deck.cards[self.current_card_index].add_review(Grade::Easy);
                            self.next_card(deck.cards.len());
                        }
                    });
                } else {
                    if ui.button("Show Answer").clicked() {
                        self.show_answer = true;
                    }
                }
            } else {
                ui.label("No cards in this deck");
            }
        });
    }

    fn next_card(&mut self, total_cards: usize) {
        self.show_answer = false;
        self.current_card_index += 1;
        if self.current_card_index >= total_cards {
            self.current_card_index = 0;
        }
    }
}

pub struct DeckManagerUI {
    pub selected_deck_id: Option<u64>,
    pub new_deck_name: String,
    pub new_deck_description: String,
    pub new_card_front: String,
    pub new_card_back: String,
}

impl DeckManagerUI {
    pub fn new() -> Self {
        Self {
            selected_deck_id: None,
            new_deck_name: String::new(),
            new_deck_description: String::new(),
            new_card_front: String::new(),
            new_card_back: String::new(),
        }
    }

    pub fn display(&mut self, ui: &mut egui::Ui, decks: &mut Vec<Deck>) -> bool {
        let mut needs_save = false;

        ui.horizontal(|ui| {
            // Deck list panel
            egui::SidePanel::left("decks_panel")
                .resizable(true)
                .default_width(200.0)
                .show_inside(ui, |ui| {
                    let decks_clone = decks.clone(); // Create a clone for iteration
                    ui.heading("Decks");
                    ui.separator();

                    for deck in &decks_clone {
                        if ui
                            .selectable_label(self.selected_deck_id == Some(deck.id), &deck.name)
                            .clicked()
                        {
                            self.selected_deck_id = Some(deck.id);
                        }
                    }

                    ui.separator();
                    ui.heading("Create New Deck");
                    ui.add(egui::TextEdit::singleline(&mut self.new_deck_name).hint_text("Deck name"));
                    ui.add(
                        egui::TextEdit::singleline(&mut self.new_deck_description)
                            .hint_text("Description (optional)"),
                    );

                    if ui.button("Create Deck").clicked() && !self.new_deck_name.is_empty() {
                        let deck = Deck::new(
                            self.new_deck_name.clone(),
                            if self.new_deck_description.is_empty() {
                                None
                            } else {
                                Some(self.new_deck_description.clone())
                            },
                        );
                        decks.push(deck);
                        self.new_deck_name.clear();
                        self.new_deck_description.clear();
                        needs_save = true;
                    }
                });

            // Deck content panel
            if let Some(deck_id) = self.selected_deck_id {
                if let Some(deck) = decks.iter_mut().find(|d| d.id == deck_id) {
                    ui.vertical(|ui| {
                        ui.heading(&deck.name);
                        if let Some(desc) = &deck.description {
                            ui.label(desc);
                        }

                        ui.separator();
                        ui.heading("Add New Card");
                        egui::TextEdit::multiline(&mut self.new_card_front)
                            .hint_text("Front (question)")
                            .show(ui);
                        egui::TextEdit::multiline(&mut self.new_card_back)
                            .hint_text("Back (answer)")
                            .show(ui);

                        if ui.button("Add Card").clicked()
                            && !self.new_card_front.is_empty()
                            && !self.new_card_back.is_empty()
                        {
                            deck.add_card(self.new_card_front.clone(), self.new_card_back.clone());
                            self.new_card_front.clear();
                            self.new_card_back.clear();
                            needs_save = true;
                        }

                        ui.separator();
                        ui.heading("Cards in Deck");
                        for card in &deck.cards {
                            ui.collapsing(format!("Card #{}", card.id), |ui| {
                                ui.label(egui::RichText::new("Front:").strong());
                                ui.label(&card.front);
                                ui.label(egui::RichText::new("Back:").strong());
                                ui.label(&card.back);
                            });
                        }
                    });
                }
            } else {
                ui.label("Select a deck or create a new one");
            }
        });

        needs_save
    }
}
