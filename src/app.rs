use crate::data::StudyData;
use crate::debug::DebugTools;
use crate::file_drop_handler::FileDropHandler;
use crate::keyboard_handler::KeyboardHandler;
use crate::settings::{AppSettings, NavigationLayout};
use crate::split_view_ui::SplitViewUI;
use crate::tab_manager::{SplitDirection, TabManager};
use crate::tab_selector_ui::TabSelectorUI;
use crate::terminal::TerminalEmulator;
use crate::timer::Timer;
use crate::ui;
use crate::ui::flashcard_ui::{DeckManagerUI, FlashcardReviewer};
use crate::weather::WeatherWidget;

use eframe::{egui, CreationContext};
use serde::{Deserialize, Serialize};
use std::time::Instant;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Tab {
    Timer,
    Stats,
    Record,
    Graph,
    Todo,
    Calculator,
    Markdown,
    Reminder,
    Terminal,
    Settings,
    Flashcards,
}

pub struct StatusMessage {
    message: String,
    time: Option<Instant>,
}

impl StatusMessage {
    pub fn new() -> Self {
        Self {
            message: String::new(),
            time: None,
        }
    }

    pub fn show(&mut self, message: &str) {
        self.message = message.to_string();
        self.time = Some(Instant::now());
    }

    pub fn render(&mut self, ui: &mut egui::Ui) {
        if let Some(status_time) = self.time {
            if status_time.elapsed().as_secs() < 5 && !self.message.is_empty() {
                ui.add_space(20.0);
                ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                    ui.label(egui::RichText::new(&self.message).italics());
                });
            } else {
                self.message.clear();
                self.time = None;
            }
        }
    }
}

pub struct StudyTimerApp {
    pub timer: Timer,
    pub study_data: StudyData,
    pub settings: AppSettings,
    pub current_tab: Tab,
    pub status: StatusMessage,
    pub debug_tools: DebugTools,
    pub markdown_editor: Option<crate::ui::markdown_editor::MarkdownEditor>,
    pub terminal: TerminalEmulator,
    pub tab_manager: TabManager,
    pub keyboard_handler: KeyboardHandler,
    pub tab_selector: TabSelectorUI,
    pub file_drop_handler: FileDropHandler,
    pub dragging_tab_id: Option<String>,
    pub drag_start_pos: Option<egui::Pos2>,
    pub last_used_split_pane: bool,
    pub flashcard_reviewer: FlashcardReviewer,
    pub deck_manager_ui: DeckManagerUI,
    pub weather_widget: WeatherWidget,
}

impl StudyTimerApp {
    pub fn new(_cc: &CreationContext<'_>) -> Self {
        let study_data = StudyData::load().unwrap_or_default();
        let settings = AppSettings::load().unwrap_or_default();
        let current_tab = settings.get_first_enabled_tab();
        let tab_manager = TabManager::new(&settings);
        let weather_widget = WeatherWidget::load().unwrap_or_default();

        Self {
            timer: Timer::new(),
            study_data,
            settings,
            current_tab,
            status: StatusMessage::new(),
            debug_tools: DebugTools::new(),
            markdown_editor: None,
            terminal: TerminalEmulator::new(),
            tab_manager,
            keyboard_handler: KeyboardHandler::new(),
            tab_selector: TabSelectorUI::new(),
            file_drop_handler: FileDropHandler::new(),
            dragging_tab_id: None,
            drag_start_pos: None,
            last_used_split_pane: false,
            flashcard_reviewer: FlashcardReviewer::new(),
            deck_manager_ui: DeckManagerUI::new(),
            weather_widget,
        }
    }

