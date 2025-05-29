use crate::app::Tab;
use crate::settings::AppSettings;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
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
            Tab::Todo => "Todo and Habits".to_string(),
            Tab::Calculator => "Calculator".to_string(),
            Tab::Flashcards => "Flashcards".to_string(),
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SplitDirection {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SplitPane {
    pub left_tab_id: String,
    pub right_tab_id: String,
    pub direction: SplitDirection,
    pub split_ratio: f32, // 0.0 to 1.0, position of the divider
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabManagerState {
    pub tabs: Vec<TabInstance>,
    pub active_tab_id: String,
    pub split_pane: Option<SplitPane>,
    pub last_active_tab_id: Option<String>,
}

impl Default for TabManagerState {
    fn default() -> Self {
        let default_tab = TabInstance::new(Tab::Timer);
        let active_tab_id = default_tab.id.clone();

        Self {
            tabs: vec![default_tab, TabInstance::new(Tab::Settings)],
            active_tab_id,
            split_pane: None,
            last_active_tab_id: None,
        }
    }
}

impl TabManagerState {
    fn get_save_path() -> PathBuf {
        let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("study_timer");
        path.push("tab_manager.json");
        path
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let save_path = Self::get_save_path();

        // Create directory if it doesn't exist
        if let Some(parent) = save_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let json = serde_json::to_string_pretty(self)?;
        fs::write(save_path, json)?;
        Ok(())
    }

    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let save_path = Self::get_save_path();

        if !save_path.exists() {
            return Ok(Self::default());
        }

        let json = fs::read_to_string(save_path)?;
        let mut state: TabManagerState = serde_json::from_str(&json)?;

        // Validate that we have at least one tab and a Settings tab
        if state.tabs.is_empty() {
            state = Self::default();
        } else {
            // Ensure Settings tab exists
            if !state.tabs.iter().any(|t| t.tab_type == Tab::Settings) {
                state.tabs.push(TabInstance::new(Tab::Settings));
            }

            // Validate active tab exists
            if !state.tabs.iter().any(|t| t.id == state.active_tab_id) {
                if let Some(first_tab) = state.tabs.first() {
                    state.active_tab_id = first_tab.id.clone();
                }
            }

            // Validate last active tab exists (if set)
            if let Some(ref last_tab_id) = state.last_active_tab_id {
                if !state.tabs.iter().any(|t| t.id == *last_tab_id) {
                    state.last_active_tab_id = None;
                }
            }
        }

        Ok(state)
    }
}

pub struct TabManager {
    pub tabs: Vec<TabInstance>,
    pub active_tab_id: String,
    pub last_active_tab_id: Option<String>,
    pub split_pane: Option<SplitPane>,
    pub tab_data: HashMap<String, Box<dyn std::any::Any>>, // Store tab-specific data
}

impl TabManager {
    pub fn new(settings: &AppSettings) -> Self {
        // Try to load saved state first
        let state = TabManagerState::load().unwrap_or_else(|_| {
            // If loading fails, create default state based on settings
            let mut tabs = Vec::new();
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

            TabManagerState {
                tabs,
                active_tab_id,
                split_pane: None,
                last_active_tab_id: None,
            }
        });

        Self {
            tabs: state.tabs,
            active_tab_id: state.active_tab_id,
            last_active_tab_id: state.last_active_tab_id,
            split_pane: state.split_pane,
            tab_data: HashMap::new(),
        }
    }

    pub fn save_state(&self) {
        let state = TabManagerState {
            tabs: self.tabs.clone(),
            active_tab_id: self.active_tab_id.clone(),
            split_pane: self.split_pane.clone(),
            last_active_tab_id: self.last_active_tab_id.clone(),
        };

        if let Err(e) = state.save() {
            eprintln!("Failed to save tab manager state: {}", e);
        }
    }

    pub fn add_tab(&mut self, tab_type: Tab) -> String {
        let new_tab = TabInstance::new(tab_type);
        let tab_id = new_tab.id.clone();
        self.tabs.push(new_tab);

        // Update last active tab before changing active tab
        self.last_active_tab_id = Some(self.active_tab_id.clone());
        self.active_tab_id = tab_id.clone();

        self.save_state();
        tab_id
    }

    pub fn add_file_tab(&mut self, tab_type: Tab, file_path: String) -> String {
        let new_tab = TabInstance::new_with_file(tab_type, file_path);
        let tab_id = new_tab.id.clone();
        self.tabs.push(new_tab);

        // Update last active tab before changing active tab
        self.last_active_tab_id = Some(self.active_tab_id.clone());
        self.active_tab_id = tab_id.clone();

        self.save_state();
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
                // Try to go to the next tab, or previous if it was the last tab
                let next_tab_id = if pos < self.tabs.len() {
                    // Go to the tab that's now at the same position (next tab)
                    self.tabs[pos].id.clone()
                } else if !self.tabs.is_empty() {
                    // Was the last tab, go to the new last tab
                    self.tabs[self.tabs.len() - 1].id.clone()
                } else {
                    // No tabs left, will be handled below
                    String::new()
                };

                if !next_tab_id.is_empty() {
                    self.active_tab_id = next_tab_id;
                }
            }

            // Update last active tab if it was the closed tab
            if self.last_active_tab_id.as_ref() == Some(&tab_id.to_string()) {
                self.last_active_tab_id = None;
            }

            // Ensure at least one tab exists
            if self.tabs.is_empty() {
                let default_tab = TabInstance::new(Tab::Timer);
                self.active_tab_id = default_tab.id.clone();
                self.tabs.push(default_tab);
            }

            self.save_state();
            true
        } else {
            false
        }
    }

    pub fn get_active_tab(&self) -> Option<&TabInstance> {
        self.tabs.iter().find(|t| t.id == self.active_tab_id)
    }

    pub fn get_tab(&self, tab_id: &str) -> Option<&TabInstance> {
        self.tabs.iter().find(|t| t.id == tab_id)
    }

    #[allow(dead_code)]
    pub fn get_tab_mut(&mut self, tab_id: &str) -> Option<&mut TabInstance> {
        self.tabs.iter_mut().find(|t| t.id == tab_id)
    }

    pub fn set_active_tab(&mut self, tab_id: &str) {
        if self.tabs.iter().any(|t| t.id == tab_id) && self.active_tab_id != tab_id {
            // Update last active tab before changing active tab
            self.last_active_tab_id = Some(self.active_tab_id.clone());
            self.active_tab_id = tab_id.to_string();
            self.save_state();
        }
    }

    /// Switch to tab by index (0-based), returns true if successful
    #[allow(dead_code)]
    pub fn set_active_tab_by_index(&mut self, index: usize) -> bool {
        if index < self.tabs.len() {
            let new_tab_id = self.tabs[index].id.clone();
            if self.active_tab_id != new_tab_id {
                // Update last active tab before changing active tab
                self.last_active_tab_id = Some(self.active_tab_id.clone());
                self.active_tab_id = new_tab_id;
                self.save_state();
            }
            true
        } else {
            false
        }
    }

    /// Switch to the last active tab
    pub fn switch_to_last_tab(&mut self) -> bool {
        if let Some(last_tab_id) = self.last_active_tab_id.clone() {
            if self.tabs.iter().any(|t| t.id == last_tab_id) && self.active_tab_id != last_tab_id {
                // Swap current and last active tabs
                let current_active = self.active_tab_id.clone();
                self.active_tab_id = last_tab_id;
                self.last_active_tab_id = Some(current_active);
                self.save_state();
                return true;
            }
        }
        false
    }

    /// Get the current active tab index
    #[allow(dead_code)]
    pub fn get_active_tab_index(&self) -> Option<usize> {
        self.tabs.iter().position(|t| t.id == self.active_tab_id)
    }

    /// Navigate to next tab (wraps around)
    #[allow(dead_code)]
    pub fn next_tab(&mut self) {
        if self.tabs.len() <= 1 {
            return;
        }

        let current_index = self.get_active_tab_index().unwrap_or(0);
        let next_index = if current_index + 1 >= self.tabs.len() {
            0
        } else {
            current_index + 1
        };

        self.set_active_tab_by_index(next_index);
    }

    /// Navigate to previous tab (wraps around)
    #[allow(dead_code)]
    pub fn previous_tab(&mut self) {
        if self.tabs.len() <= 1 {
            return;
        }

        let current_index = self.get_active_tab_index().unwrap_or(0);
        let prev_index = if current_index == 0 {
            self.tabs.len() - 1
        } else {
            current_index - 1
        };

        self.set_active_tab_by_index(prev_index);
    }

    pub fn create_split(&mut self, direction: SplitDirection) {
        if self.tabs.len() >= 2 && self.split_pane.is_none() {
            let left_tab_id = self.active_tab_id.clone();

            // Find a suitable tab for the right pane (prefer non-Settings tabs)
            let right_tab_id = self
                .tabs
                .iter()
                .find(|t| t.id != left_tab_id && t.tab_type != Tab::Settings)
                .or_else(|| self.tabs.iter().find(|t| t.id != left_tab_id))
                .map(|t| t.id.clone())
                .unwrap_or_else(|| {
                    // Create a new tab for the split (avoid Settings unless explicitly requested)
                    let active_tab = self.get_active_tab();
                    let new_tab_type = if active_tab.map(|t| &t.tab_type) == Some(&Tab::Settings) {
                        Tab::Settings
                    } else {
                        Tab::Markdown
                    };
                    self.add_tab(new_tab_type)
                });

            self.split_pane = Some(SplitPane {
                left_tab_id,
                right_tab_id,
                direction,
                split_ratio: 0.5,
            });

            self.save_state();
        }
    }

    pub fn close_split(&mut self) {
        self.split_pane = None;
        self.save_state();
    }

    pub fn is_split_active(&self) -> bool {
        self.split_pane.is_some()
    }

    pub fn update_split_ratio(&mut self, ratio: f32) {
        if let Some(ref mut split) = self.split_pane {
            split.split_ratio = ratio.clamp(0.1, 0.9);
            self.save_state();
        }
    }

    #[allow(dead_code)]
    pub fn set_tab_modified(&mut self, tab_id: &str, modified: bool) {
        if let Some(tab) = self.get_tab_mut(tab_id) {
            tab.is_modified = modified;
            self.save_state();
        }
    }

    #[allow(dead_code)]
    pub fn set_tab_title(&mut self, tab_id: &str, title: String) {
        if let Some(tab) = self.get_tab_mut(tab_id) {
            tab.title = title;
            self.save_state();
        }
    }

    #[allow(dead_code)]
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

    #[allow(dead_code)]
    pub fn get_available_tab_types(&self, settings: &AppSettings) -> Vec<Tab> {
        settings
            .get_enabled_tabs()
            .iter()
            .map(|config| config.tab_type.clone())
            .collect()
    }

    pub fn reorder_tab(&mut self, tab_id: &str, new_index: usize) {
        if let Some(old_index) = self.tabs.iter().position(|t| t.id == tab_id) {
            if old_index != new_index && new_index < self.tabs.len() {
                let tab = self.tabs.remove(old_index);
                let insert_index = if new_index > old_index {
                    new_index.saturating_sub(1)
                } else {
                    new_index
                };
                self.tabs.insert(insert_index.min(self.tabs.len()), tab);
                self.save_state();
            }
        }
    }

    pub fn move_tab_to_split(&mut self, tab_id: &str, is_right_pane: bool) -> bool {
        if let Some(ref mut split) = self.split_pane {
            if is_right_pane {
                split.right_tab_id = tab_id.to_string();
            } else {
                split.left_tab_id = tab_id.to_string();
            }
            self.save_state();
            true
        } else {
            false
        }
    }

    pub fn swap_split_tabs(&mut self) {
        if let Some(ref mut split) = self.split_pane {
            std::mem::swap(&mut split.left_tab_id, &mut split.right_tab_id);
            self.save_state();
        }
    }

    pub fn get_split_pane(&self) -> Option<&SplitPane> {
        self.split_pane.as_ref()
    }

    pub fn set_split_active_tab(&mut self, tab_id: &str, is_right_pane: bool) {
        if let Some(ref mut split) = self.split_pane {
            if is_right_pane {
                split.right_tab_id = tab_id.to_string();
            } else {
                split.left_tab_id = tab_id.to_string();
            }
            // Also set as the globally active tab
            self.set_active_tab(tab_id);
        }
    }
}

