use crate::app::StatusMessage;
use crate::data::StudyData;
use egui::{ScrollArea, TextEdit};
use std::cell::RefCell;
use std::collections::HashMap;

// We'll use thread-local storage instead of once_cell
thread_local! {
    static NEW_TODO: RefCell<String> = RefCell::new(String::new());
    static EDITING_MAP: RefCell<HashMap<u64, String>> = RefCell::new(HashMap::new());
}

pub fn display(ui: &mut egui::Ui, study_data: &mut StudyData, status: &mut StatusMessage) {
    ui.heading("Todo List");

    // Add new todo section
    ui.horizontal(|ui| {
        ui.label("New Task:");

        // Use thread_local with with() to access the value
        NEW_TODO.with(|todo_ref| {
            let mut new_todo = todo_ref.borrow_mut();

            let text_edit = ui.add(
                TextEdit::singleline(&mut *new_todo)
                    .hint_text("Enter a new task...")
                    .desired_width(280.0),
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

            if ui.button("Add").clicked() && !new_todo.is_empty() {
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

    // Buttons for clearing todos
    ui.horizontal(|ui| {
        if ui.button("Clear Completed").clicked() {
            if let Err(e) = study_data.clear_completed_todos() {
                status.show(&format!("Error clearing completed todos: {}", e));
            } else {
                status.show("Completed todos cleared!");
            }
        }

        if ui.button("Clear All").clicked() {
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
            ui.label("No todos yet. Add one above!");
            return;
        }

        // Use thread_local with with() to access the editing map
        EDITING_MAP.with(|map_ref| {
            let mut editing_map = map_ref.borrow_mut();

            // Display todos without changing them in this loop
            for todo in &study_data.todos {
                let is_editing = editing_map.contains_key(&todo.id);

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
                            // Create a mutable String reference for the TextEdit widget

                            if ui.button("Save").clicked() && !edit_text.is_empty() {
                                // Clone the String before moving it
                                let text_to_save = edit_text.clone();
                                edit_todos.push((todo.id, text_to_save));
                                cancel_editing.push(todo.id);
                            }

                            if ui.button("Cancel").clicked() {
                                cancel_editing.push(todo.id);
                            }
                        }
                    } else {
                        // Display the todo text with strikethrough if completed
                        let text = if todo.completed {
                            egui::RichText::new(&todo.text).strikethrough()
                        } else {
                            egui::RichText::new(&todo.text)
                        };
                        ui.label(text);

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("❌").clicked() {
                                delete_todos.push(todo.id);
                            }

                            if ui.button("✏️").clicked() {
                                start_editing.push((todo.id, todo.text.clone()));
                            }
                        });
                    }
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

