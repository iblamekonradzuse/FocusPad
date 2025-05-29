use crate::timer::Timer;
use eframe::egui;
use eframe::egui::Ui;

pub struct DebugTools {
    pub enabled: bool,
    pub time_to_add: f64, // Minutes to add
}

impl DebugTools {
    pub fn new() -> Self {
        Self {
            enabled: false,
            time_to_add: 5.0, // Default 5 minutes
        }
    }

    pub fn ui(&mut self, ui: &mut Ui, timer: &mut Timer) -> Option<String> {
        let mut message = None;

        if !self.enabled {
            if ui.button("🐞 Debug Mode").clicked() {
                self.enabled = true;
                message = Some("Debug mode enabled".to_string());
            }
            return message;
        }

        ui.horizontal(|ui| {
            if ui.button("❌ Close Debug").clicked() {
                self.enabled = false;
                message = Some("Debug mode disabled".to_string());
            }
        });

        ui.separator();
        ui.heading("Debug Tools");

        ui.horizontal(|ui| {
            ui.label("Add time (minutes):");
            ui.add(egui::DragValue::new(&mut self.time_to_add).speed(1.0));

            if ui.button("Add Time").clicked() {
                timer.add_time(self.time_to_add);
                message = Some(format!("Added {:.1} minutes to timer", self.time_to_add));
            }
        });

        ui.horizontal(|ui| {
            if ui.button("Add 30 minutes").clicked() {
                timer.add_time(30.0);
                message = Some("Added 30 minutes to timer".to_string());
            }
            if ui.button("Add 1 hour").clicked() {
                timer.add_time(60.0);
                message = Some("Added 1 hour to timer".to_string());
            }
        });

        // Add time manipulation options
        ui.horizontal(|ui| {
            if ui.button("Reset time offset").clicked() {
                timer.time_offset = std::time::Duration::from_secs(0);
                message = Some("Time offset reset to zero".to_string());
            }
        });

        // Show current time offset
        let hours = timer.time_offset.as_secs() / 3600;
        let minutes = (timer.time_offset.as_secs() % 3600) / 60;
        let seconds = timer.time_offset.as_secs() % 60;
        ui.label(format!(
            "Current time offset: {:02}:{:02}:{:02}",
            hours, minutes, seconds
        ));

        message
    }
}

