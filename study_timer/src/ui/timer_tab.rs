use crate::app::StatusMessage;
use crate::data::StudyData;
use crate::debug::DebugTools;
use crate::timer::Timer;
use chrono::Local;
use eframe::egui;
use eframe::egui::Ui;

thread_local! {
    static DESCRIPTION: std::cell::RefCell<String> = std::cell::RefCell::new(String::new());
}

pub fn display(
    ui: &mut Ui,
    timer: &mut Timer,
    study_data: &mut StudyData,
    debug_tools: &mut DebugTools,
    status: &mut StatusMessage,
) {
    let elapsed_minutes = timer.get_elapsed_minutes();
    let hours = (elapsed_minutes as i32) / 60;
    let minutes = (elapsed_minutes as i32) % 60;
    let seconds = ((elapsed_minutes * 60.0) as i32) % 60;

    // Display the timer in large font
    ui.vertical_centered(|ui| {
        ui.add_space(20.0);
        ui.heading(format!("{:02}:{:02}:{:02}", hours, minutes, seconds));
        ui.add_space(30.0);
    });

    // Optional description for the session
    DESCRIPTION.with(|description| {
        let mut description = description.borrow_mut();
        ui.horizontal(|ui| {
            ui.label("Description (optional):");
            ui.text_edit_singleline(&mut *description);
        });
    });

    ui.add_space(10.0);

    // Control buttons
    ui.horizontal(|ui| {
        ui.with_layout(
            egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
            |ui| {
                if timer.is_running {
                    if ui.button("⏸ Pause").clicked() {
                        timer.pause();
                        status.show("Timer paused");
                    }
                } else {
                    if ui.button("▶ Start").clicked() {
                        timer.start();
                        status.show("Timer started");
                    }
                }

                if ui.button("💾 Save").clicked() {
                    let minutes = timer.get_elapsed_minutes();
                    if minutes > 0.0 {
                        let today = Local::now().date_naive().format("%Y-%m-%d").to_string();

                        // Get the description
                        let description = DESCRIPTION.with(|desc| {
                            let desc = desc.borrow();
                            if desc.is_empty() {
                                None
                            } else {
                                Some(desc.clone())
                            }
                        });

                        if let Err(e) = study_data.add_session(today, minutes, description) {
                            status.show(&format!("Error saving: {}", e));
                        } else {
                            status.show(&format!("Saved {:.1} minutes to today's total", minutes));
                            // Reset accumulated time but keep running if it was running
                            let was_running = timer.is_running;
                            timer.reset();
                            if was_running {
                                timer.start();
                            }

                            // Clear description
                            DESCRIPTION.with(|desc| {
                                *desc.borrow_mut() = String::new();
                            });
                        }
                    } else {
                        status.show("No time to save");
                    }
                }

                if ui.button("⏹ Stop").clicked() {
                    let minutes = timer.get_elapsed_minutes();
                    if minutes > 0.0 {
                        let today = Local::now().date_naive().format("%Y-%m-%d").to_string();

                        // Get the description
                        let description = DESCRIPTION.with(|desc| {
                            let desc = desc.borrow();
                            if desc.is_empty() {
                                None
                            } else {
                                Some(desc.clone())
                            }
                        });

                        if let Err(e) = study_data.add_session(today, minutes, description) {
                            status.show(&format!("Error saving: {}", e));
                        } else {
                            status.show(&format!("Saved {:.1} minutes to today's total", minutes));

                            // Clear description
                            DESCRIPTION.with(|desc| {
                                *desc.borrow_mut() = String::new();
                            });
                        }
                    }
                    timer.reset();
                    status.show("Timer stopped and reset");
                }
            },
        );
    });

    ui.add_space(20.0);

    // Today's study time
    let today_minutes = study_data.get_today_minutes();

    ui.vertical_centered(|ui| {
        ui.label("Today's total study time:");
        ui.label(format!(
            "{:.1} minutes ({:.1} hours)",
            today_minutes,
            today_minutes / 60.0
        ));
    });

    ui.add_space(20.0);

    // Debug tools
    if let Some(debug_message) = debug_tools.ui(ui, timer) {
        status.show(&debug_message);
    }

    // Status message
    status.render(ui);
}