    fn handle_keyboard_shortcuts(&mut self) {
        if self.keyboard_handler.new_tab_requested {
            self.tab_selector.show();
        }

        if self.keyboard_handler.close_tab_requested {
            let active_tab_id = self.tab_manager.active_tab_id.clone();
            if !self.tab_manager.close_tab(&active_tab_id) {
                self.status.show("Cannot close this tab");
            }
        }

        if self.keyboard_handler.split_horizontal_requested {
            self.tab_manager.create_split(SplitDirection::Horizontal);
        }

        if self.keyboard_handler.split_vertical_requested {
            self.tab_manager.create_split(SplitDirection::Vertical);
        }

        if self.keyboard_handler.close_split_requested {
            self.tab_manager.close_split();
        }

        // Handle tab switching by number
        if let Some(tab_index) = self.keyboard_handler.tab_number_requested {
            // Collect the target tab ID first, before any mutable borrows
            let target_tab_id = {
                let non_settings_tabs: Vec<_> = self
                    .tab_manager
                    .tabs
                    .iter()
                    .filter(|tab| tab.tab_type != Tab::Settings)
                    .collect();

                if tab_index < non_settings_tabs.len() {
                    Some(non_settings_tabs[tab_index].id.clone())
                } else {
                    None
                }
            };

            // Now handle the tab switching with the collected ID
            match target_tab_id {
                Some(target_id) => {
                    if self.tab_manager.is_split_active() {
                        self.tab_manager
                            .set_split_active_tab(&target_id, self.last_used_split_pane);
                    } else {
                        self.tab_manager.set_active_tab(&target_id);
                    }
                }
                None => {
                    self.status
                        .show(&format!("Tab {} does not exist", tab_index + 1));
                }
            }
        }

        // Handle switching to last used tab
        if self.keyboard_handler.switch_to_last_tab_requested {
            if !self.tab_manager.switch_to_last_tab() {
                self.status.show("No previous tab to switch to");
            }
        }
    }

