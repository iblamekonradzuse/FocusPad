use crate::app::{StatusMessage, Tab};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct DroppedFile {
    pub path: PathBuf,
    pub tab_type: Tab,
}

pub struct FileDropHandler {
    pub dropped_files: Vec<DroppedFile>,
}

impl FileDropHandler {
    pub fn new() -> Self {
        Self {
            dropped_files: Vec::new(),
        }
    }

    pub fn handle_dropped_files(
        &mut self,
        ctx: &eframe::egui::Context,
        status: &mut StatusMessage,
    ) -> Vec<DroppedFile> {
        let mut processed_files = Vec::new();

        ctx.input(|i| {
            if !i.raw.dropped_files.is_empty() {
                for file in &i.raw.dropped_files {
                    if let Some(path) = &file.path {
                        match self.determine_tab_type(path) {
                            Some(tab_type) => {
                                let dropped_file = DroppedFile {
                                    path: path.clone(),
                                    tab_type,
                                };
                                processed_files.push(dropped_file);
                                status.show(&format!("File opened: {}", path.display()));
                            }
                            None => {
                                status.show(&format!(
                                    "Unsupported file type: {}",
                                    path.extension()
                                        .and_then(|ext| ext.to_str())
                                        .unwrap_or("unknown")
                                ));
                            }
                        }
                    }
                }
            }
        });

        processed_files
    }

    fn determine_tab_type(&self, path: &PathBuf) -> Option<Tab> {
        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase();

        match extension.as_str() {
            "md" | "markdown" => Some(Tab::Markdown),
            "txt" | "text" => Some(Tab::Markdown),
            "json" => Some(Tab::Markdown),
            "rs" | "rust" => Some(Tab::Markdown),
            "py" | "python" => Some(Tab::Markdown),
            "js" | "javascript" => Some(Tab::Markdown),
            "ts" | "typescript" => Some(Tab::Markdown),
            "ts" | "typescript" => Some(Tab::Markdown),
            "html" | "htm" => Some(Tab::Markdown),
            "css" => Some(Tab::Markdown),
            "xml" => Some(Tab::Markdown),
            "yaml" | "yml" => Some(Tab::Markdown),
            "toml" => Some(Tab::Markdown),
            "ini" | "cfg" | "conf" => Some(Tab::Markdown),
            "log" => Some(Tab::Markdown),
            _ => None, // Unsupported file type
        }
    }

    pub fn is_supported_file(&self, path: &PathBuf) -> bool {
        self.determine_tab_type(path).is_some()
    }

    pub fn get_supported_extensions(&self) -> Vec<&'static str> {
        vec![
            "md",
            "markdown",
            "txt",
            "text",
            "json",
            "rs",
            "rust",
            "py",
            "python",
            "js",
            "javascript",
            "ts",
            "typescript",
            "html",
            "htm",
            "css",
            "xml",
            "yaml",
            "yml",
            "toml",
            "ini",
            "cfg",
            "conf",
            "log",
        ]
    }
}
