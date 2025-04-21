use chrono::{Local, NaiveDate};
use eframe::{egui, CreationContext};
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;
use std::time::{Duration, Instant};

// Define the study data structures
#[derive(Debug, Clone, Serialize, Deserialize)]
struct StudySession {
    date: String, // YYYY-MM-DD format
    minutes: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct StudyData {
    sessions: Vec<StudySession>,
}

struct Timer {
    start_time: Option<Instant>,
    accumulated_time: Duration,
    is_running: bool,
}

impl Timer {
    fn new() -> Self {
        Timer {
            start_time: None,
            accumulated_time: Duration::from_secs(0),
            is_running: false,
        }
    }

    fn start(&mut self) {
        if !self.is_running {
            self.start_time = Some(Instant::now());
            self.is_running = true;
        }
    }

    fn pause(&mut self) {
        if self.is_running {
            if let Some(start) = self.start_time {
                self.accumulated_time += start.elapsed();
                self.start_time = None;
                self.is_running = false;
            }
        }
    }

    fn reset(&mut self) {
        self.start_time = None;
        self.accumulated_time = Duration::from_secs(0);
        self.is_running = false;
    }

    fn get_elapsed_time(&self) -> Duration {
        if self.is_running {
            if let Some(start) = self.start_time {
                self.accumulated_time + start.elapsed()
            } else {
                self.accumulated_time
            }
        } else {
            self.accumulated_time
        }
    }

    fn get_elapsed_minutes(&self) -> f64 {
        self.get_elapsed_time().as_secs_f64() / 60.0
    }
}

struct StudyTimerApp {
    timer: Timer,
    study_data: StudyData,
    current_tab: Tab,
    status_message: String,
    status_time: Option<Instant>,
}

enum Tab {
    Timer,
    Stats,
}

impl StudyTimerApp {
    fn new(_cc: &CreationContext<'_>) -> Self {
        let study_data = Self::load_study_data().unwrap_or_default();

        Self {
            timer: Timer::new(),
            study_data,
            current_tab: Tab::Timer,
            status_message: String::new(),
            status_time: None,
        }
    }

    fn load_study_data() -> Result<StudyData, Box<dyn std::error::Error>> {
        let data_path = Path::new("study_data.json");

        if !data_path.exists() {
            return Ok(StudyData {
                sessions: Vec::new(),
            });
        }

        let mut file = File::open(data_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let data: StudyData = serde_json::from_str(&contents)?;
        Ok(data)
    }

    fn save_study_data(&self) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(&self.study_data)?;
        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open("study_data.json")?;

        file.write_all(json.as_bytes())?;
        Ok(())
    }

    fn add_study_session(&mut self, minutes: f64) -> Result<(), Box<dyn std::error::Error>> {
        if minutes <= 0.0 {
            return Ok(());
        }

        let today = Local::now().date_naive().format("%Y-%m-%d").to_string();

        // Check if there's already a session for today
        if let Some(session) = self
            .study_data
            .sessions
            .iter_mut()
            .find(|s| s.date == today)
        {
            session.minutes += minutes;
        } else {
            self.study_data.sessions.push(StudySession {
                date: today,
                minutes,
            });
        }

        self.save_study_data()?;
        self.show_status(format!("Saved {:.1} minutes to today's total", minutes));
        Ok(())
    }