    fn render_tab_bar(&mut self, ui: &mut egui::Ui) {
        let colors = self.settings.get_current_colors();

        // Tab bar with proper margins to keep content visible
        let tab_bar_frame = egui::Frame::default()
            .fill(colors.navigation_background_color32())
            .shadow(egui::epaint::Shadow {
                extrusion: 2.0,
                color: egui::Color32::from_black_alpha(20),
            })
            .inner_margin(egui::Margin {
                left: 8.0,
                right: 8.0,
                top: -4.0, // Move buttons top edge out of screen
                bottom: 6.0,
            })
            .rounding(egui::Rounding::same(3.0));

        tab_bar_frame.show(ui, |ui| {
            ui.horizontal(|ui| {
                // Left section - scrollable tabs (takes most of the space)
                ui.push_id("tab_bar_left_section", |ui| {
                    // Allocate space for tabs, leaving room for right controls
                    let available_width = ui.available_width() - 250.0; // Reserve space for right controls

                    ui.allocate_ui_with_layout(
                        [available_width, ui.available_height()].into(),
                        egui::Layout::left_to_right(egui::Align::Center),
                        |ui| {
                            egui::ScrollArea::horizontal()
                                .id_source("tab_bar_scroll")
                                .auto_shrink([false, true])
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        ui.spacing_mut().item_spacing.x = 4.0;

                                        let tabs_to_render: Vec<_> = self
                                            .tab_manager
                                            .tabs
                                            .iter()
                                            .filter(|tab| tab.tab_type != Tab::Settings)
                                            .cloned()
                                            .collect();

                                        for (index, tab) in tabs_to_render.iter().enumerate() {
                                            ui.push_id(format!("tab_{}", tab.id), |ui| {
                                                let is_active =
                                                    tab.id == self.tab_manager.active_tab_id;
                                                self.render_enhanced_tab(ui, tab, is_active, index);
                                            });
                                        }

                                        // New tab button with same padding as tabs
                                        ui.push_id("new_tab_button", |ui| {
                                            // Use same spacing as between tabs
                                            ui.add_space(4.0);

                                            // Allocate space for the new tab button with exact same dimensions as tabs
                                            let tab_width = 90.0;
                                            let tab_height = 50.0;
                                            let button_rect = ui
                                                .allocate_space(egui::Vec2::new(
                                                    tab_width, tab_height,
                                                ))
                                                .1;

                                            // Draw button background with rounded corners (same as tabs)
                                            ui.painter().rect_filled(
                                                button_rect,
                                                egui::Rounding::same(6.0),
                                                colors.background_color32(),
                                            );

                                            // Draw border/stroke (same as inactive tabs)
                                            ui.painter().rect_stroke(
                                                button_rect,
                                                egui::Rounding::same(6.0),
                                                egui::Stroke::new(1.0, colors.accent_color32()),
                                            );

                                            // Draw "+" icon at the top center
                                            let icon_y = button_rect.min.y + 12.0;
                                            ui.painter().text(
                                                egui::Pos2::new(button_rect.center().x, icon_y),
                                                egui::Align2::CENTER_TOP,
                                                "+",
                                                egui::FontId::new(
                                                    16.0,
                                                    egui::FontFamily::Proportional,
                                                ),
                                                colors.text_primary_color32(),
                                            );

                                            // Draw "New" text at the bottom center
                                            let text_y = button_rect.max.y - 8.0;
                                            ui.painter().text(
                                                egui::Pos2::new(button_rect.center().x, text_y),
                                                egui::Align2::CENTER_BOTTOM,
                                                "New",
                                                egui::FontId::new(
                                                    10.0,
                                                    egui::FontFamily::Proportional,
                                                ),
                                                colors.text_primary_color32(),
                                            );

                                            // Handle button interaction
                                            let button_response = ui.interact(
                                                button_rect,
                                                egui::Id::new("new_tab_button_interact"),
                                                egui::Sense::click(),
                                            );

                                            if button_response.clicked() {
                                                self.tab_selector.show();
                                            }
                                        });
                                    });
                                });
                        },
                    );
                });

                // Right section - fixed layout to prevent overlapping
                ui.push_id("tab_bar_right_section", |ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Settings tab (rightmost, fixed position)
                        ui.push_id("settings_tab_section", |ui| {
                            self.render_settings_tab_button(ui);
                        });

                        ui.add_space(8.0); // Space between settings and other controls

                        // Weather and split controls in a compact horizontal layout
                        ui.push_id("right_controls_horizontal", |ui| {
                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing.x = 4.0;

                                // Weather widget first
                                ui.push_id("weather_widget_section", |ui| {
                                    self.render_weather_widget_compact(ui);
                                });

                                // Split controls after weather (only if not in split mode)
                                if !self.tab_manager.is_split_active() {
                                    ui.push_id("split_controls_section", |ui| {
                                        self.render_split_controls_compact(ui);
                                    });
                                }
                            });
                        });
                    });
                });
            });
        });
    }

    fn render_split_controls_compact(&mut self, ui: &mut egui::Ui) {
        let colors = self.settings.get_current_colors();

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 2.0; // Small spacing between buttons

            let split_button_style = |ui: &mut egui::Ui, icon: &str, button_id: &str| {
                ui.push_id(button_id, |ui| {
                    let button = egui::Button::new(
                        egui::RichText::new(icon)
                            .color(colors.text_secondary_color32())
                            .size(10.0), // Slightly larger for better visibility
                    )
                    .fill(colors.background_color32())
                    .rounding(egui::Rounding::same(3.0))
                    .min_size(egui::Vec2::new(20.0, 16.0)); // Slightly larger

                    ui.add(button)
                })
                .inner
            };

            if split_button_style(ui, "⬍", "horizontal_split").clicked() {
                self.tab_manager.create_split(SplitDirection::Horizontal);
            }

            if split_button_style(ui, "⬌", "vertical_split").clicked() {
                self.tab_manager.create_split(SplitDirection::Vertical);
            }
        });
    }

    fn render_enhanced_tab(
        &mut self,
        ui: &mut egui::Ui,
        tab: &crate::tab_manager::TabInstance,
        is_active: bool,
        index: usize,
    ) {
        let colors = self.settings.get_current_colors();

        // Create a vertical layout for the tab content with unique ID
        ui.push_id(format!("enhanced_tab_{}", tab.id), |ui| {
            // Wider tab dimensions with spacing
            let tab_width = 90.0;
            let tab_height = 50.0;

            let (button_color, text_color, stroke_width) = if is_active {
                (
                    colors.active_tab_color32(),
                    colors.text_primary_color32(),
                    2.0,
                )
            } else {
                (
                    colors.background_color32(),
                    colors.text_secondary_color32(),
                    1.0,
                )
            };

            let tab_icon = match tab.tab_type {
                Tab::Timer => "⏱",
                Tab::Stats => "📊",
                Tab::Record => "📝",
                Tab::Graph => "📈",
                Tab::Todo => "✅",
                Tab::Calculator => "=",
                Tab::Markdown => "📄",
                Tab::Reminder => "🔔",
                Tab::Terminal => "💻",
                Tab::Flashcards => "🃏",
                Tab::Settings => "⚙",
            };

            // Get display name (shortened if needed)
            let full_title = tab.get_display_title();
            let display_name = match tab.tab_type {
                Tab::Todo => "Todo",
                _ => full_title.split_whitespace().next().unwrap_or("Tab"),
            };

            // Allocate space for the entire tab
            let tab_rect = ui.allocate_space(egui::Vec2::new(tab_width, tab_height)).1;

            // Draw tab background with rounded corners
            ui.painter()
                .rect_filled(tab_rect, egui::Rounding::same(6.0), button_color);

            // Draw border/stroke
            ui.painter().rect_stroke(
                tab_rect,
                egui::Rounding::same(6.0),
                egui::Stroke::new(stroke_width, colors.accent_color32()),
            );

            // Draw icon at the top center (moved down to be more visible)
            let icon_y = tab_rect.min.y + 12.0;
            ui.painter().text(
                egui::Pos2::new(tab_rect.center().x, icon_y),
                egui::Align2::CENTER_TOP,
                tab_icon,
                egui::FontId::new(16.0, egui::FontFamily::Proportional),
                text_color,
            );

            // Draw text at the bottom center (moved up to be more visible)
            let text_y = tab_rect.max.y - 8.0;
            ui.painter().text(
                egui::Pos2::new(tab_rect.center().x, text_y),
                egui::Align2::CENTER_BOTTOM,
                display_name,
                egui::FontId::new(10.0, egui::FontFamily::Proportional),
                text_color,
            );

            // Handle close button first (if it exists)
            let close_button_clicked = if tab.can_close {
                let close_rect = egui::Rect::from_min_size(
                    egui::Pos2::new(tab_rect.max.x - 18.0, tab_rect.min.y + 8.0),
                    egui::Vec2::new(14.0, 14.0),
                );

                let close_response = ui.interact(
                    close_rect,
                    egui::Id::new(format!("close_btn_{}", tab.id)),
                    egui::Sense::click(),
                );

                // Draw close button "×" without any hover effects
                ui.painter().text(
                    close_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "×",
                    egui::FontId::new(12.0, egui::FontFamily::Proportional),
                    if is_active {
                        colors.text_primary_color32()
                    } else {
                        colors.text_secondary_color32().gamma_multiply(0.7)
                    },
                );

                close_response.clicked()
            } else {
                false
            };

            // Handle main tab area (excluding close button area)
            let main_tab_rect = if tab.can_close {
                // Exclude the close button area from the clickable tab area
                egui::Rect::from_min_max(
                    tab_rect.min,
                    egui::Pos2::new(tab_rect.max.x - 20.0, tab_rect.max.y),
                )
            } else {
                tab_rect
            };

            let tab_response = ui.interact(
                main_tab_rect,
                egui::Id::new(format!("tab_main_{}", tab.id)),
                egui::Sense::click_and_drag(),
            );

            // Process close button click
            if close_button_clicked {
                self.tab_manager.close_tab(&tab.id);
            }
            // Process tab click (only if close button wasn't clicked)
            else if tab_response.clicked() {
                if self.tab_manager.is_split_active() {
                    self.tab_manager
                        .set_split_active_tab(&tab.id, self.last_used_split_pane);
                } else {
                    self.tab_manager.set_active_tab(&tab.id);
                }
            }
            // Handle drag operations (only if no clicks happened)
            else {
                if tab_response.drag_started() {
                    self.dragging_tab_id = Some(tab.id.clone());
                    self.drag_start_pos = tab_response.interact_pointer_pos();
                }

                if tab_response.dragged() && self.dragging_tab_id == Some(tab.id.clone()) {
                    ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::Grabbing);

                    if let Some(pointer_pos) = tab_response.interact_pointer_pos() {
                        ui.painter().circle_filled(
                            pointer_pos,
                            8.0,
                            colors.accent_color32().gamma_multiply(0.8),
                        );
                        ui.painter().circle_stroke(
                            pointer_pos,
                            8.0,
                            egui::Stroke::new(2.0, colors.text_primary_color32()),
                        );
                    }
                }

                if tab_response.drag_released() && self.dragging_tab_id == Some(tab.id.clone()) {
                    if let Some(drop_pos) = tab_response.interact_pointer_pos() {
                        self.handle_tab_drop(drop_pos, &tab.id);
                    }
                    self.dragging_tab_id = None;
                    self.drag_start_pos = None;
                }
            }

            // Drop zone indicator - only show during drag operations, no hover effects
            if self.dragging_tab_id.is_some() && self.dragging_tab_id != Some(tab.id.clone()) {
                if ui.input(|i| i.pointer.any_released()) && tab_response.hovered() {
                    if let Some(dragging_id) = &self.dragging_tab_id {
                        self.tab_manager.reorder_tab(dragging_id, index);
                    }
                }
            }
        });
    }

    fn render_weather_widget_compact(&mut self, ui: &mut egui::Ui) {
        let colors = self.settings.get_current_colors();

        ui.push_id("weather_widget_compact", |ui| {
            let weather_rect = ui.allocate_space(egui::Vec2::new(80.0, 24.0)).1;

            // Use navigation background color to match the navbar
            let weather_frame = egui::Frame::default()
                .fill(colors.navigation_background_color32()) // Back to navigation background color
                .inner_margin(egui::Margin::same(4.0))
                .rounding(egui::Rounding::same(3.0));

            ui.allocate_ui_at_rect(weather_rect, |ui| {
                weather_frame.show(ui, |ui| {
                    if self.weather_widget.render(ui) {
                        let _ = self.weather_widget.save();
                    }
                });
            });
        });
    }

    fn render_settings_tab_button(&mut self, ui: &mut egui::Ui) {
        let colors = self.settings.get_current_colors();

        let settings_tab_info = self
            .tab_manager
            .tabs
            .iter()
            .find(|tab| tab.tab_type == Tab::Settings)
            .map(|tab| (tab.id.clone(), tab.id == self.tab_manager.active_tab_id));

        if let Some((settings_tab_id, is_active)) = settings_tab_info {
            let (button_color, text_color, stroke_width) = if is_active {
                (
                    colors.active_tab_color32(),
                    colors.text_primary_color32(),
                    2.0,
                )
            } else {
                (
                    colors.background_color32(),
                    colors.text_secondary_color32(),
                    1.0,
                )
            };

            ui.push_id("settings_button", |ui| {
                // Use same dimensions as regular tabs
                let tab_width = 90.0;
                let tab_height = 50.0;
                let button_rect = ui.allocate_space(egui::Vec2::new(tab_width, tab_height)).1;

                // Draw button background with rounded corners (same as tabs)
                ui.painter()
                    .rect_filled(button_rect, egui::Rounding::same(6.0), button_color);

                // Draw border/stroke (same as tabs)
                ui.painter().rect_stroke(
                    button_rect,
                    egui::Rounding::same(6.0),
                    egui::Stroke::new(stroke_width, colors.accent_color32()),
                );

                // Draw settings icon at the top center
                let icon_y = button_rect.min.y + 12.0;
                ui.painter().text(
                    egui::Pos2::new(button_rect.center().x, icon_y),
                    egui::Align2::CENTER_TOP,
                    "⚙", // Use gear symbol instead of emoji
                    egui::FontId::new(16.0, egui::FontFamily::Proportional),
                    text_color,
                );

                // Draw "Settings" text at the bottom center
                let text_y = button_rect.max.y - 8.0;
                ui.painter().text(
                    egui::Pos2::new(button_rect.center().x, text_y),
                    egui::Align2::CENTER_BOTTOM,
                    "Settings",
                    egui::FontId::new(10.0, egui::FontFamily::Proportional),
                    text_color,
                );

                // Handle button interaction
                let button_response = ui.interact(
                    button_rect,
                    egui::Id::new("settings_button_interact"),
                    egui::Sense::click(),
                );

                if button_response.clicked() {
                    if self.tab_manager.is_split_active() {
                        self.tab_manager
                            .set_split_active_tab(&settings_tab_id, self.last_used_split_pane);
                    } else {
                        self.tab_manager.set_active_tab(&settings_tab_id);
                    }
                }
            });
        }
    }

    fn handle_tab_drop(&mut self, _drop_pos: egui::Pos2, _tab_id: &str) {
        if self.tab_manager.is_split_active() {
            self.status
                .show("Tab dropped - split functionality needs enhancement");
        }
    }

    pub fn update_last_used_split_pane(&mut self, is_right_pane: bool) {
        self.last_used_split_pane = is_right_pane;
    }

    fn render_navigation(&mut self, ui: &mut egui::Ui) {
        if self.tab_manager.tabs.is_empty() {
            let enabled_tabs = self.settings.get_enabled_tabs();
            let colors = self.settings.get_current_colors();

            match self.settings.navigation_layout {
                NavigationLayout::Horizontal => {
                    // Enhanced horizontal navigation
                    let nav_frame = egui::Frame::default()
                        .fill(colors.navigation_background_color32())
                        .shadow(egui::epaint::Shadow {
                            extrusion: 3.0,
                            color: egui::Color32::from_black_alpha(20),
                        })
                        .inner_margin(egui::Margin::symmetric(12.0, 8.0))
                        .rounding(egui::Rounding::same(8.0));

                    nav_frame.show(ui, |ui| {
                        ui.horizontal(|ui| {
                            for (i, config) in enabled_tabs.iter().enumerate() {
                                let is_current = self.current_tab == config.tab_type;
                                let display_name = config.get_display_name();

                                let (button_color, text_color, stroke_width) = if is_current {
                                    (
                                        colors.active_tab_color32(),
                                        colors.text_primary_color32(),
                                        2.0,
                                    )
                                } else {
                                    (
                                        colors.inactive_tab_color32().gamma_multiply(0.6),
                                        colors.text_secondary_color32(),
                                        1.0,
                                    )
                                };

                                let button = egui::Button::new(
                                    egui::RichText::new(&display_name)
                                        .color(text_color)
                                        .size(if is_current { 13.0 } else { 12.0 }),
                                )
                                .fill(button_color)
                                .stroke(egui::Stroke::new(stroke_width, colors.accent_color32()))
                                .rounding(egui::Rounding::same(8.0))
                                .min_size(egui::Vec2::new(80.0, 36.0));

                                if ui.add(button).clicked() {
                                    self.current_tab = config.tab_type.clone();
                                }

                                // Add separator between tabs
                                if i < enabled_tabs.len() - 1 {
                                    ui.add_space(6.0);
                                }
                            }
                        });
                    });
                }
                NavigationLayout::Vertical => {
                    // Enhanced vertical navigation
                    egui::SidePanel::left("navigation_panel")
                        .resizable(true)
                        .default_width(160.0)
                        .width_range(140.0..=280.0)
                        .frame(
                            egui::Frame::default()
                                .fill(colors.navigation_background_color32())
                                .shadow(egui::epaint::Shadow {
                                    extrusion: 4.0,
                                    color: egui::Color32::from_black_alpha(25),
                                })
                                .inner_margin(egui::Margin::same(12.0))
                                .rounding(egui::Rounding::same(0.0)),
                        )
                        .show_inside(ui, |ui| {
                            ui.vertical(|ui| {
                                // Enhanced header
                                ui.heading(
                                    egui::RichText::new("🎯 Navigation")
                                        .color(colors.text_primary_color32())
                                        .size(16.0),
                                );
                                ui.add_space(4.0);
                                ui.separator();
                                ui.add_space(8.0);

                                for config in enabled_tabs {
                                    let is_current = self.current_tab == config.tab_type;
                                    let display_name = config.get_display_name();

                                    let (button_color, text_color, stroke_width) = if is_current {
                                        (
                                            colors.active_tab_color32(),
                                            colors.text_primary_color32(),
                                            2.0,
                                        )
                                    } else {
                                        (
                                            egui::Color32::TRANSPARENT,
                                            colors.text_secondary_color32(),
                                            0.0,
                                        )
                                    };

                                    // Tab icon
                                    let tab_icon = match config.tab_type {
                                        Tab::Timer => "⏱️",
                                        Tab::Stats => "📊",
                                        Tab::Record => "📝",
                                        Tab::Graph => "📈",
                                        Tab::Todo => "✅",
                                        Tab::Calculator => "🧮",
                                        Tab::Markdown => "📄",
                                        Tab::Reminder => "🔔",
                                        Tab::Terminal => "💻",
                                        Tab::Flashcards => "🃏",
                                        Tab::Settings => "⚙️",
                                    };

                                    let button = egui::Button::new(
                                        egui::RichText::new(format!(
                                            "{} {}",
                                            tab_icon, &display_name
                                        ))
                                        .color(text_color)
                                        .size(if is_current { 13.0 } else { 12.0 }),
                                    )
                                    .fill(button_color)
                                    .stroke(egui::Stroke::new(
                                        stroke_width,
                                        colors.accent_color32(),
                                    ))
                                    .rounding(egui::Rounding::same(8.0));

                                    if ui.add_sized([ui.available_width(), 36.0], button).clicked()
                                    {
                                        self.current_tab = config.tab_type.clone();
                                    }
                                    ui.add_space(4.0);
                                }
                            });
                        });
                }
            }
        }
    }

    pub fn save_on_exit(&mut self) {
        self.tab_manager.save_state();
    }

    fn render_main_content(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        if self.tab_manager.is_split_active() {
            SplitViewUI::display(ui, self, ctx);
        } else if let Some(active_tab) = self.tab_manager.get_active_tab() {
            let colors = self.settings.get_current_colors();
            let content_frame = egui::Frame::default()
                .fill(colors.panel_background_color32())
                .inner_margin(egui::Margin::same(10.0));

            let tab_type = active_tab.tab_type.clone();
            content_frame.show(ui, |ui| {
                self.render_tab_content(ui, ctx, &tab_type);
            });
        } else {
            let colors = self.settings.get_current_colors();
            let content_frame = egui::Frame::default()
                .fill(colors.panel_background_color32())
                .inner_margin(egui::Margin::same(10.0));

            let current_tab = self.current_tab.clone();
            content_frame.show(ui, |ui| {
                self.render_tab_content(ui, ctx, &current_tab);
            });
        }
    }

    fn render_tab_content(&mut self, ui: &mut egui::Ui, ctx: &egui::Context, tab_type: &Tab) {
        match tab_type {
            Tab::Timer => ui::timer_tab::display(
                ui,
                &mut self.timer,
                &mut self.study_data,
                &mut self.debug_tools,
                &mut self.status,
            ),
            Tab::Stats => ui::stats_tab::display(ui, &mut self.study_data, &mut self.status),
            Tab::Record => {
                ui::record_tab::display(ui, &mut self.study_data, &mut self.status, &self.timer)
            }
            Tab::Flashcards => ui::flashcard_tab_ui::display(ui, ctx, self),
            Tab::Graph => ui::graph_tab::display(ui, &self.study_data, &mut self.status),
            Tab::Todo => {
                ui::todo_tab::display(ui, &mut self.study_data, &mut self.status, &self.settings)
            }
            Tab::Reminder => ui::reminder_tab::display(ui, &mut self.study_data, &mut self.status),
            Tab::Calculator => ui::calculator_tab::display(ui, &mut self.status),
            Tab::Markdown => ui::markdown_tab_ui::display(ui, self, ctx),
            Tab::Terminal => ui::terminal_tab_ui::display(ui, &mut self.terminal, &mut self.status),
            Tab::Settings => ui::settings_tab_ui::display(
                ui,
                &mut self.settings,
                &mut self.status,
                &mut self.current_tab,
            ),
        }
    }
}

