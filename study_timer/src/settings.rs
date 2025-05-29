use eframe::egui::Color32;
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NavigationLayout {
    Horizontal,
    Vertical,
}

impl Default for NavigationLayout {
    fn default() -> Self {
        NavigationLayout::Horizontal
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PresetTheme {
    Default,
    Dark,
    Ocean,
    Forest,
    Sunset,
    Purple,
    Gruvbox,
    RosePine,
    Nord,
    Dracula,
    Monokai,
    Custom,
}

impl PresetTheme {
    pub fn get_colors(&self) -> ColorTheme {
        match self {
            PresetTheme::Default => ColorTheme::default(),
            PresetTheme::Dark => ColorTheme {
                background: [18, 18, 18, 255],
                navigation_background: [28, 28, 28, 255],
                active_tab: [64, 128, 255, 255],
                inactive_tab: [128, 128, 128, 255],
                text_primary: [255, 255, 255, 255],
                text_secondary: [200, 200, 200, 255],
                accent: [69, 74, 108, 255],
                panel_background: [32, 32, 32, 255],
            },
            PresetTheme::Ocean => ColorTheme {
                background: [25, 42, 86, 255],
                navigation_background: [30, 50, 100, 255],
                active_tab: [100, 200, 255, 255],
                inactive_tab: [150, 150, 180, 255],
                text_primary: [240, 248, 255, 255],
                text_secondary: [200, 220, 240, 255],
                accent: [100, 200, 255, 255],
                panel_background: [35, 55, 110, 255],
            },
            PresetTheme::Forest => ColorTheme {
                background: [34, 49, 34, 255],
                navigation_background: [45, 60, 45, 255],
                active_tab: [144, 238, 144, 255],
                inactive_tab: [128, 128, 128, 255],
                text_primary: [240, 255, 240, 255],
                text_secondary: [200, 220, 200, 255],
                accent: [144, 238, 144, 255],
                panel_background: [40, 55, 40, 255],
            },
            PresetTheme::Sunset => ColorTheme {
                background: [60, 30, 30, 255],
                navigation_background: [80, 40, 40, 255],
                active_tab: [255, 165, 0, 255],
                inactive_tab: [200, 150, 100, 255],
                text_primary: [255, 240, 220, 255],
                text_secondary: [220, 200, 180, 255],
                accent: [255, 165, 0, 255],
                panel_background: [70, 35, 35, 255],
            },
            PresetTheme::Purple => ColorTheme {
                background: [45, 35, 65, 255],
                navigation_background: [55, 45, 75, 255],
                active_tab: [186, 85, 211, 255],
                inactive_tab: [150, 120, 170, 255],
                text_primary: [248, 240, 255, 255],
                text_secondary: [220, 200, 240, 255],
                accent: [186, 85, 211, 255],
                panel_background: [50, 40, 70, 255],
            },
            PresetTheme::Gruvbox => ColorTheme {
                background: [40, 40, 40, 255],            // #282828
                navigation_background: [50, 48, 47, 255], // #32302f
                active_tab: [250, 189, 47, 255],          // #fabd2f
                inactive_tab: [146, 131, 116, 255],       // #928374
                text_primary: [235, 219, 178, 255],       // #ebdbb2
                text_secondary: [168, 153, 132, 255],     // #a89984
                accent: [254, 128, 25, 255],              // #fe8019
                panel_background: [60, 56, 54, 255],      // #3c3836
            },
            PresetTheme::RosePine => ColorTheme {
                background: [25, 23, 36, 255],            // #191724
                navigation_background: [31, 29, 46, 255], // #1f1d2e
                active_tab: [235, 188, 186, 255],         // #ebbcba
                inactive_tab: [110, 106, 134, 255],       // #6e6a86
                text_primary: [224, 222, 244, 255],       // #e0def4
                text_secondary: [144, 140, 170, 255],     // #908caa
                accent: [196, 167, 231, 255],             // #c4a7e7
                panel_background: [38, 35, 58, 255],      // #26233a
            },
            PresetTheme::Nord => ColorTheme {
                background: [46, 52, 64, 255],            // #2e3440
                navigation_background: [59, 66, 82, 255], // #3b4252
                active_tab: [136, 192, 208, 255],         // #88c0d0
                inactive_tab: [76, 86, 106, 255],         // #4c566a
                text_primary: [236, 239, 244, 255],       // #eceff4
                text_secondary: [216, 222, 233, 255],     // #d8dee9
                accent: [94, 129, 172, 255],              // #5e81ac
                panel_background: [67, 76, 94, 255],      // #434c5e
            },
            PresetTheme::Dracula => ColorTheme {
                background: [40, 42, 54, 255],            // #282a36
                navigation_background: [68, 71, 90, 255], // #44475a
                active_tab: [255, 121, 198, 255],         // #ff79c6
                inactive_tab: [98, 114, 164, 255],        // #6272a4
                text_primary: [248, 248, 242, 255],       // #f8f8f2
                text_secondary: [189, 147, 249, 255],     // #bd93f9
                accent: [139, 233, 253, 255],             // #8be9fd
                panel_background: [68, 71, 90, 255],      // #44475a
            },
            PresetTheme::Monokai => ColorTheme {
                background: [39, 40, 34, 255],            // #272822
                navigation_background: [73, 72, 62, 255], // #49483e
                active_tab: [249, 38, 114, 255],          // #f92672
                inactive_tab: [117, 113, 94, 255],        // #75715e
                text_primary: [248, 248, 242, 255],       // #f8f8f2
                text_secondary: [166, 226, 46, 255],      // #a6e22e
                accent: [174, 129, 255, 255],             // #ae81ff
                panel_background: [73, 72, 62, 255],      // #49483e
            },
            PresetTheme::Custom => ColorTheme::default(), // Will use custom values
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            PresetTheme::Default => "Default",
            PresetTheme::Dark => "Dark",
            PresetTheme::Ocean => "Ocean",
            PresetTheme::Forest => "Forest",
            PresetTheme::Sunset => "Sunset",
            PresetTheme::Purple => "Purple",
            PresetTheme::Gruvbox => "Gruvbox",
            PresetTheme::RosePine => "Rose Pine",
            PresetTheme::Nord => "Nord",
            PresetTheme::Dracula => "Dracula",
            PresetTheme::Monokai => "Monokai",
            PresetTheme::Custom => "Custom",
        }
    }

    pub fn all_presets() -> Vec<PresetTheme> {
        vec![
            PresetTheme::Default,
            PresetTheme::Dark,
            PresetTheme::Ocean,
            PresetTheme::Forest,
            PresetTheme::Sunset,
            PresetTheme::Purple,
            PresetTheme::Gruvbox,
            PresetTheme::RosePine,
            PresetTheme::Nord,
            PresetTheme::Dracula,
            PresetTheme::Monokai,
            PresetTheme::Custom,
        ]
    }

    /// Get themes arranged in rows for UI display (2 rows x 6 themes each)
    pub fn get_theme_rows() -> Vec<Vec<PresetTheme>> {
        let all_themes = Self::all_presets();
        let mut rows = Vec::new();

        // First row - 6 themes
        rows.push(vec![
            PresetTheme::Default,
            PresetTheme::Dark,
            PresetTheme::Ocean,
            PresetTheme::Forest,
            PresetTheme::Sunset,
            PresetTheme::Purple,
        ]);

        // Second row - 6 themes (including Custom at the end)
        rows.push(vec![
            PresetTheme::Gruvbox,
            PresetTheme::RosePine,
            PresetTheme::Nord,
            PresetTheme::Dracula,
            PresetTheme::Monokai,
        ]);

        // Third row for Custom (if you want to keep it separate)
        rows.push(vec![PresetTheme::Custom]);

        rows
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorTheme {
    pub background: [u8; 4],
    pub navigation_background: [u8; 4],
    pub active_tab: [u8; 4],
    pub inactive_tab: [u8; 4],
    pub text_primary: [u8; 4],
    pub text_secondary: [u8; 4],
    pub accent: [u8; 4],
    pub panel_background: [u8; 4],
}

impl Default for ColorTheme {
    fn default() -> Self {
        Self {
            background: [32, 32, 32, 255],
            navigation_background: [48, 48, 48, 255],
            active_tab: [68, 68, 68, 255],
            inactive_tab: [96, 96, 96, 255],
            text_primary: [255, 255, 255, 255],
            text_secondary: [180, 180, 180, 255],
            accent: [69, 74, 108, 255],
            panel_background: [40, 40, 40, 255],
        }
    }
}

impl ColorTheme {
    pub fn to_color32(&self, color: [u8; 4]) -> Color32 {
        Color32::from_rgba_unmultiplied(color[0], color[1], color[2], color[3])
    }

    pub fn background_color32(&self) -> Color32 {
        self.to_color32(self.background)
    }

    pub fn navigation_background_color32(&self) -> Color32 {
        self.to_color32(self.navigation_background)
    }

    pub fn active_tab_color32(&self) -> Color32 {
        self.to_color32(self.active_tab)
    }

    pub fn inactive_tab_color32(&self) -> Color32 {
        self.to_color32(self.inactive_tab)
    }

    pub fn text_primary_color32(&self) -> Color32 {
        self.to_color32(self.text_primary)
    }

    pub fn text_secondary_color32(&self) -> Color32 {
        self.to_color32(self.text_secondary)
    }

    pub fn accent_color32(&self) -> Color32 {
        self.to_color32(self.accent)
    }

    pub fn panel_background_color32(&self) -> Color32 {
        self.to_color32(self.panel_background)
    }

    pub fn from_color32(color: Color32) -> [u8; 4] {
        let rgba = color.to_array();
        [rgba[0], rgba[1], rgba[2], rgba[3]]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabConfig {
    pub tab_type: crate::app::Tab,
    pub enabled: bool,
    pub custom_name: Option<String>,
}

impl TabConfig {
    pub fn new(tab_type: crate::app::Tab, enabled: bool) -> Self {
        Self {
            tab_type,
            enabled,
            custom_name: None,
        }
    }

    pub fn get_display_name(&self) -> String {
        if let Some(custom_name) = &self.custom_name {
            custom_name.clone()
        } else {
            self.get_default_name()
        }
    }

    pub fn get_default_name(&self) -> String {
        match self.tab_type {
            crate::app::Tab::Timer => "Timer".to_string(),
            crate::app::Tab::Stats => "Statistics".to_string(),
            crate::app::Tab::Record => "Record".to_string(),
            crate::app::Tab::Graph => "Graph".to_string(),
            crate::app::Tab::Todo => "Todo and Habits".to_string(),
            crate::app::Tab::Flashcards => "Flashcards".to_string(),
            crate::app::Tab::Calculator => "Calculator".to_string(),
            crate::app::Tab::Markdown => "Markdown".to_string(),
            crate::app::Tab::Reminder => "Reminder".to_string(),
            crate::app::Tab::Terminal => "Terminal".to_string(),
            crate::app::Tab::Settings => "Settings".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub navigation_layout: NavigationLayout,
    pub tab_configs: Vec<TabConfig>,
    pub theme_preset: PresetTheme,
    pub custom_colors: ColorTheme,
}

impl Default for AppSettings {
    fn default() -> Self {
        let default_tabs = vec![
            TabConfig::new(crate::app::Tab::Timer, true),
            TabConfig::new(crate::app::Tab::Record, true),
            TabConfig::new(crate::app::Tab::Stats, true),
            TabConfig::new(crate::app::Tab::Graph, true),
            TabConfig::new(crate::app::Tab::Todo, true),
            TabConfig::new(crate::app::Tab::Flashcards, true),
            TabConfig::new(crate::app::Tab::Reminder, true),
            TabConfig::new(crate::app::Tab::Calculator, true),
            TabConfig::new(crate::app::Tab::Markdown, true),
            TabConfig::new(crate::app::Tab::Terminal, true),
            TabConfig::new(crate::app::Tab::Settings, true),
        ];

        Self {
            navigation_layout: NavigationLayout::default(),
            tab_configs: default_tabs,
            theme_preset: PresetTheme::Default,
            custom_colors: ColorTheme::default(),
        }
    }
}

impl AppSettings {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let settings_path = Path::new("app_settings.json");

        if !settings_path.exists() {
            return Ok(AppSettings::default());
        }

        let mut file = File::open(settings_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let mut settings: AppSettings = serde_json::from_str(&contents)?;

        // Ensure all tabs are present (for compatibility with older versions)
        settings.ensure_all_tabs_present();

        Ok(settings)
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(&self)?;
        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open("app_settings.json")?;

        file.write_all(json.as_bytes())?;
        Ok(())
    }

    pub fn get_current_colors(&self) -> ColorTheme {
        if self.theme_preset == PresetTheme::Custom {
            self.custom_colors.clone()
        } else {
            self.theme_preset.get_colors()
        }
    }

    pub fn apply_theme(&self, ctx: &eframe::egui::Context) {
        let colors = self.get_current_colors();

        let mut visuals = ctx.style().visuals.clone();

        // Apply background colors
        visuals.panel_fill = colors.background_color32();
        visuals.window_fill = colors.panel_background_color32();
        visuals.extreme_bg_color = colors.navigation_background_color32();

        // Apply widget colors
        visuals.widgets.active.bg_fill = colors.active_tab_color32();
        visuals.widgets.hovered.bg_fill = colors.accent_color32();
        visuals.widgets.inactive.bg_fill = colors.inactive_tab_color32();

        // Apply selection colors
        visuals.selection.bg_fill = colors.accent_color32();
        visuals.selection.stroke.color = colors.accent_color32();

        // Apply text colors through override_text_color
        visuals.override_text_color = Some(colors.text_primary_color32());

        ctx.set_visuals(visuals);
    }

    pub fn is_tab_enabled(&self, tab: &crate::app::Tab) -> bool {
        if *tab == crate::app::Tab::Settings {
            return true; // Settings tab is always enabled
        }

        self.tab_configs
            .iter()
            .find(|config| config.tab_type == *tab)
            .map(|config| config.enabled)
            .unwrap_or(false)
    }

    pub fn get_first_enabled_tab(&self) -> crate::app::Tab {
        for config in &self.tab_configs {
            if config.enabled {
                return config.tab_type.clone();
            }
        }

        // If no tabs are enabled, fallback to settings
        crate::app::Tab::Settings
    }

    pub fn get_enabled_tabs(&self) -> Vec<&TabConfig> {
        self.tab_configs
            .iter()
            .filter(|config| config.enabled)
            .collect()
    }

    pub fn get_tab_config_mut(&mut self, tab: &crate::app::Tab) -> Option<&mut TabConfig> {
        self.tab_configs
            .iter_mut()
            .find(|config| config.tab_type == *tab)
    }

    pub fn move_tab_up(&mut self, index: usize) {
        if index > 0 && index < self.tab_configs.len() {
            self.tab_configs.swap(index - 1, index);
        }
    }

    pub fn move_tab_down(&mut self, index: usize) {
        if index < self.tab_configs.len() - 1 {
            self.tab_configs.swap(index, index + 1);
        }
    }

    pub fn reset_tab_name(&mut self, tab: &crate::app::Tab) {
        if let Some(config) = self.get_tab_config_mut(tab) {
            config.custom_name = None;
        }
    }

    pub fn reset_tab_order(&mut self) {
        // Reset to default order
        let default_order = vec![
            crate::app::Tab::Timer,
            crate::app::Tab::Record,
            crate::app::Tab::Stats,
            crate::app::Tab::Graph,
            crate::app::Tab::Todo,
            crate::app::Tab::Reminder,
            crate::app::Tab::Calculator,
            crate::app::Tab::Markdown,
            crate::app::Tab::Terminal,
            crate::app::Tab::Settings,
        ];

        let mut new_configs = Vec::new();

        // Reorder according to default order, preserving settings
        for tab_type in default_order {
            if let Some(config) = self
                .tab_configs
                .iter()
                .find(|c| c.tab_type == tab_type)
                .cloned()
            {
                new_configs.push(config);
            }
        }

        self.tab_configs = new_configs;
    }

    fn ensure_all_tabs_present(&mut self) {
        let all_tabs = vec![
            crate::app::Tab::Timer,
            crate::app::Tab::Record,
            crate::app::Tab::Stats,
            crate::app::Tab::Graph,
            crate::app::Tab::Todo,
            crate::app::Tab::Reminder,
            crate::app::Tab::Flashcards,
            crate::app::Tab::Calculator,
            crate::app::Tab::Markdown,
            crate::app::Tab::Terminal,
            crate::app::Tab::Settings,
        ];

        for tab in all_tabs {
            if !self.tab_configs.iter().any(|config| config.tab_type == tab) {
                self.tab_configs.push(TabConfig::new(tab, true));
            }
        }
    }
}

