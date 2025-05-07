use std::collections::VecDeque;
use std::fs::{self, File};
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct TerminalEmulator {
    pub current_directory: PathBuf,
    pub command_history: VecDeque<String>,
    pub output_history: Vec<TerminalEntry>,
    pub current_input: String,
    pub history_index: Option<usize>,
    pub max_history: usize,
    pub fuzzy_results: Vec<PathBuf>,
    pub fuzzy_index: usize,
    pub fuzzy_mode: bool,
    pub fuzzy_query: String,
    pub pager_content: Option<String>,
    pub pager_offset: usize,
}

pub enum TerminalEntryType {
    Command,
    Output,
    Error,
}

pub struct TerminalEntry {
    pub content: String,
    pub entry_type: TerminalEntryType,
}

impl TerminalEmulator {
    pub fn new() -> Self {
        // Default to the "files" directory where notes are stored
        let current_directory = PathBuf::from("files");

        // Create the directory if it doesn't exist
        if !current_directory.exists() {
            let _ = std::fs::create_dir_all(&current_directory);
        }

        let mut terminal = Self {
            current_directory,
            command_history: VecDeque::with_capacity(100),
            output_history: Vec::new(),
            current_input: String::new(),
            history_index: None,
            max_history: 100,
            fuzzy_results: Vec::new(),
            fuzzy_index: 0,
            fuzzy_mode: false,
            fuzzy_query: String::new(),
            pager_content: None,
            pager_offset: 0,
        };

        // Add welcome message
        terminal.output_history.push(TerminalEntry {
            content: "Terminal initialized. Type 'help' for available commands.".to_string(),
            entry_type: TerminalEntryType::Output,
        });

        terminal
    }

    pub fn execute_command(&mut self) {
        if self.current_input.trim().is_empty() {
            return;
        }

        let command = self.current_input.clone();

        // Add command to history
        self.output_history.push(TerminalEntry {
            content: format!("> {}", command),
            entry_type: TerminalEntryType::Command,
        });

        // Add to command history for up/down recall
        self.command_history.push_front(command.clone());
        if self.command_history.len() > self.max_history {
            self.command_history.pop_back();
        }

        // Reset history navigation
        self.history_index = None;

        // Process the command
        let (output, is_error) = self.process_command(&command);

        // Add output to history
        self.output_history.push(TerminalEntry {
            content: output,
            entry_type: if is_error {
                TerminalEntryType::Error
            } else {
                TerminalEntryType::Output
            },
        });

        // Clear current input
        self.current_input.clear();
    }

    pub fn navigate_history(&mut self, up: bool) {
        if self.command_history.is_empty() {
            return;
        }

        if up {
            // Go up in history
            let new_index = match self.history_index {
                None => 0,
                Some(idx) if idx < self.command_history.len() - 1 => idx + 1,
                Some(idx) => idx,
            };

            if let Some(cmd) = self.command_history.get(new_index) {
                self.current_input = cmd.clone();
                self.history_index = Some(new_index);
            }
        } else {
            // Go down in history
            let new_index = match self.history_index {
                None => None,
                Some(0) => None,
                Some(idx) => Some(idx - 1),
            };

            if let Some(idx) = new_index {
                if let Some(cmd) = self.command_history.get(idx) {
                    self.current_input = cmd.clone();
                }
            } else {
                self.current_input.clear();
            }

            self.history_index = new_index;
        }
    }

    pub fn enter_fuzzy_mode(&mut self, query: &str) {
        self.fuzzy_mode = true;
        self.fuzzy_query = query.to_string();
        self.fuzzy_index = 0;
        self.update_fuzzy_results();
    }

    pub fn exit_fuzzy_mode(&mut self) {
        self.fuzzy_mode = false;
        self.fuzzy_results.clear();
    }

    pub fn update_fuzzy_results(&mut self) {
        self.fuzzy_results.clear();

        if self.fuzzy_query.is_empty() {
            return;
        }

        self.fuzzy_index = 0;

        // Search
        fn search_dir(
            dir: &Path,
            query: &str,
            results: &mut Vec<PathBuf>,
            max_results: usize,
        ) -> io::Result<()> {
            if results.len() >= max_results {
                return Ok(());
            }

            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries {
                    if let Ok(entry) = entry {
                        let path = entry.path();
                        let name = path
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_lowercase();

                        if name.contains(&query.to_lowercase()) {
                            results.push(path.clone());
                            if results.len() >= max_results {
                                return Ok(());
                            }
                        }

                        if path.is_dir() {
                            search_dir(&path, query, results, max_results)?;
                        }
                    }
                }
            }

            Ok(())
        }

