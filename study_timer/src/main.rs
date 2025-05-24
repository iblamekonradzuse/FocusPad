mod app;
mod data;
mod debug;
mod file_drop_handler;
mod keyboard_handler;
mod settings;
mod split_view_ui;
mod tab_manager;
mod tab_selector_ui;
mod terminal;
mod timer;
mod ui;
use app::StudyTimerApp;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0]) // Increased default size for split view
            .with_min_inner_size([600.0, 450.0])
            .with_drag_and_drop(true), // Enable drag and drop
        ..Default::default()
    };

    eframe::run_native(
        "Study Timer - Enhanced",
        options,
        Box::new(|cc| Box::new(StudyTimerApp::new(cc))),
    )
}
