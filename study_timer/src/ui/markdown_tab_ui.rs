use crate::app::StudyTimerApp;
use eframe::egui::{self, Color32, RichText};
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

const FILES_DIR: &str = "files";

#[derive(PartialEq)]
pub enum EditorMode {
    Edit,
    Preview,
    Split,
}

pub struct MarkdownEditor {
    current_file: Option<PathBuf>,
    current_content: String,
    selected_entry: Option<PathBuf>,
    editor_mode: EditorMode,
    zoom_level: f32,
    new_file_name: String,
    new_folder_name: String,
    rename_buffer: String,
    show_rename_dialog: bool,
    file_browser_collapsed: bool,
    // Track selected folder for creating files inside it
    selected_folder: Option<PathBuf>,
}

impl Default for MarkdownEditor {
    fn default() -> Self {
        // Ensure the files directory exists
        if !Path::new(FILES_DIR).exists() {
            let _ = fs::create_dir_all(FILES_DIR);
        }

        Self {
            current_file: None,
            current_content: String::new(),
            selected_entry: None,
            editor_mode: EditorMode::Split,
            zoom_level: 1.0,
            new_file_name: String::new(),
            new_folder_name: String::new(),
            rename_buffer: String::new(),
            show_rename_dialog: false,
            file_browser_collapsed: false,
            selected_folder: None,
        }
    }
}