        let _ = search_dir(
            &self.current_directory,
            &self.fuzzy_query,
            &mut self.fuzzy_results,
            100,
        );
    }

    pub fn select_next_fuzzy_result(&mut self) {
        if !self.fuzzy_results.is_empty() {
            self.fuzzy_index = (self.fuzzy_index + 1) % self.fuzzy_results.len();
        }
    }

    pub fn select_prev_fuzzy_result(&mut self) {
        if !self.fuzzy_results.is_empty() {
            self.fuzzy_index = if self.fuzzy_index == 0 {
                self.fuzzy_results.len() - 1
            } else {
                self.fuzzy_index - 1
            };
        }
    }

    pub fn get_selected_fuzzy_result(&self) -> Option<PathBuf> {
        self.fuzzy_results.get(self.fuzzy_index).cloned()
    }

    pub fn start_pager(&mut self, content: String) {
        self.pager_content = Some(content);
        self.pager_offset = 0;
    }

    pub fn exit_pager(&mut self) {
        self.pager_content = None;
        self.pager_offset = 0;
    }

    pub fn scroll_pager(&mut self, lines: i32, page_height: usize) {
        if let Some(content) = &self.pager_content {
            let line_count = content.lines().count();

            if lines > 0 {
                // Scroll down
                self.pager_offset = self.pager_offset.saturating_add(lines as usize);
                if self.pager_offset > line_count.saturating_sub(page_height) {
                    self.pager_offset = line_count.saturating_sub(page_height);
                }
            } else {
                // Scroll up
                self.pager_offset = self.pager_offset.saturating_sub((-lines) as usize);
            }
        }
    }

    fn process_command(&mut self, command: &str) -> (String, bool) {
        // Split the command into parts, respecting quotes
        let mut parts = Vec::new();
        let mut current_part = String::new();
        let mut in_quotes = false;

        for c in command.trim().chars() {
            match c {
                '"' => in_quotes = !in_quotes,
                ' ' if !in_quotes => {
                    if !current_part.is_empty() {
                        parts.push(current_part);
                        current_part = String::new();
                    }
                }
                _ => current_part.push(c),
            }
        }

        if !current_part.is_empty() {
            parts.push(current_part);
        }

        if parts.is_empty() {
            return ("".to_string(), false);
        }

        // Handle built-in commands
        match parts[0].as_str() {
            "cd" => self.cmd_cd(&parts),
            "pwd" => self.cmd_pwd(),
            "ls" => self.cmd_ls(&parts),
            "mkdir" => self.cmd_mkdir(&parts),
            "touch" => self.cmd_touch(&parts),
            "rm" => self.cmd_rm(&parts),
            "cp" => self.cmd_cp(&parts),
            "mv" => self.cmd_mv(&parts),
            "cat" => self.cmd_cat(&parts),
            "less" | "more" => self.cmd_less(&parts),
            "tree" => self.cmd_tree(&parts),
            "grep" => self.cmd_grep(&parts),
            "fuzzy" => self.cmd_fuzzy(&parts),
            "clear" => self.cmd_clear(),
            "help" => self.cmd_help(),
            "exit" => self.cmd_exit(),
            // Execute system command
            _ => self.execute_system_command(&parts),
        }
    }

    fn cmd_cd(&mut self, parts: &[String]) -> (String, bool) {
        if parts.len() < 2 {
            return ("Usage: cd <directory>".to_string(), true);
        }

        let new_dir = if parts[1].starts_with('/') {
            // Absolute path
            PathBuf::from(&parts[1])
        } else {
            // Relative path
            self.current_directory.join(&parts[1])
        };

        if new_dir.is_dir() {
            self.current_directory = new_dir;
            (
                format!("Changed directory to: {}", self.current_directory.display()),
                false,
            )
        } else {
            (format!("Directory not found: {}", parts[1]), true)
        }
    }

    fn cmd_pwd(&self) -> (String, bool) {
        (format!("{}", self.current_directory.display()), false)
    }

    fn cmd_ls(&self, parts: &[String]) -> (String, bool) {
        let mut show_hidden = false;
        let mut path = self.current_directory.clone();

        // Parse options and path
        for i in 1..parts.len() {
            if parts[i].starts_with('-') {
                if parts[i].contains('a') {
                    show_hidden = true;
                }
            } else {
                path = if parts[i].starts_with('/') {
                    PathBuf::from(&parts[i])
                } else {
                    self.current_directory.join(&parts[i])
                };
            }
        }

        if !path.exists() {
            return (format!("Path not found: {}", path.display()), true);
        }

        match std::fs::read_dir(&path) {
            Ok(entries) => {
                let mut result = String::new();
                let mut files = Vec::new();
                let mut dirs = Vec::new();

                for entry in entries {
                    if let Ok(entry) = entry {
                        let path = entry.path();
                        let name = path.file_name().unwrap().to_string_lossy().to_string();

                        // Skip hidden files unless -a is specified
                        if !show_hidden && name.starts_with('.') {
                            continue;
                        }

                        if path.is_dir() {
                            dirs.push(format!("{}/", name));
                        } else {
                            files.push(name);
                        }
                    }
                }

                // Sort alphabetically
                dirs.sort();
                files.sort();

                // Directories first, then files
                for dir in dirs {
                    result.push_str(&format!("{}\n", dir));
                }

                for file in files {
                    result.push_str(&format!("{}\n", file));
                }

                if result.is_empty() {
                    result = "Directory is empty.".to_string();
                }

                (result, false)
            }
            Err(e) => (format!("Error reading directory: {}", e), true),
        }
    }

    fn cmd_mkdir(&mut self, parts: &[String]) -> (String, bool) {
        if parts.len() < 2 {
            return ("Usage: mkdir <directory>".to_string(), true);
        }

        let dir_path = if parts[1].starts_with('/') {
            PathBuf::from(&parts[1])
        } else {
            self.current_directory.join(&parts[1])
        };

        match fs::create_dir_all(&dir_path) {
            Ok(_) => (format!("Created directory: {}", dir_path.display()), false),
            Err(e) => (format!("Failed to create directory: {}", e), true),
        }
    }

    fn cmd_touch(&mut self, parts: &[String]) -> (String, bool) {
        if parts.len() < 2 {
            return ("Usage: touch <file>".to_string(), true);
        }

        let file_path = if parts[1].starts_with('/') {
            PathBuf::from(&parts[1])
        } else {
            self.current_directory.join(&parts[1])
        };

        match File::create(&file_path) {
            Ok(_) => (format!("Created file: {}", file_path.display()), false),
            Err(e) => (format!("Failed to create file: {}", e), true),
        }
    }

    fn cmd_rm(&mut self, parts: &[String]) -> (String, bool) {
        if parts.len() < 2 {
            return ("Usage: rm [-r] <path>".to_string(), true);
        }

        let mut recursive = false;
        let mut path_index = 1;

        // Check for options
        if parts[1] == "-r" || parts[1] == "-rf" {
            recursive = true;
            path_index = 2;
            if parts.len() < 3 {
                return ("Usage: rm [-r] <path>".to_string(), true);
            }
        }

        let path = if parts[path_index].starts_with('/') {
            PathBuf::from(&parts[path_index])
        } else {
            self.current_directory.join(&parts[path_index])
        };

        if !path.exists() {
            return (format!("Path not found: {}", path.display()), true);
        }

        let result = if path.is_dir() {
            if recursive {
                fs::remove_dir_all(&path)
            } else {
                fs::remove_dir(&path)
            }
        } else {
            fs::remove_file(&path)
        };

        match result {
            Ok(_) => (format!("Removed: {}", path.display()), false),
            Err(e) => (format!("Failed to remove: {}", e), true),
        }
    }

    fn cmd_cp(&mut self, parts: &[String]) -> (String, bool) {
        if parts.len() < 3 {
            return ("Usage: cp [-r] <source> <destination>".to_string(), true);
        }

        let mut recursive = false;
        let mut src_index = 1;
        let mut dst_index = 2;

        // Check for options
        if parts[1] == "-r" {
            recursive = true;
            src_index = 2;
            dst_index = 3;
            if parts.len() < 4 {
                return ("Usage: cp [-r] <source> <destination>".to_string(), true);
            }
        }

        let src = if parts[src_index].starts_with('/') {
            PathBuf::from(&parts[src_index])
        } else {
            self.current_directory.join(&parts[src_index])
        };

        let dst = if parts[dst_index].starts_with('/') {
            PathBuf::from(&parts[dst_index])
        } else {
            self.current_directory.join(&parts[dst_index])
        };

        if !src.exists() {
            return (format!("Source not found: {}", src.display()), true);
        }

        // Function to copy a directory recursively
        fn copy_dir_all(src: &Path, dst: &Path) -> io::Result<()> {
            if !dst.exists() {
                fs::create_dir_all(dst)?;
            }

            for entry in fs::read_dir(src)? {
                let entry = entry?;
                let ty = entry.file_type()?;
                let src_path = entry.path();
                let file_name = src_path.file_name().unwrap();
                let dst_path = dst.join(file_name);

                if ty.is_dir() {
                    copy_dir_all(&src_path, &dst_path)?;
                } else {
                    fs::copy(&src_path, &dst_path)?;
                }
            }

            Ok(())
        }

        let result = if src.is_dir() {
            if recursive {
                copy_dir_all(&src, &dst)
            } else {
                Err(io::Error::new(
                    io::ErrorKind::Other,
                    "Cannot copy directory without -r flag",
                ))
            }
        } else {
            // Create parent directories if they don't exist
            if let Some(parent) = dst.parent() {
                if !parent.exists() {
                    if let Err(e) = fs::create_dir_all(parent) {
                        return (format!("Failed to create parent directories: {}", e), true);
                    }
                }
            }

            fs::copy(&src, &dst).map(|_| ())
        };

        match result {
            Ok(_) => (
                format!("Copied from {} to {}", src.display(), dst.display()),
                false,
            ),
            Err(e) => (format!("Failed to copy: {}", e), true),
        }
    }

    fn cmd_mv(&mut self, parts: &[String]) -> (String, bool) {
        if parts.len() < 3 {
            return ("Usage: mv <source> <destination>".to_string(), true);
        }

        let src = if parts[1].starts_with('/') {
            PathBuf::from(&parts[1])
        } else {
            self.current_directory.join(&parts[1])
        };

        let dst = if parts[2].starts_with('/') {
            PathBuf::from(&parts[2])
        } else {
            self.current_directory.join(&parts[2])
        };

        if !src.exists() {
            return (format!("Source not found: {}", src.display()), true);
        }

        // Create parent directories if they don't exist
        if let Some(parent) = dst.parent() {
            if !parent.exists() {
                if let Err(e) = fs::create_dir_all(parent) {
                    return (format!("Failed to create parent directories: {}", e), true);
                }
            }
        }

        match fs::rename(&src, &dst) {
            Ok(_) => (
                format!("Moved from {} to {}", src.display(), dst.display()),
                false,
            ),
            Err(e) => (format!("Failed to move: {}", e), true),
        }
    }

    fn cmd_cat(&mut self, parts: &[String]) -> (String, bool) {
        if parts.len() < 2 {
            return ("Usage: cat <file>".to_string(), true);
        }

        let file_path = if parts[1].starts_with('/') {
            PathBuf::from(&parts[1])
        } else {
            self.current_directory.join(&parts[1])
        };

        if !file_path.exists() {
            return (format!("File not found: {}", file_path.display()), true);
        }

        if file_path.is_dir() {
            return (format!("{} is a directory", file_path.display()), true);
        }

        match fs::read_to_string(&file_path) {
            Ok(content) => (content, false),
            Err(e) => (format!("Failed to read file: {}", e), true),
        }
    }

    fn cmd_less(&mut self, parts: &[String]) -> (String, bool) {
        if parts.len() < 2 {
            return (format!("Usage: {} <file>", parts[0]), true);
        }

        let file_path = if parts[1].starts_with('/') {
            PathBuf::from(&parts[1])
        } else {
            self.current_directory.join(&parts[1])
        };

        if !file_path.exists() {
            return (format!("File not found: {}", file_path.display()), true);
        }

        if file_path.is_dir() {
            return (format!("{} is a directory", file_path.display()), true);
        }

        match fs::read_to_string(&file_path) {
            Ok(content) => {
                self.start_pager(content);
                (
                    format!(
                        "Viewing file: {} (Press j/k to scroll, q to exit)",
                        file_path.display()
                    ),
                    false,
                )
            }
            Err(e) => (format!("Failed to read file: {}", e), true),
        }
    }

    fn cmd_tree(&mut self, parts: &[String]) -> (String, bool) {
        let path = if parts.len() > 1 {
            if parts[1].starts_with('/') {
                PathBuf::from(&parts[1])
            } else {
                self.current_directory.join(&parts[1])
            }
        } else {
            self.current_directory.clone()
        };

        if !path.exists() {
            return (format!("Path not found: {}", path.display()), true);
        }

        if !path.is_dir() {
            return (format!("{} is not a directory", path.display()), true);
        }

        let mut result = String::new();
        let mut total_dirs = 0;
        let mut total_files = 0;

        fn visit_dir(
            dir: &Path,
            prefix: &str,
            result: &mut String,
            dirs: &mut usize,
            files: &mut usize,
        ) -> io::Result<()> {
            *dirs += 1;

            let entries = fs::read_dir(dir)?
                .collect::<Result<Vec<_>, _>>()?
                .into_iter()
                .filter(|e| {
                    let file_name = e.file_name();
                    let name = file_name.to_string_lossy();
                    !name.starts_with('.')
                })
                .collect::<Vec<_>>();

            for (i, entry) in entries.iter().enumerate() {
                let is_last_entry = i == entries.len() - 1;
                let path = entry.path();
                let name = path.file_name().unwrap().to_string_lossy();

                let branch = if is_last_entry {
                    "└── "
                } else {
                    "├── "
                };
                result.push_str(&format!("{}{}{}\n", prefix, branch, name));

                if path.is_dir() {
                    let new_prefix =
                        format!("{}{}", prefix, if is_last_entry { "    " } else { "│   " });
                    visit_dir(&path, &new_prefix, result, dirs, files)?;
                } else {
                    *files += 1;
                }
            }

            Ok(())
        }

        result.push_str(&format!("{}\n", path.display()));
        if let Err(e) = visit_dir(&path, "", &mut result, &mut total_dirs, &mut total_files) {
            return (format!("Error reading directory: {}", e), true);
        }

        result.push_str(&format!(
            "\n{} directories, {} files",
            total_dirs - 1,
            total_files
        ));

        (result, false)
    }

    fn cmd_grep(&mut self, parts: &[String]) -> (String, bool) {
        if parts.len() < 3 {
            return ("Usage: grep <pattern> <file or path>".to_string(), true);
        }

        let pattern = &parts[1];
        let path = if parts[2].starts_with('/') {
            PathBuf::from(&parts[2])
        } else {
            self.current_directory.join(&parts[2])
        };

        if !path.exists() {
            return (format!("Path not found: {}", path.display()), true);
        }

        let mut result = String::new();

        if path.is_dir() {
            match fs::read_dir(&path) {
                Ok(entries) => {
                    let mut found = false;
                    for entry in entries {
                        if let Ok(entry) = entry {
                            let file_path = entry.path();
                            if file_path.is_file() {
                                match search_in_file(&file_path, pattern) {
                                    Ok(matches) => {
                                        if !matches.is_empty() {
                                            found = true;
                                            result.push_str(&format!(
                                                "File: {}\n",
                                                file_path.display()
                                            ));
                                            for line in matches {
                                                result.push_str(&format!("{}\n", line));
                                            }
                                            result.push_str("\n");
                                        }
                                    }
                                    Err(e) => {
                                        result.push_str(&format!(
                                            "Error searching in {}: {}\n",
                                            file_path.display(),
                                            e
                                        ));
                                    }
                                }
                            }
                        }
                    }

                    if !found {
                        result =
                            format!("No matches found for '{}' in {}", pattern, path.display());
                    }
                }
                Err(e) => return (format!("Error reading directory: {}", e), true),
            }
        } else {
            // Search in a single file
            match search_in_file(&path, pattern) {
                Ok(matches) => {
                    if matches.is_empty() {
                        result =
                            format!("No matches found for '{}' in {}", pattern, path.display());
                    } else {
                        for line in matches {
                            result.push_str(&format!("{}\n", line));
                        }
                    }
                }
                Err(e) => return (format!("Error searching in file: {}", e), true),
            }
        }

        (result, false)
    }

    fn cmd_fuzzy(&mut self, parts: &[String]) -> (String, bool) {
        if parts.len() < 2 {
            return ("Usage: fuzzy <search_term>".to_string(), true);
        }

        self.enter_fuzzy_mode(&parts[1]);

        if self.fuzzy_results.is_empty() {
            self.exit_fuzzy_mode();
            return (format!("No matches found for '{}'", parts[1]), false);
        }

        let mut result = format!(
            "Found {} matches for '{}'\n",
            self.fuzzy_results.len(),
            parts[1]
        );
        result.push_str("Use arrow keys to navigate, Enter to select, Esc to cancel\n\n");

        for (i, path) in self.fuzzy_results.iter().enumerate() {
            let marker = if i == self.fuzzy_index { ">> " } else { "   " };
            result.push_str(&format!("{}{}\n", marker, path.display()));
        }

        (result, false)
    }

    fn cmd_clear(&mut self) -> (String, bool) {
        self.output_history.clear();
        ("".to_string(), false)
    }

    fn cmd_help(&self) -> (String, bool) {
        (
            "Available commands:\n\
            Navigation:\n\
            cd <dir>       - Change directory\n\
            pwd            - Print working directory\n\
            ls [-a] [path] - List directory contents (-a: show hidden files)\n\
            \n\
            File Operations:\n\
            mkdir <dir>    - Create directory\n\
            touch <file>   - Create empty file\n\
            rm [-r] <path> - Remove file or directory (-r: recursive)\n\
            cp [-r] <src> <dst> - Copy file or directory (-r: recursive)\n\
            mv <src> <dst> - Move/rename file or directory\n\
            \n\
            File Viewing:\n\
            cat <file>     - Display file content\n\
            less/more <file> - View file with paging (j/k to scroll, q to exit)\n\
            tree [path]    - Display directory structure as a tree\n\
            grep <pattern> <path> - Search for pattern in file(s)\n\
            \n\
            Utilities:\n\
            fuzzy <term>   - Fuzzy search for files\n\
            clear          - Clear terminal output\n\
            help           - Show this help message\n\
            exit           - (Note: In this environment, use the tab system to exit)\n\
            \n\
            You can also run system commands like 'echo', 'cat', etc."
                .to_string(),
            false,
        )
    }

    fn cmd_exit(&self) -> (String, bool) {
        (
            "Terminal emulator cannot be exited in this environment.".to_string(),
            false,
        )
    }

    fn execute_system_command(&self, parts: &[String]) -> (String, bool) {
        let command = &parts[0];
        let args = &parts[1..];

        // Create command with current directory
        let output = if cfg!(target_os = "windows") {
            Command::new("cmd")
                .args(["/C", command])
                .args(args)
                .current_dir(&self.current_directory)
                .output()
        } else {
            Command::new(command)
                .args(args)
                .current_dir(&self.current_directory)
                .output()
        };

        match output {
            Ok(output) => {
                let mut result = String::new();
                let success = output.status.success();

                if !output.stdout.is_empty() {
                    result.push_str(&String::from_utf8_lossy(&output.stdout));
                }

                if !output.stderr.is_empty() {
                    result.push_str(&String::from_utf8_lossy(&output.stderr));
                }

                if result.is_empty() {
                    result = if success {
                        "Command executed successfully.".to_string()
                    } else {
                        format!("Command failed with exit code: {}", output.status)
                    };
                }

                (result, !success)
            }
            Err(e) => (format!("Failed to execute command: {}", e), true),
        }
    }
}

// Helper function for grep
fn search_in_file(file_path: &Path, pattern: &str) -> io::Result<Vec<String>> {
    let mut file = File::open(file_path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;

    let mut matches = Vec::new();
    for (line_num, line) in content.lines().enumerate() {
        if line.contains(pattern) {
            matches.push(format!("{}: {}", line_num + 1, line));
        }
    }

    Ok(matches)
}

