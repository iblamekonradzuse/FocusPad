use chrono::{Local, NaiveDate};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Grade {
    Again,
    Hard,
    Good,
    Easy,
}

impl Grade {
    pub fn value(&self) -> u8 {
        match self {
            Grade::Again => 0,
            Grade::Hard => 1,
            Grade::Good => 2,
            Grade::Easy => 3,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Review {
    pub date: String, // YYYY-MM-DD format
    pub grade: Grade,
    pub interval: u32,
    pub ease_factor: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Card {
    pub id: u64,
    pub deck_id: u64,
    pub front: String,
    pub back: String,
    pub tags: HashSet<String>,
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
            tags: HashSet::new(),
            created_at: now.clone(),
            reviews: Vec::new(),
            current_interval: 1,
            current_ease_factor: 2.5,
            due_date: now,
            is_new: true,
        }
    }

    pub fn add_review(&mut self, grade: Grade) {
        let now = Local::now().format("%Y-%m-%d").to_string();

        // Apply SM-2 algorithm
        let (new_interval, new_ease_factor) = match grade {
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
                let interval =
                    (self.current_interval as f32 * self.current_ease_factor * 1.3).round() as u32;
                (interval.max(1), (self.current_ease_factor + 0.15).min(2.5))
            }
        };

        let review = Review {
            date: now.clone(),
            grade,
            interval: new_interval,
            ease_factor: new_ease_factor,
        };

        self.reviews.push(review);
        self.current_interval = new_interval;
        self.current_ease_factor = new_ease_factor;
        self.due_date = NaiveDate::parse_from_str(&now, "%Y-%m-%d")
            .ok()
            .and_then(|d| d.checked_add_days(chrono::Days::new(new_interval as u64)))
            .map(|d| d.format("%Y-%m-%d").to_string())
            .unwrap_or(now);
        self.is_new = false;
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

    pub fn add_card(&mut self, front: String, back: String) -> u64 {
        let card_id = self.get_next_card_id();
        let mut card = Card::new(self.id, front, back);
        card.id = card_id;
        self.cards.push(card);
        card_id
    }

    pub fn get_due_cards(&self) -> Vec<&Card> {
        let today = Local::now().format("%Y-%m-%d").to_string();
        self.cards
            .iter()
            .filter(|card| card.due_date <= today)
            .collect()
    }

    pub fn get_card(&mut self, card_id: u64) -> Option<&mut Card> {
        self.cards.iter_mut().find(|c| c.id == card_id)
    }

    fn get_next_card_id(&self) -> u64 {
        if let Some(max_id) = self.cards.iter().map(|c| c.id).max() {
            max_id + 1
        } else {
            1
        }
    }
}

