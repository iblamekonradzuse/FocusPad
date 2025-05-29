use crate::app::StatusMessage;
use crate::data::StudyData;
use crate::settings::AppSettings;
use chrono::{Datelike, Duration, Local, NaiveDate};
use egui::{ComboBox, ScrollArea, TextEdit, Window};
use std::cell::RefCell;
use std::collections::HashMap;

// We'll use thread-local storage instead of once_cell
thread_local! {
    static NEW_TODO: RefCell<String> = RefCell::new(String::new());
    static NEW_HABIT: RefCell<String> = RefCell::new(String::new());
    static NEW_HABIT_CATEGORY: RefCell<String> = RefCell::new(String::from("General"));
    static EDITING_MAP: RefCell<HashMap<u64, String>> = RefCell::new(HashMap::new());
    static SELECTED_TAB: RefCell<HabitTab> = RefCell::new(HabitTab::Todos);
    static SELECTED_CATEGORY_FILTER: RefCell<String> = RefCell::new(String::from("All"));
    static MONTHLY_VIEW_HABIT: RefCell<Option<u64>> = RefCell::new(None);
    static MONTHLY_VIEW_DATE: RefCell<NaiveDate> = RefCell::new(Local::now().date_naive());
}

#[derive(Debug, Clone, PartialEq)]
enum HabitTab {
    Todos,
    Habits,
}

impl HabitTab {
    fn as_str(&self) -> &str {
        match self {
            HabitTab::Todos => "Todos",
            HabitTab::Habits => "Habits",
        }
    }
}

pub fn display(
    ui: &mut egui::Ui,
    study_data: &mut StudyData,
    status: &mut StatusMessage,
    settings: &AppSettings,
) {
    let colors = settings.get_current_colors();

    ui.heading(egui::RichText::new("Tasks & Habits").color(colors.text_primary_color32()));

    // Tab selection
    ui.horizontal(|ui| {
        SELECTED_TAB.with(|tab_ref| {
            let mut current_tab = tab_ref.borrow_mut();

            for tab in [HabitTab::Todos, HabitTab::Habits].iter() {
                let is_selected = *current_tab == *tab;
                let button_color = if is_selected {
                    colors.active_tab_color32()
                } else {
                    colors.inactive_tab_color32()
                };

                let tab_button = egui::Button::new(
                    egui::RichText::new(tab.as_str()).color(colors.text_primary_color32()),
                )
                .fill(button_color)
                .stroke(egui::Stroke::new(1.0, colors.accent_color32()));

                if ui.add(tab_button).clicked() {
                    *current_tab = tab.clone();
                }
            }
        });
    });

    ui.separator();

    SELECTED_TAB.with(|tab_ref| {
        let current_tab = tab_ref.borrow();
        match *current_tab {
            HabitTab::Todos => display_todos(ui, study_data, status, settings),
            HabitTab::Habits => display_habits(ui, study_data, status, settings),
        }
    });

    // Show monthly view popup if a habit is selected
    display_monthly_view_popup(ui, study_data, settings);

    // Show status message
    status.render(ui);
}

