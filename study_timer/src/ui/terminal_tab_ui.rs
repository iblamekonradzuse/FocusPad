use crate::terminal::{TerminalEmulator, TerminalEntryType};
use eframe::egui::{self, Color32, Key, RichText, TextEdit};

pub fn display(
    ui: &mut egui::Ui,
    terminal: &mut TerminalEmulator,
    _status: &mut crate::app::StatusMessage,
) {
    ui.vertical(|ui| {
        // Directory header
        ui.horizontal(|ui| {
            ui.label(RichText::new("Current directory: ").strong());
            ui.label(&terminal.current_directory.display().to_string());
        });

        ui.separator();

        // Handle different modes (normal, pager, fuzzy)
        if let Some(content) = &terminal.pager_content.clone() {
            // PAGER MODE
            display_pager(ui, terminal, content);
        } else if terminal.fuzzy_mode {
            // FUZZY FINDER MODE
            display_fuzzy_finder(ui, terminal);
        } else {
            // NORMAL MODE
            let available_height = ui.available_height();

            // Terminal output area with scrolling (now first)
            // Create a ScrollArea that always scrolls to bottom
            let scroll = egui::ScrollArea::vertical()
                .stick_to_bottom(true)
                .auto_shrink([false; 2])
                .max_height(available_height - 40.0);

            // Show the scroll area with terminal output
            scroll.show(ui, |ui| {
                for entry in &terminal.output_history {
                    match entry.entry_type {
                        TerminalEntryType::Command => {
                            ui.label(RichText::new(&entry.content).color(Color32::GREEN).strong());
                        }
                        TerminalEntryType::Output => {
                            ui.label(&entry.content);
                        }
                        TerminalEntryType::Error => {
                            ui.label(RichText::new(&entry.content).color(Color32::RED));
                        }
                    }
                }
            });

            ui.add_space(8.0);

            // Command input at bottom (after output area)
            ui.horizontal(|ui| {
                // Terminal prompt
                ui.label(RichText::new("> ").strong().color(Color32::GREEN));

                // Create text edit for command input
                let text_edit = TextEdit::singleline(&mut terminal.current_input)
                    .desired_width(f32::INFINITY)
                    .hint_text("Type a command and press Enter");

                let response = ui.add(text_edit);

                // Handle Enter key to execute command
                if response.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter)) {
                    terminal.execute_command();
                    response.request_focus();
                }

                // Handle up/down keys for history navigation
                if response.has_focus() {
                    if ui.input(|i| i.key_pressed(Key::ArrowUp)) {
                        terminal.navigate_history(true);
                    } else if ui.input(|i| i.key_pressed(Key::ArrowDown)) {
                        terminal.navigate_history(false);
                    }
                }

                // Visual hint about the key commands
                ui.add_space(10.0);
                ui.label(RichText::new("↑↓: History").weak().italics());
            });
        }
    });
}

