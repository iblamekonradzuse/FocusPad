use crate::ui::markdown_editor::MarkdownEditor;
use eframe::egui;
use std::fs;
use std::path::Path;

pub fn render_file_browser(
    ui: &mut egui::Ui,
    editor: &mut MarkdownEditor,
    status_update: impl FnMut(&str),
) {
    let mut status_update = status_update;

    ui.heading("Files");
    ui.separator();

    // Show current selected folder
    if let Some(folder) = &editor.selected_folder {
        let folder_name = folder
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown");
        ui.label(format!("Current folder: 📁 {}", folder_name));
        if ui.button("📤 Go Up").clicked() {
            if let Some(parent) = folder.parent() {
                if parent == Path::new("files") {
                    editor.selected_folder = None;
                } else if parent.starts_with("files") {
                    editor.selected_folder = Some(parent.to_path_buf());
                }
            } else {
                editor.selected_folder = None;
            }
        }
        ui.separator();
    }

    // Create new file input
    ui.horizontal(|ui| {
        ui.label("New file:");
        let response = ui.text_edit_singleline(&mut editor.new_file_name);
        if response.lost_focus()
            && ui.input(|i| i.key_pressed(egui::Key::Enter))
            && !editor.new_file_name.is_empty()
        {
            let file_name = editor.new_file_name.clone();
            if let Ok(file_path) = editor.create_file(&file_name) {
                status_update(&format!("Created file: {}", file_name));
                editor.selected_entry = Some(file_path.clone());
                let _ = editor.open_file(&file_path);
                editor.new_file_name.clear();
            } else {
                status_update("Failed to create file");
            }
        }
        if ui.button("📄").clicked() && !editor.new_file_name.is_empty() {
            let file_name = editor.new_file_name.clone();
            if let Ok(file_path) = editor.create_file(&file_name) {
                status_update(&format!("Created file: {}", file_name));
                editor.selected_entry = Some(file_path.clone());
                let _ = editor.open_file(&file_path);
                editor.new_file_name.clear();
            } else {
                status_update("Failed to create file");
            }
        }
    });

    // Create new folder input
    ui.horizontal(|ui| {
        ui.label("New folder:");
        let response = ui.text_edit_singleline(&mut editor.new_folder_name);
        if response.lost_focus()
            && ui.input(|i| i.key_pressed(egui::Key::Enter))
            && !editor.new_folder_name.is_empty()
        {
            let folder_name = editor.new_folder_name.clone();
            if let Ok(folder_path) = editor.create_folder(&folder_name) {
                status_update(&format!("Created folder: {}", folder_name));
                editor.new_folder_name.clear();
                // Auto-select the newly created folder
                editor.selected_folder = Some(folder_path);
            } else {
                status_update("Failed to create folder");
            }
        }
        if ui.button("📁").clicked() && !editor.new_folder_name.is_empty() {
            let folder_name = editor.new_folder_name.clone();
            if let Ok(folder_path) = editor.create_folder(&folder_name) {
                status_update(&format!("Created folder: {}", folder_name));
                editor.new_folder_name.clear();
                // Auto-select the newly created folder
                editor.selected_folder = Some(folder_path);
            } else {
                status_update("Failed to create folder");
            }
        }
    });

    ui.separator();

    // File list with collapsible directory tree
    egui::ScrollArea::vertical().show(ui, |ui| {
        let start_path = if let Some(folder) = &editor.selected_folder {
            folder.clone()
        } else {
            Path::new("files").to_path_buf()
        };

        // If we're at the root, show all top-level entries
        if start_path == Path::new("files") {
            if let Ok(entries) = fs::read_dir(&start_path) {
                let mut entries: Vec<_> = entries.flatten().collect();

                // Sort directories first, then files
                entries.sort_by(|a, b| {
                    let a_is_dir = a.path().is_dir();
                    let b_is_dir = b.path().is_dir();
                    if a_is_dir && !b_is_dir {
                        std::cmp::Ordering::Less
                    } else if !a_is_dir && b_is_dir {
                        std::cmp::Ordering::Greater
                    } else {
                        a.file_name().cmp(&b.file_name())
                    }
                });

                for entry in entries {
                    let path = entry.path();
                    render_file_entry(ui, editor, &path, &mut status_update);
                }
            }
        } else {
            // We're inside a specific folder, show only its contents
            if let Ok(entries) = fs::read_dir(&start_path) {
                let mut entries: Vec<_> = entries.flatten().collect();

                // Sort directories first, then files
                entries.sort_by(|a, b| {
                    let a_is_dir = a.path().is_dir();
                    let b_is_dir = b.path().is_dir();
                    if a_is_dir && !b_is_dir {
                        std::cmp::Ordering::Less
                    } else if !a_is_dir && b_is_dir {
                        std::cmp::Ordering::Greater
                    } else {
                        a.file_name().cmp(&b.file_name())
                    }
                });

                for entry in entries {
                    let path = entry.path();
                    render_file_entry(ui, editor, &path, &mut status_update);
                }
            }
        }
    });
}

