use chrono::{Local, NaiveDate};
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;

// Define the study data structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudySession {
    pub date: String, // YYYY-MM-DD format
    pub minutes: f64,
    pub description: Option<String>, // Optional description of the study session
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StudyData {
    pub sessions: Vec<StudySession>,
    pub todos: Vec<Todo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Todo {
    pub id: u64,
    pub text: String,
    pub completed: bool,
    pub created_at: String, // ISO date format
}

impl StudyData {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let data_path = Path::new("study_data.json");

        if !data_path.exists() {
            return Ok(StudyData {
                sessions: Vec::new(),
                todos: Vec::new(), // Added missing todos field
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

        // Check if there's already a session for this date with the same description
        // If description is None, combine with any existing session for that date with no description
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
            completed = todo.completed; // Store completion state before saving
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
}