fn display_todos(
    ui: &mut egui::Ui,
    study_data: &mut StudyData,
    status: &mut StatusMessage,
    settings: &AppSettings,
) {
    let colors = settings.get_current_colors();

    // Add new todo section with themed colors
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("New Task:").color(colors.text_secondary_color32()));

        // Use thread_local with with() to access the value
        NEW_TODO.with(|todo_ref| {
            let mut new_todo = todo_ref.borrow_mut();

            let text_edit = ui.add(
                TextEdit::singleline(&mut *new_todo)
                    .hint_text("Enter a new task...")
                    .desired_width(280.0)
                    .text_color(colors.text_primary_color32()),
            );

            if text_edit.lost_focus()
                && ui.input(|i| i.key_pressed(egui::Key::Enter))
                && !new_todo.is_empty()
            {
                if let Err(e) = study_data.add_todo(new_todo.clone()) {
                    status.show(&format!("Error adding todo: {}", e));
                } else {
                    status.show("Todo added successfully!");
                    new_todo.clear();
                }
            }

            let add_button =
                egui::Button::new(egui::RichText::new("Add").color(colors.text_primary_color32()))
                    .fill(colors.accent_color32())
                    .stroke(egui::Stroke::new(1.0, colors.active_tab_color32()));

            if ui.add(add_button).clicked() && !new_todo.is_empty() {
                if let Err(e) = study_data.add_todo(new_todo.clone()) {
                    status.show(&format!("Error adding todo: {}", e));
                } else {
                    status.show("Todo added successfully!");
                    new_todo.clear();
                }
            }
        });
    });

    ui.separator();

    // Buttons for clearing todos with themed colors
    ui.horizontal(|ui| {
        let clear_completed_button = egui::Button::new(
            egui::RichText::new("Clear Completed").color(colors.text_primary_color32()),
        )
        .fill(colors.inactive_tab_color32())
        .stroke(egui::Stroke::new(1.0, colors.accent_color32()));

        if ui.add(clear_completed_button).clicked() {
            if let Err(e) = study_data.clear_completed_todos() {
                status.show(&format!("Error clearing completed todos: {}", e));
            } else {
                status.show("Completed todos cleared!");
            }
        }

        let clear_all_button = egui::Button::new(
            egui::RichText::new("Clear All").color(colors.text_primary_color32()),
        )
        .fill(colors.inactive_tab_color32())
        .stroke(egui::Stroke::new(1.0, colors.accent_color32()));

        if ui.add(clear_all_button).clicked() {
            if let Err(e) = study_data.clear_todos() {
                status.show(&format!("Error clearing todos: {}", e));
            } else {
                status.show("All todos cleared!");
            }
        }
    });

    ui.separator();

    display_todo_list(ui, study_data, status, &colors);
}

fn display_habits(
    ui: &mut egui::Ui,
    study_data: &mut StudyData,
    status: &mut StatusMessage,
    settings: &AppSettings,
) {
    let colors = settings.get_current_colors();

    // Add new habit section
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("New Habit:").color(colors.text_secondary_color32()));

        NEW_HABIT.with(|habit_ref| {
            let mut new_habit = habit_ref.borrow_mut();

            let text_edit = ui.add(
                TextEdit::singleline(&mut *new_habit)
                    .hint_text("Enter a new habit...")
                    .desired_width(200.0)
                    .text_color(colors.text_primary_color32()),
            );

            NEW_HABIT_CATEGORY.with(|category_ref| {
                let mut category = category_ref.borrow_mut();

                ui.label(egui::RichText::new("Category:").color(colors.text_secondary_color32()));
                ComboBox::from_id_source("habit_category")
                    .selected_text(&*category)
                    .width(100.0)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut *category, "General".to_string(), "General");
                        ui.selectable_value(&mut *category, "Health".to_string(), "Health");
                        ui.selectable_value(&mut *category, "Study".to_string(), "Study");
                        ui.selectable_value(&mut *category, "Exercise".to_string(), "Exercise");
                        ui.selectable_value(
                            &mut *category,
                            "Productivity".to_string(),
                            "Productivity",
                        );
                        ui.selectable_value(&mut *category, "Self-Care".to_string(), "Self-Care");
                    });

                if (text_edit.lost_focus()
                    && ui.input(|i| i.key_pressed(egui::Key::Enter))
                    && !new_habit.is_empty())
                    || ui.button("Add Habit").clicked() && !new_habit.is_empty()
                {
                    if let Err(e) = study_data.add_habit(new_habit.clone(), category.clone()) {
                        status.show(&format!("Error adding habit: {}", e));
                    } else {
                        status.show("Habit added successfully!");
                        new_habit.clear();
                    }
                }
            });
        });
    });

    ui.separator();

    // Category filter and management buttons
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Filter:").color(colors.text_secondary_color32()));

        SELECTED_CATEGORY_FILTER.with(|filter_ref| {
            let mut filter = filter_ref.borrow_mut();
            let categories = study_data.get_habit_categories();

            ComboBox::from_id_source("category_filter")
                .selected_text(&*filter)
                .width(120.0)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut *filter, "All".to_string(), "All");
                    for category in categories {
                        ui.selectable_value(&mut *filter, category.clone(), &category);
                    }
                });
        });

        ui.separator();

        let clear_completed_button = egui::Button::new(
            egui::RichText::new("Clear Completed").color(colors.text_primary_color32()),
        )
        .fill(colors.inactive_tab_color32())
        .stroke(egui::Stroke::new(1.0, colors.accent_color32()));

        if ui.add(clear_completed_button).clicked() {
            if let Err(e) = study_data.clear_completed_habits() {
                status.show(&format!("Error clearing completed habits: {}", e));
            } else {
                status.show("Completed habits cleared!");
            }
        }
    });

    ui.separator();

    display_habit_list(ui, study_data, status, &colors);
}

