use crate::image_handler::ImageManager;
use crate::ui::flashcard::Deck;
use chrono::{Duration, Local, NaiveDate};
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
pub struct Habit {
    pub id: u64,
    pub name: String,
    pub category: String,
    pub created_at: String,
    pub completion_dates: HashSet<String>, // Store dates as "YYYY-MM-DD" strings
    pub target_frequency: HabitFrequency,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HabitFrequency {
    Daily,
    Weekly,
    Custom(u32), // Every N days
}

impl Habit {
    pub fn calculate_current_streak(&self) -> u32 {
        let today = Local::now().date_naive();
        let mut streak = 0;
        let mut current_date = today;

        // Check if today is completed (for ongoing streak)
        let today_str = today.format("%Y-%m-%d").to_string();
        let mut checking_today = true;

        loop {
            let date_str = current_date.format("%Y-%m-%d").to_string();

            if self.completion_dates.contains(&date_str) {
                streak += 1;
                checking_today = false;
            } else if !checking_today {
                // If we've moved past today and hit a gap, break the streak
                break;
            } else {
                // Today isn't completed, but we might have a streak ending yesterday
                checking_today = false;
            }

            // Move to previous day
            current_date = current_date - Duration::days(1);

            // Don't go back more than a reasonable amount (e.g., 1 year)
            if (today - current_date).num_days() > 365 {
                break;
            }
        }

        streak
    }

    pub fn get_completion_rate_last_n_days(&self, days: u32) -> f32 {
        let today = Local::now().date_naive();
        let mut completed_days = 0;

        for i in 0..days {
            let date = today - Duration::days(i as i64);
            let date_str = date.format("%Y-%m-%d").to_string();
            if self.completion_dates.contains(&date_str) {
                completed_days += 1;
            }
        }

        completed_days as f32 / days as f32
    }
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
    pub habits: Vec<Habit>,
    pub reminders: Vec<Reminder>,
    pub decks: Vec<Deck>,
    pub next_deck_id: u64,
    pub image_manager: ImageManager,
}

impl StudyData {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let data_path = Path::new("study_data.json");
        if !data_path.exists() {
            return Ok(StudyData {
                sessions: Vec::new(),
                todos: Vec::new(),
                habits: Vec::new(),
                reminders: Vec::new(),
                decks: Vec::new(),
                image_manager: ImageManager::new(),
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

    // Todo methods
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

    // Habit methods
    pub fn add_habit(
        &mut self,
        name: String,
        category: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let now = Local::now();
        let habit = Habit {
            id: self.get_next_habit_id(),
            name,
            category,
            created_at: now.format("%Y-%m-%d %H:%M:%S").to_string(),
            completion_dates: HashSet::new(),
            target_frequency: HabitFrequency::Daily,
        };

        self.habits.push(habit);
        self.save()?;
        Ok(())
    }

    pub fn mark_habit_complete_today(&mut self, id: u64) -> Result<(), Box<dyn std::error::Error>> {
        let today = Local::now().date_naive().format("%Y-%m-%d").to_string();

        if let Some(habit) = self.habits.iter_mut().find(|h| h.id == id) {
            habit.completion_dates.insert(today);
            self.save()?;
        }
        Ok(())
    }

    pub fn unmark_habit_complete(
        &mut self,
        id: u64,
        date: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(habit) = self.habits.iter_mut().find(|h| h.id == id) {
            habit.completion_dates.remove(&date);
            self.save()?;
        }
        Ok(())
    }

    pub fn delete_habit(&mut self, id: u64) -> Result<(), Box<dyn std::error::Error>> {
        self.habits.retain(|h| h.id != id);
        self.save()?;
        Ok(())
    }

    pub fn clear_completed_habits(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let today = Local::now().date_naive().format("%Y-%m-%d").to_string();
        self.habits.retain(|h| !h.completion_dates.contains(&today));
        self.save()?;
        Ok(())
    }

    pub fn get_habit_categories(&self) -> Vec<String> {
        let mut categories: Vec<String> = self
            .habits
            .iter()
            .map(|h| h.category.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        categories.sort();
        categories
    }

    pub fn get_habits_by_category(&self, category: &str) -> Vec<&Habit> {
        self.habits
            .iter()
            .filter(|h| h.category == category)
            .collect()
    }

    pub fn get_habit_stats(&self, id: u64) -> Option<HabitStats> {
        if let Some(habit) = self.habits.iter().find(|h| h.id == id) {
            let current_streak = habit.calculate_current_streak();
            let total_completions = habit.completion_dates.len();
            let completion_rate_7_days = habit.get_completion_rate_last_n_days(7);
            let completion_rate_30_days = habit.get_completion_rate_last_n_days(30);

            Some(HabitStats {
                current_streak,
                total_completions,
                completion_rate_7_days,
                completion_rate_30_days,
            })
        } else {
            None
        }
    }

    fn get_next_habit_id(&self) -> u64 {
        let todo_max = self.todos.iter().map(|t| t.id).max().unwrap_or(0);
        let habit_max = self.habits.iter().map(|h| h.id).max().unwrap_or(0);
        std::cmp::max(todo_max, habit_max) + 1
    }

    // Reminder methods
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

    pub fn get_due_cards_count(&self) -> usize {
        self.decks
            .iter()
            .flat_map(|deck| deck.get_due_cards(true))
            .count()
    }
}

#[derive(Debug, Clone)]
pub struct HabitStats {
    pub current_streak: u32,
    pub total_completions: usize,
    pub completion_rate_7_days: f32,
    pub completion_rate_30_days: f32,
}

