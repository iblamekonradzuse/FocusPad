use crate::app::StatusMessage;
use crate::data::StudyData;
use chrono::{Local, NaiveDate};
use eframe::egui;
use eframe::egui::Ui;

pub struct RecordState {
    pub date: String,
    pub hours: String,
    pub minutes: String,
    pub description: String,
}

impl Default for RecordState {
    fn default() -> Self {
        Self {
            date: Local::now().date_naive().format("%Y-%m-%d").to_string(),
            hours: "0".to_string(),
            minutes: "0".to_string(),
            description: String::new(),
        }
    }
}

thread_local! {
    static RECORD_STATE: std::cell::RefCell<RecordState> = std::cell::RefCell::new(RecordState::default());
}

pub fn display(ui: &mut Ui, study_data: &mut StudyData, status: &mut StatusMessage) {
    ui.heading("Record Study Session");
    ui.add_space(20.0);

    RECORD_STATE.with(|state| {
        let mut state = state.borrow_mut();

        // Date selector
        ui.horizontal(|ui| {
            ui.label("Date:");
            ui.text_edit_singleline(&mut state.date);
            if ui.button("Today").clicked() {
                state.date = Local::now().date_naive().format("%Y-%m-%d").to_string();
            }
        });

        // Validate date format
        let date_valid = NaiveDate::parse_from_str(&state.date, "%Y-%m-%d").is_ok();
        if !date_valid {
            ui.colored_label(egui::Color32::RED, "Invalid date format. Use YYYY-MM-DD.");
        }

        ui.add_space(10.0);

        // Time input
        ui.horizontal(|ui| {
            ui.label("Study time:");

            // Hours
            let hours_width = 60.0;
            ui.add_sized(
                [hours_width, ui.available_height()],
                egui::TextEdit::singleline(&mut state.hours).hint_text("0"),
            );
            ui.label("h");

            // Minutes
            ui.add_sized(
                [hours_width, ui.available_height()],
                egui::TextEdit::singleline(&mut state.minutes).hint_text("0"),
            );
            ui.label("m");
        });

        // Description
        ui.horizontal(|ui| {
            ui.label("Description (optional):");
            ui.text_edit_singleline(&mut state.description);
        });

        ui.add_space(20.0);

        // Submit button
        let hours_valid = state.hours.parse::<f64>().is_ok();
        let minutes_valid = state.minutes.parse::<f64>().is_ok();

        if ui
            .add_enabled(
                date_valid && hours_valid && minutes_valid,
                egui::Button::new("Save Session"),
            )
            .clicked()
        {
            let hours = state.hours.parse::<f64>().unwrap_or(0.0);
            let minutes = state.minutes.parse::<f64>().unwrap_or(0.0);
            let total_minutes = (hours * 60.0) + minutes;

            let description = if state.description.trim().is_empty() {
                None
            } else {
                Some(state.description.clone())
            };

            if total_minutes > 0.0 {
                if let Err(e) =
                    study_data.add_session(state.date.clone(), total_minutes, description)
                {
                    status.show(&format!("Error saving: {}", e));
                } else {
                    status.show(&format!(
                        "Saved {:.1} minutes ({:.1} hours) of study time",
                        total_minutes,
                        total_minutes / 60.0
                    ));

                    // Reset fields except date
                    state.hours = "0".to_string();
                    state.minutes = "0".to_string();
                    state.description.clear();
                }
            } else {
                status.show("Study time must be greater than zero");
            }
        }

        // Status message
        status.render(ui);
    });

    // Display recent sessions
    ui.add_space(20.0);
    ui.heading("Recent Sessions");
    ui.add_space(10.0);

    // Sort sessions by date (newest first)
    let mut sessions = study_data.sessions.clone();
    sessions.sort_by(|a, b| b.date.cmp(&a.date));

    // Take only the last 5 sessions
    let recent_sessions = sessions.into_iter().take(5).collect::<Vec<_>>();

    if recent_sessions.is_empty() {
        ui.label("No sessions recorded yet.");
    } else {
        egui::Grid::new("recent_sessions_grid")
            .num_columns(4)
            .spacing([20.0, 4.0])
            .striped(true)
            .show(ui, |ui| {
                // Header
                ui.strong("Date");
                ui.strong("Minutes");
                ui.strong("Hours");
                ui.strong("Description");
                ui.end_row();

                for session in recent_sessions {
                    ui.label(session.date);
                    ui.label(format!("{:.1}", session.minutes));
                    ui.label(format!("{:.1}", session.minutes / 60.0));
                    ui.label(session.description.unwrap_or_else(|| "-".to_string()));
                    ui.end_row();
                }
            });
    }
}