fn display_todo_list(
    ui: &mut egui::Ui,
    study_data: &mut StudyData,
    status: &mut StatusMessage,
    colors: &crate::settings::ColorTheme,
) {
    // Track actions to perform after UI rendering
    let mut toggle_todos: Vec<u64> = Vec::new();
    let mut delete_todos: Vec<u64> = Vec::new();
    let mut edit_todos: Vec<(u64, String)> = Vec::new();
    let mut start_editing: Vec<(u64, String)> = Vec::new();
    let mut cancel_editing: Vec<u64> = Vec::new();

    // Display todos in a scrollable area
    ScrollArea::vertical().show(ui, |ui| {
        if study_data.todos.is_empty() {
            ui.label(
                egui::RichText::new("No todos yet. Add one above!")
                    .color(colors.text_secondary_color32()),
            );
            return;
        }

        // Use thread_local with with() to access the editing map
        EDITING_MAP.with(|map_ref| {
            let mut editing_map = map_ref.borrow_mut();

            // Display todos without changing them in this loop
            for todo in &study_data.todos {
                let is_editing = editing_map.contains_key(&todo.id);

                // Create a frame for each todo item with theme-appropriate background
                let todo_frame = egui::Frame::default()
                    .fill(if todo.completed {
                        // Slightly darker background for completed todos
                        egui::Color32::from_rgba_unmultiplied(
                            colors.panel_background_color32().r().saturating_sub(10),
                            colors.panel_background_color32().g().saturating_sub(10),
                            colors.panel_background_color32().b().saturating_sub(10),
                            colors.panel_background_color32().a(),
                        )
                    } else {
                        colors.panel_background_color32()
                    })
                    .inner_margin(egui::Margin::same(8.0))
                    .outer_margin(egui::Margin::symmetric(0.0, 2.0))
                    .stroke(egui::Stroke::new(
                        1.0,
                        if todo.completed {
                            colors.text_secondary_color32()
                        } else {
                            colors.accent_color32()
                        },
                    ));

                todo_frame.show(ui, |ui| {
                    ui.horizontal(|ui| {
                        // Checkbox for marking todo as complete
                        let mut is_completed = todo.completed;
                        let checkbox = ui.checkbox(&mut is_completed, "");
                        if checkbox.clicked() {
                            toggle_todos.push(todo.id);
                        }

                        // Display todo text or edit field
                        if is_editing {
                            if let Some(edit_text) = editing_map.get_mut(&todo.id) {
                                // Text edit field with theme colors
                                ui.add(
                                    TextEdit::singleline(edit_text)
                                        .desired_width(ui.available_width() - 120.0)
                                        .text_color(colors.text_primary_color32()),
                                );

                                let save_button = egui::Button::new(
                                    egui::RichText::new("Save")
                                        .color(colors.text_primary_color32()),
                                )
                                .fill(colors.accent_color32())
                                .stroke(egui::Stroke::new(1.0, colors.active_tab_color32()));

                                if ui.add(save_button).clicked() && !edit_text.is_empty() {
                                    // Clone the String before moving it
                                    let text_to_save = edit_text.clone();
                                    edit_todos.push((todo.id, text_to_save));
                                    cancel_editing.push(todo.id);
                                }

                                let cancel_button = egui::Button::new(
                                    egui::RichText::new("Cancel")
                                        .color(colors.text_primary_color32()),
                                )
                                .fill(colors.inactive_tab_color32())
                                .stroke(egui::Stroke::new(1.0, colors.accent_color32()));

                                if ui.add(cancel_button).clicked() {
                                    cancel_editing.push(todo.id);
                                }
                            }
                        } else {
                            // Display the todo text with strikethrough if completed
                            let text_color = if todo.completed {
                                colors.text_secondary_color32()
                            } else {
                                colors.text_primary_color32()
                            };

                            let text = if todo.completed {
                                egui::RichText::new(&todo.text)
                                    .strikethrough()
                                    .color(text_color)
                            } else {
                                egui::RichText::new(&todo.text).color(text_color)
                            };
                            ui.label(text);

                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    // Delete button with theme colors
                                    let delete_button = egui::Button::new(
                                        egui::RichText::new("❌")
                                            .color(colors.text_primary_color32()),
                                    )
                                    .fill(egui::Color32::from_rgba_unmultiplied(200, 50, 50, 100))
                                    .stroke(egui::Stroke::new(
                                        1.0,
                                        egui::Color32::from_rgba_unmultiplied(200, 50, 50, 200),
                                    ));

                                    if ui.add(delete_button).clicked() {
                                        delete_todos.push(todo.id);
                                    }

                                    // Edit button with theme colors
                                    let edit_button = egui::Button::new(
                                        egui::RichText::new("✏️")
                                            .color(colors.text_primary_color32()),
                                    )
                                    .fill(colors.inactive_tab_color32())
                                    .stroke(egui::Stroke::new(1.0, colors.accent_color32()));

                                    if ui.add(edit_button).clicked() {
                                        start_editing.push((todo.id, todo.text.clone()));
                                    }
                                },
                            );
                        }
                    });
                });
            }
        });
    });

    // Process the collected actions
    for id in toggle_todos {
        if let Err(e) = study_data.toggle_todo(id) {
            status.show(&format!("Error toggling todo: {}", e));
        }
    }

    for id in delete_todos {
        if let Err(e) = study_data.delete_todo(id) {
            status.show(&format!("Error deleting todo: {}", e));
        } else {
            status.show("Todo deleted successfully!");
        }
    }

    for (id, text) in edit_todos {
        if let Err(e) = study_data.update_todo_text(id, text) {
            status.show(&format!("Error updating todo: {}", e));
        } else {
            status.show("Todo updated successfully!");
        }
    }

    // Update the editing map with new edits or cancellations
    EDITING_MAP.with(|map_ref| {
        let mut editing_map = map_ref.borrow_mut();

        for (id, text) in start_editing {
            editing_map.insert(id, text);
        }

        for id in cancel_editing {
            editing_map.remove(&id);
        }
    });
}