impl MarkdownEditor {
    fn open_file(&mut self, path: &PathBuf) -> Result<(), std::io::Error> {
        let mut file = File::open(path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;

        self.current_content = content;
        self.current_file = Some(path.clone());
        Ok(())
    }

    fn save_file(&mut self) -> Result<(), std::io::Error> {
        if let Some(path) = &self.current_file {
            let mut file = OpenOptions::new()
                .write(true)
                .truncate(true)
                .create(true)
                .open(path)?;

            file.write_all(self.current_content.as_bytes())?;
        }
        Ok(())
    }

    fn create_file(&mut self, name: &str) -> Result<PathBuf, std::io::Error> {
        // Determine the directory where the file should be created
        let parent_dir = if let Some(folder) = &self.selected_folder {
            if folder.is_dir() {
                folder.clone()
            } else {
                Path::new(FILES_DIR).to_path_buf()
            }
        } else {
            Path::new(FILES_DIR).to_path_buf()
        };

        // Add .md extension if not present
        let file_name = if !name.ends_with(".md") {
            format!("{}.md", name)
        } else {
            name.to_string()
        };

        let file_path = parent_dir.join(file_name);

        // Create an empty file
        let mut file = File::create(&file_path)?;
        file.write_all(b"")?;

        Ok(file_path)
    }

    fn create_folder(&mut self, name: &str) -> Result<PathBuf, std::io::Error> {
        // Determine the parent directory
        let parent_dir = if let Some(folder) = &self.selected_folder {
            if folder.is_dir() {
                folder.clone()
            } else {
                Path::new(FILES_DIR).to_path_buf()
            }
        } else {
            Path::new(FILES_DIR).to_path_buf()
        };

        let folder_path = parent_dir.join(name);
        fs::create_dir_all(&folder_path)?;
        Ok(folder_path)
    }

    fn delete_entry(&mut self, path: &Path) -> Result<(), std::io::Error> {
        if path.is_file() {
            fs::remove_file(path)?;
            if self.current_file.as_ref().map_or(false, |p| p == path) {
                self.current_file = None;
                self.current_content.clear();
            }
        } else if path.is_dir() {
            fs::remove_dir_all(path)?;
            if self
                .current_file
                .as_ref()
                .map_or(false, |p| p.starts_with(path))
            {
                self.current_file = None;
                self.current_content.clear();
            }
            // Clear selected folder if we just deleted it
            if self.selected_folder.as_ref().map_or(false, |p| p == path) {
                self.selected_folder = None;
            }
        }
        Ok(())
    }

    fn rename_entry(&mut self, path: &Path, new_name: &str) -> Result<PathBuf, std::io::Error> {
        let parent = path.parent().unwrap_or(Path::new(""));
        let new_path = parent.join(new_name);

        fs::rename(path, &new_path)?;

        // Update current_file if the renamed file was open
        if self.current_file.as_ref().map_or(false, |p| p == path) {
            self.current_file = Some(new_path.clone());
        }

        // Update selected folder if we just renamed it
        if self.selected_folder.as_ref().map_or(false, |p| p == path) {
            self.selected_folder = Some(new_path.clone());
        }

        Ok(new_path)
    }

    fn list_files(&self) -> Vec<PathBuf> {
        let mut result = Vec::new();
        if let Ok(entries) = fs::read_dir(FILES_DIR) {
            for entry in entries.flatten() {
                result.push(entry.path());
            }
        }
        // Sort directories first, then files
        result.sort_by(|a, b| {
            let a_is_dir = a.is_dir();
            let b_is_dir = b.is_dir();
            if a_is_dir && !b_is_dir {
                std::cmp::Ordering::Less
            } else if !a_is_dir && b_is_dir {
                std::cmp::Ordering::Greater
            } else {
                a.file_name().cmp(&b.file_name())
            }
        });
        result
    }

    // Add markdown formatting to selected text
    fn add_formatting(&mut self, format_type: &str) {
        // This is a placeholder - in a real implementation,
        // we would need to handle text selection which requires more complex UI state management
        match format_type {
            "bold" => {
                self.current_content.push_str("**Bold Text**");
            }
            "italic" => {
                self.current_content.push_str("*Italic Text*");
            }
            "red" => {
                self.current_content.push_str("<color=red>Red Text</color>");
            }
            "green" => {
                self.current_content
                    .push_str("<color=green>Green Text</color>");
            }
            "blue" => {
                self.current_content
                    .push_str("<color=blue>Blue Text</color>");
            }
            "bold_italic" => {
                self.current_content.push_str("***Bold and Italic***");
            }
            _ => {}
        }
    }

    // Enhanced markdown rendering to properly display formatted text
    fn render_markdown(&self, ui: &mut egui::Ui, markdown: &str) {
        let font_size = 14.0 * self.zoom_level;
        ui.style_mut()
            .text_styles
            .get_mut(&egui::TextStyle::Body)
            .unwrap()
            .size = font_size;

        let lines = markdown.lines();

        for line in lines {
            let trimmed = line.trim();

            // Handle headers
            if trimmed.starts_with("# ") {
                ui.heading(RichText::new(&trimmed[2..]).size(font_size * 1.8).strong());
                ui.add_space(5.0);
            } else if trimmed.starts_with("## ") {
                ui.heading(RichText::new(&trimmed[3..]).size(font_size * 1.5).strong());
                ui.add_space(4.0);
            } else if trimmed.starts_with("### ") {
                ui.heading(RichText::new(&trimmed[4..]).size(font_size * 1.3).strong());
                ui.add_space(3.0);
            } else if trimmed.starts_with("#### ") {
                ui.heading(RichText::new(&trimmed[5..]).size(font_size * 1.1).strong());
                ui.add_space(2.0);
            }
            // Handle bold and italic text together (***text***)
            else if trimmed.contains("***") {
                let parts: Vec<&str> = line.split("***").collect();

                ui.horizontal(|ui| {
                    for (i, part) in parts.iter().enumerate() {
                        if i % 2 == 0 {
                            if !part.is_empty() {
                                ui.label(RichText::new(*part).size(font_size));
                            }
                        } else {
                            ui.label(RichText::new(*part).size(font_size).strong().italics());
                        }
                    }
                });
            }
            // Handle bold text (**text**)
            else if trimmed.contains("**") {
                let parts: Vec<&str> = line.split("**").collect();

                ui.horizontal(|ui| {
                    for (i, part) in parts.iter().enumerate() {
                        if i % 2 == 0 {
                            if !part.is_empty() {
                                ui.label(RichText::new(*part).size(font_size));
                            }
                        } else {
                            ui.label(RichText::new(*part).size(font_size).strong());
                        }
                    }
                });
            }
            // Handle italic text (*text*)
            else if trimmed.contains("*") {
                let parts: Vec<&str> = line.split("*").collect();

                ui.horizontal(|ui| {
                    for (i, part) in parts.iter().enumerate() {
                        if i % 2 == 0 {
                            if !part.is_empty() {
                                ui.label(RichText::new(*part).size(font_size));
                            }
                        } else {
                            ui.label(RichText::new(*part).size(font_size).italics());
                        }
                    }
                });
            }
            // Handle colored text - red
            else if trimmed.contains("<color=red>") && trimmed.contains("</color>") {
                let parts: Vec<&str> = line.split("<color=red>").collect();
                ui.horizontal(|ui| {
                    if !parts[0].is_empty() {
                        ui.label(RichText::new(parts[0]).size(font_size));
                    }

                    for i in 1..parts.len() {
                        let color_parts: Vec<&str> = parts[i].split("</color>").collect();
                        if !color_parts[0].is_empty() {
                            ui.label(
                                RichText::new(color_parts[0])
                                    .color(Color32::RED)
                                    .size(font_size),
                            );
                        }
                        if color_parts.len() > 1 && !color_parts[1].is_empty() {
                            ui.label(RichText::new(color_parts[1]).size(font_size));
                        }
                    }
                });
            }
            // Handle colored text - green
            else if trimmed.contains("<color=green>") && trimmed.contains("</color>") {
                let parts: Vec<&str> = line.split("<color=green>").collect();
                ui.horizontal(|ui| {
                    if !parts[0].is_empty() {
                        ui.label(RichText::new(parts[0]).size(font_size));
                    }

                    for i in 1..parts.len() {
                        let color_parts: Vec<&str> = parts[i].split("</color>").collect();
                        if !color_parts[0].is_empty() {
                            ui.label(
                                RichText::new(color_parts[0])
                                    .color(Color32::GREEN)
                                    .size(font_size),
                            );
                        }
                        if color_parts.len() > 1 && !color_parts[1].is_empty() {
                            ui.label(RichText::new(color_parts[1]).size(font_size));
                        }
                    }
                });
            }
            // Handle colored text - blue
            else if trimmed.contains("<color=blue>") && trimmed.contains("</color>") {
                let parts: Vec<&str> = line.split("<color=blue>").collect();
                ui.horizontal(|ui| {
                    if !parts[0].is_empty() {
                        ui.label(RichText::new(parts[0]).size(font_size));
                    }

                    for i in 1..parts.len() {
                        let color_parts: Vec<&str> = parts[i].split("</color>").collect();
                        if !color_parts[0].is_empty() {
                            ui.label(
                                RichText::new(color_parts[0])
                                    .color(Color32::BLUE)
                                    .size(font_size),
                            );
                        }
                        if color_parts.len() > 1 && !color_parts[1].is_empty() {
                            ui.label(RichText::new(color_parts[1]).size(font_size));
                        }
                    }
                });
            }
            // Handle bullet points
            else if trimmed.starts_with("- ") {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("•").size(font_size));
                    ui.label(RichText::new(&trimmed[2..]).size(font_size));
                });
            }
            // Handle numbered lists
            else if trimmed.len() > 2
                && trimmed.chars().nth(0).unwrap().is_numeric()
                && trimmed.chars().nth(1).unwrap() == '.'
            {
                let content = &trimmed[2..].trim();
                ui.horizontal(|ui| {
                    ui.label(RichText::new(&trimmed[..2]).size(font_size));
                    ui.label(RichText::new(*content).size(font_size));
                });
            }
            // Handle code blocks
            else if trimmed.starts_with("```") || trimmed.ends_with("```") {
                ui.monospace(
                    RichText::new(if trimmed == "```" { "" } else { trimmed })
                        .size(font_size * 0.9),
                );
            }
            // Handle inline code
            else if trimmed.contains("`") {
                let parts: Vec<&str> = line.split("`").collect();

                ui.horizontal(|ui| {
                    for (i, part) in parts.iter().enumerate() {
                        if i % 2 == 0 {
                            if !part.is_empty() {
                                ui.label(RichText::new(*part).size(font_size));
                            }
                        } else {
                            ui.monospace(RichText::new(*part).size(font_size * 0.9));
                        }
                    }
                });
            }
            // Handle horizontal rule
            else if trimmed == "---" || trimmed == "***" || trimmed == "___" {
                ui.separator();
            }
            // Handle blockquotes
            else if trimmed.starts_with("> ") {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("▌").size(font_size));
                    ui.label(RichText::new(&trimmed[2..]).italics().size(font_size));
                });
            }
            // Regular text
            else if !trimmed.is_empty() {
                ui.label(RichText::new(line).size(font_size));
            } else {
                ui.add_space(font_size * 0.5);
            }
        }
    }
}

