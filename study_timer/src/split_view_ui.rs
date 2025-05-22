use crate::app::StudyTimerApp;
use crate::tab_manager::{SplitDirection, SplitPane};
use eframe::egui;

pub struct SplitViewUI;

impl SplitViewUI {
    pub fn display(ui: &mut egui::Ui, app: &mut StudyTimerApp, ctx: &egui::Context) {
        if let Some(ref split_pane) = app.tab_manager.split_pane.clone() {
            match split_pane.direction {
                SplitDirection::Horizontal => {
                    Self::render_horizontal_split(ui, app, ctx, split_pane);
                }
                SplitDirection::Vertical => {
                    Self::render_vertical_split(ui, app, ctx, split_pane);
                }
            }
        }
    }

    fn render_horizontal_split(
        ui: &mut egui::Ui,
        app: &mut StudyTimerApp,
        ctx: &egui::Context,
        split_pane: &SplitPane,
    ) {
        let available_rect = ui.available_rect_before_wrap();
        let split_pos = available_rect.height() * split_pane.split_ratio;

        // Top pane
        let top_rect = egui::Rect::from_min_size(
            available_rect.min,
            egui::Vec2::new(available_rect.width(), split_pos - 2.0),
        );

        // Bottom pane
        let bottom_rect = egui::Rect::from_min_size(
            egui::Pos2::new(available_rect.min.x, available_rect.min.y + split_pos + 2.0),
            egui::Vec2::new(
                available_rect.width(),
                available_rect.height() - split_pos - 2.0,
            ),
        );

        // Render top pane
        let mut top_ui = ui.child_ui(top_rect, egui::Layout::top_down(egui::Align::LEFT));
        Self::render_tab_content(&mut top_ui, app, ctx, &split_pane.left_tab_id);

        // Render splitter
        let splitter_rect = egui::Rect::from_min_size(
            egui::Pos2::new(available_rect.min.x, available_rect.min.y + split_pos - 2.0),
            egui::Vec2::new(available_rect.width(), 4.0),
        );

        let splitter_response = ui.allocate_rect(splitter_rect, egui::Sense::drag());
        ui.painter().rect_filled(
            splitter_rect,
            egui::Rounding::ZERO,
            app.settings.get_current_colors().accent_color32(),
        );

        if splitter_response.dragged() {
            let new_ratio = (splitter_response.interact_pointer_pos().unwrap().y
                - available_rect.min.y)
                / available_rect.height();
            app.tab_manager.update_split_ratio(new_ratio);
        }

        // Change cursor when hovering over splitter
        if splitter_response.hovered() {
            ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::ResizeVertical);
        }

        // Render bottom pane
        let mut bottom_ui = ui.child_ui(bottom_rect, egui::Layout::top_down(egui::Align::LEFT));
        Self::render_tab_content(&mut bottom_ui, app, ctx, &split_pane.right_tab_id);
    }

    fn render_vertical_split(
        ui: &mut egui::Ui,
        app: &mut StudyTimerApp,
        ctx: &egui::Context,
        split_pane: &SplitPane,
    ) {
        let available_rect = ui.available_rect_before_wrap();
        let split_pos = available_rect.width() * split_pane.split_ratio;

        // Left pane
        let left_rect = egui::Rect::from_min_size(
            available_rect.min,
            egui::Vec2::new(split_pos - 2.0, available_rect.height()),
        );

        // Right pane
        let right_rect = egui::Rect::from_min_size(
            egui::Pos2::new(available_rect.min.x + split_pos + 2.0, available_rect.min.y),
            egui::Vec2::new(
                available_rect.width() - split_pos - 2.0,
                available_rect.height(),
            ),
        );

        // Render left pane
        let mut left_ui = ui.child_ui(left_rect, egui::Layout::top_down(egui::Align::LEFT));
        Self::render_tab_content(&mut left_ui, app, ctx, &split_pane.left_tab_id);

        // Render splitter
        let splitter_rect = egui::Rect::from_min_size(
            egui::Pos2::new(available_rect.min.x + split_pos - 2.0, available_rect.min.y),
            egui::Vec2::new(4.0, available_rect.height()),
        );

        let splitter_response = ui.allocate_rect(splitter_rect, egui::Sense::drag());
        ui.painter().rect_filled(
            splitter_rect,
            egui::Rounding::ZERO,
            app.settings.get_current_colors().accent_color32(),
        );

        if splitter_response.dragged() {
            let new_ratio = (splitter_response.interact_pointer_pos().unwrap().x
                - available_rect.min.x)
                / available_rect.width();
            app.tab_manager.update_split_ratio(new_ratio);
        }

        // Change cursor when hovering over splitter
        if splitter_response.hovered() {
            ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::ResizeHorizontal);
        }

        // Render right pane
        let mut right_ui = ui.child_ui(right_rect, egui::Layout::top_down(egui::Align::LEFT));
        Self::render_tab_content(&mut right_ui, app, ctx, &split_pane.right_tab_id);
    }

    fn render_tab_content(
        ui: &mut egui::Ui,
        app: &mut StudyTimerApp,
        ctx: &egui::Context,
        tab_id: &str,
    ) {
        // Get the tab information first to avoid borrowing conflicts
        let tab_info = app
            .tab_manager
            .get_tab(tab_id)
            .map(|tab| (tab.get_display_title(), tab.tab_type.clone()));

        if let Some((title, tab_type)) = tab_info {
            let colors = app.settings.get_current_colors();

            // Apply content background
            let content_frame = egui::Frame::default()
                .fill(colors.panel_background_color32())
                .inner_margin(egui::Margin::same(5.0))
                .stroke(egui::Stroke::new(1.0, colors.accent_color32()));

            content_frame.show(ui, |ui| {
                // Show tab title
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(&title).strong());

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("❌").clicked() {
                            app.tab_manager.close_split();
                        }
                    });
                });

                ui.separator();

                // Render tab content based on type
                match tab_type {
                    crate::app::Tab::Timer => crate::ui::timer_tab::display(
                        ui,
                        &mut app.timer,
                        &mut app.study_data,
                        &mut app.debug_tools,
                        &mut app.status,
                    ),
                    crate::app::Tab::Stats => {
                        crate::ui::stats_tab::display(ui, &mut app.study_data, &mut app.status)
                    }
                    crate::app::Tab::Record => crate::ui::record_tab::display(
                        ui,
                        &mut app.study_data,
                        &mut app.status,
                        &app.timer,
                    ),
                    crate::app::Tab::Graph => {
                        crate::ui::graph_tab::display(ui, &app.study_data, &mut app.status)
                    }
                    crate::app::Tab::Todo => crate::ui::todo_tab::display(
                        ui,
                        &mut app.study_data,
                        &mut app.status,
                        &app.settings,
                    ),
                    crate::app::Tab::Reminder => {
                        crate::ui::reminder_tab::display(ui, &mut app.study_data, &mut app.status)
                    }
                    crate::app::Tab::Calculator => {
                        crate::ui::calculator_tab::display(ui, &mut app.status)
                    }
                    crate::app::Tab::Markdown => crate::ui::markdown_tab_ui::display(ui, app, ctx),
                    crate::app::Tab::Terminal => {
                        crate::ui::terminal_tab_ui::display(ui, &mut app.terminal, &mut app.status)
                    }
                    crate::app::Tab::Settings => crate::ui::settings_tab_ui::display(
                        ui,
                        &mut app.settings,
                        &mut app.status,
                        &mut app.current_tab,
                    ),
                }
            });
        }
    }
}