fn display_habit_list(
    ui: &mut egui::Ui,
    study_data: &mut StudyData,
    status: &mut StatusMessage,
    colors: &crate::settings::ColorTheme,
) {
    let mut mark_habit_complete: Vec<u64> = Vec::new();
    let mut delete_habits: Vec<u64> = Vec::new();
    let mut show_monthly_view: Option<u64> = None;

    // Get filtered habits
    let filtered_habits = SELECTED_CATEGORY_FILTER.with(|filter_ref| {
        let filter = filter_ref.borrow();
        if filter.as_str() == "All" {
            study_data.habits.clone()
        } else {
            study_data
                .habits
                .iter()
                .filter(|h| h.category == *filter)
                .cloned()
                .collect()
        }
    });

    ScrollArea::vertical().show(ui, |ui| {
        if filtered_habits.is_empty() {
            ui.label(
                egui::RichText::new("No habits yet. Add one above!")
                    .color(colors.text_secondary_color32()),
            );
            return;
        }

        // Group habits by category and sort categories to prevent glitching
        let mut categories: std::collections::HashMap<String, Vec<_>> =
            std::collections::HashMap::new();
        for habit in &filtered_habits {
            categories
                .entry(habit.category.clone())
                .or_insert_with(Vec::new)
                .push(habit);
        }

        // Sort categories alphabetically to prevent UI glitching
        let mut sorted_categories: Vec<(String, Vec<_>)> = categories.into_iter().collect();
        sorted_categories.sort_by(|a, b| a.0.cmp(&b.0));

        for (category, habits) in sorted_categories {
            if SELECTED_CATEGORY_FILTER.with(|f| f.borrow().as_str() == "All") {
                ui.group(|ui| {
                    ui.label(
                        egui::RichText::new(&category)
                            .heading()
                            .color(colors.accent_color32()),
                    );

                    for habit in habits {
                        if let Some(habit_id) = display_habit_item(
                            ui,
                            habit,
                            colors,
                            &mut mark_habit_complete,
                            &mut delete_habits,
                        ) {
                            show_monthly_view = Some(habit_id);
                        }
                    }
                });
                ui.add_space(10.0);
            } else {
                for habit in habits {
                    if let Some(habit_id) = display_habit_item(
                        ui,
                        habit,
                        colors,
                        &mut mark_habit_complete,
                        &mut delete_habits,
                    ) {
                        show_monthly_view = Some(habit_id);
                    }
                }
            }
        }
    });

    // Process actions
    for id in mark_habit_complete {
        if let Err(e) = study_data.mark_habit_complete_today(id) {
            status.show(&format!("Error marking habit complete: {}", e));
        } else {
            status.show("Habit marked complete for today!");
        }
    }

    for id in delete_habits {
        if let Err(e) = study_data.delete_habit(id) {
            status.show(&format!("Error deleting habit: {}", e));
        } else {
            status.show("Habit deleted successfully!");
        }
    }

    // Show monthly view if requested
    if let Some(habit_id) = show_monthly_view {
        MONTHLY_VIEW_HABIT.with(|habit_ref| {
            *habit_ref.borrow_mut() = Some(habit_id);
        });
        MONTHLY_VIEW_DATE.with(|date_ref| {
            *date_ref.borrow_mut() = Local::now().date_naive();
        });
    }
}

