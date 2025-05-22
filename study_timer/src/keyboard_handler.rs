use eframe::egui::{self, Key, Modifiers};

pub struct KeyboardHandler {
    pub new_tab_requested: bool,
    pub close_tab_requested: bool,
    pub split_horizontal_requested: bool,
    pub split_vertical_requested: bool,
    pub close_split_requested: bool,
}

impl KeyboardHandler {
    pub fn new() -> Self {
        Self {
            new_tab_requested: false,
            close_tab_requested: false,
            split_horizontal_requested: false,
            split_vertical_requested: false,
            close_split_requested: false,
        }
    }

    pub fn handle_input(&mut self, ctx: &egui::Context) {
        // Reset flags
        self.new_tab_requested = false;
        self.close_tab_requested = false;
        self.split_horizontal_requested = false;
        self.split_vertical_requested = false;
        self.close_split_requested = false;

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
        });
    }

    pub fn reset(&mut self) {
        self.new_tab_requested = false;
        self.close_tab_requested = false;
        self.split_horizontal_requested = false;
        self.split_vertical_requested = false;
        self.close_split_requested = false;
    }
}