fn display_fuzzy_finder(ui: &mut egui::Ui, terminal: &mut TerminalEmulator) {
    // Title
    ui.vertical_centered(|ui| {
        ui.label(
            RichText::new(format!("Fuzzy search: '{}'", terminal.fuzzy_query))
                .size(18.0)
                .strong(),
        );
        ui.label(
            RichText::new("Press ↑↓: Navigate, Enter: Select, Esc: Cancel")
                .size(14.0)
                .italics(),
        );
    });

    ui.add_space(10.0);

    // Results
    egui::ScrollArea::vertical()
        .max_height(ui.available_height() - 100.0)
        .show(ui, |ui| {
            if terminal.fuzzy_results.is_empty() {
                ui.vertical_centered(|ui| {
                    ui.label(RichText::new("No matches found").color(Color32::RED));
                });
            } else {
                for (i, path) in terminal.fuzzy_results.iter().enumerate() {
                    let is_selected = i == terminal.fuzzy_index;
                    let text = format!("{}", path.display());
                    let mut text = RichText::new(text);

                    if is_selected {
                        text = text
                            .background_color(Color32::from_rgb(50, 50, 80))
                            .color(Color32::WHITE)
                            .strong();

                        ui.horizontal(|ui| {
                            ui.label("▶");
                            ui.add(egui::Label::new(text).wrap(false));
                        });
                    } else {
                        ui.horizontal(|ui| {
                            ui.label(" ");
                            ui.add(egui::Label::new(text).wrap(false));
                        });
                    }
                }
            }
        });

    // Handle keyboard navigation
    if ui.input(|i| i.key_pressed(Key::ArrowUp)) {
        terminal.select_prev_fuzzy_result();
    } else if ui.input(|i| i.key_pressed(Key::ArrowDown)) {
        terminal.select_next_fuzzy_result();
    } else if ui.input(|i| i.key_pressed(Key::Enter)) {
        if let Some(selected) = terminal.get_selected_fuzzy_result() {
            // If it's a directory, cd to it, otherwise cat the file
            if selected.is_dir() {
                let path_display = selected.display().to_string(); // Get display string before move
                terminal.current_directory = selected.clone();
                terminal
                    .output_history
                    .push(crate::terminal::TerminalEntry {
                        content: format!("> cd {}", path_display),
                        entry_type: TerminalEntryType::Command,
                    });

                terminal
                    .output_history
                    .push(crate::terminal::TerminalEntry {
                        content: format!("Changed directory to: {}", selected.display()),
                        entry_type: TerminalEntryType::Output,
                    });
            } else {
                // Cat the file content
                terminal
                    .output_history
                    .push(crate::terminal::TerminalEntry {
                        content: format!("> cat {}", selected.display()),
                        entry_type: TerminalEntryType::Command,
                    });

                match std::fs::read_to_string(&selected) {
                    Ok(content) => {
                        terminal
                            .output_history
                            .push(crate::terminal::TerminalEntry {
                                content,
                                entry_type: TerminalEntryType::Output,
                            });
                    }
                    Err(e) => {
                        terminal
                            .output_history
                            .push(crate::terminal::TerminalEntry {
                                content: format!("Failed to read file: {}", e),
                                entry_type: TerminalEntryType::Error,
                            });
                    }
                }
            }
        }
        terminal.exit_fuzzy_mode();
    } else if ui.input(|i| i.key_pressed(Key::Escape)) {
        terminal.exit_fuzzy_mode();
    }
}

fn display_pager(ui: &mut egui::Ui, terminal: &mut TerminalEmulator, content: &str) {
    // Pager title
    ui.vertical_centered(|ui| {
        ui.label(RichText::new("Pager Mode").size(18.0).strong());
        ui.label(
            RichText::new("j/k: Scroll, Space: Page down, q: Exit")
                .size(14.0)
                .italics(),
        );
    });

    ui.add_space(10.0);

    // Calculate visible height (in text lines)
    let approx_line_height = 18.0; // Estimated pixels per line
    let available_height = ui.available_height() - 80.0; // Reserve space for UI elements
    let visible_lines = (available_height / approx_line_height).floor() as usize;

    // Content display
    let lines: Vec<&str> = content.lines().collect();
    let total_lines = lines.len();

    let start_line = terminal.pager_offset;
    let end_line = (start_line + visible_lines).min(total_lines);

    egui::ScrollArea::vertical()
        .max_height(available_height)
        .show(ui, |ui| {
            // Show line numbers and content
            for i in start_line..end_line {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(format!("{:4} ", i + 1)).color(Color32::GRAY));
                    ui.label(lines[i]);
                });
            }

            // Show scroll position
            ui.add_space(5.0);
            ui.separator();
            ui.horizontal(|ui| {
                let percentage = if total_lines > 0 {
                    ((end_line as f32) / (total_lines as f32) * 100.0).min(100.0)
                } else {
                    100.0
                };

                ui.label(format!(
                    "Lines {}-{} of {} ({}%)",
                    start_line + 1,
                    end_line,
                    total_lines,
                    percentage.round() as i32
                ));
            });
        });

    // Handle keyboard navigation
    if ui.input(|i| i.key_pressed(Key::J)) {
        terminal.scroll_pager(1, visible_lines);
    } else if ui.input(|i| i.key_pressed(Key::K)) {
        terminal.scroll_pager(-1, visible_lines);
    } else if ui.input(|i| i.key_pressed(Key::Space)) {
        terminal.scroll_pager(visible_lines as i32 - 2, visible_lines);
    } else if ui.input(|i| i.key_pressed(Key::Q)) || ui.input(|i| i.key_pressed(Key::Escape)) {
        terminal.exit_pager();
    }
}
