use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

const FILES_DIR: &str = "files";

#[derive(PartialEq)]
pub enum EditorMode {
    Edit,
    Preview,
    Split,
}

pub struct MarkdownEditor {
    pub current_file: Option<PathBuf>,
    pub current_content: String,
    pub selected_entry: Option<PathBuf>,
    pub editor_mode: EditorMode,
    pub zoom_level: f32,
    pub new_file_name: String,
    pub new_folder_name: String,
    pub rename_buffer: String,
    pub show_rename_dialog: bool,
    pub file_browser_collapsed: bool,
    // Track selected folder for creating files inside it
    pub selected_folder: Option<PathBuf>,
    // Track expanded folders for the tree view
    pub expanded_folders: Vec<PathBuf>,
}

impl Default for MarkdownEditor {
    fn default() -> Self {
        // Ensure the files directory exists
        if !Path::new(FILES_DIR).exists() {
            let _ = fs::create_dir_all(FILES_DIR);
        }

        Self {
            current_file: None,
            current_content: String::new(),
            selected_entry: None,
            editor_mode: EditorMode::Split,
            zoom_level: 1.0,
            new_file_name: String::new(),
            new_folder_name: String::new(),
            rename_buffer: String::new(),
            show_rename_dialog: false,
            file_browser_collapsed: false,
            selected_folder: None,
            expanded_folders: Vec::new(),
        }
    }
}

impl MarkdownEditor {
    pub fn open_file(&mut self, path: &PathBuf) -> Result<(), std::io::Error> {
        let mut file = File::open(path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;

        self.current_content = content;
        self.current_file = Some(path.clone());
        Ok(())
    }

    pub fn save_file(&mut self) -> Result<(), std::io::Error> {
        if let Some(path) = &self.current_file {
            let mut file = OpenOptions::new()
                .write(true)
                .truncate(true)
                .create(true)
                .open(path)?;

            file.write_all(self.current_content.as_bytes())?;
        }
        Ok(())
    }

    pub fn create_file(&mut self, name: &str) -> Result<PathBuf, std::io::Error> {
        // Determine the directory where the file should be created
        let parent_dir = if let Some(folder) = &self.selected_folder {
            if folder.is_dir() {
                folder.clone()
            } else {
                Path::new(FILES_DIR).to_path_buf()
            }
        } else {
            Path::new(FILES_DIR).to_path_buf()
        };

        // Add .md extension if not present
        let file_name = if !name.ends_with(".md") {
            format!("{}.md", name)
        } else {
            name.to_string()
        };

        let file_path = parent_dir.join(file_name);

        // Create an empty file
        let mut file = File::create(&file_path)?;
        file.write_all(b"")?;

        Ok(file_path)
    }

    pub fn create_folder(&mut self, name: &str) -> Result<PathBuf, std::io::Error> {
        // Determine the parent directory
        let parent_dir = if let Some(folder) = &self.selected_folder {
            if folder.is_dir() {
                folder.clone()
            } else {
                Path::new(FILES_DIR).to_path_buf()
            }
        } else {
            Path::new(FILES_DIR).to_path_buf()
        };

        let folder_path = parent_dir.join(name);
        fs::create_dir_all(&folder_path)?;

        // Auto-expand the newly created folder
        self.expanded_folders.push(folder_path.clone());

        Ok(folder_path)
    }

    pub fn delete_entry(&mut self, path: &Path) -> Result<(), std::io::Error> {
        if path.is_file() {
            fs::remove_file(path)?;
            if self.current_file.as_ref().map_or(false, |p| p == path) {
                self.current_file = None;
                self.current_content.clear();
            }
        } else if path.is_dir() {
            fs::remove_dir_all(path)?;
            if self
                .current_file
                .as_ref()
                .map_or(false, |p| p.starts_with(path))
            {
                self.current_file = None;
                self.current_content.clear();
            }
            // Clear selected folder if we just deleted it
            if self.selected_folder.as_ref().map_or(false, |p| p == path) {
                self.selected_folder = None;
            }

            // Remove from expanded folders list
            self.expanded_folders.retain(|p| !p.starts_with(path));
        }
        Ok(())
    }

    pub fn rename_entry(&mut self, path: &Path, new_name: &str) -> Result<PathBuf, std::io::Error> {
        let parent = path.parent().unwrap_or(Path::new(""));
        let new_path = parent.join(new_name);

        fs::rename(path, &new_path)?;

        // Update current_file if the renamed file was open
        if self.current_file.as_ref().map_or(false, |p| p == path) {
            self.current_file = Some(new_path.clone());
        }

        // Update selected folder if we just renamed it
        if self.selected_folder.as_ref().map_or(false, |p| p == path) {
            self.selected_folder = Some(new_path.clone());
        }

        // Update expanded folders list
        if path.is_dir() {
            if let Some(pos) = self.expanded_folders.iter().position(|p| p == path) {
                self.expanded_folders[pos] = new_path.clone();
            }
        }

        Ok(new_path)
    }

    pub fn is_folder_expanded(&self, path: &Path) -> bool {
        self.expanded_folders.iter().any(|p| p == path)
    }

    pub fn toggle_folder_expansion(&mut self, path: &Path) {
        if let Some(pos) = self.expanded_folders.iter().position(|p| p == path) {
            self.expanded_folders.remove(pos);
        } else if path.is_dir() {
            self.expanded_folders.push(path.to_path_buf());
        }
    }

    // Add markdown formatting to selected text
    pub fn add_formatting(&mut self, format_type: &str) {
        match format_type {
            "bold" => {
                self.current_content.push_str("**Bold Text**");
            }
            "italic" => {
                self.current_content.push_str("*Italic Text*");
            }
            "red" => {
                self.current_content.push_str("<color=red>Red Text</color>");
            }
            "green" => {
                self.current_content
                    .push_str("<color=green>Green Text</color>");
            }
            "blue" => {
                self.current_content
                    .push_str("<color=blue>Blue Text</color>");
            }
            "bold_italic" => {
                self.current_content.push_str("***Bold and Italic***");
            }
            _ => {}
        }
    }
}
