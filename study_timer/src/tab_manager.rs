use crate::app::Tab;
use crate::settings::AppSettings;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TabInstance {
    pub id: String,
    pub tab_type: Tab,
    pub title: String,
    pub file_path: Option<String>,
    pub is_modified: bool,
    pub can_close: bool,
}

impl TabInstance {
    pub fn new(tab_type: Tab) -> Self {
        let title = match tab_type {
            Tab::Timer => "Timer".to_string(),
            Tab::Stats => "Statistics".to_string(),
            Tab::Record => "Record".to_string(),
            Tab::Graph => "Graph".to_string(),
            Tab::Todo => "Todo".to_string(),
            Tab::Calculator => "Calculator".to_string(),
            Tab::Markdown => "New Markdown".to_string(),
            Tab::Reminder => "Reminder".to_string(),
            Tab::Terminal => "Terminal".to_string(),
            Tab::Settings => "Settings".to_string(),
        };

        let can_close = tab_type != Tab::Settings; // Settings tab cannot be closed

        Self {
            id: Uuid::new_v4().to_string(),
            tab_type,
            title,
            file_path: None,
            is_modified: false,
            can_close,
        }
    }

    pub fn new_with_file(tab_type: Tab, file_path: String) -> Self {
        let file_name = std::path::Path::new(&file_path)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("Unknown File")
            .to_string();

        Self {
            id: Uuid::new_v4().to_string(),
            tab_type,
            title: file_name,
            file_path: Some(file_path),
            is_modified: false,
            can_close: true,
        }
    }

    pub fn get_display_title(&self) -> String {
        let modified_indicator = if self.is_modified { "●" } else { "" };
        format!("{}{}", self.title, modified_indicator)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SplitDirection {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone)]
pub struct SplitPane {
    pub left_tab_id: String,
    pub right_tab_id: String,
    pub direction: SplitDirection,
    pub split_ratio: f32, // 0.0 to 1.0, position of the divider
}

pub struct TabManager {
    pub tabs: Vec<TabInstance>,
    pub active_tab_id: String,
    pub split_pane: Option<SplitPane>,
    pub tab_data: HashMap<String, Box<dyn std::any::Any>>, // Store tab-specific data
}

impl TabManager {
    pub fn new(settings: &AppSettings) -> Self {
        let mut tabs = Vec::new();

        // Create initial tabs based on enabled settings
        let enabled_tabs = settings.get_enabled_tabs();
        if !enabled_tabs.is_empty() {
            let first_tab = TabInstance::new(enabled_tabs[0].tab_type.clone());
            tabs.push(first_tab);
        }

        // Always ensure Settings tab exists
        if !tabs.iter().any(|t| t.tab_type == Tab::Settings) {
            tabs.push(TabInstance::new(Tab::Settings));
        }

        let active_tab_id = tabs.first().map(|t| t.id.clone()).unwrap_or_default();

        Self {
            tabs,
            active_tab_id,
            split_pane: None,
            tab_data: HashMap::new(),
        }
    }

    pub fn add_tab(&mut self, tab_type: Tab) -> String {
        let new_tab = TabInstance::new(tab_type);
        let tab_id = new_tab.id.clone();
        self.tabs.push(new_tab);
        self.active_tab_id = tab_id.clone();
        tab_id
    }

    pub fn add_file_tab(&mut self, tab_type: Tab, file_path: String) -> String {
        let new_tab = TabInstance::new_with_file(tab_type, file_path);
        let tab_id = new_tab.id.clone();
        self.tabs.push(new_tab);
        self.active_tab_id = tab_id.clone();
        tab_id
    }

    pub fn close_tab(&mut self, tab_id: &str) -> bool {
        if let Some(pos) = self.tabs.iter().position(|t| t.id == tab_id) {
            let tab = &self.tabs[pos];

            if !tab.can_close {
                return false; // Cannot close this tab
            }

            // Remove from split pane if it's part of one
            if let Some(ref split) = self.split_pane {
                if split.left_tab_id == tab_id || split.right_tab_id == tab_id {
                    self.split_pane = None;
                }
            }

            self.tabs.remove(pos);
            self.tab_data.remove(tab_id);

            // Update active tab if necessary
            if self.active_tab_id == tab_id {
                if let Some(next_tab) = self.tabs.first() {
                    self.active_tab_id = next_tab.id.clone();
                }
            }

            // Ensure at least one tab exists
            if self.tabs.is_empty() {
                let default_tab = TabInstance::new(Tab::Timer);
                self.active_tab_id = default_tab.id.clone();
                self.tabs.push(default_tab);
            }

            true
        } else {
            false
        }
    }

    pub fn get_active_tab(&self) -> Option<&TabInstance> {
        self.tabs.iter().find(|t| t.id == self.active_tab_id)
    }

    pub fn get_active_tab_mut(&mut self) -> Option<&mut TabInstance> {
        self.tabs.iter_mut().find(|t| t.id == self.active_tab_id)
    }

    pub fn get_tab(&self, tab_id: &str) -> Option<&TabInstance> {
        self.tabs.iter().find(|t| t.id == tab_id)
    }

    pub fn get_tab_mut(&mut self, tab_id: &str) -> Option<&mut TabInstance> {
        self.tabs.iter_mut().find(|t| t.id == tab_id)
    }

    pub fn set_active_tab(&mut self, tab_id: &str) {
        if self.tabs.iter().any(|t| t.id == tab_id) {
            self.active_tab_id = tab_id.to_string();
        }
    }

    pub fn create_split(&mut self, direction: SplitDirection) {
        if self.tabs.len() >= 2 && self.split_pane.is_none() {
            let left_tab_id = self.active_tab_id.clone();
            let right_tab_id = self
                .tabs
                .iter()
                .find(|t| t.id != left_tab_id)
                .map(|t| t.id.clone())
                .unwrap_or_else(|| {
                    // Create a new tab for the split
                    self.add_tab(Tab::Markdown)
                });

            self.split_pane = Some(SplitPane {
                left_tab_id,
                right_tab_id,
                direction,
                split_ratio: 0.5,
            });
        }
    }

    pub fn close_split(&mut self) {
        self.split_pane = None;
    }

    pub fn is_split_active(&self) -> bool {
        self.split_pane.is_some()
    }

    pub fn update_split_ratio(&mut self, ratio: f32) {
        if let Some(ref mut split) = self.split_pane {
            split.split_ratio = ratio.clamp(0.1, 0.9);
        }
    }

    pub fn set_tab_modified(&mut self, tab_id: &str, modified: bool) {
        if let Some(tab) = self.get_tab_mut(tab_id) {
            tab.is_modified = modified;
        }
    }

    pub fn set_tab_title(&mut self, tab_id: &str, title: String) {
        if let Some(tab) = self.get_tab_mut(tab_id) {
            tab.title = title;
        }
    }

    pub fn handle_file_drop(&mut self, file_path: String) -> Option<String> {
        // Determine tab type based on file extension
        let extension = std::path::Path::new(&file_path)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase();

        let tab_type = match extension.as_str() {
            "md" | "markdown" | "txt" => Tab::Markdown,
            _ => Tab::Markdown, // Default to markdown for unknown files
        };

        Some(self.add_file_tab(tab_type, file_path))
    }

    pub fn get_available_tab_types(&self, settings: &AppSettings) -> Vec<Tab> {
        settings
            .get_enabled_tabs()
            .iter()
            .map(|config| config.tab_type.clone())
            .collect()
    }
}

