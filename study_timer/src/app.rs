use crate::data::StudyData;
use crate::debug::DebugTools;
use crate::timer::Timer;
use crate::ui;

use eframe::{egui, CreationContext};
use std::time::Instant;

pub enum Tab {
    Timer,
    Stats,
    Record,
    Graph,
    Todo,
    Calculator,
    Markdown,
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
    pub current_tab: Tab,
    pub status: StatusMessage,
    pub debug_tools: DebugTools,
    pub markdown_editor: Option<crate::ui::markdown_editor::MarkdownEditor>,
}

impl StudyTimerApp {
    pub fn new(_cc: &CreationContext<'_>) -> Self {
        let study_data = StudyData::load().unwrap_or_default();

        Self {
            timer: Timer::new(),
            study_data,
            current_tab: Tab::Timer,
            status: StatusMessage::new(),
            debug_tools: DebugTools::new(),
            markdown_editor: None, // Will be initialized when first accessed
        }
    }
}

impl eframe::App for StudyTimerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Request a repaint frequently if the timer is running
        if self.timer.is_running {
            ctx.request_repaint();
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui
                    .selectable_label(matches!(self.current_tab, Tab::Timer), "⏱ Timer")
                    .clicked()
                {
                    self.current_tab = Tab::Timer;
                }

                if ui
                    .selectable_label(matches!(self.current_tab, Tab::Record), "✍️ Record")
                    .clicked()
                {
                    self.current_tab = Tab::Record;
                }

                if ui
                    .selectable_label(matches!(self.current_tab, Tab::Stats), "📊 Statistics")
                    .clicked()
                {
                    self.current_tab = Tab::Stats;
                }

                if ui
                    .selectable_label(matches!(self.current_tab, Tab::Graph), "📈 Graph")
                    .clicked()
                {
                    self.current_tab = Tab::Graph;
                }

                if ui
                    .selectable_label(matches!(self.current_tab, Tab::Todo), "✓ Todo")
                    .clicked()
                {
                    self.current_tab = Tab::Todo;
                }

                if ui
                    .selectable_label(matches!(self.current_tab, Tab::Calculator), "🔢 Calculator")
                    .clicked()
                {
                    self.current_tab = Tab::Calculator;
                }

                if ui
                    .selectable_label(matches!(self.current_tab, Tab::Markdown), "📝 Markdown")
                    .clicked()
                {
                    self.current_tab = Tab::Markdown;
                }
            });

            ui.separator();

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
                Tab::Calculator => ui::calculator_tab::display(ui, &mut self.status),
                Tab::Markdown => ui::markdown_tab_ui::display(ui, self), // Add the markdown tab display
            }
        });
    }
}