impl eframe::App for StudyTimerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.settings.apply_theme(ctx);

        self.keyboard_handler.handle_input(ctx);
        self.handle_keyboard_shortcuts();

        // Update weather widget
        self.weather_widget.update();

        let dropped_files = self
            .file_drop_handler
            .handle_dropped_files(ctx, &mut self.status);
        for dropped_file in dropped_files {
            if let Some(path_str) = dropped_file.path.to_str() {
                self.tab_manager
                    .add_file_tab(dropped_file.tab_type, path_str.to_string());
            }
        }

        if let Some(selected_tab) = self
            .tab_selector
            .display(ctx, &self.settings, &mut self.status)
        {
            let new_tab_id = self.tab_manager.add_tab(selected_tab);

            if self.tab_manager.is_split_active() {
                self.tab_manager
                    .set_split_active_tab(&new_tab_id, self.last_used_split_pane);
            }
        }

        if self.timer.is_running {
            ctx.request_repaint();
        }

        let colors = self.settings.get_current_colors();

        let main_frame = egui::Frame::default()
            .fill(colors.background_color32())
            .inner_margin(egui::Margin::ZERO);

        egui::CentralPanel::default()
            .frame(main_frame)
            .show(ctx, |ui| {
                if !self.tab_manager.tabs.is_empty() {
                    self.render_tab_bar(ui);
                    ui.separator();
                }

                self.render_navigation(ui);

                if !self.tab_manager.tabs.is_empty()
                    || self.settings.navigation_layout == NavigationLayout::Horizontal
                {
                    ui.separator();
                }

                self.render_main_content(ui, ctx);
            });
    }
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.save_on_exit();
    }
}

