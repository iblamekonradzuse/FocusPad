use crate::ui::flashcard::{ Deck, Grade};
use eframe::egui;

pub struct FlashcardReviewer {
    current_card_index: usize,
    show_answer: bool,
    selected_deck_id: Option<u64>,
    new_deck_name: String,
    new_deck_description: String,
    new_card_front: String,
    new_card_back: String,
    pub is_fullscreen: bool, // Made public so it can be accessed from flashcard_tab_ui
    edit_card_id: Option<u64>,
    edit_card_front: String,
    edit_card_back: String,
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
            is_fullscreen: false,
            edit_card_id: None,
            edit_card_front: String::new(),
            edit_card_back: String::new(),
        }
    }

    pub fn display(&mut self, ui: &mut egui::Ui, deck: &mut Deck) {
        if self.is_fullscreen {
            self.display_fullscreen(ui, deck);
        } else {
            self.display_normal(ui, deck);
        }
    }

    fn display_fullscreen(&mut self, ui: &mut egui::Ui, deck: &mut Deck) {
        ui.vertical_centered(|ui| {
            // Exit fullscreen button
            ui.horizontal(|ui| {
                if ui.button("📱 Exit Fullscreen").clicked() {
                    self.is_fullscreen = false;
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(format!("Card {} of {}", self.current_card_index + 1, deck.cards.len()));
                });
            });

            ui.add_space(20.0);

            if let Some(card) = deck.cards.get(self.current_card_index) {
                // Question
                ui.add_space(40.0);
                ui.label(egui::RichText::new("Question").size(24.0).strong());
                ui.add_space(20.0);
                
                ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                    ui.label(egui::RichText::new(&card.front).size(32.0));
                });

                ui.add_space(40.0);

                if self.show_answer {
                    ui.label(egui::RichText::new("Answer").size(24.0).strong());
                    ui.add_space(20.0);
                    
                    ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                        ui.label(egui::RichText::new(&card.back).size(32.0));
                    });

                    ui.add_space(40.0);

                    // Grade buttons
                    ui.horizontal(|ui| {
                        ui.spacing_mut().button_padding = egui::vec2(20.0, 15.0);
                        
                        if ui.button(egui::RichText::new("Again").size(18.0)).clicked() {
                            deck.cards[self.current_card_index].add_review(Grade::Again);
                            self.next_card(deck.cards.len());
                        }
                        if ui.button(egui::RichText::new("Hard").size(18.0)).clicked() {
                            deck.cards[self.current_card_index].add_review(Grade::Hard);
                            self.next_card(deck.cards.len());
                        }
                        if ui.button(egui::RichText::new("Good").size(18.0)).clicked() {
                            deck.cards[self.current_card_index].add_review(Grade::Good);
                            self.next_card(deck.cards.len());
                        }
                        if ui.button(egui::RichText::new("Easy").size(18.0)).clicked() {
                            deck.cards[self.current_card_index].add_review(Grade::Easy);
                            self.next_card(deck.cards.len());
                        }
                    });
                } else {
                    if ui.button(egui::RichText::new("Show Answer").size(20.0)).clicked() {
                        self.show_answer = true;
                    }
                }
            } else {
                ui.label(egui::RichText::new("No cards in this deck").size(24.0));
            }
        });
    }

    fn display_normal(&mut self, ui: &mut egui::Ui, deck: &mut Deck) {
        ui.vertical(|ui| {
            // Header with navigation
            ui.horizontal(|ui| {
                ui.heading(&deck.name);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("🔍 Fullscreen").clicked() {
                        self.is_fullscreen = true;
                    }
                });
            });

            if let Some(desc) = &deck.description {
                ui.label(desc);
            }

            ui.separator();
            
            if let Some(card) = deck.cards.get(self.current_card_index) {
                // Card counter
                ui.label(format!("Card {} of {}", self.current_card_index + 1, deck.cards.len()));
                ui.add_space(10.0);

                // Question
                ui.group(|ui| {
                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new("Question:").size(16.0).strong());
                        ui.add_space(5.0);
                        ui.label(egui::RichText::new(&card.front).size(14.0));
                    });
                });
                
                ui.add_space(10.0);

                if self.show_answer {
                    // Answer
                    ui.group(|ui| {
                        ui.vertical(|ui| {
                            ui.label(egui::RichText::new("Answer:").size(16.0).strong());
                            ui.add_space(5.0);
                            ui.label(egui::RichText::new(&card.back).size(14.0));
                        });
                    });

                    ui.add_space(15.0);
                    
                    // Grade buttons
                    ui.horizontal(|ui| {
                        ui.spacing_mut().button_padding = egui::vec2(12.0, 8.0);
                        
                        if ui.button(egui::RichText::new("Again").color(egui::Color32::from_rgb(220, 53, 69))).clicked() {
                            deck.cards[self.current_card_index].add_review(Grade::Again);
                            self.next_card(deck.cards.len());
                        }
                        if ui.button(egui::RichText::new("Hard").color(egui::Color32::from_rgb(255, 193, 7))).clicked() {
                            deck.cards[self.current_card_index].add_review(Grade::Hard);
                            self.next_card(deck.cards.len());
                        }
                        if ui.button(egui::RichText::new("Good").color(egui::Color32::from_rgb(40, 167, 69))).clicked() {
                            deck.cards[self.current_card_index].add_review(Grade::Good);
                            self.next_card(deck.cards.len());
                        }
                        if ui.button(egui::RichText::new("Easy").color(egui::Color32::from_rgb(23, 162, 184))).clicked() {
                            deck.cards[self.current_card_index].add_review(Grade::Easy);
                            self.next_card(deck.cards.len());
                        }
                    });
                } else {
                    if ui.button(egui::RichText::new("Show Answer").size(16.0)).clicked() {
                        self.show_answer = true;
                    }
                }
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label(egui::RichText::new("No cards in this deck").size(18.0));
                });
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

