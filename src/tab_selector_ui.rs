use crate::app::{StatusMessage, Tab};
use crate::settings::AppSettings;
use eframe::egui;

#[derive(Debug, Clone)]
pub struct TabSelectorUI {
    pub selected_tab: Option<Tab>,
    pub is_open: bool,
    pub search_text: String,
    pub filtered_tabs: Vec<Tab>,
}

impl TabSelectorUI {
    pub fn new() -> Self {
        Self {
            selected_tab: None,
            is_open: false,
            search_text: String::new(),
            filtered_tabs: Vec::new(),
        }
    }

    pub fn show(&mut self) {
        self.is_open = true;
        self.selected_tab = None;
        self.search_text.clear();
        self.filtered_tabs.clear();
    }

    pub fn hide(&mut self) {
        self.is_open = false;
        self.selected_tab = None;
        self.search_text.clear();
        self.filtered_tabs.clear();
    }

    fn update_filtered_tabs(&mut self, available_tabs: &[&crate::settings::TabConfig]) {
        if self.search_text.is_empty() {
            self.filtered_tabs = available_tabs.iter().map(|config| config.tab_type.clone()).collect();
        } else {
            self.filtered_tabs = available_tabs
                .iter()
                .filter(|config| {
                    let search_lower = self.search_text.to_lowercase();
                    let tab_name = get_tab_search_name(&config.tab_type).to_lowercase();
                    let display_name = config.get_display_name().to_lowercase();
                    
                    // Check if search text matches beginning of tab name or display name
                    tab_name.starts_with(&search_lower) || 
                    display_name.starts_with(&search_lower) ||
                    // Also check if any word in display name starts with search text
                    display_name.split_whitespace().any(|word| word.starts_with(&search_lower))
                })
                .map(|config| config.tab_type.clone())
                .collect();
        }
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
                ui.heading("Create New Tab");
                ui.add_space(10.0);

                // Search input
                ui.horizontal(|ui| {
                    ui.label("🔍 Type to search:");
                    let search_response = ui.add(
                        egui::TextEdit::singleline(&mut self.search_text)
                            .hint_text("e.g., 'ti' for Timer, 'to' for Todo...")
                            .desired_width(200.0)
                    );

                    // Auto-focus the search box when dialog opens
                    if !self.search_text.is_empty() || ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
                        // Keep focus unless escape is pressed
                        if !ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
                            search_response.request_focus();
                        }
                    } else {
                        search_response.request_focus();
                    }
                });

                let available_tabs = settings.get_enabled_tabs();
                self.update_filtered_tabs(&available_tabs);

                // Handle Enter key to select first filtered result
                if ctx.input(|i| i.key_pressed(egui::Key::Enter)) && !self.filtered_tabs.is_empty() {
                    selected_tab = Some(self.filtered_tabs[0].clone());
                    self.hide();
                }

                // Handle Escape key to close dialog
                if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
                    self.hide();
                }

                ui.add_space(15.0);

                let colors = settings.get_current_colors();

                // Fixed height container to maintain consistent dialog size
                ui.allocate_ui_with_layout(
                    egui::Vec2::new(ui.available_width(), 200.0), // Fixed height of 200px
                    egui::Layout::top_down(egui::Align::Center),
                    |ui| {
                        // Show filtered results
                        if self.filtered_tabs.is_empty() && !self.search_text.is_empty() {
                            // Center the "no match" message within the fixed area
                            ui.allocate_ui_with_layout(
                                egui::Vec2::new(ui.available_width(), 100.0),
                                egui::Layout::centered_and_justified(egui::Direction::TopDown),
                                |ui| {
                                    ui.colored_label(colors.text_primary_color32(), "No matching tabs found");
                                    ui.add_space(5.0);
                                    ui.colored_label(
                                        colors.text_primary_color32().linear_multiply(0.7),
                                        "Try a different search term"
                                    );
                                }
                            );
                        } else {
                            // Show search results or all tabs - collect outside the closure to avoid borrow conflicts
                            let tabs_to_show = if self.search_text.is_empty() {
                                available_tabs.iter().map(|config| &config.tab_type).collect::<Vec<_>>()
                            } else {
                                self.filtered_tabs.iter().collect::<Vec<_>>()
                            };
                            
                            // Clone tabs to avoid borrow checker issues
                            let tabs_clone: Vec<Tab> = tabs_to_show.iter().map(|&tab| tab.clone()).collect();

                            if !self.search_text.is_empty() && tabs_clone.len() == 1 {
                                // Highlight the single match
                                ui.horizontal(|ui| {
                                    ui.label("Press Enter to create:");
                                    ui.strong(get_tab_display_name(&tabs_clone[0]));
                                });
                                ui.add_space(5.0);
                            }

                            // Create a grid of tab options
                            let columns = 3;
                            let mut current_column = 0;

                            ui.horizontal_wrapped(|ui| {
                                for (index, tab_type) in tabs_clone.iter().enumerate() {
                                    let display_name = get_tab_display_name(tab_type);
                                    let icon = get_tab_icon(tab_type);
                                    let description = get_tab_description(tab_type);

                                    let button_size = egui::Vec2::new(120.0, 80.0);

                                    // Create a custom button with centered content
                                    let (rect, response) =
                                        ui.allocate_exact_size(button_size, egui::Sense::click());

                                    let visuals = ui.style().interact(&response);
                                    
                                    // Highlight first result when searching
                                    let is_first_result = !self.search_text.is_empty() && index == 0;
                                    let fill_color = if is_first_result {
                                        colors.accent_color32().linear_multiply(0.6)
                                    } else if response.hovered() {
                                        colors.accent_color32().linear_multiply(0.8)
                                    } else {
                                        colors.panel_background_color32()
                                    };

                                    ui.painter()
                                        .rect_filled(rect, egui::Rounding::same(4.0), fill_color);

                                    let stroke_color = if is_first_result {
                                        colors.accent_color32()
                                    } else {
                                        colors.accent_color32().linear_multiply(0.7)
                                    };

                                    ui.painter().rect_stroke(
                                        rect,
                                        egui::Rounding::same(4.0),
                                        egui::Stroke::new(if is_first_result { 2.0 } else { 1.0 }, stroke_color),
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
                        }
                    }
                );

                ui.add_space(20.0);
                ui.separator();
                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    if ui.button("❌ Cancel").clicked() {
                        self.hide();
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if self.search_text.is_empty() {
                            ui.label("💡 Tip: Type tab names for quick access (e.g., 'ti' for Timer)");
                        } else {
                            ui.label("⏎ Press Enter to create highlighted tab");
                        }
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

fn get_tab_display_name(tab_type: &Tab) -> &'static str {
    match tab_type {
        Tab::Timer => "Timer",
        Tab::Stats => "Statistics",
        Tab::Record => "Record",
        Tab::Graph => "Graph",
        Tab::Todo => "Todo and Habits",
        Tab::Calculator => "Calculator",
        Tab::Flashcards => "Flashcards",
        Tab::Markdown => "Markdown",
        Tab::Reminder => "Reminder",
        Tab::Terminal => "Terminal",
        Tab::Settings => "Settings",
    }
}

fn get_tab_search_name(tab_type: &Tab) -> &'static str {
    match tab_type {
        Tab::Timer => "timer",
        Tab::Stats => "stats statistics",
        Tab::Record => "record",
        Tab::Graph => "graph",
        Tab::Todo => "todo task habits",
        Tab::Calculator => "calculator calc",
        Tab::Flashcards => "flashcards cards flash",
        Tab::Markdown => "markdown md text",
        Tab::Reminder => "reminder remind",
        Tab::Terminal => "terminal term console",
        Tab::Settings => "settings config",
    }
}
