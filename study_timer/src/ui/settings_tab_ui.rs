use crate::app::{StatusMessage, Tab};
use crate::settings::{AppSettings, NavigationLayout};
use eframe::egui;

pub fn display(
    ui: &mut egui::Ui,
    settings: &mut AppSettings,
    status: &mut StatusMessage,
    current_tab: &mut Tab,
) {
    ui.heading("Settings");
    ui.add_space(20.0);

    egui::ScrollArea::vertical().show(ui, |ui| {
        // Navigation Layout Section
        ui.group(|ui| {
            ui.heading("Navigation Layout");
            ui.add_space(10.0);

            let mut layout_changed = false;

            ui.horizontal(|ui| {
                if ui
                    .radio_value(
                        &mut settings.navigation_layout,
                        NavigationLayout::Horizontal,
                        "Horizontal",
                    )
                    .clicked()
                {
                    layout_changed = true;
                }
                if ui
                    .radio_value(
                        &mut settings.navigation_layout,
                        NavigationLayout::Vertical,
                        "Vertical",
                    )
                    .clicked()
                {
                    layout_changed = true;
                }
            });

            if layout_changed {
                if let Err(e) = settings.save() {
                    status.show(&format!("Failed to save layout setting: {}", e));
                } else {
                    status.show("Layout setting saved!");
                }
            }
        });

        ui.add_space(20.0);

        // Tab Management Section
        ui.group(|ui| {
            ui.heading("Tab Management");
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
                    if index > 0 && ui.button("U").clicked() {
                        move_up_index = Some(index);
                    }
                    if index < settings.tab_configs.len() - 2 && ui.button("D").clicked() {
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
                    if config.custom_name.is_some() && ui.button("Reset Name").clicked() {
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
            ui.heading("Reset Options");
            ui.add_space(10.0);

            ui.horizontal(|ui| {
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
            ui.heading("Information");
            ui.add_space(5.0);
            ui.label("• Use U/D buttons to reorder tabs");
            ui.label("• Custom names will be saved and remembered");
            ui.label("• The Settings tab is always visible and enabled");
            ui.label("• Disabled tabs will be hidden from the navigation");
            ui.label("• Changes are automatically saved");
        });
    });
}

