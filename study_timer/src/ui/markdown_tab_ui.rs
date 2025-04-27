use crate::ui::markdown_editor::{EditorMode, MarkdownEditor};
use crate::ui::markdown_renderer;
use eframe::egui::{self, Color32, RichText};

pub fn render_editor(
    ui: &mut egui::Ui,
    editor: &mut MarkdownEditor,
    mut status_update: impl FnMut(&str),
) {
    // Get the path string outside any closures if it exists
    let file_path = match &editor.current_file {
        Some(path) => path.display().to_string(),
        None => {
            ui.centered_and_justified(|ui| {
                ui.heading("Select or create a file to start editing");
            });
            return;
        }
    };

    ui.horizontal(|ui| {
        ui.label(format!("Editing: {}", file_path));

        // Move the mode toggles to their own section on the right
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let save_clicked = ui.button("💾 Save").clicked();
            if save_clicked {
                if let Err(e) = editor.save_file() {
                    status_update(&format!("Error saving file: {}", e));
                } else {
                    status_update("File saved");
                }
            }
            ui.separator();

            // Editor mode toggle
            ui.selectable_value(&mut editor.editor_mode, EditorMode::Edit, "Edit");
            ui.selectable_value(&mut editor.editor_mode, EditorMode::Preview, "Preview");
            ui.selectable_value(&mut editor.editor_mode, EditorMode::Split, "Split");
        });
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
            render_edit_mode(ui, editor);
        }
        EditorMode::Preview => {
            render_preview_mode(ui, editor);
        }
        EditorMode::Split => {
            render_split_mode(ui, editor);
        }
    }
}

fn render_edit_mode(ui: &mut egui::Ui, editor: &mut MarkdownEditor) {
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

fn render_preview_mode(ui: &mut egui::Ui, editor: &mut MarkdownEditor) {
    // Full preview
    egui::ScrollArea::vertical()
        .id_source("preview_scroll")
        .show(ui, |ui| {
            markdown_renderer::render_markdown(ui, &editor.current_content, editor.zoom_level);
        });
}

fn render_split_mode(ui: &mut egui::Ui, editor: &mut MarkdownEditor) {
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
                    markdown_renderer::render_markdown(
                        ui,
                        &editor.current_content,
                        editor.zoom_level,
                    );
                });
        });
    });
}

pub fn display(ui: &mut egui::Ui, app: &mut crate::app::StudyTimerApp) {
    // Initialize the markdown editor if it's not already initialized
    if app.markdown_editor.is_none() {
        app.markdown_editor = Some(crate::ui::markdown_editor::MarkdownEditor::default());
    }

    // Get a mutable reference to the editor
    if let Some(editor) = &mut app.markdown_editor {
        // Add toggle button for file browser at the top
        ui.horizontal(|ui| {
            let collapse_text = if editor.file_browser_collapsed {
                "📂 Show Explorer"
            } else {
                "📁 Hide Explorer"
            };

            if ui.button(collapse_text).clicked() {
                editor.file_browser_collapsed = !editor.file_browser_collapsed;
            }
        });

        ui.separator();

        if editor.file_browser_collapsed {
            // Only show editor when file browser is collapsed
            ui.with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |ui| {
                render_editor(ui, editor, |message| {
                    app.status.show(message);
                });
            });
        } else {
            // Use columns to have file browser on left and editor on right
            ui.columns(2, |columns| {
                // File browser on the left column
                columns[0].with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |ui| {
                    crate::ui::file_browser::render_file_browser(ui, editor, |message| {
                        app.status.show(message);
                    });
                });

                // Editor on the right column
                columns[1].with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |ui| {
                    render_editor(ui, editor, |message| {
                        app.status.show(message);
                    });
                });
            });
        }
    }
}

