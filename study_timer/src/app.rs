use crate::data::StudyData;
use crate::debug::DebugTools;
use crate::settings::{AppSettings, NavigationLayout};
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
    pub current_tab: Tab,
    pub status: StatusMessage,
    pub debug_tools: DebugTools,
    pub markdown_editor: Option<crate::ui::markdown_editor::MarkdownEditor>,
    pub terminal: TerminalEmulator,
}

impl StudyTimerApp {
    pub fn new(_cc: &CreationContext<'_>) -> Self {
        let study_data = StudyData::load().unwrap_or_default();
        let settings = AppSettings::load().unwrap_or_default();
        let current_tab = settings.get_first_enabled_tab();

        Self {
            timer: Timer::new(),
            study_data,
            settings,
            current_tab,
            status: StatusMessage::new(),
            debug_tools: DebugTools::new(),
            markdown_editor: None,
            terminal: TerminalEmulator::new(),
        }
    }

    fn render_navigation(&mut self, ui: &mut egui::Ui) {
        let enabled_tabs = self.settings.get_enabled_tabs();

        match self.settings.navigation_layout {
            NavigationLayout::Horizontal => {
                ui.horizontal(|ui| {
                    for config in enabled_tabs {
                        let is_current = self.current_tab == config.tab_type;
                        let display_name = config.get_display_name();

                        if ui.selectable_label(is_current, display_name).clicked() {
                            self.current_tab = config.tab_type.clone();
                        }
                    }
                });
            }
            NavigationLayout::Vertical => {
                egui::SidePanel::left("navigation_panel")
                    .resizable(true)
                    .default_width(150.0)
                    .width_range(120.0..=250.0)
                    .show_inside(ui, |ui| {
                        ui.vertical(|ui| {
                            ui.heading("Navigation");
                            ui.separator();
                            ui.add_space(5.0);

                            for config in enabled_tabs {
                                let is_current = self.current_tab == config.tab_type;
                                let display_name = config.get_display_name();

                                if ui.selectable_label(is_current, display_name).clicked() {
                                    self.current_tab = config.tab_type.clone();
                                }
                                ui.add_space(2.0);
                            }
                        });
                    });
            }
        }
    }

    fn render_main_content(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        match self.current_tab {
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
            Tab::Todo => ui::todo_tab::display(ui, &mut self.study_data, &mut self.status),
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
        // Request a repaint frequently if the timer is running
        if self.timer.is_running {
            ctx.request_repaint();
        }

        match self.settings.navigation_layout {
            NavigationLayout::Horizontal => {
                egui::CentralPanel::default().show(ctx, |ui| {
                    self.render_navigation(ui);
                    ui.separator();
                    self.render_main_content(ui, ctx);
                });
            }
            NavigationLayout::Vertical => {
                egui::CentralPanel::default().show(ctx, |ui| {
                    self.render_navigation(ui);

                    // Create a separate area for main content
                    let available_rect = ui.available_rect_before_wrap();
                    let content_rect = egui::Rect::from_min_size(
                        egui::pos2(available_rect.left() + 160.0, available_rect.top()),
                        egui::vec2(available_rect.width() - 160.0, available_rect.height()),
                    );

                    let mut content_ui =
                        ui.child_ui(content_rect, egui::Layout::top_down(egui::Align::LEFT));
                    self.render_main_content(&mut content_ui, ctx);
                });
            }
        }
    }
}

