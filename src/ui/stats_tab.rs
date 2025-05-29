use crate::app::StatusMessage;
use crate::data::{StudyData, StudySession};
use eframe::egui;
use eframe::egui::Ui;
use std::cell::RefCell;

// Add state for session editing
pub struct EditSessionState {
    pub session_index: Option<usize>,
    pub date: String,
    pub minutes: String,
    pub description: String,
    pub show_dialog: bool, // Added to control dialog visibility
}

impl Default for EditSessionState {
    fn default() -> Self {
        Self {
            session_index: None,
            date: String::new(),
            minutes: String::new(),
            description: String::new(),
            show_dialog: false,
        }
    }
}

thread_local! {
    static EDIT_STATE: RefCell<EditSessionState> = RefCell::new(EditSessionState::default());
}

pub fn display(ui: &mut Ui, study_data: &mut StudyData, status: &mut StatusMessage) {
    if study_data.sessions.is_empty() {
        ui.vertical_centered(|ui| {
            ui.add_space(50.0);
            ui.label("No study sessions recorded yet.");
        });
        return;
    }

    // Get stats
    let today_minutes = study_data.get_today_minutes();
    let total_minutes = study_data.get_total_minutes();
    let last_week_minutes = study_data.get_last_n_days_minutes(7);

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

    // Show edit dialog if needed
    EDIT_STATE.with(|state| {
        if state.borrow().show_dialog {
            render_edit_dialog(ui, study_data, status);
        }
    });

    // Show all records in a table
    ui.heading("All Study Sessions");
    ui.add_space(10.0);

    egui::ScrollArea::vertical().show(ui, |ui| {
        // Create header manually
        ui.horizontal(|ui| {
            ui.strong("Date");
            ui.add_space(40.0);
            ui.strong("Minutes");
            ui.add_space(20.0);
            ui.strong("Hours");
            ui.add_space(20.0);
            ui.strong("Description");
            ui.add_space(20.0);
            ui.strong("Actions");
        });
        ui.add_space(4.0);
        ui.separator();
        ui.add_space(4.0);

        // Table contents
        egui::Grid::new("sessions_grid")
            .num_columns(6) // Increased to add action buttons
            .spacing([20.0, 4.0])
            .striped(true)
            .show(ui, |ui| {
                // Sort sessions by date (newest first)
                let mut sessions = study_data.sessions.clone();
                sessions.sort_by(|a, b| b.date.cmp(&a.date));

                for (idx, session) in sessions.iter().enumerate() {
                    ui.label(&session.date);
                    ui.label(format!("{:.1}", session.minutes));
                    ui.label(format!("{:.1}", session.minutes / 60.0));
                    ui.label(
                        session
                            .description
                            .clone()
                            .unwrap_or_else(|| "-".to_string()),
                    );

                    // Add edit and delete buttons
                    ui.horizontal(|ui| {
                        if ui.button("Edit").clicked() {
                            EDIT_STATE.with(|state| {
                                let mut state = state.borrow_mut();
                                state.session_index = Some(idx);
                                state.date = session.date.clone();
                                state.minutes = format!("{}", session.minutes);
                                state.description = session.description.clone().unwrap_or_default();
                                state.show_dialog = true;
                            });
                        }

                        if ui.button("Delete").clicked() {
                            if let Some(real_idx) = find_session_index(study_data, idx, &sessions) {
                                study_data.sessions.remove(real_idx);
                                if let Err(e) = study_data.save() {
                                    status.show(&format!("Error saving: {}", e));
                                } else {
                                    status.show("Session deleted");
                                }
                            }
                        }
                    });

                    ui.end_row();
                }
            });
    });

    // Status message
    status.render(ui);
}

fn find_session_index(
    study_data: &StudyData,
    display_idx: usize,
    sorted_sessions: &[StudySession],
) -> Option<usize> {
    if display_idx >= sorted_sessions.len() {
        return None;
    }

    let target_session = &sorted_sessions[display_idx];

    // Find the actual index in the original vector
    study_data.sessions.iter().position(|s| {
        s.date == target_session.date
            && (s.minutes - target_session.minutes).abs() < f64::EPSILON
            && s.description == target_session.description
    })
}

fn render_edit_dialog(ui: &mut Ui, study_data: &mut StudyData, status: &mut StatusMessage) {
    let mut keep_open = true;

    egui::Window::new("Edit Study Session")
        .collapsible(false)
        .resizable(false)
        .open(&mut keep_open)
        .show(ui.ctx(), |ui| {
            let mut save_clicked = false;
            let mut cancel_clicked = false;

            EDIT_STATE.with(|state| {
                let mut state = state.borrow_mut();

                // Date
                ui.horizontal(|ui| {
                    ui.label("Date:");
                    ui.text_edit_singleline(&mut state.date);
                });

                // Time
                ui.horizontal(|ui| {
                    ui.label("Minutes:");
                    ui.text_edit_singleline(&mut state.minutes);
                });

                // Description
                ui.horizontal(|ui| {
                    ui.label("Description:");
                    ui.text_edit_singleline(&mut state.description);
                });

                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    save_clicked = ui.button("Save").clicked();
                    cancel_clicked = ui.button("Cancel").clicked();
                });
            });

            // Handle button clicks outside of the EDIT_STATE borrow
            if save_clicked {
                // Save changes
                EDIT_STATE.with(|state| {
                    let state = state.borrow();
                    if let Some(idx) = state.session_index {
                        // Get sorted sessions
                        let mut sorted_sessions = study_data.sessions.clone();
                        sorted_sessions.sort_by(|a, b| b.date.cmp(&a.date));

                        if let Some(real_idx) =
                            find_session_index(study_data, idx, &sorted_sessions)
                        {
                            // Parse minutes
                            if let Ok(minutes) = state.minutes.parse::<f64>() {
                                if minutes > 0.0 {
                                    let description = if state.description.trim().is_empty() {
                                        None
                                    } else {
                                        Some(state.description.clone())
                                    };

                                    // Update session
                                    study_data.sessions[real_idx].date = state.date.clone();
                                    study_data.sessions[real_idx].minutes = minutes;
                                    study_data.sessions[real_idx].description = description;

                                    // Save data
                                    if let Err(e) = study_data.save() {
                                        status.show(&format!("Error saving: {}", e));
                                    } else {
                                        status.show("Session updated successfully");
                                    }
                                } else {
                                    status.show("Minutes must be greater than zero");
                                }
                            } else {
                                status.show("Invalid minutes value");
                            }
                        }
                    }
                });

                // Close dialog
                EDIT_STATE.with(|state| {
                    let mut state = state.borrow_mut();
                    state.show_dialog = false;
                    state.session_index = None;
                });
            }

            if cancel_clicked {
                // Cancel edit
                EDIT_STATE.with(|state| {
                    let mut state = state.borrow_mut();
                    state.show_dialog = false;
                    state.session_index = None;
                });
            }
        });

    // If window was closed (X button)
    if !keep_open {
        EDIT_STATE.with(|state| {
            let mut state = state.borrow_mut();
            state.show_dialog = false;
            state.session_index = None;
        });
    }
}

