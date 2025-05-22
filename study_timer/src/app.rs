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
    pub current_tab: Tab, // Keep for backward compatibility
    pub status: StatusMessage,
    pub debug_tools: DebugTools,
    pub markdown_editor: Option<crate::ui::markdown_editor::MarkdownEditor>,
    pub terminal: TerminalEmulator,
    // New tab management system
    pub tab_manager: TabManager,
    pub keyboard_handler: KeyboardHandler,
    pub tab_selector: TabSelectorUI,
    pub file_drop_handler: FileDropHandler,
}

impl StudyTimerApp {
    pub fn new(_cc: &CreationContext<'_>) -> Self {
        let study_data = StudyData::load().unwrap_or_default();
        let settings = AppSettings::load().unwrap_or_default();
        let current_tab = settings.get_first_enabled_tab();
        let tab_manager = TabManager::new(&settings);

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
    }

    fn render_tab_bar(&mut self, ui: &mut egui::Ui) {
        let colors = self.settings.get_current_colors();

        // Tab bar background
        let tab_bar_frame = egui::Frame::default()
            .fill(colors.navigation_background_color32())
            .inner_margin(egui::Margin::symmetric(8.0, 4.0));

        tab_bar_frame.show(ui, |ui| {
            egui::ScrollArea::horizontal()
                .id_source("tab_bar_scroll")
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        for tab in self.tab_manager.tabs.clone() {
                            let is_active = tab.id == self.tab_manager.active_tab_id;

                            ui.horizontal(|ui| {
                                let button_color = if is_active {
                                    colors.active_tab_color32()
                                } else {
                                    colors.inactive_tab_color32()
                                };

                                let text_color = if is_active {
                                    colors.text_primary_color32()
                                } else {
                                    colors.text_secondary_color32()
                                };

                                let button = egui::Button::new(
                                    egui::RichText::new(&tab.get_display_title()).color(text_color),
                                )
                                .fill(button_color)
                                .stroke(egui::Stroke::new(1.0, colors.accent_color32()));

                                if ui.add(button).clicked() {
                                    self.tab_manager.set_active_tab(&tab.id);
                                }

                                // Close button for closeable tabs
                                if tab.can_close {
                                    let close_button = egui::Button::new("×")
                                        .fill(egui::Color32::TRANSPARENT)
                                        .stroke(egui::Stroke::NONE)
                                        .min_size(egui::Vec2::new(16.0, 16.0));

                                    if ui.add(close_button).clicked() {
                                        self.tab_manager.close_tab(&tab.id);
                                    }
                                }
                            });
                        }

                        // New tab button
                        let new_tab_button = egui::Button::new("+")
                            .fill(colors.accent_color32())
                            .stroke(egui::Stroke::new(1.0, colors.text_primary_color32()));

                        if ui.add(new_tab_button).clicked() {
                            self.tab_selector.show();
                        }

                        // Split controls
                        ui.separator();

                        if !self.tab_manager.is_split_active() {
                            if ui.button("⬌ Split V").clicked() {
                                self.tab_manager.create_split(SplitDirection::Vertical);
                            }
                            if ui.button("⬍ Split H").clicked() {
                                self.tab_manager.create_split(SplitDirection::Horizontal);
                            }
                        } else {
                            if ui.button("❌ Close Split").clicked() {
                                self.tab_manager.close_split();
                            }
                        }
                    });
                });
        });
    }

    fn render_navigation(&mut self, ui: &mut egui::Ui) {
        // Only render old navigation if not using the new tab system
        if self.tab_manager.tabs.is_empty() {
            let enabled_tabs = self.settings.get_enabled_tabs();
            let colors = self.settings.get_current_colors();

            match self.settings.navigation_layout {
                NavigationLayout::Horizontal => {
                    let nav_frame = egui::Frame::default()
                        .fill(colors.navigation_background_color32())
                        .inner_margin(egui::Margin::same(8.0));

                    nav_frame.show(ui, |ui| {
                        ui.horizontal(|ui| {
                            for config in enabled_tabs {
                                let is_current = self.current_tab == config.tab_type;
                                let display_name = config.get_display_name();

                                let button_color = if is_current {
                                    colors.active_tab_color32()
                                } else {
                                    colors.inactive_tab_color32()
                                };

                                let text_color = if is_current {
                                    colors.text_primary_color32()
                                } else {
                                    colors.text_secondary_color32()
                                };

                                let button = egui::Button::new(
                                    egui::RichText::new(&display_name).color(text_color),
                                )
                                .fill(button_color)
                                .stroke(egui::Stroke::new(1.0, colors.accent_color32()));

                                if ui.add(button).clicked() {
                                    self.current_tab = config.tab_type.clone();
                                }
                            }
                        });
                    });
                }
                NavigationLayout::Vertical => {
                    egui::SidePanel::left("navigation_panel")
                        .resizable(true)
                        .default_width(150.0)
                        .width_range(120.0..=250.0)
                        .frame(egui::Frame::default().fill(colors.navigation_background_color32()))
                        .show_inside(ui, |ui| {
                            ui.vertical(|ui| {
                                ui.heading(
                                    egui::RichText::new("Navigation")
                                        .color(colors.text_primary_color32()),
                                );
                                ui.separator();
                                ui.add_space(5.0);

                                for config in enabled_tabs {
                                    let is_current = self.current_tab == config.tab_type;
                                    let display_name = config.get_display_name();

                                    let button_color = if is_current {
                                        colors.active_tab_color32()
                                    } else {
                                        egui::Color32::TRANSPARENT
                                    };

                                    let text_color = if is_current {
                                        colors.text_primary_color32()
                                    } else {
                                        colors.text_secondary_color32()
                                    };

                                    let button = egui::Button::new(
                                        egui::RichText::new(&display_name).color(text_color),
                                    )
                                    .fill(button_color)
                                    .stroke(egui::Stroke::new(
                                        if is_current { 2.0 } else { 0.0 },
                                        colors.accent_color32(),
                                    ));

                                    if ui.add_sized([ui.available_width(), 30.0], button).clicked()
                                    {
                                        self.current_tab = config.tab_type.clone();
                                    }
                                    ui.add_space(2.0);
                                }
                            });
                        });
                }
            }
        }
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
            // Fallback to old system - fix borrowing issue here
            let colors = self.settings.get_current_colors();
            let content_frame = egui::Frame::default()
                .fill(colors.panel_background_color32())
                .inner_margin(egui::Margin::same(10.0));

            // Clone the current_tab to avoid borrowing conflict
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
        // Apply theme to the entire application
        self.settings.apply_theme(ctx);

        // Handle keyboard shortcuts
        self.keyboard_handler.handle_input(ctx);
        self.handle_keyboard_shortcuts();

        // Handle file drops
        let dropped_files = self
            .file_drop_handler
            .handle_dropped_files(ctx, &mut self.status);
        for dropped_file in dropped_files {
            if let Some(path_str) = dropped_file.path.to_str() {
                self.tab_manager
                    .add_file_tab(dropped_file.tab_type, path_str.to_string());
            }
        }

        // Handle tab selector
        if let Some(selected_tab) = self
            .tab_selector
            .display(ctx, &self.settings, &mut self.status)
        {
            self.tab_manager.add_tab(selected_tab);
        }

        // Request a repaint frequently if the timer is running
        if self.timer.is_running {
            ctx.request_repaint();
        }

        let colors = self.settings.get_current_colors();

        // Set the main background color
        let main_frame = egui::Frame::default()
            .fill(colors.background_color32())
            .inner_margin(egui::Margin::ZERO);

        egui::CentralPanel::default()
            .frame(main_frame)
            .show(ctx, |ui| {
                // Render tab bar (new system)
                if !self.tab_manager.tabs.is_empty() {
                    self.render_tab_bar(ui);
                    ui.separator();
                }

                // Render navigation (old system for backward compatibility)
                self.render_navigation(ui);

                if !self.tab_manager.tabs.is_empty()
                    || self.settings.navigation_layout == NavigationLayout::Horizontal
                {
                    ui.separator();
                }

                self.render_main_content(ui, ctx);
            });
    }
}

