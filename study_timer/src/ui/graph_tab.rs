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

            if ui.add_sized([button_width, 24.0], egui::Button::new("← Previous Week")).clicked() {
                state.week_offset -= 1;
                week_changed = true;
            }

            if ui.add_sized([button_width, 24.0], egui::Button::new("Current Week")).clicked() {
                state.week_offset = 0;
                week_changed = true;
            }

            if ui.add_sized([button_width, 24.0], egui::Button::new("Next Week →")).clicked() {
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
    
    let inner_width = available_width - (padding_left + padding_right);
    let inner_height = available_height - (padding_top + padding_bottom);

    // Create and render the chart
    let chart = Chart::new()
        .title(Title::new().text("Daily Study Minutes"))
        .x_axis(Axis::new().type_(AxisType::Category).data(week_days.clone()))
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
            egui::pos2(rect.right() - padding_right, rect.bottom() - padding_bottom)
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
            
            let bar_height = if value > 0.0 { value as f32 * y_scale } else { 0.0 };
            let bar_rect = egui::Rect::from_min_size(
                egui::pos2(x_center - bar_width/2.0, inner_rect.bottom() - bar_height),
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
        let grid_steps = 5;  // Number of grid lines
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
