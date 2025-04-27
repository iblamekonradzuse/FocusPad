use crate::app::StatusMessage;
use crate::data::{NotificationPeriod, Reminder, StudyData};
use chrono::{Local, NaiveDate};
use egui::{ ScrollArea, TextEdit};
use std::cell::RefCell;
use std::collections::HashMap;

thread_local! {
    static NEW_REMINDER_TITLE: RefCell<String> = RefCell::new(String::new());
    static NEW_REMINDER_DESC: RefCell<String> = RefCell::new(String::new());
    static NEW_REMINDER_DATE: RefCell<String> = RefCell::new(String::new());
    static EDITING_MAP: RefCell<HashMap<u64, EditingReminder>> = RefCell::new(HashMap::new());
    static CUSTOM_DAYS: RefCell<String> = RefCell::new(String::from("5"));
}
#[derive(Clone)]
struct EditingReminder {
    title: String,
    description: String,
    due_date: String,
    notification_periods: Vec<NotificationPeriod>,
}

pub fn display(ui: &mut egui::Ui, study_data: &mut StudyData, status: &mut StatusMessage) {
    ui.heading("Reminders");

    // Auto-fill due date with today's date if empty
    NEW_REMINDER_DATE.with(|date_ref| {
        let mut due_date = date_ref.borrow_mut();
        if due_date.is_empty() {
            let today = Local::now().date_naive();
            *due_date = today.format("%Y-%m-%d").to_string();
        }
    });
    // Check for due reminders
    check_due_reminders(study_data, status);

    // Add new reminder section
    ui.collapsing("Add New Reminder", |ui| {
        NEW_REMINDER_TITLE.with(|title_ref| {
            NEW_REMINDER_DESC.with(|desc_ref| {
                NEW_REMINDER_DATE.with(|date_ref| {
                    let mut title = title_ref.borrow_mut();
                    let mut desc = desc_ref.borrow_mut();
                    let mut due_date = date_ref.borrow_mut();

                    ui.horizontal(|ui| {
                        ui.label("Title:");
                        ui.add(
                            TextEdit::singleline(&mut *title)
                                .hint_text("Enter reminder title")
                                .desired_width(280.0),
                        );
                    });

                    ui.horizontal(|ui| {
                        ui.label("Description:");
                        ui.add(
                            TextEdit::multiline(&mut *desc)
                                .hint_text("Enter description (optional)")
                                .desired_width(280.0)
                                .desired_rows(2),
                        );
                    });

                    ui.horizontal(|ui| {
                        ui.label("Due Date:");
                        ui.add(
                            TextEdit::singleline(&mut *due_date)
                                .hint_text("YYYY-MM-DD")
                                .desired_width(280.0),
                        );
                    });

                    // Notification periods selection
                    ui.label("Notification Periods:");

                    // Store notification periods in static variables 
                    thread_local! {
                        static ONE_DAY: RefCell<bool> = RefCell::new(false);
                        static THREE_DAYS: RefCell<bool> = RefCell::new(false);
                        static ONE_WEEK: RefCell<bool> = RefCell::new(false);
                        static CUSTOM: RefCell<bool> = RefCell::new(false);
                    }

                    ONE_DAY.with(|one_day| {
                        THREE_DAYS.with(|three_days| {
                            ONE_WEEK.with(|one_week| {
                                CUSTOM.with(|custom| {
                                    let mut one_day_val = one_day.borrow_mut();
                                    let mut three_days_val = three_days.borrow_mut();
                                    let mut one_week_val = one_week.borrow_mut();
                                    let mut custom_val = custom.borrow_mut();

                                    ui.horizontal(|ui| {
                                        ui.checkbox(&mut *one_day_val, "1 Day Before");
                                        ui.checkbox(&mut *three_days_val, "3 Days Before");
                                        ui.checkbox(&mut *one_week_val, "1 Week Before");
                                    });

                                    ui.horizontal(|ui| {
                                        ui.checkbox(&mut *custom_val, "Custom:");

                                        CUSTOM_DAYS.with(|days_ref| {
                                            let mut custom_days = days_ref.borrow_mut();
                                            ui.add_enabled(
                                                *custom_val,
                                                TextEdit::singleline(&mut *custom_days)
                                                    .hint_text("Days")
                                                    .desired_width(50.0),
                                            );
                                            ui.label("days before");
                                        });
                                    });

                                    if ui.button("Add Reminder").clicked() {
                                        if title.is_empty() {
                                            status.show("Reminder title cannot be empty!");
                                            return;
                                        }

                                        if due_date.is_empty() {
                                            status.show("Due date cannot be empty!");
                                            return;
                                        }

                                        // Validate date format
                                        if NaiveDate::parse_from_str(&due_date, "%Y-%m-%d").is_err() {
                                            status.show("Invalid date format! Use YYYY-MM-DD");
                                            return;
                                        }

                                        // Create notification periods
                                        let mut periods = Vec::new();
                                        if *one_day_val {
                                            periods.push(NotificationPeriod::OneDay);
                                        }
                                        if *three_days_val {
                                            periods.push(NotificationPeriod::ThreeDays);
                                        }
                                        if *one_week_val {
                                            periods.push(NotificationPeriod::OneWeek);
                                        }
                                        if *custom_val {
                                            CUSTOM_DAYS.with(|days_ref| {
                                                let custom_days = days_ref.borrow();
                                                if let Ok(days) = custom_days.parse::<u32>() {
                                                    if days > 0 {
                                                        periods.push(NotificationPeriod::Custom(days));
                                                    }
                                                }
                                            });
                                        }

                                        let description = if desc.is_empty() {
                                            None
                                        } else {
                                            Some(desc.clone())
                                        };

                                        if let Err(e) = study_data.add_reminder(
                                            title.clone(),
                                            description,
                                            due_date.clone(),
                                            periods,
                                        ) {
                                            status.show(&format!("Error adding reminder: {}", e));
                                        } else {
                                            status.show("Reminder added successfully!");
                                            title.clear();
                                            desc.clear();
                                            due_date.clear();
                                            *one_day_val = false;
                                            *three_days_val = false;
                                            *one_week_val = false;
                                            *custom_val = false;
                                        }
                                    }
                                });
                            });
                        });
                    });
                });
            });
        });
    });

    ui.separator();

    // Filter options
    ui.horizontal(|ui| {
        if ui.button("Clear Completed").clicked() {
            if let Err(e) = study_data.clear_completed_reminders() {
                status.show(&format!("Error clearing completed reminders: {}", e));
            } else {
                status.show("Completed reminders cleared!");
            }
        }

        if ui.button("Clear All").clicked() {
            if let Err(e) = study_data.clear_reminders() {
                status.show(&format!("Error clearing reminders: {}", e));
            } else {
                status.show("All reminders cleared!");
            }
        }
    });

    ui.separator();

    // Track actions to perform after UI rendering
    let mut toggle_reminders: Vec<u64> = Vec::new();
    let mut delete_reminders: Vec<u64> = Vec::new();
    let mut edit_reminders: Vec<(u64, EditingReminder)> = Vec::new();
    let mut start_editing: Vec<(u64, Reminder)> = Vec::new();
    let mut cancel_editing: Vec<u64> = Vec::new();

    // Display reminders in a scrollable area
    ScrollArea::vertical().show(ui, |ui| {
        if study_data.reminders.is_empty() {
            ui.label("No reminders yet. Add one above!");
            return;
        }

        // Sort reminders by due date
        let mut sorted_reminders = study_data.reminders.clone();
        sorted_reminders.sort_by(|a, b| a.due_date.cmp(&b.due_date));

        EDITING_MAP.with(|map_ref| {
            let mut editing_map = map_ref.borrow_mut();

            for reminder in &sorted_reminders {
                let is_editing = editing_map.contains_key(&reminder.id);

                // Calculate days until due
                let days_until = days_until_due(&reminder.due_date);
                let due_text = match days_until {
                    Some(days) if days == 0 => " (Due today)".to_string(),
                    Some(days) if days < 0 => format!(" (Overdue by {} days)", -days),
                    Some(days) => format!(" (Due in {} days)", days),
                    None => " (Invalid date)".to_string(),
                };

                let frame = if let Some(days) = days_until {
                    if days < 0 {
                        // Overdue - deep red-blue
                        egui::Frame::none()
                            .fill(egui::Color32::from_rgb(120, 130, 180))
                            .inner_margin(egui::style::Margin::same(8.0))
                            .rounding(egui::Rounding::same(5.0))
                    } else if days == 0 {
                        // Due today - bright blue
                        egui::Frame::none()
                            .fill(egui::Color32::from_rgb(100, 150, 230))
                            .inner_margin(egui::style::Margin::same(8.0))
                            .rounding(egui::Rounding::same(5.0))
                    } else if days <= 3 {
                        // Due soon - medium blue
                        egui::Frame::none()
                            .fill(egui::Color32::from_rgb(80, 120, 200))
                            .inner_margin(egui::style::Margin::same(8.0))
                            .rounding(egui::Rounding::same(5.0))
                    } else {
                        // Plenty of time - light blue
                        egui::Frame::none()
                            .fill(egui::Color32::from_rgb(120, 170, 230))
                            .inner_margin(egui::style::Margin::same(8.0))
                            .rounding(egui::Rounding::same(5.0))
                    }
                } else {
                    // Invalid date - neutral blue-grey
                    egui::Frame::none()
                        .fill(egui::Color32::from_rgb(150, 160, 190))
                        .inner_margin(egui::style::Margin::same(8.0))
                        .rounding(egui::Rounding::same(5.0))
                };


                frame.show(ui, |ui| {
                    let text_style = egui::TextStyle::Body;
                    let font_id = ui.style().text_styles.get(&text_style).unwrap().clone();
                    let mut font = font_id.clone();
                    font.size = font_id.size;

                    ui.style_mut().override_font_id = Some(font);
                    ui.visuals_mut().override_text_color = Some(egui::Color32::from_rgb(10, 10, 30));
                    if is_editing {
                        if let Some(editing_reminder) = editing_map.get_mut(&reminder.id) {
                            ui.horizontal(|ui| {
                                ui.label("Title:");
                                ui.add(
                                    TextEdit::singleline(&mut editing_reminder.title)
                                        .desired_width(280.0),
                                );
                            });

                            ui.horizontal(|ui| {
                                ui.label("Description:");
                                ui.add(
                                    TextEdit::multiline(&mut editing_reminder.description)
                                        .desired_width(280.0)
                                        .desired_rows(2),
                                );
                            });

                            ui.horizontal(|ui| {
                                ui.label("Due Date:");
                                ui.add(
                                    TextEdit::singleline(&mut editing_reminder.due_date)
                                        .hint_text("YYYY-MM-DD")
                                        .desired_width(280.0),
                                );
                            });

                            ui.label("Notification Periods:");

                            let mut has_one_day = false;
                            let mut has_three_days = false;
                            let mut has_one_week = false;
                            let mut has_custom = false;
                            let mut custom_days = 5;

                            // Set initial values based on existing notification periods
                            for period in &editing_reminder.notification_periods {
                                match period {
                                    NotificationPeriod::OneDay => has_one_day = true,
                                    NotificationPeriod::ThreeDays => has_three_days = true,
                                    NotificationPeriod::OneWeek => has_one_week = true,
                                    NotificationPeriod::Custom(days) => {
                                        has_custom = true;
                                        custom_days = *days;
                                    }
                                }
                            }

                            // Store checkbox state to prevent auto-reset
                            thread_local! {
                                static EDIT_ONE_DAY: RefCell<HashMap<u64, bool>> = RefCell::new(HashMap::new());
                                static EDIT_THREE_DAYS: RefCell<HashMap<u64, bool>> = RefCell::new(HashMap::new());
                                static EDIT_ONE_WEEK: RefCell<HashMap<u64, bool>> = RefCell::new(HashMap::new());
                                static EDIT_CUSTOM: RefCell<HashMap<u64, bool>> = RefCell::new(HashMap::new());
                                static EDIT_CUSTOM_DAYS: RefCell<HashMap<u64, String>> = RefCell::new(HashMap::new());
                            }

                            // Initialize the stored values if not present
                            EDIT_ONE_DAY.with(|map| {
                                let mut map = map.borrow_mut();
                                if !map.contains_key(&reminder.id) {
                                    map.insert(reminder.id, has_one_day);
                                }
                                has_one_day = *map.get(&reminder.id).unwrap();
                            });

                            EDIT_THREE_DAYS.with(|map| {
                                let mut map = map.borrow_mut();
                                if !map.contains_key(&reminder.id) {
                                    map.insert(reminder.id, has_three_days);
                                }
                                has_three_days = *map.get(&reminder.id).unwrap();
                            });

                            EDIT_ONE_WEEK.with(|map| {
                                let mut map = map.borrow_mut();
                                if !map.contains_key(&reminder.id) {
                                    map.insert(reminder.id, has_one_week);
                                }
                                has_one_week = *map.get(&reminder.id).unwrap();
                            });

                            EDIT_CUSTOM.with(|map| {
                                let mut map = map.borrow_mut();
                                if !map.contains_key(&reminder.id) {
                                    map.insert(reminder.id, has_custom);
                                }
                                has_custom = *map.get(&reminder.id).unwrap();
                            });

                            EDIT_CUSTOM_DAYS.with(|map| {
                                let mut map = map.borrow_mut();
                                if !map.contains_key(&reminder.id) {
                                    map.insert(reminder.id, custom_days.to_string());
                                }
                            });

                            ui.horizontal(|ui| {
                                if ui.checkbox(&mut has_one_day, "1 Day Before").changed() {
                                    EDIT_ONE_DAY.with(|map| {
                                        let mut map = map.borrow_mut();
                                        map.insert(reminder.id, has_one_day);
                                    });
                                }

                                if ui.checkbox(&mut has_three_days, "3 Days Before").changed() {
                                    EDIT_THREE_DAYS.with(|map| {
                                        let mut map = map.borrow_mut();
                                        map.insert(reminder.id, has_three_days);
                                    });
                                }

                                if ui.checkbox(&mut has_one_week, "1 Week Before").changed() {
                                    EDIT_ONE_WEEK.with(|map| {
                                        let mut map = map.borrow_mut();
                                        map.insert(reminder.id, has_one_week);
                                    });
                                }
                            });

                            ui.horizontal(|ui| {
                                if ui.checkbox(&mut has_custom, "Custom:").changed() {
                                    EDIT_CUSTOM.with(|map| {
                                        let mut map = map.borrow_mut();
                                        map.insert(reminder.id, has_custom);
                                    });
                                }

                                let mut custom_days_str = String::new();
                                EDIT_CUSTOM_DAYS.with(|map| {
                                    let map = map.borrow_mut();
                                    custom_days_str = map.get(&reminder.id).unwrap_or(&custom_days.to_string()).clone();
                                });

                                if ui
                                    .add_enabled(
                                        has_custom,
                                        TextEdit::singleline(&mut custom_days_str)
                                            .hint_text("Days")
                                            .desired_width(50.0),
                                    )
                                    .changed()
                                {
                                    EDIT_CUSTOM_DAYS.with(|map| {
                                        let mut map = map.borrow_mut();
                                        map.insert(reminder.id, custom_days_str.clone());
                                    });
                                }
                                ui.label("days before");
                            });

                            ui.horizontal(|ui| {
                                if ui.button("Save").clicked() {
                                    if editing_reminder.title.is_empty() {
                                        status.show("Reminder title cannot be empty!");
                                        return;
                                    }

                                    if editing_reminder.due_date.is_empty() {
                                        status.show("Due date cannot be empty!");
                                        return;
                                    }

                                    // Validate date format
                                    if NaiveDate::parse_from_str(
                                        &editing_reminder.due_date,
                                        "%Y-%m-%d",
                                    )
                                    .is_err()
                                    {
                                        status.show("Invalid date format! Use YYYY-MM-DD");
                                        return;
                                    }

                                    // Update notification periods based on checkboxes
                                    let mut periods = Vec::new();
                                    
                                    if has_one_day {
                                        periods.push(NotificationPeriod::OneDay);
                                    }
                                    
                                    if has_three_days {
                                        periods.push(NotificationPeriod::ThreeDays);
                                    }
                                    
                                    if has_one_week {
                                        periods.push(NotificationPeriod::OneWeek);
                                    }
                                    
                                    if has_custom {
                                        let custom_days_str = EDIT_CUSTOM_DAYS.with(|map| {
                                            map.borrow().get(&reminder.id).unwrap_or(&"5".to_string()).clone()
                                        });
                                        
                                        if let Ok(days) = custom_days_str.parse::<u32>() {
                                            if days > 0 {
                                                periods.push(NotificationPeriod::Custom(days));
                                            }
                                        }
                                    }
                                    
                                    editing_reminder.notification_periods = periods;

                                    edit_reminders.push((reminder.id, editing_reminder.clone()));
                                    cancel_editing.push(reminder.id);
                                    
                                    // Clean up stored values
                                    EDIT_ONE_DAY.with(|map| {
                                        map.borrow_mut().remove(&reminder.id);
                                    });
                                    EDIT_THREE_DAYS.with(|map| {
                                        map.borrow_mut().remove(&reminder.id);
                                    });
                                    EDIT_ONE_WEEK.with(|map| {
                                        map.borrow_mut().remove(&reminder.id);
                                    });
                                    EDIT_CUSTOM.with(|map| {
                                        map.borrow_mut().remove(&reminder.id);
                                    });
                                    EDIT_CUSTOM_DAYS.with(|map| {
                                        map.borrow_mut().remove(&reminder.id);
                                    });
                                }

                                if ui.button("Cancel").clicked() {
                                    cancel_editing.push(reminder.id);
                                    
                                    // Clean up stored values
                                    EDIT_ONE_DAY.with(|map| {
                                        map.borrow_mut().remove(&reminder.id);
                                    });
                                    EDIT_THREE_DAYS.with(|map| {
                                        map.borrow_mut().remove(&reminder.id);
                                    });
                                    EDIT_ONE_WEEK.with(|map| {
                                        map.borrow_mut().remove(&reminder.id);
                                    });
                                    EDIT_CUSTOM.with(|map| {
                                        map.borrow_mut().remove(&reminder.id);
                                    });
                                    EDIT_CUSTOM_DAYS.with(|map| {
                                        map.borrow_mut().remove(&reminder.id);
                                    });
                                }
                            });
                        }
                    } else {
                        // Display reminder title and due date
                        ui.horizontal(|ui| {
                            let mut is_completed = reminder.is_completed;
                            if ui.checkbox(&mut is_completed, "").clicked() {
                                toggle_reminders.push(reminder.id);
                            }

                            let title_text = if reminder.is_completed {
                                egui::RichText::new(&reminder.title).strikethrough()
                            } else {
                                egui::RichText::new(&reminder.title).strong()
                            };

                            ui.label(title_text);
                            ui.label(egui::RichText::new(&due_text).small());
                        });

                        // Display description if available
                        if let Some(desc) = &reminder.description {
                            if !desc.is_empty() {
                                ui.indent("desc_indent", |ui| {
                                    ui.label(desc);
                                });
                            }
                        }

                        // Display notification periods
                        let periods_text =
                            format_notification_periods(&reminder.notification_periods);
                        ui.label(egui::RichText::new(&periods_text).small().italics());

                        // Action buttons
                        ui.horizontal(|ui| {
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if ui.button("❌").clicked() {
                                        delete_reminders.push(reminder.id);
                                    }

                                    if ui.button("✏️").clicked() {
                                        start_editing.push((reminder.id, reminder.clone()));
                                    }
                                },
                            );
                        });
                    }
                });

                ui.add_space(8.0);
            }
        });
    });

    // Process the collected actions
    for id in toggle_reminders {
        if let Err(e) = study_data.toggle_reminder(id) {
            status.show(&format!("Error toggling reminder: {}", e));
        }
    }

    for id in delete_reminders {
        if let Err(e) = study_data.delete_reminder(id) {
            status.show(&format!("Error deleting reminder: {}", e));
        } else {
            status.show("Reminder deleted successfully!");
        }
    }

    for (id, editing_reminder) in edit_reminders {
        let description = if editing_reminder.description.is_empty() {
            None
        } else {
            Some(editing_reminder.description)
        };

        if let Err(e) = study_data.update_reminder(
            id,
            editing_reminder.title,
            description,
            editing_reminder.due_date,
            editing_reminder.notification_periods,
        ) {
            status.show(&format!("Error updating reminder: {}", e));
        } else {
            status.show("Reminder updated successfully!");
        }
    }

    // Update the editing map with new edits or cancellations
    EDITING_MAP.with(|map_ref| {
        let mut editing_map = map_ref.borrow_mut();

        for (id, reminder) in start_editing {
            let description = reminder.description.unwrap_or_default();
            let editing_reminder = EditingReminder {
                title: reminder.title,
                description,
                due_date: reminder.due_date,
                notification_periods: reminder.notification_periods,
            };
            editing_map.insert(id, editing_reminder);
        }

        for id in cancel_editing {
            editing_map.remove(&id);
        }
    });

    // Show status message
    status.render(ui);
}


