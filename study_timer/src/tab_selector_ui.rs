use crate::app::{StatusMessage, Tab};
use crate::settings::AppSettings;
use eframe::egui;

#[derive(Debug, Clone)]
pub struct TabSelectorUI {
    pub selected_tab: Option<Tab>,
    pub is_open: bool,
}

impl TabSelectorUI {
    pub fn new() -> Self {
        Self {
            selected_tab: None,
            is_open: false,
        }
    }

    pub fn show(&mut self) {
        self.is_open = true;
        self.selected_tab = None;
    }

    pub fn hide(&mut self) {
        self.is_open = false;
        self.selected_tab = None;
    }

    pub fn display(
        &mut self,
        ctx: &egui::Context,
        settings: &AppSettings,
        _status: &mut StatusMessage,
    ) -> Option<Tab> {
        if !self.is_open {
            return None;
        }

        let mut selected_tab = None;

        egui::Window::new("📑 New Tab")
            .collapsible(false)
            .resizable(false)
            .default_width(400.0)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .show(ctx, |ui| {
                ui.heading("Select a tab type to create:");
                ui.add_space(15.0);

                let available_tabs = settings.get_enabled_tabs();
                let colors = settings.get_current_colors();

                // Create a grid of tab options
                let columns = 3;
                let mut current_column = 0;

                ui.horizontal_wrapped(|ui| {
                    for tab_config in available_tabs {
                        let tab_type = &tab_config.tab_type;
                        let display_name = tab_config.get_display_name();
                        let icon = get_tab_icon(tab_type);
                        let description = get_tab_description(tab_type);

                        let button_size = egui::Vec2::new(120.0, 80.0);

                        // Create a custom button with centered content
                        let (rect, response) =
                            ui.allocate_exact_size(button_size, egui::Sense::click());

                        let visuals = ui.style().interact(&response);
                        let fill_color = if response.hovered() {
                            colors.accent_color32().linear_multiply(0.8)
                        } else {
                            colors.panel_background_color32()
                        };

                        ui.painter()
                            .rect_filled(rect, egui::Rounding::same(4.0), fill_color);

                        ui.painter().rect_stroke(
                            rect,
                            egui::Rounding::same(4.0),
                            egui::Stroke::new(1.0, colors.accent_color32()),
                        );

                        // Draw centered emoji
                        let emoji_rect = egui::Rect::from_center_size(
                            rect.center() - egui::Vec2::new(0.0, 12.0),
                            egui::Vec2::new(32.0, 32.0),
                        );
                        ui.painter().text(
                            emoji_rect.center(),
                            egui::Align2::CENTER_CENTER,
                            icon,
                            egui::FontId::proportional(24.0),
                            visuals.text_color(),
                        );

                        // Draw centered text
                        let text_rect = egui::Rect::from_center_size(
                            rect.center() + egui::Vec2::new(0.0, 15.0),
                            egui::Vec2::new(110.0, 20.0),
                        );
                        ui.painter().text(
                            text_rect.center(),
                            egui::Align2::CENTER_CENTER,
                            display_name,
                            egui::FontId::proportional(11.0),
                            visuals.text_color(),
                        );

                        if response.clicked() {
                            selected_tab = Some(tab_type.clone());
                            self.hide();
                        }

                        // Add tooltip with description
                        if response.hovered() {
                            egui::show_tooltip(
                                ctx,
                                egui::Id::new(format!("tab_tooltip_{:?}", tab_type)),
                                |ui| {
                                    ui.label(description);
                                },
                            );
                        }

                        current_column += 1;
                        if current_column >= columns {
                            ui.end_row();
                            current_column = 0;
                        }
                    }
                });

                ui.add_space(20.0);
                ui.separator();
                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    if ui.button("❌ Cancel").clicked() {
                        self.hide();
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label("💡 Tip: Use Cmd/Ctrl+T to quickly create new tabs");
                    });
                });
            });

        selected_tab
    }
}

fn get_tab_icon(tab_type: &Tab) -> &'static str {
    match tab_type {
        Tab::Timer => "⏰",
        Tab::Stats => "📊",
        Tab::Record => "📝",
        Tab::Graph => "📈",
        Tab::Todo => "✅",
        Tab::Calculator => "🔢",
        Tab::Flashcards => "🃏",
        Tab::Markdown => "📄",
        Tab::Reminder => "🔔",
        Tab::Terminal => "💻",
        Tab::Settings => "⚙",
    }
}

fn get_tab_description(tab_type: &Tab) -> &'static str {
    match tab_type {
        Tab::Timer => "Focus timer with pomodoro technique support",
        Tab::Stats => "View your study statistics and progress",
        Tab::Record => "Record and manage study sessions",
        Tab::Graph => "Visualize your study data with charts",
        Tab::Todo => "Manage tasks, to-do items and Habits",
        Tab::Flashcards => "Anki like flashcards",
        Tab::Calculator => "Built-in calculator for quick calculations",
        Tab::Markdown => "Write and edit markdown documents",
        Tab::Reminder => "Set reminders and notifications",
        Tab::Terminal => "Built-in terminal emulator",
        Tab::Settings => "Configure application settings",
    }
}
