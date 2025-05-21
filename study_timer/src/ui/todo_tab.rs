use crate::app::StatusMessage;
use crate::data::StudyData;
use crate::settings::AppSettings;
use egui::{ScrollArea, TextEdit};
use std::cell::RefCell;
use std::collections::HashMap;

// We'll use thread-local storage instead of once_cell
thread_local! {
    static NEW_TODO: RefCell<String> = RefCell::new(String::new());
    static EDITING_MAP: RefCell<HashMap<u64, String>> = RefCell::new(HashMap::new());
}

pub fn display(
    ui: &mut egui::Ui,
    study_data: &mut StudyData,
    status: &mut StatusMessage,
    settings: &AppSettings,
) {
    let colors = settings.get_current_colors();

    ui.heading(egui::RichText::new("Todo List").color(colors.text_primary_color32()));

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

    // Show status message
    status.render(ui);
}

