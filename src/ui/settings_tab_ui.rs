use crate::app::{StatusMessage, Tab};
use crate::settings::{AppSettings, ColorTheme, PresetTheme};
use eframe::egui::{self};

pub fn display(
    ui: &mut egui::Ui,
    settings: &mut AppSettings,
    status: &mut StatusMessage,
    current_tab: &mut Tab,
) {
    ui.heading("⚙️ Settings");
    ui.add_space(20.0);

    egui::ScrollArea::vertical().show(ui, |ui| {
        // Theme Section
        ui.group(|ui| {
            ui.heading("🎨 Theme");
            ui.add_space(10.0);

            let mut theme_changed = false;
            let old_preset = settings.theme_preset.clone();

            // Display themes in 2 rows with 6 themes each
            let all_presets = PresetTheme::all_presets();
            let themes_per_row = 6;

            for row in 0..2 {
                ui.horizontal_wrapped(|ui| {
                    let start_idx = row * themes_per_row;
                    let end_idx = (start_idx + themes_per_row).min(all_presets.len());

                    for preset in &all_presets[start_idx..end_idx] {
                        let is_selected = settings.theme_preset == *preset;

                        // Create a colored button for visual preview
                        let colors = if *preset == PresetTheme::Custom {
                            settings.custom_colors.clone()
                        } else {
                            preset.get_colors()
                        };

                        let bg_color = colors.background_color32();

                        let button = egui::Button::new(preset.name())
                            .fill(if is_selected {
                                colors.active_tab_color32()
                            } else {
                                bg_color
                            })
                            .stroke(egui::Stroke::new(1.0, colors.accent_color32()));

                        if ui.add(button).clicked() {
                            settings.theme_preset = preset.clone();
                            theme_changed = true;
                        }
                    }
                });

                if row == 0 {
                    ui.add_space(5.0);
                }
            }

            // Custom color editor (only show when Custom is selected)
            if settings.theme_preset == PresetTheme::Custom {
                ui.add_space(15.0);
                ui.separator();
                ui.add_space(10.0);
                ui.heading("Custom Colors");
                ui.add_space(10.0);

                let mut custom_changed = false;

                egui::Grid::new("color_grid")
                    .num_columns(3)
                    .spacing([10.0, 8.0])
                    .show(ui, |ui| {
                        // Background
                        ui.label("Background:");
                        let mut bg_color = settings.custom_colors.background_color32();
                        if ui.color_edit_button_srgba(&mut bg_color).changed() {
                            settings.custom_colors.background = ColorTheme::from_color32(bg_color);
                            custom_changed = true;
                        }
                        ui.end_row();

                        // Navigation Background
                        ui.label("Navigation:");
                        let mut nav_color = settings.custom_colors.navigation_background_color32();
                        if ui.color_edit_button_srgba(&mut nav_color).changed() {
                            settings.custom_colors.navigation_background =
                                ColorTheme::from_color32(nav_color);
                            custom_changed = true;
                        }
                        ui.end_row();

                        // Active Tab
                        ui.label("Active Tab:");
                        let mut active_color = settings.custom_colors.active_tab_color32();
                        if ui.color_edit_button_srgba(&mut active_color).changed() {
                            settings.custom_colors.active_tab =
                                ColorTheme::from_color32(active_color);
                            custom_changed = true;
                        }
                        ui.end_row();

                        // Inactive Tab
                        ui.label("Inactive Tab:");
                        let mut inactive_color = settings.custom_colors.inactive_tab_color32();
                        if ui.color_edit_button_srgba(&mut inactive_color).changed() {
                            settings.custom_colors.inactive_tab =
                                ColorTheme::from_color32(inactive_color);
                            custom_changed = true;
                        }
                        ui.end_row();

                        // Primary Text
                        ui.label("Primary Text:");
                        let mut text_color = settings.custom_colors.text_primary_color32();
                        if ui.color_edit_button_srgba(&mut text_color).changed() {
                            settings.custom_colors.text_primary =
                                ColorTheme::from_color32(text_color);
                            custom_changed = true;
                        }
                        ui.end_row();

                        // Secondary Text
                        ui.label("Secondary Text:");
                        let mut sec_text_color = settings.custom_colors.text_secondary_color32();
                        if ui.color_edit_button_srgba(&mut sec_text_color).changed() {
                            settings.custom_colors.text_secondary =
                                ColorTheme::from_color32(sec_text_color);
                            custom_changed = true;
                        }
                        ui.end_row();

                        // Accent Color
                        ui.label("Accent:");
                        let mut accent_color = settings.custom_colors.accent_color32();
                        if ui.color_edit_button_srgba(&mut accent_color).changed() {
                            settings.custom_colors.accent = ColorTheme::from_color32(accent_color);
                            custom_changed = true;
                        }
                        ui.end_row();

                        // Panel Background
                        ui.label("Panel Background:");
                        let mut panel_color = settings.custom_colors.panel_background_color32();
                        if ui.color_edit_button_srgba(&mut panel_color).changed() {
                            settings.custom_colors.panel_background =
                                ColorTheme::from_color32(panel_color);
                            custom_changed = true;
                        }
                        ui.end_row();
                    });

                if custom_changed {
                    theme_changed = true;
                }

                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    if ui.button("🔄 Reset to Default Colors").clicked() {
                        settings.custom_colors = ColorTheme::default();
                        theme_changed = true;
                    }

                    if ui.button("📋 Copy from Current Theme").clicked() {
                        if old_preset != PresetTheme::Custom {
                            settings.custom_colors = old_preset.get_colors();
                            theme_changed = true;
                        }
                    }
                });
            }

            if theme_changed {
                if let Err(e) = settings.save() {
                    status.show(&format!("Failed to save theme: {}", e));
                } else {
                    status.show("Theme saved successfully!");
                }
            }
        });

        ui.add_space(20.0);

        // Tab Management Section
        ui.group(|ui| {
            ui.heading("📑 Tab Management");
            ui.add_space(10.0);

            ui.label("Configure tabs visibility, names, and order:");
            ui.add_space(10.0);

            let mut any_changed = false;
            let mut move_up_index = None;
            let mut move_down_index = None;

            for (index, config) in settings.tab_configs.clone().iter().enumerate() {
                if config.tab_type == Tab::Settings {
                    continue; // Skip settings tab as it's always enabled
                }

                ui.horizontal(|ui| {
                    // Move up/down buttons
                    if index > 0 && ui.button("⬆").clicked() {
                        move_up_index = Some(index);
                    }
                    if index < settings.tab_configs.len() - 2 && ui.button("⬇").clicked() {
                        // -2 because settings is last
                        move_down_index = Some(index);
                    }

                    // Enable/disable checkbox
                    let mut enabled = config.enabled;
                    if ui.checkbox(&mut enabled, "").changed() {
                        if let Some(tab_config) = settings.get_tab_config_mut(&config.tab_type) {
                            tab_config.enabled = enabled;
                            any_changed = true;
                        }
                    }

                    // Tab name input
                    let mut display_name = config.get_display_name();
                    ui.label(format!("{}:", config.get_default_name()));

                    let text_edit =
                        egui::TextEdit::singleline(&mut display_name).desired_width(150.0);

                    if ui.add(text_edit).changed() {
                        if let Some(tab_config) = settings.get_tab_config_mut(&config.tab_type) {
                            if display_name == config.get_default_name() {
                                tab_config.custom_name = None;
                            } else {
                                tab_config.custom_name = Some(display_name);
                            }
                            any_changed = true;
                        }
                    }

                    // Reset name button
                    if config.custom_name.is_some() && ui.button("🔄 Reset Name").clicked() {
                        settings.reset_tab_name(&config.tab_type);
                        any_changed = true;
                    }
                });

                ui.add_space(5.0);
            }

            // Handle tab reordering
            if let Some(index) = move_up_index {
                settings.move_tab_up(index);
                any_changed = true;
            }
            if let Some(index) = move_down_index {
                settings.move_tab_down(index);
                any_changed = true;
            }

            if any_changed {
                if let Err(e) = settings.save() {
                    status.show(&format!("Failed to save tab settings: {}", e));
                } else {
                    status.show("Tab settings saved successfully!");
                }

                // If current tab is disabled, switch to first enabled tab
                if !settings.is_tab_enabled(current_tab) {
                    *current_tab = settings.get_first_enabled_tab();
                }
            }
        });

        ui.add_space(20.0);

        // Reset Section
        ui.group(|ui| {
            ui.heading("🔧 Reset Options");
            ui.add_space(10.0);

            ui.horizontal_wrapped(|ui| {
                if ui.button("🔄 Reset All Names").clicked() {
                    for config in &mut settings.tab_configs {
                        config.custom_name = None;
                    }
                    if let Err(e) = settings.save() {
                        status.show(&format!("Failed to reset names: {}", e));
                    } else {
                        status.show("All tab names reset to defaults!");
                    }
                }

                if ui.button("🔄 Reset Tab Order").clicked() {
                    settings.reset_tab_order();
                    if let Err(e) = settings.save() {
                        status.show(&format!("Failed to reset tab order: {}", e));
                    } else {
                        status.show("Tab order reset to default!");
                    }
                }

                if ui.button("🔄 Reset Theme").clicked() {
                    settings.theme_preset = PresetTheme::Default;
                    settings.custom_colors = ColorTheme::default();
                    if let Err(e) = settings.save() {
                        status.show(&format!("Failed to reset theme: {}", e));
                    } else {
                        status.show("Theme reset to default!");
                    }
                }

                if ui.button("🔄 Reset All Settings").clicked() {
                    *settings = AppSettings::default();
                    if let Err(e) = settings.save() {
                        status.show(&format!("Failed to reset all settings: {}", e));
                    } else {
                        status.show("All settings reset to defaults!");
                    }
                    *current_tab = settings.get_first_enabled_tab();
                }
            });
        });

        ui.add_space(20.0);

        // Information Section
        ui.group(|ui| {
            ui.heading("ℹ️ Information");
            ui.add_space(5.0);
            ui.label("• Choose from preset themes or create a custom one");
            ui.label("• Custom colors are saved when you select the Custom theme");
            ui.label("• Use ⬆/⬇ buttons to reorder tabs");
            ui.label("• Custom names will be saved and remembered");
            ui.label("• The Settings tab is always visible and enabled");
            ui.label("• Disabled tabs will be hidden from the navigation");
            ui.label("• Changes are automatically saved");
            ui.label("• Theme changes apply immediately");
        });
    });
}
