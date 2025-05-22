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
        Self::render_split_pane_content(&mut top_ui, app, ctx, &split_pane.left_tab_id, false);

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

        // Handle tab drops on splitter for swapping
        if let Some(_dragging_tab_id) = &app.dragging_tab_id {
            if splitter_response.hovered() && ui.input(|i| i.pointer.any_released()) {
                app.tab_manager.swap_split_tabs();
                app.status.show("Split panes swapped");
            }
        }

        // Render bottom pane
        let mut bottom_ui = ui.child_ui(bottom_rect, egui::Layout::top_down(egui::Align::LEFT));
        Self::render_split_pane_content(&mut bottom_ui, app, ctx, &split_pane.right_tab_id, true);
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
        Self::render_split_pane_content(&mut left_ui, app, ctx, &split_pane.left_tab_id, false);

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

        // Handle tab drops on splitter for swapping
        if let Some(_dragging_tab_id) = &app.dragging_tab_id {
            if splitter_response.hovered() && ui.input(|i| i.pointer.any_released()) {
                app.tab_manager.swap_split_tabs();
                app.status.show("Split panes swapped");
            }
        }

        // Render right pane
        let mut right_ui = ui.child_ui(right_rect, egui::Layout::top_down(egui::Align::LEFT));
        Self::render_split_pane_content(&mut right_ui, app, ctx, &split_pane.right_tab_id, true);
    }

    fn render_split_pane_content(
        ui: &mut egui::Ui,
        app: &mut StudyTimerApp,
        ctx: &egui::Context,
        tab_id: &str,
        is_right_pane: bool,
    ) {
        // Track which pane is being used
        if ui.rect_contains_pointer(ui.available_rect_before_wrap()) {
            app.update_last_used_split_pane(is_right_pane);
        }

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
                // Tab header with controls - only show on one pane (left/top)
                if !is_right_pane {
                    ui.horizontal(|ui| {
                        // Tab selector dropdown
                        Self::render_split_tab_selector(ui, app, is_right_pane);

                        ui.separator();

                        // Current tab title
                        ui.label(egui::RichText::new(&title).strong());

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            // Close split button
                            if ui.button("❌ Close Split").clicked() {
                                app.tab_manager.close_split();
                            }

                            // Swap panes button
                            if ui.button("🔄 Swap").clicked() {
                                app.tab_manager.swap_split_tabs();
                            }
                        });
                    });
                } else {
                    // For right pane, just show tab selector and title
                    ui.horizontal(|ui| {
                        // Tab selector dropdown
                        Self::render_split_tab_selector(ui, app, is_right_pane);

                        ui.separator();

                        // Current tab title
                        ui.label(egui::RichText::new(&title).strong());
                    });
                }

                ui.separator();

                // Handle drop zones for tab dragging
                let pane_rect = ui.available_rect_before_wrap();
                if let Some(dragging_tab_id) = &app.dragging_tab_id {
                    if ui.rect_contains_pointer(pane_rect) {
                        // Visual feedback for drop zone
                        ui.painter().rect_stroke(
                            pane_rect,
                            egui::Rounding::same(5.0),
                            egui::Stroke::new(3.0, colors.accent_color32().gamma_multiply(0.7)),
                        );

                        // Handle drop
                        if ui.input(|i| i.pointer.any_released()) {
                            app.tab_manager
                                .move_tab_to_split(dragging_tab_id, is_right_pane);
                            app.status.show(&format!(
                                "Tab moved to {} pane",
                                if is_right_pane { "right" } else { "left" }
                            ));
                        }
                    }
                }

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

    fn render_split_tab_selector(ui: &mut egui::Ui, app: &mut StudyTimerApp, is_right_pane: bool) {
        let current_tab_id = if is_right_pane {
            app.tab_manager
                .get_split_pane()
                .map(|s| s.right_tab_id.clone())
        } else {
            app.tab_manager
                .get_split_pane()
                .map(|s| s.left_tab_id.clone())
        };

        if let Some(current_id) = current_tab_id {
            let current_title = app
                .tab_manager
                .get_tab(&current_id)
                .map(|tab| tab.get_display_title())
                .unwrap_or_else(|| "Unknown".to_string());

            // Collect tab information first to avoid borrowing conflicts
            let tab_options: Vec<(String, String)> = app
                .tab_manager
                .tabs
                .iter()
                .map(|tab| (tab.id.clone(), tab.get_display_title()))
                .collect();

            egui::ComboBox::from_id_source(format!("split_tab_selector_{}", is_right_pane))
                .selected_text(current_title)
                .width(120.0)
                .show_ui(ui, |ui| {
                    for (tab_id, tab_title) in tab_options {
                        let is_selected = tab_id == current_id;
                        let selectable = ui.selectable_label(is_selected, tab_title);

                        if selectable.clicked() && !is_selected {
                            app.tab_manager.set_split_active_tab(&tab_id, is_right_pane);
                            // Update the last used split pane when user interacts with it
                            app.update_last_used_split_pane(is_right_pane);
                        }
                    }
                });
        }
    }
}

