use eframe::egui::{self, Key};

pub struct KeyboardHandler {
    pub new_tab_requested: bool,
    pub close_tab_requested: bool,
    pub split_horizontal_requested: bool,
    pub split_vertical_requested: bool,
    pub close_split_requested: bool,
    pub tab_number_requested: Option<usize>,
    pub switch_to_last_tab_requested: bool,
}

impl KeyboardHandler {
    pub fn new() -> Self {
        Self {
            new_tab_requested: false,
            close_tab_requested: false,
            split_horizontal_requested: false,
            split_vertical_requested: false,
            close_split_requested: false,
            tab_number_requested: None,
            switch_to_last_tab_requested: false,
        }
    }

    pub fn handle_input(&mut self, ctx: &egui::Context) {
        // Reset flags
        self.new_tab_requested = false;
        self.close_tab_requested = false;
        self.split_horizontal_requested = false;
        self.split_vertical_requested = false;
        self.close_split_requested = false;
        self.tab_number_requested = None;
        self.switch_to_last_tab_requested = false;

        ctx.input(|i| {
            // Use mac_cmd for macOS and ctrl for other platforms
            let cmd_or_ctrl = if cfg!(target_os = "macos") {
                i.modifiers.mac_cmd
            } else {
                i.modifiers.ctrl
            };

            // Cmd/Ctrl + T - New Tab
            if cmd_or_ctrl && i.key_pressed(Key::T) {
                self.new_tab_requested = true;
            }

            // Cmd/Ctrl + W - Close Tab
            if cmd_or_ctrl && i.key_pressed(Key::W) {
                self.close_tab_requested = true;
            }

            // Cmd/Ctrl + Shift + H - Split Horizontal
            if cmd_or_ctrl && i.modifiers.shift && i.key_pressed(Key::H) {
                self.split_horizontal_requested = true;
            }

            // Cmd/Ctrl + Shift + V - Split Vertical
            if cmd_or_ctrl && i.modifiers.shift && i.key_pressed(Key::V) {
                self.split_vertical_requested = true;
            }

            // Cmd/Ctrl + Shift + X - Close Split
            if cmd_or_ctrl && i.modifiers.shift && i.key_pressed(Key::X) {
                self.close_split_requested = true;
            }

            // Option/Alt + Tab - Switch to last used tab
            if i.modifiers.alt && i.key_pressed(Key::Tab) {
                self.switch_to_last_tab_requested = true;
            }

            // Cmd/Ctrl + Number keys (1-9) - Switch to tab by index
            if cmd_or_ctrl {
                if i.key_pressed(Key::Num1) {
                    self.tab_number_requested = Some(0);
                } else if i.key_pressed(Key::Num2) {
                    self.tab_number_requested = Some(1);
                } else if i.key_pressed(Key::Num3) {
                    self.tab_number_requested = Some(2);
                } else if i.key_pressed(Key::Num4) {
                    self.tab_number_requested = Some(3);
                } else if i.key_pressed(Key::Num5) {
                    self.tab_number_requested = Some(4);
                } else if i.key_pressed(Key::Num6) {
                    self.tab_number_requested = Some(5);
                } else if i.key_pressed(Key::Num7) {
                    self.tab_number_requested = Some(6);
                } else if i.key_pressed(Key::Num8) {
                    self.tab_number_requested = Some(7);
                } else if i.key_pressed(Key::Num9) {
                    self.tab_number_requested = Some(8);
                }
            }
        });
    }
}