fn display_habit_item(
    ui: &mut egui::Ui,
    habit: &crate::data::Habit,
    colors: &crate::settings::ColorTheme,
    mark_complete: &mut Vec<u64>,
    delete_habits: &mut Vec<u64>,
) -> Option<u64> {
    let today = Local::now().date_naive().format("%Y-%m-%d").to_string();
    let is_complete_today = habit.completion_dates.contains(&today);
    let streak = habit.calculate_current_streak();
    let total_completions = habit.completion_dates.len();
    let mut show_monthly = false;

    let habit_frame = egui::Frame::default()
        .fill(if is_complete_today {
            egui::Color32::from_rgba_unmultiplied(50, 150, 50, 50)
        } else {
            colors.panel_background_color32()
        })
        .inner_margin(egui::Margin::same(8.0))
        .outer_margin(egui::Margin::symmetric(0.0, 2.0))
        .stroke(egui::Stroke::new(
            1.0,
            if is_complete_today {
                egui::Color32::from_rgba_unmultiplied(50, 200, 50, 200)
            } else {
                colors.accent_color32()
            },
        ));

    habit_frame.show(ui, |ui| {
        ui.horizontal(|ui| {
            // Completion checkbox
            let mut completed = is_complete_today;
            if ui.checkbox(&mut completed, "").clicked() {
                if completed != is_complete_today {
                    mark_complete.push(habit.id);
                }
            }

            // Habit name and streak info
            ui.vertical(|ui| {
                let habit_text = if is_complete_today {
                    egui::RichText::new(&habit.name)
                        .color(colors.text_primary_color32())
                        .strong()
                } else {
                    egui::RichText::new(&habit.name).color(colors.text_primary_color32())
                };

                ui.label(habit_text);

                // Streak and completion info
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(&format!("🔥 {} day streak", streak)).color(
                            if streak > 0 {
                                egui::Color32::from_rgb(255, 140, 0)
                            } else {
                                colors.text_secondary_color32()
                            },
                        ),
                    );

                    ui.separator();

                    ui.label(
                        egui::RichText::new(&format!("✅ {} total", total_completions))
                            .color(colors.text_secondary_color32()),
                    );
                });
            });

            // Visual streak indicator (last 7 days)
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let monthly_button = egui::Button::new(
                    egui::RichText::new("📅").color(colors.text_primary_color32()),
                )
                .fill(colors.inactive_tab_color32())
                .stroke(egui::Stroke::new(1.0, colors.accent_color32()));

                if ui.add(monthly_button).clicked() {
                    show_monthly = true;
                }

                ui.separator();
                // Delete button
                let delete_button = egui::Button::new(
                    egui::RichText::new("❌").color(colors.text_primary_color32()),
                )
                .fill(egui::Color32::from_rgba_unmultiplied(200, 50, 50, 100))
                .stroke(egui::Stroke::new(
                    1.0,
                    egui::Color32::from_rgba_unmultiplied(200, 50, 50, 200),
                ));

                if ui.add(delete_button).clicked() {
                    delete_habits.push(habit.id);
                }

                ui.separator();

                // Last 7 days visual indicator
                ui.horizontal(|ui| {
                    let today = Local::now().date_naive();
                    for i in (0..7).rev() {
                        let date = today - Duration::days(i);
                        let date_str = date.format("%Y-%m-%d").to_string();
                        let completed_on_date = habit.completion_dates.contains(&date_str);

                        let color = if completed_on_date {
                            egui::Color32::from_rgb(50, 200, 50)
                        } else {
                            egui::Color32::from_rgb(100, 100, 100)
                        };

                        ui.add(
                            egui::widgets::Button::new("●")
                                .fill(color)
                                .stroke(egui::Stroke::NONE),
                        )
                        .on_hover_text(&format!("{}", date.format("%m/%d")));
                    }
                });
            });
        });
    });

    if show_monthly {
        Some(habit.id)
    } else {
        None
    }
}