pub fn display(ui: &mut egui::Ui, app: &mut StudyTimerApp) {
    // Initialize the markdown editor if it doesn't exist
    if app.markdown_editor.is_none() {
        app.markdown_editor = Some(MarkdownEditor::default());
    }

    let editor = app.markdown_editor.as_mut().unwrap();

    ui.horizontal(|ui| {
        ui.heading("Markdown Files");

        // Zoom controls
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            // Add collapse/expand button
            let collapse_text = if editor.file_browser_collapsed {
                "📂 Expand"
            } else {
                "📁 Collapse"
            };
            if ui.button(collapse_text).clicked() {
                editor.file_browser_collapsed = !editor.file_browser_collapsed;
            }

            ui.separator();

            if ui.button("🔍+").clicked() {
                editor.zoom_level += 0.1;
                if editor.zoom_level > 2.0 {
                    editor.zoom_level = 2.0;
                }
            }
            if ui.button("🔍-").clicked() {
                editor.zoom_level -= 0.1;
                if editor.zoom_level < 0.5 {
                    editor.zoom_level = 0.5;
                }
            }
            ui.label(format!("Zoom: {:.1}x", editor.zoom_level));
        });
    });

    ui.separator();

    // Two-panel layout with conditionally collapsed file browser
    if !editor.file_browser_collapsed {
        egui::SidePanel::left("file_browser")
            .resizable(true)
            .default_width(150.0)
            .show_inside(ui, |ui| {
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
                        editor.selected_folder = None;
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
                            app.status.show(&format!("Created file: {}", file_name));
                            editor.selected_entry = Some(file_path.clone());
                            let _ = editor.open_file(&file_path);
                            editor.new_file_name.clear();
                        } else {
                            app.status.show("Failed to create file");
                        }
                    }
                    if ui.button("📄").clicked() && !editor.new_file_name.is_empty() {
                        let file_name = editor.new_file_name.clone();
                        if let Ok(file_path) = editor.create_file(&file_name) {
                            app.status.show(&format!("Created file: {}", file_name));
                            editor.selected_entry = Some(file_path.clone());
                            let _ = editor.open_file(&file_path);
                            editor.new_file_name.clear();
                        } else {
                            app.status.show("Failed to create file");
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
                        if let Ok(_) = editor.create_folder(&folder_name) {
                            app.status.show(&format!("Created folder: {}", folder_name));
                            editor.new_folder_name.clear();
                        } else {
                            app.status.show("Failed to create folder");
                        }
                    }
                    if ui.button("📁").clicked() && !editor.new_folder_name.is_empty() {
                        let folder_name = editor.new_folder_name.clone();
                        if let Ok(_) = editor.create_folder(&folder_name) {
                            app.status.show(&format!("Created folder: {}", folder_name));
                            editor.new_folder_name.clear();
                        } else {
                            app.status.show("Failed to create folder");
                        }
                    }
                });

                ui.separator();

                // File list
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for path in editor.list_files() {
                        let path_clone = path.clone(); // Clone path for later use
                        let file_name = path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("Unknown");

                        let icon = if path.is_dir() { "📁 " } else { "📄 " };
                        let text = format!("{}{}", icon, file_name);

                        let is_selected = editor
                            .selected_entry
                            .as_ref()
                            .map_or(false, |selected| *selected == path);

                        if ui.selectable_label(is_selected, text).clicked() {
                            editor.selected_entry = Some(path.clone());
                            if path.is_file() {
                                if let Err(e) = editor.open_file(&path) {
                                    app.status.show(&format!("Error opening file: {}", e));
                                }
                            } else if path.is_dir() {
                                // Set as selected folder when clicking on a directory
                                editor.selected_folder = Some(path.clone());
                            }
                        }

                        if is_selected {
                            ui.horizontal(|ui| {
                                if ui.button("✏️ Rename").clicked() {
                                    editor.show_rename_dialog = true;
                                    editor.rename_buffer = file_name.to_string();
                                }
                                if ui.button("🗑️ Delete").clicked() {
                                    if let Err(e) = editor.delete_entry(&path_clone) {
                                        app.status.show(&format!("Error deleting: {}", e));
                                    } else {
                                        app.status.show("Item deleted");
                                        editor.selected_entry = None;
                                    }
                                }
                            });

                            if editor.show_rename_dialog {
                                ui.horizontal(|ui| {
                                    ui.label("New name:");
                                    let response =
                                        ui.text_edit_singleline(&mut editor.rename_buffer);
                                    if response.lost_focus()
                                        && ui.input(|i| i.key_pressed(egui::Key::Enter))
                                        && !editor.rename_buffer.is_empty()
                                    {
                                        let new_name = editor.rename_buffer.clone();
                                        if let Err(e) = editor.rename_entry(&path_clone, &new_name)
                                        {
                                            app.status.show(&format!("Error renaming: {}", e));
                                        } else {
                                            app.status.show("Item renamed");
                                            editor.show_rename_dialog = false;
                                        }
                                    }
                                    if ui.button("✓").clicked() && !editor.rename_buffer.is_empty()
                                    {
                                        let new_name = editor.rename_buffer.clone();
                                        if let Err(e) = editor.rename_entry(&path_clone, &new_name)
                                        {
                                            app.status.show(&format!("Error renaming: {}", e));
                                        } else {
                                            app.status.show("Item renamed");
                                            editor.show_rename_dialog = false;
                                        }
                                    }
                                    if ui.button("❌").clicked() {
                                        editor.show_rename_dialog = false;
                                    }
                                });
                            }
                        }
                    }
                });
            });
    }

    // Editor panel
    if editor.current_file.is_some() {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                if let Some(path) = &editor.current_file {
                    ui.label(format!("Editing: {}", path.display()));

                    // Move the mode toggles to their own section on the right
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("💾 Save").clicked() {
                            if let Err(e) = editor.save_file() {
                                app.status.show(&format!("Error saving file: {}", e));
                            } else {
                                app.status.show("File saved");
                            }
                        }
                        ui.separator();

                        // Editor mode toggle
                        ui.selectable_value(&mut editor.editor_mode, EditorMode::Edit, "Edit");
                        ui.selectable_value(
                            &mut editor.editor_mode,
                            EditorMode::Preview,
                            "Preview",
                        );
                        ui.selectable_value(&mut editor.editor_mode, EditorMode::Split, "Split");
                    });
                }
            });

            // Add formatting buttons in their own row
            if editor.editor_mode == EditorMode::Edit || editor.editor_mode == EditorMode::Split {
                ui.horizontal(|ui| {
                    ui.label("Format:");
                    ui.separator();
                    if ui.button(RichText::new("B").strong()).clicked() {
                        editor.add_formatting("bold");
                    }
                    if ui.button(RichText::new("I").italics()).clicked() {
                        editor.add_formatting("italic");
                    }
                    if ui.button(RichText::new("B+I").strong().italics()).clicked() {
                        editor.add_formatting("bold_italic");
                    }

                    // Color buttons
                    ui.separator();
                    if ui.button(RichText::new("R").color(Color32::RED)).clicked() {
                        editor.add_formatting("red");
                    }
                    if ui
                        .button(RichText::new("G").color(Color32::GREEN))
                        .clicked()
                    {
                        editor.add_formatting("green");
                    }
                    if ui.button(RichText::new("B").color(Color32::BLUE)).clicked() {
                        editor.add_formatting("blue");
                    }
                });
            }

            ui.separator();

            match editor.editor_mode {
                EditorMode::Edit => {
                    // Full editor
                    let text_height = ui.available_height();
                    egui::ScrollArea::vertical()
                        .id_source("editor_scroll")
                        .show(ui, |ui| {
                            let font_size = 14.0 * editor.zoom_level;
                            let text_style = egui::TextStyle::Monospace;
                            ui.style_mut()
                                .text_styles
                                .get_mut(&text_style)
                                .unwrap()
                                .size = font_size;

                            ui.add_sized(
                                [ui.available_width(), text_height],
                                egui::TextEdit::multiline(&mut editor.current_content)
                                    .font(text_style)
                                    .desired_width(f32::INFINITY),
                            );
                        });
                }
                EditorMode::Preview => {
                    // Full preview
                    egui::ScrollArea::vertical()
                        .id_source("preview_scroll")
                        .show(ui, |ui| {
                            editor.render_markdown(ui, &editor.current_content);
                        });
                }
                EditorMode::Split => {
                    // Split view - using show_inside instead of show
                    egui::CentralPanel::default().show_inside(ui, |ui| {
                        ui.columns(2, |columns| {
                            // Editor on left column
                            columns[0].heading("Editor");
                            egui::ScrollArea::vertical()
                                .id_source("editor_scroll_split")
                                .show(&mut columns[0], |ui| {
                                    let font_size = 14.0 * editor.zoom_level;
                                    let text_style = egui::TextStyle::Monospace;
                                    ui.style_mut()
                                        .text_styles
                                        .get_mut(&text_style)
                                        .unwrap()
                                        .size = font_size;

                                    ui.add_sized(
                                        [ui.available_width(), ui.available_height()],
                                        egui::TextEdit::multiline(&mut editor.current_content)
                                            .font(text_style)
                                            .desired_width(f32::INFINITY),
                                    );
                                });

                            // Preview on right column
                            columns[1].heading("Preview");
                            egui::ScrollArea::vertical()
                                .id_source("preview_scroll_split")
                                .show(&mut columns[1], |ui| {
                                    editor.render_markdown(ui, &editor.current_content);
                                });
                        });
                    });
                }
            }
        });
    } else {
        ui.centered_and_justified(|ui| {
            ui.heading("Select or create a file to start editing");
        });
    }

    // Status message
    app.status.render(ui);
}