    fn display_timer_tab(&mut self, ui: &mut egui::Ui) {
        let elapsed_minutes = self.timer.get_elapsed_minutes();
        let hours = (elapsed_minutes as i32) / 60;
        let minutes = (elapsed_minutes as i32) % 60;
        let seconds = ((elapsed_minutes * 60.0) as i32) % 60;

        // Display the timer in large font
        ui.vertical_centered(|ui| {
            ui.add_space(20.0);
            ui.heading(format!("{:02}:{:02}:{:02}", hours, minutes, seconds));
            ui.add_space(30.0);
        });

        // Control buttons
        ui.horizontal(|ui| {
            ui.with_layout(
                egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                |ui| {
                    if self.timer.is_running {
                        if ui.button("⏸ Pause").clicked() {
                            self.timer.pause();
                            self.show_status("Timer paused".to_string());
                        }
                    } else {
                        if ui.button("▶ Start").clicked() {
                            self.timer.start();
                            self.show_status("Timer started".to_string());
                        }
                    }

                    if ui.button("💾 Save").clicked() {
                        let minutes = self.timer.get_elapsed_minutes();
                        if minutes > 0.0 {
                            if let Err(e) = self.add_study_session(minutes) {
                                self.show_status(format!("Error saving: {}", e));
                            } else {
                                // Reset accumulated time but keep running if it was running
                                let was_running = self.timer.is_running;
                                self.timer.reset();
                                if was_running {
                                    self.timer.start();
                                }
                            }
                        } else {
                            self.show_status("No time to save".to_string());
                        }
                    }

                    if ui.button("⏹ Stop").clicked() {
                        let minutes = self.timer.get_elapsed_minutes();
                        if minutes > 0.0 {
                            if let Err(e) = self.add_study_session(minutes) {
                                self.show_status(format!("Error saving: {}", e));
                            }
                        }
                        self.timer.reset();
                        self.show_status("Timer stopped and reset".to_string());
                    }
                },
            );
        });

        ui.add_space(20.0);

        // Today's study time
        let today = Local::now().date_naive().format("%Y-%m-%d").to_string();
        let today_minutes = self
            .study_data
            .sessions
            .iter()
            .find(|s| s.date == today)
            .map(|s| s.minutes)
            .unwrap_or(0.0);

        ui.vertical_centered(|ui| {
            ui.label("Today's total study time:");
            ui.label(format!(
                "{:.1} minutes ({:.1} hours)",
                today_minutes,
                today_minutes / 60.0
            ));
        });

        // Status message
        if let Some(status_time) = self.status_time {
            if status_time.elapsed().as_secs() < 5 && !self.status_message.is_empty() {
                ui.add_space(20.0);
                ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                    ui.label(egui::RichText::new(&self.status_message).italics());
                });
            } else {
                self.status_message.clear();
                self.status_time = None;
            }
        }
    }

    fn display_stats_tab(&mut self, ui: &mut egui::Ui) {
        if self.study_data.sessions.is_empty() {
            ui.vertical_centered(|ui| {
                ui.add_space(50.0);
                ui.label("No study sessions recorded yet.");
            });
            return;
        }

        // Today's stats
        let today = Local::now().date_naive().format("%Y-%m-%d").to_string();
        let today_minutes = self
            .study_data
            .sessions
            .iter()
            .find(|s| s.date == today)
            .map(|s| s.minutes)
            .unwrap_or(0.0);

        // Total stats
        let total_minutes: f64 = self.study_data.sessions.iter().map(|s| s.minutes).sum();

        // Last 7 days
        let today_date = NaiveDate::parse_from_str(&today, "%Y-%m-%d").unwrap();
        let last_week_minutes: f64 = self
            .study_data
            .sessions
            .iter()
            .filter_map(|s| {
                if let Ok(date) = NaiveDate::parse_from_str(&s.date, "%Y-%m-%d") {
                    if (today_date - date).num_days() < 7 {
                        return Some(s.minutes);
                    }
                }
                None
            })
            .sum();

        // Display summary
        ui.heading("Study Statistics");
        ui.add_space(10.0);

        egui::Grid::new("stats_grid")
            .num_columns(2)
            .spacing([40.0, 4.0])
            .striped(true)
            .show(ui, |ui| {
                ui.label("Today:");
                ui.label(format!(
                    "{:.1} minutes ({:.1} hours)",
                    today_minutes,
                    today_minutes / 60.0
                ));
                ui.end_row();

                ui.label("Last 7 days:");
                ui.label(format!(
                    "{:.1} minutes ({:.1} hours)",
                    last_week_minutes,
                    last_week_minutes / 60.0
                ));
                ui.end_row();

                ui.label("Total:");
                ui.label(format!(
                    "{:.1} minutes ({:.1} hours)",
                    total_minutes,
                    total_minutes / 60.0
                ));
                ui.end_row();
            });

        ui.add_space(20.0);

        // Show all records in a table
        ui.heading("All Study Sessions");
        ui.add_space(10.0);

        egui::ScrollArea::vertical().show(ui, |ui| {
            // Create header manually
            ui.horizontal(|ui| {
                ui.strong("Date");
                ui.add_space(50.0); // Add some space between columns
                ui.strong("Minutes");
                ui.add_space(30.0);
                ui.strong("Hours");
            });
            ui.add_space(4.0);
            ui.separator();
            ui.add_space(4.0);

            // Table contents
            egui::Grid::new("sessions_grid")
                .num_columns(3)
                .spacing([20.0, 4.0])
                .striped(true)
                .show(ui, |ui| {
                    // Sort sessions by date (newest first)
                    let mut sessions = self.study_data.sessions.clone();
                    sessions.sort_by(|a, b| b.date.cmp(&a.date));

                    for session in sessions {
                        ui.label(session.date);
                        ui.label(format!("{:.1}", session.minutes));
                        ui.label(format!("{:.1}", session.minutes / 60.0));
                        ui.end_row();
                    }
                });
        });
    }

    fn show_status(&mut self, message: String) {
        self.status_message = message;
        self.status_time = Some(Instant::now());
    }
}

impl eframe::App for StudyTimerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Request a repaint frequently if the timer is running
        if self.timer.is_running {
            ctx.request_repaint();
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui
                    .selectable_label(matches!(self.current_tab, Tab::Timer), "⏱ Timer")
                    .clicked()
                {
                    self.current_tab = Tab::Timer;
                }

                if ui
                    .selectable_label(matches!(self.current_tab, Tab::Stats), "📊 Statistics")
                    .clicked()
                {
                    self.current_tab = Tab::Stats;
                }
            });

            ui.separator();

            match self.current_tab {
                Tab::Timer => self.display_timer_tab(ui),
                Tab::Stats => self.display_stats_tab(ui),
            }
        });
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(400.0, 500.0)),
        min_window_size: Some(egui::vec2(300.0, 400.0)),
        ..Default::default()
    };

    eframe::run_native(
        "Study Timer",
        options,
        Box::new(|cc| Box::new(StudyTimerApp::new(cc))),
    )
}

