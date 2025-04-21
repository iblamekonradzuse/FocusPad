use crate::data::StudyData;
use eframe::egui;
use eframe::egui::Ui;

pub fn display(ui: &mut Ui, study_data: &StudyData) {
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
        });
        ui.add_space(4.0);
        ui.separator();
        ui.add_space(4.0);

        // Table contents
        egui::Grid::new("sessions_grid")
            .num_columns(4)
            .spacing([20.0, 4.0])
            .striped(true)
            .show(ui, |ui| {
                // Sort sessions by date (newest first)
                let mut sessions = study_data.sessions.clone();
                sessions.sort_by(|a, b| b.date.cmp(&a.date));

                for session in sessions {
                    ui.label(session.date);
                    ui.label(format!("{:.1}", session.minutes));
                    ui.label(format!("{:.1}", session.minutes / 60.0));
                    ui.label(session.description.unwrap_or_else(|| "-".to_string()));
                    ui.end_row();
                }
            });
    });
}