#[derive(Debug, Clone, PartialEq)]
pub enum ViewMode {
    DeckList,
    DeckView,
}

pub struct DeckManagerUI {
    pub selected_deck_id: Option<u64>,
    pub new_deck_name: String,
    pub new_deck_description: String,
    pub new_card_front: String,
    pub new_card_back: String,
    pub view_mode: ViewMode,
    pub edit_deck_id: Option<u64>,
    pub edit_deck_name: String,
    pub edit_deck_description: String,
    pub edit_card_id: Option<u64>,
    pub edit_card_front: String,
    pub edit_card_back: String,
    pub delete_confirmation: Option<String>, // Holds the type of item being deleted ("deck" or "card")
    pub item_to_delete: Option<u64>, // ID of item to delete
}

impl DeckManagerUI {
    pub fn new() -> Self {
        Self {
            selected_deck_id: None,
            new_deck_name: String::new(),
            new_deck_description: String::new(),
            new_card_front: String::new(),
            new_card_back: String::new(),
            view_mode: ViewMode::DeckList,
            edit_deck_id: None,
            edit_deck_name: String::new(),
            edit_deck_description: String::new(),
            edit_card_id: None,
            edit_card_front: String::new(),
            edit_card_back: String::new(),
            delete_confirmation: None,
            item_to_delete: None,
        }
    }

    pub fn display(&mut self, ui: &mut egui::Ui, decks: &mut Vec<Deck>) -> bool {
        let mut needs_save = false;

        match self.view_mode {
            ViewMode::DeckList => {
                needs_save |= self.display_deck_list(ui, decks);
            }
            ViewMode::DeckView => {
                needs_save |= self.display_deck_view(ui, decks);
            }
        }

        // Handle delete confirmation dialog
        if let Some(delete_type) = &self.delete_confirmation.clone() {
            egui::Window::new("Confirm Delete")
                .collapsible(false)
                .resizable(false)
                .show(ui.ctx(), |ui| {
                    ui.label(format!("Are you sure you want to delete this {}?", delete_type));
                    ui.separator();
                    ui.horizontal(|ui| {
                        if ui.button("Delete").clicked() {
                            if delete_type == "deck" {
                                if let Some(deck_id) = self.item_to_delete {
                                    decks.retain(|d| d.id != deck_id);
                                    if self.selected_deck_id == Some(deck_id) {
                                        self.selected_deck_id = None;
                                        self.view_mode = ViewMode::DeckList;
                                    }
                                    needs_save = true;
                                }
                            } else if delete_type == "card" {
                                if let (Some(deck_id), Some(card_id)) = (self.selected_deck_id, self.item_to_delete) {
                                    if let Some(deck) = decks.iter_mut().find(|d| d.id == deck_id) {
                                        deck.cards.retain(|c| c.id != card_id);
                                        needs_save = true;
                                    }
                                }
                            }
                            self.delete_confirmation = None;
                            self.item_to_delete = None;
                        }
                        if ui.button("Cancel").clicked() {
                            self.delete_confirmation = None;
                            self.item_to_delete = None;
                        }
                    });
                });
        }

        needs_save
    }