fn display_monthly_view_popup(ui: &mut egui::Ui, study_data: &StudyData, settings: &AppSettings) {
    let colors = settings.get_current_colors();

    MONTHLY_VIEW_HABIT.with(|habit_ref| {
        let mut habit_id_opt = habit_ref.borrow_mut();

        if let Some(habit_id) = *habit_id_opt {
            // Find the habit
            if let Some(habit) = study_data.habits.iter().find(|h| h.id == habit_id) {
                let mut open = true;

                Window::new(format!("Monthly View - {}", habit.name))
                    .open(&mut open)
                    .resizable(true)
                    .default_width(400.0)
                    .default_height(350.0)
                    .show(ui.ctx(), |ui| {
                        MONTHLY_VIEW_DATE.with(|date_ref| {
                            let mut current_date = date_ref.borrow_mut();

                            // Navigation buttons
                            ui.horizontal(|ui| {
                                if ui.button("◀ Previous").clicked() {
                                    *current_date = current_date
                                        .with_day(1)
                                        .unwrap_or(*current_date)
                                        .checked_sub_months(chrono::Months::new(1))
                                        .unwrap_or(*current_date);
                                }

                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        if ui.button("Next ▶").clicked() {
                                            *current_date = current_date
                                                .with_day(1)
                                                .unwrap_or(*current_date)
                                                .checked_add_months(chrono::Months::new(1))
                                                .unwrap_or(*current_date);
                                        }

                                        ui.with_layout(
                                            egui::Layout::left_to_right(egui::Align::Center),
                                            |ui| {
                                                ui.label(
                                                    egui::RichText::new(&format!(
                                                        "{} {}",
                                                        match current_date.month() {
                                                            1 => "January",
                                                            2 => "February",
                                                            3 => "March",
                                                            4 => "April",
                                                            5 => "May",
                                                            6 => "June",
                                                            7 => "July",
                                                            8 => "August",
                                                            9 => "September",
                                                            10 => "October",
                                                            11 => "November",
                                                            12 => "December",
                                                            _ => "Unknown",
                                                        },
                                                        current_date.year()
                                                    ))
                                                    .heading()
                                                    .color(colors.text_primary_color32()),
                                                );
                                            },
                                        );
                                    },
                                );
                            });

                            ui.separator();

                            // Calendar grid
                            display_monthly_calendar(ui, habit, *current_date, &colors);

                            ui.separator();

                            // Statistics for the month
                            let month_stats = calculate_month_stats(habit, *current_date);
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new(&format!(
                                        "Days completed: {}",
                                        month_stats.completed_days
                                    ))
                                    .color(colors.text_primary_color32()),
                                );
                                ui.separator();
                                ui.label(
                                    egui::RichText::new(&format!(
                                        "Success rate: {:.1}%",
                                        month_stats.success_rate
                                    ))
                                    .color(colors.text_primary_color32()),
                                );
                            });
                        });
                    });

                if !open {
                    *habit_id_opt = None;
                }
            } else {
                // Habit not found, close the popup
                *habit_id_opt = None;
            }
        }
    });
}

