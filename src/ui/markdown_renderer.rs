use eframe::egui::{self, Color32, RichText, TextureHandle};
use std::collections::HashMap;
use std::path::Path;

pub struct MarkdownRendererState {
    pub image_cache: HashMap<String, TextureHandle>,
}

impl Default for MarkdownRendererState {
    fn default() -> Self {
        Self {
            image_cache: HashMap::new(),
        }
    }
}

pub fn render_markdown(
    ui: &mut egui::Ui,
    markdown: &str,
    zoom_level: f32,
    renderer_state: &mut MarkdownRendererState,
    ctx: &egui::Context,
) {
    let font_size = 14.0 * zoom_level;
    ui.style_mut()
        .text_styles
        .get_mut(&egui::TextStyle::Body)
        .unwrap()
        .size = font_size;

    let lines = markdown.lines();

    for line in lines {
        let trimmed = line.trim();

        // Handle image syntax: ![alt text](path/to/image.png)
        if let Some(image_match) = regex_image_match(trimmed) {
            let (alt_text, image_path) = image_match;
            render_image(ui, &alt_text, &image_path, zoom_level, renderer_state, ctx);
        }
        // Handle headers
        else if trimmed.starts_with("# ") {
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

// Helper function to extract image details using regex
fn regex_image_match(text: &str) -> Option<(String, String)> {
    // Basic regex pattern for markdown images
    let re = regex::Regex::new(r"!\[(.*?)\]\((.*?)\)").ok()?;
    if let Some(captures) = re.captures(text) {
        let alt_text = captures.get(1)?.as_str().to_string();
        let path = captures.get(2)?.as_str().to_string();
        return Some((alt_text, path));
    }
    None
}

fn render_image(
    ui: &mut egui::Ui,
    alt_text: &str,
    image_path: &str,
    zoom_level: f32,
    renderer_state: &mut MarkdownRendererState,
    ctx: &egui::Context,
) {
    // Check if we already have this image in cache
    if !renderer_state.image_cache.contains_key(image_path) {
        // Try to load the image
        let path = Path::new(image_path);
        if !path.exists() {
            ui.label(RichText::new(format!("Image not found: {}", image_path)).color(Color32::RED));
            return;
        }

        // Load the image
        if let Ok(image_data) = std::fs::read(path) {
            if let Ok(image) = image::load_from_memory(&image_data) {
                let size = [image.width() as usize, image.height() as usize];
                let image_buffer = image.to_rgba8();
                let pixels = image_buffer.as_flat_samples();

                let color_image = egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());

                let options = egui::TextureOptions {
                    magnification: egui::TextureFilter::Linear,
                    minification: egui::TextureFilter::Linear,
                    ..Default::default()
                };
                
                let texture = ctx.load_texture(image_path, color_image, options);

                renderer_state
                    .image_cache
                    .insert(image_path.to_string(), texture);
            } else {
                ui.label(
                    RichText::new(format!("Failed to decode image: {}", image_path))
                        .color(Color32::RED),
                );
                return;
            }
        } else {
            ui.label(
                RichText::new(format!("Failed to read image: {}", image_path)).color(Color32::RED),
            );
            return;
        }
    }

    // Display the image
    if let Some(texture) = renderer_state.image_cache.get(image_path) {
        // Calculate appropriate size based on zoom level and original dimensions
        let max_width = ui.available_width() * 0.8; // Use 80% of available width max

        let mut size = texture.size_vec2();
        if size.x > max_width {
            let ratio = max_width / size.x;
            size *= ratio;
        }

        // Apply zoom
        size *= zoom_level;

        // Create Image widget from the texture handle directly
        let image = egui::Image::new(texture).fit_to_exact_size(size);
        
        // Check if the image is hovered
        let response = ui.add(image);
        
        // Show tooltip if hovered
        if response.hovered() {
            egui::show_tooltip(ui.ctx(), egui::Id::new("image_tooltip"), |ui| {
                ui.label(alt_text);
            });
        }
    }
}