    fn display_deck_list(&mut self, ui: &mut egui::Ui, decks: &mut Vec<Deck>) -> bool {
        let mut needs_save = false;

        ui.heading("📚 My Decks");
        ui.separator();

        // Deck list
        ui.spacing_mut().item_spacing.y = 8.0;
        
        for deck in decks.iter() {
            ui.horizontal(|ui| {
                // Deck info
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.label(egui::RichText::new(&deck.name).size(16.0).strong());
                            if let Some(desc) = &deck.description {
                                ui.label(egui::RichText::new(desc).size(12.0).weak());
                            }
                            ui.label(format!("{} cards", deck.cards.len()));
                        });
                        
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            // Delete deck button
                            if ui.button("🗑").clicked() {
                                self.delete_confirmation = Some("deck".to_string());
                                self.item_to_delete = Some(deck.id);
                            }
                            
                            // Edit deck button
                            if ui.button("✏").clicked() {
                                self.edit_deck_id = Some(deck.id);
                                self.edit_deck_name = deck.name.clone();
                                self.edit_deck_description = deck.description.clone().unwrap_or_default();
                            }
                            
                            // Select deck button
                            if ui.button("Open").clicked() {
                                self.selected_deck_id = Some(deck.id);
                                self.view_mode = ViewMode::DeckView;
                            }
                        });
                    });
                });
            });
        }

        ui.add_space(20.0);
        ui.separator();

        // Create new deck section
        ui.heading("➕ Create New Deck");
        ui.add_space(10.0);

        ui.horizontal(|ui| {
            ui.label("Name:");
            ui.add(egui::TextEdit::singleline(&mut self.new_deck_name).hint_text("Enter deck name"));
        });

        ui.horizontal(|ui| {
            ui.label("Description:");
            ui.add(egui::TextEdit::singleline(&mut self.new_deck_description).hint_text("Optional description"));
        });

        ui.add_space(10.0);

        if ui.button("Create Deck").clicked() && !self.new_deck_name.is_empty() {
            let mut deck = Deck::new(
                self.new_deck_name.clone(),
                if self.new_deck_description.is_empty() {
                    None
                } else {
                    Some(self.new_deck_description.clone())
                },
            );
            deck.id = self.get_next_deck_id(decks);
            decks.push(deck);
            self.new_deck_name.clear();
            self.new_deck_description.clear();
            needs_save = true;
        }

        // Edit deck dialog
        if let Some(edit_id) = self.edit_deck_id {
            egui::Window::new("Edit Deck")
                .collapsible(false)
                .resizable(false)
                .show(ui.ctx(), |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Name:");
                        ui.add(egui::TextEdit::singleline(&mut self.edit_deck_name));
                    });

                    ui.horizontal(|ui| {
                        ui.label("Description:");
                        ui.add(egui::TextEdit::singleline(&mut self.edit_deck_description));
                    });

                    ui.separator();
                    ui.horizontal(|ui| {
                        if ui.button("Save").clicked() {
                            if let Some(deck) = decks.iter_mut().find(|d| d.id == edit_id) {
                                deck.name = self.edit_deck_name.clone();
                                deck.description = if self.edit_deck_description.is_empty() {
                                    None
                                } else {
                                    Some(self.edit_deck_description.clone())
                                };
                                needs_save = true;
                            }
                            self.edit_deck_id = None;
                        }
                        if ui.button("Cancel").clicked() {
                            self.edit_deck_id = None;
                        }
                    });
                });
        }

        needs_save
    }

    fn display_deck_view(&mut self, ui: &mut egui::Ui, decks: &mut Vec<Deck>) -> bool {
        let mut needs_save = false;

        if let Some(deck_id) = self.selected_deck_id {
            if let Some(deck) = decks.iter_mut().find(|d| d.id == deck_id) {
                // Use TopBottomPanel to create better layout
                egui::TopBottomPanel::top("deck_header").show_inside(ui, |ui| {
                    // Header with back button
                    ui.horizontal(|ui| {
                        if ui.button("← Back to Decks").clicked() {
                            self.view_mode = ViewMode::DeckList;
                            self.selected_deck_id = None;
                        }
                        ui.separator();
                        ui.heading(&deck.name);
                    });

                    if let Some(desc) = &deck.description {
                        ui.label(desc);
                    }
                });

                egui::TopBottomPanel::top("add_card_section").show_inside(ui, |ui| {
                    ui.separator();
                    
                    // Add new card section
                    ui.heading("➕ Add New Card");
                    ui.add_space(5.0);

                    ui.label("Front (Question):");
                    ui.add(egui::TextEdit::multiline(&mut self.new_card_front).desired_rows(3));

                    ui.label("Back (Answer):");
                    ui.add(egui::TextEdit::multiline(&mut self.new_card_back).desired_rows(3));

                    ui.add_space(10.0);

                    if ui.button("Add Card").clicked()
                        && !self.new_card_front.is_empty()
                        && !self.new_card_back.is_empty()
                    {
                        deck.add_card(self.new_card_front.clone(), self.new_card_back.clone());
                        self.new_card_front.clear();
                        self.new_card_back.clear();
                        needs_save = true;
                    }

                    ui.add_space(10.0);
                    ui.separator();
                });

                // Cards list - now takes up remaining space
                egui::CentralPanel::default().show_inside(ui, |ui| {
                    ui.heading("📋 Cards in Deck");
                    ui.add_space(10.0);

                    if deck.cards.is_empty() {
                        ui.centered_and_justified(|ui| {
                            ui.label("No cards in this deck yet. Add some cards above!");
                        });
                    } else {
                        // Now the scroll area will take up all remaining vertical space
                        egui::ScrollArea::vertical()
                            .auto_shrink([false; 2])
                            .show(ui, |ui| {
                                for card in &deck.cards {
                                    ui.group(|ui| {
                                        ui.horizontal(|ui| {
                                            ui.vertical(|ui| {
                                                ui.label(egui::RichText::new("Front:").strong());
                                                ui.label(&card.front);
                                                ui.add_space(5.0);
                                                ui.label(egui::RichText::new("Back:").strong());
                                                ui.label(&card.back);
                                            });

                                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                                                // Delete card button
                                                if ui.button("🗑").clicked() {
                                                    self.delete_confirmation = Some("card".to_string());
                                                    self.item_to_delete = Some(card.id);
                                                }
                                                
                                                // Edit card button
                                                if ui.button("✏").clicked() {
                                                    self.edit_card_id = Some(card.id);
                                                    self.edit_card_front = card.front.clone();
                                                    self.edit_card_back = card.back.clone();
                                                }
                                            });
                                        });
                                    });
                                    ui.add_space(8.0);
                                }
                            });
                    }
                });
            }
        }

        // Edit card dialog
        if let Some(edit_id) = self.edit_card_id {
            egui::Window::new("Edit Card")
                .collapsible(false)
                .resizable(true)
                .default_size([400.0, 300.0])
                .show(ui.ctx(), |ui| {
                    ui.label("Front (Question):");
                    ui.add(egui::TextEdit::multiline(&mut self.edit_card_front).desired_rows(3));

                    ui.label("Back (Answer):");
                    ui.add(egui::TextEdit::multiline(&mut self.edit_card_back).desired_rows(3));

                    ui.separator();
                    ui.horizontal(|ui| {
                        if ui.button("Save").clicked() {
                            if let Some(deck_id) = self.selected_deck_id {
                                if let Some(deck) = decks.iter_mut().find(|d| d.id == deck_id) {
                                    if let Some(card) = deck.cards.iter_mut().find(|c| c.id == edit_id) {
                                        card.front = self.edit_card_front.clone();
                                        card.back = self.edit_card_back.clone();
                                        needs_save = true;
                                    }
                                }
                            }
                            self.edit_card_id = None;
                        }
                        if ui.button("Cancel").clicked() {
                            self.edit_card_id = None;
                        }
                    });
                });
        }

        needs_save
    }

    fn get_next_deck_id(&self, decks: &[Deck]) -> u64 {
        if let Some(max_id) = decks.iter().map(|d| d.id).max() {
            max_id + 1
        } else {
            1
        }
    }
}