fn format_notification_periods(periods: &[NotificationPeriod]) -> String {
    if periods.is_empty() {
        return "No notifications set".to_string();
    }

    let mut result = "Notify: ".to_string();
    let mut notification_texts = Vec::new();

    for period in periods {
        match period {
            NotificationPeriod::OneDay => notification_texts.push("1 day before".to_string()),
            NotificationPeriod::ThreeDays => notification_texts.push("3 days before".to_string()),
            NotificationPeriod::OneWeek => notification_texts.push("1 week before".to_string()),
            NotificationPeriod::Custom(days) => {
                notification_texts.push(format!("{} days before", days))
            }
        }
    }

    result.push_str(&notification_texts.join(", "));
    result
}

fn days_until_due(due_date: &str) -> Option<i64> {
    if let Ok(date) = NaiveDate::parse_from_str(due_date, "%Y-%m-%d") {
        let today = Local::now().date_naive();
        Some((date - today).num_days())
    } else {
        None
    }
}

fn check_due_reminders(study_data: &StudyData, status: &mut StatusMessage) {
    let today = Local::now().date_naive();
    let mut notifications = Vec::new();

    for reminder in &study_data.reminders {
        if reminder.is_completed {
            continue;
        }

        if let Ok(due_date) = NaiveDate::parse_from_str(&reminder.due_date, "%Y-%m-%d") {
            let days_until = (due_date - today).num_days();

            for period in &reminder.notification_periods {
                match period {
                    NotificationPeriod::OneDay if days_until == 1 => {
                        notifications.push(format!("\"{}\" is due tomorrow!", reminder.title));
                    }
                    NotificationPeriod::ThreeDays if days_until == 3 => {
                        notifications.push(format!("\"{}\" is due in 3 days!", reminder.title));
                    }
                    NotificationPeriod::OneWeek if days_until == 7 => {
                        notifications.push(format!("\"{}\" is due in a week!", reminder.title));
                    }
                    NotificationPeriod::Custom(custom_days)
                        if days_until == *custom_days as i64 =>
                    {
                        notifications.push(format!(
                            "\"{}\" is due in {} days!",
                            reminder.title, custom_days
                        ));
                    }
                    _ => {}
                }
            }

            // Always notify if due today
            if days_until == 0 {
                notifications.push(format!("\"{}\" is due today!", reminder.title));
            }

            // Always notify if overdue
            if days_until < 0 {
                notifications.push(format!(
                    "\"{}\" is overdue by {} days!",
                    reminder.title, -days_until
                ));
            }
        }
    }

    // Display notifications (up to 3)
    if !notifications.is_empty() {
        let mut message = "REMINDER: ".to_string();
        let display_count = notifications.len().min(3);

        for i in 0..display_count {
            message.push_str(&notifications[i]);
            if i < display_count - 1 {
                message.push_str(" | ");
            }
        }
        if notifications.len() > 3 {
            message.push_str(" (and more...)");
        }

        status.show(&message);
    }
}
