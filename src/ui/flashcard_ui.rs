use crate::image_handler::{CardImage, ImageManager};
use crate::ui::flashcard::{Deck, Grade};
use arboard::Clipboard;
use base64::Engine;
use eframe::egui;
use egui::TextureHandle;
use image;
use rand::prelude::SliceRandom;
use std::collections::HashMap;
use std::path::PathBuf;

#[allow(dead_code)]
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
    review_mode: ReviewMode,
    current_difficulty_filter: Option<Grade>,
    weighted_cards: Vec<usize>,
    pub show_image_dialog: bool,
    pub pending_image_side: Option<ImageSide>, // Front or Back
    pub selected_image_path: Option<PathBuf>,
    pub algorithm_enabled: bool,
    texture_cache: HashMap<u64, TextureHandle>,
    pub right_panel_open: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ReviewMode {
    All,
    ByDifficulty(Grade),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ImageSide {
    Front,
    Back,
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
            review_mode: ReviewMode::All,
            current_difficulty_filter: None,
            weighted_cards: Vec::new(),
            algorithm_enabled: false,
            show_image_dialog: false,
            pending_image_side: None,
            selected_image_path: None,
            texture_cache: HashMap::new(),
            right_panel_open: true,
        }
    }

    pub fn display(&mut self, ui: &mut egui::Ui, deck: &mut Deck) {
        let mut back_to_decks = false;
        let mut right_panel_open = self.right_panel_open;

        self.display_with_callbacks(ui, deck, &mut back_to_decks, &mut right_panel_open);

        self.right_panel_open = right_panel_open;
    }

    pub fn display_with_callbacks(
        &mut self,
        ui: &mut egui::Ui,
        deck: &mut Deck,
        back_to_decks: &mut bool,
        right_panel_open: &mut bool,
    ) {
        if self.is_fullscreen {
            self.display_fullscreen(ui, deck);
        } else {
            self.display_normal_with_callbacks(ui, deck, back_to_decks, right_panel_open);
        }
    }

    fn display_image(&mut self, ui: &mut egui::Ui, card_image: &CardImage, max_size: [f32; 2]) {
        // Check if texture already exists in cache
        let cache_key = format!("{}_{}", card_image.id, card_image.data.len());
        if let Some(texture_handle) =
            self.texture_cache
                .get(&cache_key.parse::<u64>().unwrap_or_else(|_| {
                    use std::collections::hash_map::DefaultHasher;
                    use std::hash::{Hash, Hasher};
                    let mut hasher = DefaultHasher::new();
                    cache_key.hash(&mut hasher);
                    hasher.finish()
                }))
        {
            // Use cached texture - display with proper size constraints
            let texture_size = texture_handle.size_vec2();
            let available_width = ui.available_width().min(max_size[0]);
            let max_height = max_size[1];

            // Calculate scale factor to fit within both width and height constraints
            let width_scale = if texture_size.x > available_width {
                available_width / texture_size.x
            } else {
                1.0
            };

            let height_scale = if texture_size.y > max_height {
                max_height / texture_size.y
            } else {
                1.0
            };

            // Use the smaller scale factor to ensure the image fits within both constraints
            let scale_factor = width_scale.min(height_scale);

            let display_width = texture_size.x * scale_factor;
            let display_height = texture_size.y * scale_factor;

            ui.add(
                egui::Image::from_texture(texture_handle)
                    .fit_to_exact_size(egui::Vec2::new(display_width, display_height)),
            );
            return;
        }

        // Only decode and load image if not in cache
        match base64::engine::general_purpose::STANDARD.decode(&card_image.data) {
            Ok(image_data) => {
                match image::load_from_memory(&image_data) {
                    Ok(dynamic_image) => {
                        let rgba_image = dynamic_image.to_rgba8();
                        let size = [rgba_image.width() as usize, rgba_image.height() as usize];
                        let pixels = rgba_image.as_flat_samples();

                        let color_image =
                            egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());

                        let texture_handle = ui.ctx().load_texture(
                            format!("card_image_{}_{}", card_image.id, card_image.data.len()),
                            color_image,
                            egui::TextureOptions::default(),
                        );

                        // Display with proper size constraints
                        let texture_size = texture_handle.size_vec2();
                        let available_width = ui.available_width().min(max_size[0]);
                        let max_height = max_size[1];

                        // Calculate scale factor to fit within both width and height constraints
                        let width_scale = if texture_size.x > available_width {
                            available_width / texture_size.x
                        } else {
                            1.0
                        };

                        let height_scale = if texture_size.y > max_height {
                            max_height / texture_size.y
                        } else {
                            1.0
                        };

                        // Use the smaller scale factor to ensure the image fits within both constraints
                        let scale_factor = width_scale.min(height_scale);

                        let display_width = texture_size.x * scale_factor;
                        let display_height = texture_size.y * scale_factor;

                        ui.add(
                            egui::Image::from_texture(&texture_handle)
                                .fit_to_exact_size(egui::Vec2::new(display_width, display_height)),
                        );

                        // Cache the texture for future use
                        let cache_key = format!("{}_{}", card_image.id, card_image.data.len());
                        let cache_key_hash = cache_key.parse::<u64>().unwrap_or_else(|_| {
                            use std::collections::hash_map::DefaultHasher;
                            use std::hash::{Hash, Hasher};
                            let mut hasher = DefaultHasher::new();
                            cache_key.hash(&mut hasher);
                            hasher.finish()
                        });
                        self.texture_cache.insert(cache_key_hash, texture_handle);
                    }
                    Err(e) => {
                        ui.colored_label(
                            egui::Color32::RED,
                            format!("Failed to load image: {}", e),
                        );
                    }
                }
            }
            Err(e) => {
                ui.colored_label(
                    egui::Color32::RED,
                    format!("Failed to decode base64: {}", e),
                );
            }
        }
    }

    fn display_normal_with_callbacks(
        &mut self,
        ui: &mut egui::Ui,
        deck: &mut Deck,
        back_to_decks: &mut bool,
        right_panel_open: &mut bool,
    ) {
        ui.vertical(|ui| {
            // Header with navigation
            ui.horizontal(|ui| {
                if ui.button("← Back to Decks").clicked() {
                    *back_to_decks = true;
                }
                ui.separator();
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("🔍 Fullscreen").clicked() {
                        self.is_fullscreen = true;
                    }
                    let toggle_text = if *right_panel_open {
                        "Hide Panel"
                    } else {
                        "Show Panel"
                    };
                    if ui.button(toggle_text).clicked() {
                        *right_panel_open = !*right_panel_open;
                    }
                });
            });

            ui.separator();

            // Review mode selection
            ui.horizontal(|ui| {
                ui.label("Review Mode:");

                if ui
                    .selectable_label(matches!(self.review_mode, ReviewMode::All), "All Cards")
                    .clicked()
                {
                    self.review_mode = ReviewMode::All;
                    self.reset_review_session(deck);
                }

                if ui
                    .selectable_label(
                        matches!(self.review_mode, ReviewMode::ByDifficulty(Grade::Again)),
                        "Again Cards",
                    )
                    .clicked()
                {
                    self.review_mode = ReviewMode::ByDifficulty(Grade::Again);
                    self.reset_review_session(deck);
                }

                if ui
                    .selectable_label(
                        matches!(self.review_mode, ReviewMode::ByDifficulty(Grade::Hard)),
                        "Hard Cards",
                    )
                    .clicked()
                {
                    self.review_mode = ReviewMode::ByDifficulty(Grade::Hard);
                    self.reset_review_session(deck);
                }

                if ui
                    .selectable_label(
                        matches!(self.review_mode, ReviewMode::ByDifficulty(Grade::Good)),
                        "Good Cards",
                    )
                    .clicked()
                {
                    self.review_mode = ReviewMode::ByDifficulty(Grade::Good);
                    self.reset_review_session(deck);
                }

                if ui
                    .selectable_label(
                        matches!(self.review_mode, ReviewMode::ByDifficulty(Grade::Easy)),
                        "Easy Cards",
                    )
                    .clicked()
                {
                    self.review_mode = ReviewMode::ByDifficulty(Grade::Easy);
                    self.reset_review_session(deck);
                }
            });

            ui.separator();

            // Algorithm toggle
            ui.horizontal(|ui| {
                ui.label("Spaced Repetition:");
                if ui
                    .checkbox(&mut self.algorithm_enabled, "Enable algorithm")
                    .changed()
                {
                    self.reset_review_session(deck);
                }
                ui.label("(When disabled, all cards are always available for review)");
            });

            ui.separator();

            // Get card data first, then drop the borrow
            let card_data = if let Some(card) = self.get_current_card(deck) {
                Some((
                    card.front.clone(),
                    card.back.clone(),
                    card.front_image.clone(),
                    card.back_image.clone(),
                ))
            } else {
                None
            };

            if let Some((card_front, card_back, front_image, back_image)) = card_data {
                // Card counter
                let total_cards = self.get_review_cards_count(deck);
                ui.label(format!(
                    "Card {} of {}",
                    self.current_card_index + 1,
                    total_cards
                ));
                ui.add_space(10.0);

                // Question
                ui.group(|ui| {
                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new("Question:").size(16.0).strong());
                        ui.add_space(5.0);

                        // Make question content scrollable with controlled height
                        ui.push_id("question_scroll", |ui| {
                            egui::ScrollArea::vertical()
                                .id_source("question_content")
                                .max_height(250.0) // Reduced from 400.0
                                .min_scrolled_height(150.0) // Reduced from 200.0
                                .auto_shrink([false; 2])
                                .show(ui, |ui| {
                                    ui.label(egui::RichText::new(&card_front).size(14.0));

                                    // Display front image if available with smaller size
                                    if let Some(front_image) = &front_image {
                                        ui.add_space(10.0);
                                        self.display_image(ui, front_image, [250.0, 150.0]);
                                        // Reduced from [300.0, 200.0]
                                    }
                                });
                        });
                    });
                });

                ui.add_space(10.0);

                if self.show_answer {
                    // Grade buttons in the middle (between question and answer)
                    ui.horizontal(|ui| {
                        ui.spacing_mut().button_padding = egui::vec2(12.0, 8.0);

                        if ui
                            .button(
                                egui::RichText::new("Again")
                                    .color(egui::Color32::from_rgb(220, 53, 69)),
                            )
                            .clicked()
                        {
                            self.grade_card(deck, Grade::Again);
                        }
                        if ui
                            .button(
                                egui::RichText::new("Hard")
                                    .color(egui::Color32::from_rgb(255, 193, 7)),
                            )
                            .clicked()
                        {
                            self.grade_card(deck, Grade::Hard);
                        }
                        if ui
                            .button(
                                egui::RichText::new("Good")
                                    .color(egui::Color32::from_rgb(40, 167, 69)),
                            )
                            .clicked()
                        {
                            self.grade_card(deck, Grade::Good);
                        }
                        if ui
                            .button(
                                egui::RichText::new("Easy")
                                    .color(egui::Color32::from_rgb(23, 162, 184)),
                            )
                            .clicked()
                        {
                            self.grade_card(deck, Grade::Easy);
                        }
                    });

                    ui.add_space(10.0);

                    // Answer
                    ui.group(|ui| {
                        ui.vertical(|ui| {
                            ui.label(egui::RichText::new("Answer:").size(16.0).strong());
                            ui.add_space(5.0);

                            // Make answer content scrollable with controlled height
                            ui.push_id("answer_scroll", |ui| {
                                egui::ScrollArea::vertical()
                                    .id_source("answer_content")
                                    .max_height(300.0) // Reduced from 600.0
                                    .min_scrolled_height(150.0) // Reduced from 200.0
                                    .auto_shrink([false; 2])
                                    .show(ui, |ui| {
                                        ui.label(egui::RichText::new(&card_back).size(14.0));

                                        // Display back image if available with smaller size
                                        if let Some(back_image) = &back_image {
                                            ui.add_space(10.0);
                                            self.display_image(ui, back_image, [250.0, 150.0]);
                                            // Reduced from [400.0, 300.0]
                                        }
                                    });
                            });
                        });
                    });
                } else {
                    if ui
                        .button(egui::RichText::new("Show Answer").size(16.0))
                        .clicked()
                    {
                        self.show_answer = true;
                    }
                }
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label(egui::RichText::new("No cards available for review").size(18.0));
                });
            }
        });
    }

    fn display_fullscreen(&mut self, ui: &mut egui::Ui, deck: &mut Deck) {
        ui.vertical_centered(|ui| {
            // Exit fullscreen button
            ui.horizontal(|ui| {
                if ui.button("📱 Exit Fullscreen").clicked() {
                    self.is_fullscreen = false;
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let total_cards = self.get_review_cards_count(deck);
                    ui.label(format!(
                        "Card {} of {}",
                        self.current_card_index + 1,
                        total_cards
                    ));
                });
            });

            ui.add_space(20.0);

            // Get card data first, then drop the borrow
            let card_data = if let Some(card) = self.get_current_card(deck) {
                Some((
                    card.front.clone(),
                    card.back.clone(),
                    card.front_image.clone(),
                    card.back_image.clone(),
                ))
            } else {
                None
            };

            if let Some((card_front, card_back, front_image, back_image)) = card_data {
                // Question
                ui.add_space(20.0);
                ui.label(egui::RichText::new("Question").size(24.0).strong());
                ui.add_space(15.0);

                // Question content with size limit
                ui.push_id("fullscreen_question_scroll", |ui| {
                    egui::ScrollArea::vertical()
                        .id_source("fullscreen_question_content")
                        .max_height(250.0) // Limit question area height
                        .show(ui, |ui| {
                            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                                ui.label(egui::RichText::new(&card_front).size(28.0)); // Reduced from 32.0

                                // Display front image if available with controlled size
                                if let Some(front_image) = &front_image {
                                    ui.add_space(10.0);
                                    self.display_image(ui, front_image, [350.0, 200.0]);
                                    // Reduced from [400.0, 300.0]
                                }
                            });
                        });
                });

                ui.add_space(20.0);

                if self.show_answer {
                    ui.label(egui::RichText::new("Answer").size(24.0).strong());
                    ui.add_space(15.0);

                    // Grade buttons FIRST - always visible at the top
                    ui.allocate_ui_with_layout(
                        egui::Vec2::new(ui.available_width(), 60.0), // Reduced height from 100.0
                        egui::Layout::left_to_right(egui::Align::Center),
                        |ui| {
                            ui.spacing_mut().button_padding = egui::vec2(15.0, 10.0); // Reduced padding

                            // Add flexible space before buttons
                            ui.allocate_space(egui::Vec2::new(
                                (ui.available_width() - 320.0) / 2.0, // Reduced from 350.0
                                0.0,
                            ));

                            if ui
                                .button(
                                    egui::RichText::new("Again")
                                        .size(16.0) // Reduced from 18.0
                                        .color(egui::Color32::from_rgb(220, 53, 69)),
                                )
                                .clicked()
                            {
                                self.grade_card(deck, Grade::Again);
                            }
                            if ui
                                .button(
                                    egui::RichText::new("Hard")
                                        .size(16.0) // Reduced from 18.0
                                        .color(egui::Color32::from_rgb(255, 193, 7)),
                                )
                                .clicked()
                            {
                                self.grade_card(deck, Grade::Hard);
                            }
                            if ui
                                .button(
                                    egui::RichText::new("Good")
                                        .size(16.0) // Reduced from 18.0
                                        .color(egui::Color32::from_rgb(40, 167, 69)),
                                )
                                .clicked()
                            {
                                self.grade_card(deck, Grade::Good);
                            }
                            if ui
                                .button(
                                    egui::RichText::new("Easy")
                                        .size(16.0) // Reduced from 18.0
                                        .color(egui::Color32::from_rgb(23, 162, 184)),
                                )
                                .clicked()
                            {
                                self.grade_card(deck, Grade::Easy);
                            }
                        },
                    );

                    ui.add_space(15.0);

                    // Answer content with size limit - scrollable if needed
                    ui.push_id("fullscreen_answer_scroll", |ui| {
                        egui::ScrollArea::vertical()
                            .id_source("fullscreen_answer_content")
                            .max_height(300.0) // Limit answer area height
                            .show(ui, |ui| {
                                ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                                    ui.label(egui::RichText::new(&card_back).size(28.0)); // Reduced from 32.0

                                    // Display back image if available with controlled size
                                    if let Some(back_image) = &back_image {
                                        ui.add_space(10.0);
                                        self.display_image(ui, back_image, [350.0, 200.0]);
                                        // Reduced from [400.0, 300.0]
                                    }
                                });
                            });
                    });
                } else {
                    if ui
                        .button(egui::RichText::new("Show Answer").size(20.0))
                        .clicked()
                    {
                        self.show_answer = true;
                    }
                }
            } else {
                ui.label(egui::RichText::new("No cards available for review").size(24.0));
            }
        });
    }

    fn get_current_card<'a>(&self, deck: &'a Deck) -> Option<&'a crate::ui::flashcard::Card> {
        match &self.review_mode {
            ReviewMode::All => {
                if self.weighted_cards.is_empty() {
                    None
                } else {
                    let card_index =
                        self.weighted_cards[self.current_card_index % self.weighted_cards.len()];
                    deck.cards.get(card_index)
                }
            }
            ReviewMode::ByDifficulty(grade) => {
                let filtered_cards =
                    deck.get_cards_by_difficulty_for_review(grade, self.algorithm_enabled);
                filtered_cards.get(self.current_card_index).copied() // Convert from &&Card to &Card
            }
        }
    }

    fn get_current_card_mut<'a>(
        &mut self,
        deck: &'a mut Deck,
    ) -> Option<&'a mut crate::ui::flashcard::Card> {
        match &self.review_mode {
            ReviewMode::All => {
                if self.weighted_cards.is_empty() {
                    None
                } else {
                    let card_index =
                        self.weighted_cards[self.current_card_index % self.weighted_cards.len()];
                    deck.cards.get_mut(card_index)
                }
            }
            ReviewMode::ByDifficulty(grade) => {
                // For difficulty-specific review, we need to find the actual card by comparing
                let filtered_cards =
                    deck.get_cards_by_difficulty_for_review(grade, self.algorithm_enabled);
                if let Some(&target_card) = filtered_cards.get(self.current_card_index) {
                    let target_id = target_card.id;
                    deck.cards.iter_mut().find(|c| c.id == target_id)
                } else {
                    None
                }
            }
        }
    }

    fn get_review_cards_count(&self, deck: &Deck) -> usize {
        match &self.review_mode {
            ReviewMode::All => self.weighted_cards.len(),
            ReviewMode::ByDifficulty(grade) => deck
                .get_cards_by_difficulty_for_review(grade, self.algorithm_enabled)
                .len(),
        }
    }

    fn grade_card(&mut self, deck: &mut Deck, grade: Grade) {
        if let Some(card) = self.get_current_card_mut(deck) {
            card.add_review(grade, self.algorithm_enabled);
        }
        self.next_card(deck);
    }

    fn next_card(&mut self, deck: &Deck) {
        self.show_answer = false;
        let total_cards = self.get_review_cards_count(deck);

        if total_cards > 0 {
            self.current_card_index = (self.current_card_index + 1) % total_cards;
        } else {
            self.current_card_index = 0;
        }

        // If we're in "All" mode and completed a cycle, refresh the weighted cards
        if matches!(self.review_mode, ReviewMode::All) && self.current_card_index == 0 {
            self.setup_weighted_cards(deck);
        }
    }

    fn reset_review_session(&mut self, deck: &Deck) {
        self.current_card_index = 0;
        self.show_answer = false;

        if matches!(self.review_mode, ReviewMode::All) {
            self.setup_weighted_cards(deck);
        }
    }

    fn setup_weighted_cards(&mut self, deck: &Deck) {
        self.weighted_cards.clear();

        let due_cards = deck.get_due_cards(self.algorithm_enabled);

        for (deck_index, card) in deck.cards.iter().enumerate() {
            // Only include due cards
            if !due_cards.iter().any(|&due_card| due_card.id == card.id) {
                continue;
            }

            let weight = match card.get_difficulty() {
                Grade::Again | Grade::Hard => 4, // High frequency for difficult cards
                Grade::Good | Grade::Easy => 2,  // Lower frequency for easier cards
            };

            for _ in 0..weight {
                self.weighted_cards.push(deck_index);
            }
        }

        // Shuffle the weighted cards for randomness
        let mut rng = rand::thread_rng();
        self.weighted_cards.shuffle(&mut rng);
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
    pub item_to_delete: Option<u64>,         // ID of item to delete
    pub show_image_dialog: bool,
    pub pending_image_side: Option<ImageSide>,
    pub pending_card_id: Option<u64>,
    pub pending_front_image: Option<CardImage>,
    pub pending_back_image: Option<CardImage>,
    pub right_panel_open: bool, // New field for toggling right panel
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
            show_image_dialog: false,
            pending_image_side: None,
            pending_card_id: None,
            pending_front_image: None,
            pending_back_image: None,
            right_panel_open: true, // Default to open
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
                    ui.label(format!(
                        "Are you sure you want to delete this {}?",
                        delete_type
                    ));
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
                                if let (Some(deck_id), Some(card_id)) =
                                    (self.selected_deck_id, self.item_to_delete)
                                {
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

        // Handle image dialog
        if self.show_image_dialog {
            egui::Window::new("Add Image")
                .collapsible(false)
                .resizable(false)
                .show(ui.ctx(), |ui| {
                    ui.label("Add image from:");
                    ui.separator();

                    // Browse files button
                    if ui.button("📁 Browse Files").clicked() {
                        if let Some(path) = crate::image_handler::open_file_dialog() {
                            self.handle_image_addition(decks, path, &mut needs_save);
                        }
                    }

                    ui.add_space(10.0);

                    // Paste from clipboard button
                    if ui.button("📋 Paste from Clipboard (Ctrl+V)").clicked() {
                        self.handle_clipboard_paste(decks, &mut needs_save);
                    }

                    ui.add_space(5.0);
                    ui.label("💡 Take a screenshot and paste it here!");

                    ui.separator();
                    if ui.button("Cancel").clicked() {
                        self.show_image_dialog = false;
                        self.pending_image_side = None;
                        self.pending_card_id = None;
                    }
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
                                ui.label(desc);
                            }
                            ui.label(format!("Cards: {}", deck.cards.len()));
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
                                self.edit_deck_description =
                                    deck.description.clone().unwrap_or_default();
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
            ui.add(
                egui::TextEdit::singleline(&mut self.new_deck_name).hint_text("Enter deck name"),
            );
        });

        ui.horizontal(|ui| {
            ui.label("Description:");
            ui.add(
                egui::TextEdit::singleline(&mut self.new_deck_description)
                    .hint_text("Optional description"),
            );
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
                // Only show the entire panel if toggle is enabled
                if self.right_panel_open {
                    // Use TopBottomPanel to create better layout
                    egui::TopBottomPanel::top("deck_header").show_inside(ui, |ui| {
                        // Header
                        ui.horizontal(|ui| {
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

                        ui.horizontal(|ui| {
                            if ui.button("📷 Add Image to Front").clicked() {
                                self.show_image_dialog = true;
                                self.pending_image_side = Some(ImageSide::Front);
                                self.pending_card_id = None; // For new cards
                            }
                        });

                        ui.label("Back (Answer):");
                        ui.add(egui::TextEdit::multiline(&mut self.new_card_back).desired_rows(3));

                        ui.horizontal(|ui| {
                            if ui.button("📷 Add Image to Back").clicked() {
                                self.show_image_dialog = true;
                                self.pending_image_side = Some(ImageSide::Back);
                                self.pending_card_id = None; // For new cards
                            }
                        });

                        ui.add_space(10.0);

                        if ui.button("Add Card").clicked()
                            && !self.new_card_front.is_empty()
                            && !self.new_card_back.is_empty()
                        {
                            let mut new_card = crate::ui::flashcard::Card::new(
                                deck.id,
                                self.new_card_front.clone(),
                                self.new_card_back.clone(),
                            );

                            // Add pending images if they exist
                            if let Some(front_image) = self.pending_front_image.take() {
                                new_card.front_image = Some(front_image);
                            }
                            if let Some(back_image) = self.pending_back_image.take() {
                                new_card.back_image = Some(back_image);
                            }

                            deck.cards.push(new_card);
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
                                                    ui.label(
                                                        egui::RichText::new("Front:").strong(),
                                                    );
                                                    ui.label(&card.front);
                                                    ui.add_space(5.0);
                                                    ui.label(egui::RichText::new("Back:").strong());
                                                    ui.label(&card.back);
                                                });

                                                ui.with_layout(
                                                    egui::Layout::right_to_left(egui::Align::Min),
                                                    |ui| {
                                                        // Delete card button
                                                        if ui.button("🗑").clicked() {
                                                            self.delete_confirmation =
                                                                Some("card".to_string());
                                                            self.item_to_delete = Some(card.id);
                                                        }

                                                        // Edit card button
                                                        if ui.button("✏").clicked() {
                                                            self.edit_card_id = Some(card.id);
                                                            self.edit_card_front =
                                                                card.front.clone();
                                                            self.edit_card_back = card.back.clone();
                                                        }
                                                    },
                                                );
                                            });
                                        });
                                        ui.add_space(8.0);
                                    }
                                });
                        }
                    });
                } else {
                    // When panel is hidden, just show a simple message
                    ui.separator();
                    ui.centered_and_justified(|ui| {
                        ui.label(
                            "Panel hidden. Management interface collapsed to give more space.",
                        );
                    });
                }
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
                                    if let Some(card) =
                                        deck.cards.iter_mut().find(|c| c.id == edit_id)
                                    {
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

    fn handle_image_addition(
        &mut self,
        decks: &mut Vec<Deck>,
        path: PathBuf,
        needs_save: &mut bool,
    ) {
        let image_manager = ImageManager::new();

        match image_manager.add_image_from_file(&path) {
            Ok(card_image) => {
                self.apply_image_to_card(decks, card_image, needs_save);
            }
            Err(e) => {
                eprintln!("Error loading image: {}", e);
            }
        }
    }

    fn handle_clipboard_paste(&mut self, decks: &mut Vec<Deck>, needs_save: &mut bool) {
        match Clipboard::new() {
            Ok(mut clipboard) => {
                match clipboard.get_image() {
                    Ok(img_data) => {
                        // Convert clipboard image to CardImage
                        match self.clipboard_image_to_card_image(img_data) {
                            Ok(card_image) => {
                                self.apply_image_to_card(decks, card_image, needs_save);
                            }
                            Err(e) => {
                                eprintln!("Error processing clipboard image: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("No image found in clipboard or error: {}", e);
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to access clipboard: {}", e);
            }
        }
    }

    fn clipboard_image_to_card_image(
        &self,
        img_data: arboard::ImageData,
    ) -> Result<CardImage, Box<dyn std::error::Error>> {
        // Convert RGBA data to image format
        let image = image::RgbaImage::from_raw(
            img_data.width as u32,
            img_data.height as u32,
            img_data.bytes.into_owned(),
        )
        .ok_or("Failed to create image from clipboard data")?;

        // Convert to PNG format
        let mut png_data = Vec::new();
        image.write_to(
            &mut std::io::Cursor::new(&mut png_data),
            image::ImageFormat::Png,
        )?;

        // Encode to base64
        let base64_data = base64::engine::general_purpose::STANDARD.encode(&png_data);

        // Generate unique ID
        let id = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;

        Ok(CardImage {
            id: id.to_string(),
            data: base64_data,
            filename: "clipboard_image.png".to_string(),
            mime_type: "image/png".to_string(),
            size: png_data.len(),
        })
    }

    fn apply_image_to_card(
        &mut self,
        decks: &mut Vec<Deck>,
        card_image: CardImage,
        needs_save: &mut bool,
    ) {
        if let Some(deck_id) = self.selected_deck_id {
            if let Some(deck) = decks.iter_mut().find(|d| d.id == deck_id) {
                if let Some(card_id) = self.pending_card_id {
                    // Adding to existing card
                    if let Some(card) = deck.cards.iter_mut().find(|c| c.id == card_id) {
                        match self.pending_image_side.as_ref().unwrap() {
                            ImageSide::Front => card.front_image = Some(card_image),
                            ImageSide::Back => card.back_image = Some(card_image),
                        }
                        *needs_save = true;
                    }
                } else {
                    // Adding to new card being created
                    match self.pending_image_side.as_ref().unwrap() {
                        ImageSide::Front => self.pending_front_image = Some(card_image),
                        ImageSide::Back => self.pending_back_image = Some(card_image),
                    }
                }
            }
        }

        self.show_image_dialog = false;
        self.pending_image_side = None;
        self.pending_card_id = None;
    }
}
