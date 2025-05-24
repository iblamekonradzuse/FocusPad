use crate::ui::flashcard::{Card, Deck, Grade};
use chrono::{Local, NaiveDate};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudySession {
    pub date: String,
    pub minutes: f64,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Todo {
    pub id: u64,
    pub text: String,
    pub completed: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reminder {
    pub id: u64,
    pub title: String,
    pub description: Option<String>,
    pub due_date: String,
    pub created_at: String,
    pub notification_periods: Vec<NotificationPeriod>,
    pub is_completed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationPeriod {
    OneDay,
    ThreeDays,
    OneWeek,
    Custom(u32),
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StudyData {
    pub sessions: Vec<StudySession>,
    pub todos: Vec<Todo>,
    pub reminders: Vec<Reminder>,
    pub decks: Vec<Deck>,
    pub next_deck_id: u64,
}

impl StudyData {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let data_path = Path::new("study_data.json");
        if !data_path.exists() {
            return Ok(StudyData {
                sessions: Vec::new(),
                todos: Vec::new(),
                reminders: Vec::new(),
                decks: Vec::new(),
                next_deck_id: 1,
            });
        }

        let mut file = File::open(data_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let data: StudyData = serde_json::from_str(&contents)?;
        Ok(data)
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(&self)?;
        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open("study_data.json")?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }

    pub fn add_session(
        &mut self,
        date: String,
        minutes: f64,
        description: Option<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if minutes <= 0.0 {
            return Ok(());
        }

        if let Some(description) = &description {
            if let Some(session) = self
                .sessions
                .iter_mut()
                .find(|s| s.date == date && s.description.as_ref() == Some(description))
            {
                session.minutes += minutes;
            } else {
                self.sessions.push(StudySession {
                    date,
                    minutes,
                    description: Some(description.clone()),
                });
            }
        } else {
            if let Some(session) = self
                .sessions
                .iter_mut()
                .find(|s| s.date == date && s.description.is_none())
            {
                session.minutes += minutes;
            } else {
                self.sessions.push(StudySession {
                    date,
                    minutes,
                    description: None,
                });
            }
        }

        self.save()?;
        Ok(())
    }

    pub fn get_today_minutes(&self) -> f64 {
        let today = Local::now().date_naive().format("%Y-%m-%d").to_string();
        self.sessions
            .iter()
            .filter(|s| s.date == today)
            .map(|s| s.minutes)
            .sum()
    }

    pub fn get_total_minutes(&self) -> f64 {
        self.sessions.iter().map(|s| s.minutes).sum()
    }

    pub fn get_last_n_days_minutes(&self, days: i64) -> f64 {
        let today = Local::now().date_naive();
        self.sessions
            .iter()
            .filter_map(|s| {
                if let Ok(date) = NaiveDate::parse_from_str(&s.date, "%Y-%m-%d") {
                    if (today - date).num_days() < days {
                        return Some(s.minutes);
                    }
                }
                None
            })
            .sum()
    }

    pub fn add_todo(&mut self, text: String) -> Result<(), Box<dyn std::error::Error>> {
        let now = Local::now();
        let todo = Todo {
            id: self.get_next_todo_id(),
            text,
            completed: false,
            created_at: now.format("%Y-%m-%d %H:%M:%S").to_string(),
        };

        self.todos.push(todo);
        self.save()?;
        Ok(())
    }

    pub fn toggle_todo(&mut self, id: u64) -> Result<bool, Box<dyn std::error::Error>> {
        let mut completed = false;
        if let Some(todo) = self.todos.iter_mut().find(|t| t.id == id) {
            todo.completed = !todo.completed;
            completed = todo.completed;
        }
        self.save()?;
        Ok(completed)
    }

    pub fn update_todo_text(
        &mut self,
        id: u64,
        text: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(todo) = self.todos.iter_mut().find(|t| t.id == id) {
            todo.text = text;
            self.save()?;
        }
        Ok(())
    }

    pub fn delete_todo(&mut self, id: u64) -> Result<(), Box<dyn std::error::Error>> {
        self.todos.retain(|t| t.id != id);
        self.save()?;
        Ok(())
    }

    pub fn clear_todos(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.todos.clear();
        self.save()?;
        Ok(())
    }

    pub fn clear_completed_todos(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.todos.retain(|t| !t.completed);
        self.save()?;
        Ok(())
    }

    fn get_next_todo_id(&self) -> u64 {
        if let Some(max_id) = self.todos.iter().map(|t| t.id).max() {
            max_id + 1
        } else {
            1
        }
    }

    pub fn add_reminder(
        &mut self,
        title: String,
        description: Option<String>,
        due_date: String,
        notification_periods: Vec<NotificationPeriod>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let now = Local::now();
        let reminder = Reminder {
            id: self.get_next_reminder_id(),
            title,
            description,
            due_date,
            created_at: now.format("%Y-%m-%d %H:%M:%S").to_string(),
            notification_periods,
            is_completed: false,
        };

        self.reminders.push(reminder);
        self.save()?;
        Ok(())
    }

    pub fn update_reminder(
        &mut self,
        id: u64,
        title: String,
        description: Option<String>,
        due_date: String,
        notification_periods: Vec<NotificationPeriod>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(reminder) = self.reminders.iter_mut().find(|r| r.id == id) {
            reminder.title = title;
            reminder.description = description;
            reminder.due_date = due_date;
            reminder.notification_periods = notification_periods;
            self.save()?;
        }
        Ok(())
    }

    pub fn toggle_reminder(&mut self, id: u64) -> Result<bool, Box<dyn std::error::Error>> {
        let mut completed = false;
        if let Some(reminder) = self.reminders.iter_mut().find(|r| r.id == id) {
            reminder.is_completed = !reminder.is_completed;
            completed = reminder.is_completed;
        }
        self.save()?;
        Ok(completed)
    }

    pub fn delete_reminder(&mut self, id: u64) -> Result<(), Box<dyn std::error::Error>> {
        self.reminders.retain(|r| r.id != id);
        self.save()?;
        Ok(())
    }

    pub fn clear_reminders(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.reminders.clear();
        self.save()?;
        Ok(())
    }

    pub fn clear_completed_reminders(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.reminders.retain(|r| !r.is_completed);
        self.save()?;
        Ok(())
    }

    fn get_next_reminder_id(&self) -> u64 {
        if let Some(max_id) = self.reminders.iter().map(|r| r.id).max() {
            max_id + 1
        } else {
            1
        }
    }

    pub fn add_deck(
        &mut self,
        name: String,
        description: Option<String>,
    ) -> Result<u64, Box<dyn std::error::Error>> {
        let deck_id = self.next_deck_id;
        let mut deck = Deck::new(name, description);
        deck.id = deck_id;
        self.decks.push(deck);
        self.next_deck_id += 1;
        self.save()?;
        Ok(deck_id)
    }

    pub fn get_deck(&mut self, deck_id: u64) -> Option<&mut Deck> {
        self.decks.iter_mut().find(|d| d.id == deck_id)
    }

    pub fn delete_deck(&mut self, deck_id: u64) -> Result<(), Box<dyn std::error::Error>> {
        self.decks.retain(|d| d.id != deck_id);
        self.save()?;
        Ok(())
    }

    pub fn get_due_cards_count(&self) -> usize {
        self.decks
            .iter()
            .flat_map(|deck| deck.get_due_cards())
            .count()
    }
}

