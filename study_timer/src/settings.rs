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
            crate::app::Tab::Todo => "Todo".to_string(),
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
}

impl Default for AppSettings {
    fn default() -> Self {
        let default_tabs = vec![
            TabConfig::new(crate::app::Tab::Timer, true),
            TabConfig::new(crate::app::Tab::Record, true),
            TabConfig::new(crate::app::Tab::Stats, true),
            TabConfig::new(crate::app::Tab::Graph, true),
            TabConfig::new(crate::app::Tab::Todo, true),
            TabConfig::new(crate::app::Tab::Reminder, true),
            TabConfig::new(crate::app::Tab::Calculator, true),
            TabConfig::new(crate::app::Tab::Markdown, true),
            TabConfig::new(crate::app::Tab::Terminal, true),
            TabConfig::new(crate::app::Tab::Settings, true),
        ];

        Self {
            navigation_layout: NavigationLayout::default(),
            tab_configs: default_tabs,
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

    pub fn get_tab_config(&self, tab: &crate::app::Tab) -> Option<&TabConfig> {
        self.tab_configs
            .iter()
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

