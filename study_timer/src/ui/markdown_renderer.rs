use eframe::egui::{self, Color32, RichText};

pub fn render_markdown(ui: &mut egui::Ui, markdown: &str, zoom_level: f32) {
    let font_size = 14.0 * zoom_level;
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
                RichText::new(if trimmed == "```" { "" } else { trimmed }).size(font_size * 0.9),
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
