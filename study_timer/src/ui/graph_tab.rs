use crate::app::StatusMessage;
use crate::data::StudyData;
use charming::{
    component::{Axis, Title},
    element::AxisType,
    series::Line,
    Chart,
};
use chrono::{Datelike, Duration, Local, NaiveDate};
use eframe::egui;
use eframe::egui::Ui;
use std::cell::RefCell;

pub struct GraphState {
    week_offset: i64, // 0 is current week, -1 is last week, 1 is next week, etc.
}

impl Default for GraphState {
    fn default() -> Self {
        Self { week_offset: 0 }
    }
}

thread_local! {
    static GRAPH_STATE: RefCell<GraphState> = RefCell::new(GraphState::default());
}

// Helper function to get the start of a week containing the given date
fn start_of_week(date: NaiveDate) -> NaiveDate {
    // Start of week is Monday
    let days_from_monday = date.weekday().num_days_from_monday() as i64;
    date - Duration::days(days_from_monday)
}

pub fn display(ui: &mut Ui, study_data: &StudyData, status: &mut StatusMessage) {
    ui.vertical_centered(|ui| {
        ui.heading("Weekly Study Graph");
    });
    ui.add_space(10.0);

    // Navigation buttons for week selection with fixed width buttons
    ui.horizontal(|ui| {
        // Split available width evenly between buttons
        let button_width = ui.available_width() / 3.0 - 10.0; // Subtract some margin

        let mut week_changed = false;

        GRAPH_STATE.with(|state| {
            let mut state = state.borrow_mut();

            if ui
                .add_sized([button_width, 24.0], egui::Button::new("Previous Week"))
                .clicked()
            {
                state.week_offset -= 1;
                week_changed = true;
            }

            if ui
                .add_sized([button_width, 24.0], egui::Button::new("Current Week"))
                .clicked()
            {
                state.week_offset = 0;
                week_changed = true;
            }

            if ui
                .add_sized([button_width, 24.0], egui::Button::new("Next Week "))
                .clicked()
            {
                state.week_offset += 1;
                week_changed = true;
            }
        });

        if week_changed {
            status.show("Week changed");
        }
    });

    ui.add_space(10.0);

    // Generate week dates and labels
    let today = Local::now().date_naive();
    let week_offset = GRAPH_STATE.with(|state| state.borrow().week_offset);

    let week_start = start_of_week(today) + Duration::days(week_offset * 7);
    let week_end = week_start + Duration::days(6);

    // Display week range
    ui.vertical_centered(|ui| {
        ui.label(format!(
            "Week: {} to {}",
            week_start.format("%b %d, %Y"),
            week_end.format("%b %d, %Y")
        ));
    });

    ui.add_space(20.0);

    // Create week days and data for the chart
    let week_days = (0..7)
        .map(|day| {
            let date = week_start + Duration::days(day);
            date.format("%a").to_string() // Mon, Tue, etc.
        })
        .collect::<Vec<_>>();

    let week_data = (0..7)
        .map(|day| {
            let date = week_start + Duration::days(day);
            let date_str = date.format("%Y-%m-%d").to_string();

            // Sum minutes for this day
            study_data
                .sessions
                .iter()
                .filter(|s| s.date == date_str)
                .map(|s| s.minutes)
                .sum::<f64>()
        })
        .collect::<Vec<_>>();

    // Make the chart even smaller to ensure all days are visible
    // Reduce width further to 70% of available width
    let available_width = ui.available_width() * 0.7;
    let available_height = 250.0;

    // Define padding for the graph with more generous right padding
    let padding_left = 50.0;
    let padding_right = 50.0; // Increased right padding even more
    let padding_top = 20.0;
    let padding_bottom = 30.0;

    // Create and render the chart
    let _chart = Chart::new()
        .title(Title::new().text("Daily Study Minutes"))
        .x_axis(
            Axis::new()
                .type_(AxisType::Category)
                .data(week_days.clone()),
        )
        .y_axis(Axis::new().type_(AxisType::Value))
        .series(Line::new().data(week_data.clone()));

    // Display the chart - center it
    ui.vertical_centered(|ui| {
        let size = egui::vec2(available_width, available_height);
        let (rect, _) = ui.allocate_at_least(size, egui::Sense::hover());

        let painter = ui.painter();

        // Fill background with dark grey (#1B1B1B)
        painter.rect_filled(rect, 4.0, egui::Color32::from_rgb(27, 27, 27));

        // Create inner rectangle with padding
        let inner_rect = egui::Rect::from_min_max(
            egui::pos2(rect.left() + padding_left, rect.top() + padding_top),
            egui::pos2(rect.right() - padding_right, rect.bottom() - padding_bottom),
        );

        // Render chart data
        let max_value = week_data.iter().fold(0.0, |acc: f64, &x| acc.max(x));

        // Ensure there's always some scale, even if data is empty
        let max_value_display = if max_value < 1.0 { 10.0 } else { max_value };

        let y_scale = if max_value > 0.0 {
            inner_rect.height() / max_value as f32
        } else {
            inner_rect.height() / 10.0 // Default scale if no data
        };

        // Distribute bars evenly with narrower bars
        let day_count = 7.0;
        let bar_spacing = inner_rect.width() / day_count;
        let bar_width = bar_spacing * 0.5; // Reduced from 0.6 to 0.5 to make bars narrower

        // Define colors
        let dark_blue = egui::Color32::from_rgb(66, 133, 244); // Bar color
        let text_color = egui::Color32::from_rgb(220, 220, 220); // Light grey text
        let grid_color = egui::Color32::from_rgb(70, 70, 70); // Subtle grid lines

        // Draw bars
        for (i, &value) in week_data.iter().enumerate() {
            // Calculate x position with even spacing
            let x_center = inner_rect.left() + (i as f32 + 0.5) * bar_spacing;

            let bar_height = if value > 0.0 {
                value as f32 * y_scale
            } else {
                0.0
            };
            let bar_rect = egui::Rect::from_min_size(
                egui::pos2(x_center - bar_width / 2.0, inner_rect.bottom() - bar_height),
                egui::vec2(bar_width, bar_height),
            );

            painter.rect_filled(bar_rect, 3.0, dark_blue);

            // Draw day label below x-axis
            let day_label = &week_days[i];
            painter.text(
                egui::pos2(x_center, inner_rect.bottom() + 5.0),
                egui::Align2::CENTER_TOP,
                day_label,
                egui::FontId::default(),
                text_color,
            );

            // Draw value above bar
            if value > 0.0 {
                painter.text(
                    egui::pos2(x_center, bar_rect.top() - 5.0),
                    egui::Align2::CENTER_BOTTOM,
                    format!("{:.1}", value),
                    egui::FontId::default(),
                    text_color,
                );
            }
        }

        // Draw axes
        painter.line_segment(
            [
                egui::pos2(inner_rect.left(), inner_rect.bottom()),
                egui::pos2(inner_rect.right(), inner_rect.bottom()),
            ],
            egui::Stroke::new(1.5, text_color),
        );

        painter.line_segment(
            [
                egui::pos2(inner_rect.left(), inner_rect.bottom()),
                egui::pos2(inner_rect.left(), inner_rect.top()),
            ],
            egui::Stroke::new(1.5, text_color),
        );

        // Draw horizontal grid lines
        let grid_steps = 5; // Number of grid lines
        let grid_step_value = max_value_display / grid_steps as f64;

        for i in 1..=grid_steps {
            let y_pos = inner_rect.bottom() - (grid_step_value * i as f64) as f32 * y_scale;

            // Draw grid line
            painter.line_segment(
                [
                    egui::pos2(inner_rect.left(), y_pos),
                    egui::pos2(inner_rect.right(), y_pos),
                ],
                egui::Stroke::new(0.5, grid_color),
            );

            // Draw y-axis label
            painter.text(
                egui::pos2(inner_rect.left() - 5.0, y_pos),
                egui::Align2::RIGHT_CENTER,
                format!("{:.0}", grid_step_value * i as f64),
                egui::FontId::default(),
                text_color,
            );
        }
    });

    ui.add_space(20.0);

    // Add GitHub-style yearly commit streak heatmap
    ui.vertical_centered(|ui| {
        ui.heading("Annual Study Activity");
        ui.add_space(10.0);
    });

    // Create a heatmap with GitHub-like appearance
    // This will always show the full year regardless of week selection
    render_heatmap(ui, study_data, week_start);

    ui.add_space(10.0);

    // Week summary
    let week_total = week_data.iter().sum::<f64>();
    let avg_per_day = if week_total > 0.0 {
        week_total / 7.0
    } else {
        0.0
    };

    ui.vertical_centered(|ui| {
        ui.label(format!(
            "Week total: {:.1} minutes ({:.1} hours)",
            week_total,
            week_total / 60.0
        ));
        ui.label(format!(
            "Daily average: {:.1} minutes ({:.1} hours)",
            avg_per_day,
            avg_per_day / 60.0
        ));
    });

    // Status message
    status.render(ui);
}