fn display_monthly_calendar(
    ui: &mut egui::Ui,
    habit: &crate::data::Habit,
    current_date: NaiveDate,
    colors: &crate::settings::ColorTheme,
) {
    // Get the first day of the month
    let first_day = current_date.with_day(1).unwrap_or(current_date);

    // Get the number of days in the month
    let days_in_month = if current_date.month() == 12 {
        NaiveDate::from_ymd_opt(current_date.year() + 1, 1, 1)
            .unwrap()
            .signed_duration_since(first_day)
            .num_days()
    } else {
        NaiveDate::from_ymd_opt(current_date.year(), current_date.month() + 1, 1)
            .unwrap()
            .signed_duration_since(first_day)
            .num_days()
    };

    // Get the day of week for the first day (0 = Sunday, 6 = Saturday)
    let first_day_weekday = first_day.weekday().num_days_from_sunday() as i32;

    // Day headers
    ui.horizontal(|ui| {
        for day in ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"] {
            ui.allocate_ui_with_layout(
                egui::Vec2::new(40.0, 20.0),
                egui::Layout::centered_and_justified(egui::Direction::TopDown),
                |ui| {
                    ui.label(
                        egui::RichText::new(day)
                            .color(colors.text_secondary_color32())
                            .small(),
                    );
                },
            );
        }
    });

    ui.separator();

    // Calendar grid
    egui::Grid::new("monthly_calendar")
        .num_columns(7)
        .spacing([2.0, 2.0])
        .show(ui, |ui| {
            let mut day_counter = 1;

            // Calculate total cells needed (6 weeks maximum)
            for week in 0..6 {
                for weekday in 0..7 {
                    let cell_day = day_counter - first_day_weekday;

                    if week == 0 && weekday < first_day_weekday {
                        // Empty cell before month starts
                        ui.allocate_ui_with_layout(
                            egui::Vec2::new(40.0, 40.0),
                            egui::Layout::centered_and_justified(egui::Direction::TopDown),
                            |ui| {
                                ui.label("");
                            },
                        );
                    } else if cell_day >= days_in_month as i32 {
                        // Empty cell after month ends
                        ui.allocate_ui_with_layout(
                            egui::Vec2::new(40.0, 40.0),
                            egui::Layout::centered_and_justified(egui::Direction::TopDown),
                            |ui| {
                                ui.label("");
                            },
                        );
                    } else {
                        // Valid day in month
                        let day_number = cell_day + 1;
                        let date = first_day.with_day(day_number as u32).unwrap();
                        let date_str = date.format("%Y-%m-%d").to_string();
                        let is_completed = habit.completion_dates.contains(&date_str);
                        let is_today = date == Local::now().date_naive();

                        let (bg_color, text_color, border_color) = if is_completed {
                            (
                                egui::Color32::from_rgba_unmultiplied(50, 200, 50, 100),
                                colors.text_primary_color32(),
                                egui::Color32::from_rgb(50, 200, 50),
                            )
                        } else if is_today {
                            (
                                colors.accent_color32(),
                                colors.text_primary_color32(),
                                colors.accent_color32(),
                            )
                        } else {
                            (
                                colors.panel_background_color32(),
                                colors.text_primary_color32(),
                                colors.text_secondary_color32(),
                            )
                        };

                        ui.allocate_ui_with_layout(
                            egui::Vec2::new(40.0, 40.0),
                            egui::Layout::centered_and_justified(egui::Direction::TopDown),
                            |ui| {
                                let frame = egui::Frame::default()
                                    .fill(bg_color)
                                    .stroke(egui::Stroke::new(1.0, border_color))
                                    .inner_margin(egui::Margin::same(4.0));

                                frame.show(ui, |ui| {
                                    ui.label(
                                        egui::RichText::new(&day_number.to_string())
                                            .color(text_color)
                                            .size(12.0),
                                    );
                                });
                            },
                        );
                    }

                    day_counter += 1;
                }
                ui.end_row();

                // Break if we've shown all days of the month
                if day_counter - first_day_weekday >= days_in_month as i32 {
                    break;
                }
            }
        });
}

struct MonthStats {
    completed_days: usize,
    success_rate: f64,
}

fn calculate_month_stats(habit: &crate::data::Habit, current_date: NaiveDate) -> MonthStats {
    let first_day = current_date.with_day(1).unwrap_or(current_date);
    let days_in_month = if current_date.month() == 12 {
        NaiveDate::from_ymd_opt(current_date.year() + 1, 1, 1)
            .unwrap()
            .signed_duration_since(first_day)
            .num_days()
    } else {
        NaiveDate::from_ymd_opt(current_date.year(), current_date.month() + 1, 1)
            .unwrap()
            .signed_duration_since(first_day)
            .num_days()
    } as u32;

    let today = Local::now().date_naive();
    let mut completed_days = 0;
    let mut valid_days = 0;

    for day in 1..=days_in_month {
        if let Some(date) = first_day.with_day(day) {
            // Only count days up to today (don't count future days)
            if date <= today {
                valid_days += 1;
                let date_str = date.format("%Y-%m-%d").to_string();
                if habit.completion_dates.contains(&date_str) {
                    completed_days += 1;
                }
            }
        }
    }

    let success_rate = if valid_days > 0 {
        (completed_days as f64 / valid_days as f64) * 100.0
    } else {
        0.0
    };

    MonthStats {
        completed_days,
        success_rate,
    }
}

