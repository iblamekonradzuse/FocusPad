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
    }

    fn render_tab_bar(&mut self, ui: &mut egui::Ui) {
        let colors = self.settings.get_current_colors();

        let tab_bar_frame = egui::Frame::default()
            .fill(colors.navigation_background_color32())
            .inner_margin(egui::Margin::symmetric(8.0, 4.0));

        tab_bar_frame.show(ui, |ui| {
            ui.horizontal(|ui| {
                egui::ScrollArea::horizontal()
                    .id_source("tab_bar_scroll")
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            let tabs_to_render: Vec<_> = self
                                .tab_manager
                                .tabs
                                .iter()
                                .filter(|tab| tab.tab_type != Tab::Settings)
                                .cloned()
                                .collect();

                            for (index, tab) in tabs_to_render.iter().enumerate() {
                                let is_active = tab.id == self.tab_manager.active_tab_id;
                                self.render_draggable_tab(ui, tab, is_active, index);
                            }

                            let new_tab_button = egui::Button::new("+")
                                .fill(colors.accent_color32())
                                .stroke(egui::Stroke::new(1.0, colors.text_primary_color32()));

                            if ui.add(new_tab_button).clicked() {
                                self.tab_selector.show();
                            }
                        });
                    });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let settings_tab_info = self
                        .tab_manager
                        .tabs
                        .iter()
                        .find(|tab| tab.tab_type == Tab::Settings)
                        .map(|tab| (tab.id.clone(), tab.id == self.tab_manager.active_tab_id));

                    if let Some((settings_tab_id, is_active)) = settings_tab_info {
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

                        let button =
                            egui::Button::new(egui::RichText::new("⚙️ Settings").color(text_color))
                                .fill(button_color)
                                .stroke(egui::Stroke::new(1.0, colors.accent_color32()));

                        if ui.add(button).clicked() {
                            if self.tab_manager.is_split_active() {
                                self.tab_manager.set_split_active_tab(
                                    &settings_tab_id,
                                    self.last_used_split_pane,
                                );
                            } else {
                                self.tab_manager.set_active_tab(&settings_tab_id);
                            }
                        }
                    }

                    // Add weather widget
                    ui.separator();
                    if self.weather_widget.render(ui) {
                        // Save weather settings when city changes
                        let _ = self.weather_widget.save();
                    }

                    // Add split buttons next to weather widget
                    if !self.tab_manager.is_split_active() {
                        ui.separator();

                        if ui.button("⬍").clicked() {
                            self.tab_manager.create_split(SplitDirection::Horizontal);
                        }
                        if ui.button("⬌").clicked() {
                            self.tab_manager.create_split(SplitDirection::Vertical);
                        }
                    }
                });
            });
        });
    }

    fn render_draggable_tab(
        &mut self,
        ui: &mut egui::Ui,
        tab: &crate::tab_manager::TabInstance,
        is_active: bool,
        index: usize,
    ) {
        let colors = self.settings.get_current_colors();

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

            let button =
                egui::Button::new(egui::RichText::new(&tab.get_display_title()).color(text_color))
                    .fill(button_color)
                    .stroke(egui::Stroke::new(1.0, colors.accent_color32()));

            let response = ui.add(button);

            if response.clicked() && self.dragging_tab_id.is_none() {
                if self.tab_manager.is_split_active() {
                    self.tab_manager
                        .set_split_active_tab(&tab.id, self.last_used_split_pane);
                } else {
                    self.tab_manager.set_active_tab(&tab.id);
                }
            }

            if response.drag_started() {
                self.dragging_tab_id = Some(tab.id.clone());
                self.drag_start_pos = response.interact_pointer_pos();
            }

            if response.dragged() && self.dragging_tab_id == Some(tab.id.clone()) {
                ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::Grabbing);

                if let Some(pointer_pos) = response.interact_pointer_pos() {
                    ui.painter().circle_filled(
                        pointer_pos,
                        8.0,
                        colors.accent_color32().gamma_multiply(0.7),
                    );
                }
            }

            if response.drag_released() && self.dragging_tab_id == Some(tab.id.clone()) {
                if let Some(drop_pos) = response.interact_pointer_pos() {
                    self.handle_tab_drop(drop_pos, &tab.id);
                }
                self.dragging_tab_id = None;
                self.drag_start_pos = None;
            }

            if self.dragging_tab_id.is_some() && self.dragging_tab_id != Some(tab.id.clone()) {
                if response.hovered() {
                    let rect = response.rect;
                    ui.painter().rect_stroke(
                        rect.expand(2.0),
                        egui::Rounding::same(3.0),
                        egui::Stroke::new(2.0, colors.accent_color32()),
                    );

                    if ui.input(|i| i.pointer.any_released()) {
                        if let Some(dragging_id) = &self.dragging_tab_id {
                            self.tab_manager.reorder_tab(dragging_id, index);
                        }
                    }
                }
            }

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
}