// Redesigned render_heatmap function in GitHub style showing a full year

fn render_heatmap(ui: &mut Ui, study_data: &StudyData, _current_week_start: NaiveDate) {
    // Use today's date to determine the year to display
    let today = Local::now().date_naive();
    let current_year = today.year();

    // Start from beginning of current year
    let year_start = NaiveDate::from_ymd_opt(current_year, 1, 1).unwrap();

    // End of current year (for complete year display)
    let year_end = NaiveDate::from_ymd_opt(current_year, 12, 31).unwrap();

    // Calculate available width and height for the heatmap
    let available_width = ui.available_width() * 0.95;
    let available_height = 160.0;

    ui.vertical_centered(|ui| {
        // Add year label
        ui.label(format!("Study Activity for {}", current_year));
        ui.add_space(5.0);

        let size = egui::vec2(available_width, available_height);
        let (rect, _) = ui.allocate_at_least(size, egui::Sense::hover());

        let painter = ui.painter();

        // Fill background with dark grey
        painter.rect_filled(rect, 4.0, egui::Color32::from_rgb(27, 27, 27));

        // Define grid layout
        let day_count = 7; // 7 days per week

        // Calculate total weeks in the year plus padding
        let first_day_offset = year_start.weekday().num_days_from_sunday() as usize;
        let total_days_in_year = if year_end.leap_year() { 366 } else { 365 };
        let total_weeks = (total_days_in_year + first_day_offset + 6) / 7;

        // Use total weeks in the year instead of current date
        let week_count = total_weeks;

        // Calculate cell size based on available space
        let horizontal_padding = 60.0;
        let vertical_padding = 30.0;
        let grid_width = rect.width() - horizontal_padding;
        let grid_height = rect.height() - 2.0 * vertical_padding;

        let cell_width = grid_width / week_count as f32;
        let cell_height = grid_height / day_count as f32;
        let cell_size = cell_width.min(cell_height) * 0.85;
        let cell_margin = cell_size * 0.2;

        // Draw month labels at the top
        let months = [
            "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
        ];
        for month in 0..12 {
            // Calculate the first day of the month
            let month_date = NaiveDate::from_ymd_opt(current_year, month as u32 + 1, 1).unwrap();
            // Calculate the week number (0-based) for this month
            let days_since_start = (month_date - year_start).num_days();
            let week_num = (days_since_start / 7) as f32;

            // Position the month label
            let x_pos = rect.left() + horizontal_padding + week_num * (cell_size + cell_margin);

            painter.text(
                egui::pos2(x_pos, rect.top() + vertical_padding / 2.0),
                egui::Align2::LEFT_CENTER,
                months[month],
                egui::FontId::default(),
                egui::Color32::from_rgb(180, 180, 180),
            );
        }

        // Draw day labels (vertical axis)
        let days = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
        for (i, day) in days.iter().enumerate() {
            let y_pos = rect.top()
                + vertical_padding
                + (i as f32) * (cell_size + cell_margin)
                + cell_size / 2.0;
            painter.text(
                egui::pos2(rect.left() + horizontal_padding / 2.0, y_pos),
                egui::Align2::RIGHT_CENTER,
                day,
                egui::FontId::default(),
                egui::Color32::from_rgb(180, 180, 180),
            );
        }

        // Pre-calculate activity data for all days in the year for efficient lookup
        let mut activity_by_date: std::collections::HashMap<String, f64> =
            std::collections::HashMap::new();
        for session in &study_data.sessions {
            let entry = activity_by_date.entry(session.date.clone()).or_insert(0.0);
            *entry += session.minutes;
        }

        // Track which cell is being hovered to draw tooltip last (on top of everything)
        let mut hovered_cell: Option<(NaiveDate, egui::Rect, f64)> = None;

        // Draw cells for the entire year
        let mut current_date = year_start;

        // Continue until end of year instead of just today
        while current_date <= year_end {
            let week_of_year = (current_date.ordinal0() + first_day_offset as u32) / 7;
            let day_of_week = current_date.weekday().num_days_from_sunday() as usize;

            let date_str = current_date.format("%Y-%m-%d").to_string();

            // Look up activity using our pre-calculated HashMap for better performance
            let activity_level = *activity_by_date.get(&date_str).unwrap_or(&0.0);

            // Map activity level to a color intensity - keep the same color mappings
            let color = if activity_level <= 0.0 {
                egui::Color32::from_rgb(40, 40, 40) // Empty cell
            } else if activity_level < 30.0 {
                egui::Color32::from_rgb(0, 109, 50) // Light green
            } else if activity_level < 60.0 {
                egui::Color32::from_rgb(38, 166, 65) // Medium green
            } else if activity_level < 120.0 {
                egui::Color32::from_rgb(57, 211, 83) // Strong green
            } else {
                egui::Color32::from_rgb(105, 255, 124) // Very intense green
            };

            // Position cells using week_of_year which properly accounts for the position in the year
            let x = rect.left()
                + horizontal_padding
                + (week_of_year as f32) * (cell_size + cell_margin);
            let y =
                rect.top() + vertical_padding + (day_of_week as f32) * (cell_size + cell_margin);

            let cell_rect =
                egui::Rect::from_min_size(egui::pos2(x, y), egui::vec2(cell_size, cell_size));

            painter.rect_filled(cell_rect, 2.0, color);

            // Store information about hovered cell to draw tooltip later
            if activity_level > 0.0 && ui.rect_contains_pointer(cell_rect) {
                hovered_cell = Some((current_date, cell_rect, activity_level));
            }

            // Move to next day
            current_date += Duration::days(1);
        }

        // Draw tooltip for hovered cell (after all cells, to be on top)
        if let Some((date, cell_rect, activity_level)) = hovered_cell {
            let tooltip_text =
                format!("{}: {:.1} minutes", date.format("%Y-%m-%d"), activity_level);

            // Position tooltip above the cell
            let tooltip_pos = egui::pos2(cell_rect.center().x, cell_rect.top() - 5.0);

            // Use egui's built-in text size calculation
            let galley = ui.painter().layout_no_wrap(
                tooltip_text.clone(),
                egui::FontId::default(),
                egui::Color32::WHITE,
            );
            let text_size = galley.size();

            // Tooltip background with increased z-index
            let tooltip_padding = 5.0;
            let tooltip_rect = egui::Rect::from_min_size(
                egui::pos2(
                    tooltip_pos.x - text_size.x / 2.0 - tooltip_padding,
                    tooltip_pos.y - text_size.y - tooltip_padding,
                ),
                egui::vec2(
                    text_size.x + 2.0 * tooltip_padding,
                    text_size.y + 2.0 * tooltip_padding,
                ),
            );

            // Draw with solid background to cover any grid elements
            painter.rect_filled(tooltip_rect, 4.0, egui::Color32::from_rgb(60, 60, 60));
            painter.text(
                egui::pos2(tooltip_pos.x, tooltip_pos.y - tooltip_padding),
                egui::Align2::CENTER_BOTTOM,
                tooltip_text,
                egui::FontId::default(),
                egui::Color32::WHITE,
            );
        }

        // Add a legend for the heatmap with increased spacing
        let legend_y = rect.bottom() - vertical_padding / 2.0;
        let legend_width = cell_size * 0.8;
        let legend_height = cell_size * 0.8;
        let legend_start_x = rect.left() + horizontal_padding;

        // Increase spacing between legend items to prevent overlap
        let legend_spacing = legend_width * 8.5; // Increased from 2.5 to 3.5

        // Draw legend label with more space
        painter.text(
            egui::pos2(legend_start_x - legend_width * 1.5, legend_y),
            egui::Align2::RIGHT_CENTER,
            "Activity:",
            egui::FontId::default(),
            egui::Color32::from_rgb(180, 180, 180),
        );

        // Draw legend color boxes with labels
        let legend_colors = [
            (egui::Color32::from_rgb(40, 40, 40), "None"),
            (egui::Color32::from_rgb(0, 109, 50), "<30m"),
            (egui::Color32::from_rgb(38, 166, 65), "<60m"),
            (egui::Color32::from_rgb(57, 211, 83), "<120m"),
            (egui::Color32::from_rgb(105, 255, 124), "≥120m"),
        ];

        for (i, (color, label)) in legend_colors.iter().enumerate() {
            let x = legend_start_x + (i as f32) * legend_spacing;

            // Draw color box
            painter.rect_filled(
                egui::Rect::from_min_size(
                    egui::pos2(x, legend_y - legend_height / 2.0),
                    egui::vec2(legend_width, legend_height),
                ),
                2.0,
                *color,
            );

            // Draw label with more spacing
            painter.text(
                egui::pos2(x + legend_width + 5.0, legend_y),
                egui::Align2::LEFT_CENTER,
                label,
                egui::FontId::default(),
                egui::Color32::from_rgb(180, 180, 180),
            );
        }
    });
}
