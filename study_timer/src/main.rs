mod app;
mod data;
mod debug;
mod terminal;
mod timer;
mod ui;

use app::StudyTimerApp;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([400.0, 550.0])
            .with_min_inner_size([300.0, 450.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Study Timer",
        options,
        Box::new(|cc| Box::new(StudyTimerApp::new(cc))),
    )
}