fn render_file_entry(
    ui: &mut egui::Ui,
    editor: &mut MarkdownEditor,
    path: &Path,
    status_update: &mut impl FnMut(&str),
) {
    if !path.exists() {
        return;
    }

    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("Unknown");

    let is_folder = path.is_dir();
    let is_selected = editor
        .selected_entry
        .as_ref()
        .map_or(false, |selected| *selected == path);
    let is_expanded = editor.is_folder_expanded(path);

    let indent_level = path.components().count() - Path::new("files").components().count();
    let indent = " ".repeat(indent_level * 2);

    ui.horizontal(|ui| {
        ui.label(&indent);

        if is_folder {
            // Folder toggle button
            let toggle_icon = if is_expanded { "🔽" } else { "▶️" };
            if ui.button(toggle_icon).clicked() {
                editor.toggle_folder_expansion(path);
            }

            // Folder label
            let folder_text = format!("📁 {}", file_name);
            if ui.selectable_label(is_selected, folder_text).clicked() {
                editor.selected_entry = Some(path.to_path_buf());
                editor.selected_folder = Some(path.to_path_buf());
            }
        } else {
            // File spacing (for alignment with folders)
            ui.add_space(20.0);

            // File label
            let file_text = format!("📄 {}", file_name);
            if ui.selectable_label(is_selected, file_text).clicked() {
                editor.selected_entry = Some(path.to_path_buf());
                if let Err(e) = editor.open_file(&path.to_path_buf()) {
                    status_update(&format!("Error opening file: {}", e));
                }
            }
        }
    });

    if is_selected {
        ui.horizontal(|ui| {
            ui.add_space((indent_level * 2 + 4) as f32 * 10.0); // Indent for action buttons

            if ui.button("✏️ Rename").clicked() {
                editor.show_rename_dialog = true;
                editor.rename_buffer = file_name.to_string();
            }

            if ui.button("🗑️ Delete").clicked() {
                if let Err(e) = editor.delete_entry(path) {
                    status_update(&format!("Error deleting: {}", e));
                } else {
                    status_update("Item deleted");
                    editor.selected_entry = None;
                }
            }
        });

        if editor.show_rename_dialog {
            ui.horizontal(|ui| {
                ui.add_space((indent_level * 2 + 4) as f32 * 10.0); // Indent for rename dialog
                ui.label("New name:");
                let response = ui.text_edit_singleline(&mut editor.rename_buffer);
                if response.lost_focus()
                    && ui.input(|i| i.key_pressed(egui::Key::Enter))
                    && !editor.rename_buffer.is_empty()
                {
                    let new_name = editor.rename_buffer.clone();
                    if let Err(e) = editor.rename_entry(path, &new_name) {
                        status_update(&format!("Error renaming: {}", e));
                    } else {
                        status_update("Item renamed");
                        editor.show_rename_dialog = false;
                    }
                }
                if ui.button("✓").clicked() && !editor.rename_buffer.is_empty() {
                    let new_name = editor.rename_buffer.clone();
                    if let Err(e) = editor.rename_entry(path, &new_name) {
                        status_update(&format!("Error renaming: {}", e));
                    } else {
                        status_update("Item renamed");
                        editor.show_rename_dialog = false;
                    }
                }
                if ui.button("❌").clicked() {
                    editor.show_rename_dialog = false;
                }
            });
        }
    }

    // Render children if this is an expanded folder
    if is_folder && is_expanded {
        if let Ok(entries) = fs::read_dir(path) {
            let mut entries: Vec<_> = entries.flatten().collect();

            // Sort directories first, then files
            entries.sort_by(|a, b| {
                let a_is_dir = a.path().is_dir();
                let b_is_dir = b.path().is_dir();
                if a_is_dir && !b_is_dir {
                    std::cmp::Ordering::Less
                } else if !a_is_dir && b_is_dir {
                    std::cmp::Ordering::Greater
                } else {
                    a.file_name().cmp(&b.file_name())
                }
            });

            for entry in entries {
                render_file_entry(ui, editor, &entry.path(), status_update);
            }
        }
    }
}
