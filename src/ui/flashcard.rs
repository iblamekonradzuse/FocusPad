use crate::image_handler::CardImage;
use chrono::{Local, NaiveDate};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Grade {
    Again,
    Hard,
    Good,
    Easy,
}

impl Grade {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Review {
    pub date: String, // YYYY-MM-DD format
    pub grade: Grade,
    pub interval: u32,
    pub ease_factor: f32,
    pub algorithm_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Card {
    pub id: u64,
    pub deck_id: u64,
    pub front: String,
    pub back: String,
    pub tags: HashSet<String>,
    pub front_image: Option<CardImage>,
    pub back_image: Option<CardImage>,
    pub created_at: String, // ISO date format
    pub reviews: Vec<Review>,
    pub current_interval: u32,
    pub current_ease_factor: f32,
    pub due_date: String, // YYYY-MM-DD format
    pub is_new: bool,
}

impl Card {
    pub fn new(deck_id: u64, front: String, back: String) -> Self {
        let now = Local::now().format("%Y-%m-%d").to_string();
        Card {
            id: 0, // Will be set when added to a deck
            deck_id,
            front,
            back,
            front_image: None,
            back_image: None,
            tags: HashSet::new(),
            created_at: now.clone(),
            reviews: Vec::new(),
            current_interval: 1,
            current_ease_factor: 2.5,
            due_date: now,
            is_new: true,
        }
    }

    pub fn add_review(&mut self, grade: Grade, algorithm_enabled: bool) {
        let now = Local::now().format("%Y-%m-%d").to_string();

        // Apply SM-2 algorithm only if enabled
        let (new_interval, new_ease_factor) = if algorithm_enabled {
            match grade {
                Grade::Again => (1, (self.current_ease_factor - 0.15).max(1.3)),
                Grade::Hard => {
                    let interval = (self.current_interval as f32 * 1.2).round() as u32;
                    (interval.max(1), (self.current_ease_factor - 0.15).max(1.3))
                }
                Grade::Good => {
                    let interval =
                        (self.current_interval as f32 * self.current_ease_factor).round() as u32;
                    (interval.max(1), self.current_ease_factor)
                }
                Grade::Easy => {
                    let interval = (self.current_interval as f32 * self.current_ease_factor * 1.3)
                        .round() as u32;
                    (interval.max(1), (self.current_ease_factor + 0.15).min(2.5))
                }
            }
        } else {
            // When algorithm is disabled, keep cards immediately available
            (0, self.current_ease_factor)
        };

        let review = Review {
            date: now.clone(),
            grade,
            interval: new_interval,
            ease_factor: new_ease_factor,
            algorithm_enabled,
        };

        self.reviews.push(review);
        self.current_interval = new_interval;
        self.current_ease_factor = new_ease_factor;

        // Set due date - if algorithm disabled, make it available today
        self.due_date = if algorithm_enabled {
            NaiveDate::parse_from_str(&now, "%Y-%m-%d")
                .ok()
                .and_then(|d| d.checked_add_days(chrono::Days::new(new_interval as u64)))
                .map(|d| d.format("%Y-%m-%d").to_string())
                .unwrap_or(now.clone())
        } else {
            now.clone() // Always available today when algorithm is off
        };

        self.is_new = false;
    }

    pub fn get_difficulty(&self) -> Grade {
        if self.reviews.is_empty() {
            Grade::Again // New cards are considered "Again"
        } else {
            self.reviews.last().unwrap().grade.clone()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deck {
    pub id: u64,
    pub name: String,
    pub description: Option<String>,
    pub created_at: String, // ISO date format
    pub cards: Vec<Card>,
}

impl Deck {
    pub fn new(name: String, description: Option<String>) -> Self {
        let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        Deck {
            id: 0, // Will be set when added to study data
            name,
            description,
            created_at: now,
            cards: Vec::new(),
        }
    }

    pub fn get_due_cards(&self, algorithm_enabled: bool) -> Vec<&Card> {
        if algorithm_enabled {
            let today = Local::now().format("%Y-%m-%d").to_string();
            self.cards
                .iter()
                .filter(|card| card.due_date <= today)
                .collect()
        } else {
            // When algorithm is disabled, all cards are always available
            self.cards.iter().collect()
        }
    }

    pub fn get_cards_by_difficulty_for_review(
        &self,
        difficulty: &Grade,
        algorithm_enabled: bool,
    ) -> Vec<&Card> {
        let today = Local::now().format("%Y-%m-%d").to_string();
        self.cards
            .iter()
            .filter(|card| {
                let is_due = if algorithm_enabled { card.due_date <= today } else { true };
                is_due && matches!(card.get_difficulty(), d if std::mem::discriminant(&d) == std::mem::discriminant(difficulty))
            })
            .collect()
    }
}
