// app.rs
use copypasta::ClipboardProvider;
use std::cell::Cell;
use std::env;
use std::fs;
use std::path::PathBuf;

#[cfg(any(target_os = "linux", target_os = "macos"))]
use std::process::{Command, Stdio};

#[cfg(target_os = "windows")]
use std::process::Command;

const HELP_TEXT: &str = r#"
Navigation:
  Up/Down Arrow  - Move selection
  j/k            - Move selection up/down
  Enter          - Copy selected command
  /              - Start search (in input mode)
  h              - Toggle help
  q              - Quit

Search Mode:
  Type to filter history
  Press ESC to cancel search

Bookmark Mode:
  b - Add current command to bookmarks
  B - Toggle bookmark/history mode
  d - Delete selected bookmark
"#;

const HISTORY_LIMIT: usize = 5000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveDirection {
    Up,
    Down,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Shell {
    PowerShell,
    Zsh,
    Bash,
    Fish,
    Unknown(String),
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct App {
    bookmark_path: PathBuf,
    history: Vec<String>,
    queried_history: Vec<String>,
    pub selected: usize,
    pub search_mode: bool,
    pub search_query: String,
    pub skipped_items: usize,
    pub size: Cell<usize>,
    pub show_help: bool,
    pub should_quit: bool,
    pub message: String,
    pub bookmarks: Vec<String>,
    pub bookmark_mode: bool,
    current_shell: Shell,
}

impl Shell {
    pub fn detect() -> Self {
        #[cfg(target_os = "windows")]
        {
            Shell::PowerShell
        }

        #[cfg(not(target_os = "windows"))]
        {
            let shell_path = env::var("SHELL").unwrap_or_else(|_| "/bin/bash".into());
            let shell_name = shell_path.to_lowercase();

            if shell_name.contains("pwsh") || shell_name.contains("powershell") {
                Shell::PowerShell
            } else if shell_name.contains("zsh") {
                Shell::Zsh
            } else if shell_name.contains("fish") {
                Shell::Fish
            } else if shell_name.contains("bash") {
                Shell::Bash
            } else {
                Shell::Unknown(shell_path)
            }
        }
    }

    pub fn history_path(&self) -> PathBuf {
        let base_dirs = directories::BaseDirs::new().unwrap();
        let mut path = base_dirs.home_dir().to_path_buf();

        match self {
            Shell::PowerShell => {
                #[cfg(target_os = "windows")]
                path.push("AppData\\Roaming\\Microsoft\\Windows\\PowerShell\\PSReadLine\\ConsoleHost_history.txt");
                
                #[cfg(not(target_os = "windows"))]
                path.push(".local/share/powershell/PSReadLine/ConsoleHost_history.txt");
            }
            Shell::Zsh => path.push(".zsh_history"),
            Shell::Bash => path.push(".bash_history"),
            Shell::Fish => path.push(".local/share/fish/fish_history"),
            Shell::Unknown(_) => path.push(".bash_history"),
        }
        path
    }

    pub fn parse_history(&self, content: Vec<u8>) -> Vec<String> {
        match self {
            Shell::PowerShell => Self::parse_powershell(content),
            Shell::Zsh => Self::parse_zsh(content),
            Shell::Bash => Self::parse_bash(content),
            Shell::Fish => Self::parse_fish(content),
            Shell::Unknown(_) => Self::parse_bash(content),
        }
    }

    fn parse_powershell(content: Vec<u8>) -> Vec<String> {
        String::from_utf8(content)
            .expect("Failed to decode PowerShell history")
            .lines()
            .rev()
            .map(|line| line.trim().to_string())
            .filter(|line| !line.is_empty())
            .take(HISTORY_LIMIT)
            .collect()
    }

    fn parse_zsh(content: Vec<u8>) -> Vec<String> {
        let mut decoded = Vec::new();
        let mut p = 0;

        while p < content.len() && content[p] != 0x83 {
            decoded.push(content[p]);
            p += 1;
        }

        while p < content.len() {
            let current_char = content[p];
            if current_char == 0x83 {
                p += 1;
                if p < content.len() {
                    decoded.push(content[p] ^ 32);
                }
            } else {
                decoded.push(current_char);
            }
            p += 1;
        }

        String::from_utf8(decoded)
            .expect("Failed to decode Zsh history")
            .lines()
            .filter_map(|line| line.splitn(2, ':').last())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .rev()
            .take(HISTORY_LIMIT)
            .collect()
    }

    fn parse_bash(content: Vec<u8>) -> Vec<String> {
        String::from_utf8(content)
            .expect("Failed to decode Bash history")
            .lines()
            .filter(|line| !line.starts_with('#'))
            .map(|line| line
                 .trim_start_matches(|c: char| c.is_numeric() || c == '#')
                 .trim()
                 .to_string())
            .rev()
            .take(HISTORY_LIMIT)
            .collect()
    }

    fn parse_fish(content: Vec<u8>) -> Vec<String> {
        String::from_utf8(content)
            .expect("Failed to decode Fish history")
            .lines()
            .filter_map(|line| line.strip_prefix("- cmd: "))
            .flat_map(|cmd| cmd.split("\\n"))
            .map(str::trim)
            .map(String::from)
            .rev()
            .take(HISTORY_LIMIT)
            .collect()
    }
}

impl App {
    pub fn new() -> Self {
        let current_shell = Shell::detect();
        let history = Self::load_history(&current_shell);

        let mut app = Self {
            bookmarks: Vec::new(),
            bookmark_mode: false,
            bookmark_path: Self::get_bookmark_path(),
            queried_history: history.clone(),
            history,
            selected: 0,
            search_mode: false,
            search_query: String::new(),
            skipped_items: 0,
            size: Cell::new(0),
            show_help: false,
            should_quit: false,
            message: String::new(),
            current_shell,
        };

        app.load_bookmarks();
        app
    }

    fn load_history(shell: &Shell) -> Vec<String> {
        let history_path = shell.history_path();

        // Debug output
        println!("Loading history from: {}", history_path.display());
        if let Ok(metadata) = fs::metadata(&history_path) {
            println!("File size: {} bytes", metadata.len());
        } else {
            println!("File not found");
        }

        fs::read(&history_path)
            .map(|content| shell.parse_history(content))
            .unwrap_or_else(|e| vec![format!("Error: {}", e)])
    }

    pub fn search_query(&self) -> &str {
        &self.search_query
    }
    
    pub fn push_query(&mut self, c: char) {
        self.search_query.push(c);
        self.update_queried_history();
    }

    pub fn pop_query(&mut self) {
        self.search_query.pop();
        self.update_queried_history();
    }

    pub fn clear_query(&mut self) {
        self.search_query.clear();
        self.queried_history = self.history.clone();
    }

    fn update_queried_history(&mut self) {
        let query = self.search_query.to_lowercase();
        self.queried_history = self.history
            .iter()
            .filter(|cmd| cmd.to_lowercase().contains(&query))
            .cloned()
            .take(HISTORY_LIMIT)
            .collect();
        self.selected = self.selected.min(self.queried_history.len().saturating_sub(1));
    }

    pub fn move_selection(&mut self, direction: MoveDirection) {
        let max_index = self.current_list().len().saturating_sub(1);

        match direction {
            MoveDirection::Up if self.selected > 0 => self.selected -= 1,
            MoveDirection::Down if self.selected < max_index => self.selected += 1,
            _ => (),
        }

        if self.selected < self.skipped_items {
            self.skipped_items = self.selected;
        } else if self.selected >= self.skipped_items + self.size.get() {
            self.skipped_items += 1;
        }
    }

    pub fn copy_selected(&mut self) {
        let selected_cmd = match self.current_list().get(self.selected) {
            Some(cmd) => cmd,
            None => {
                self.message = "No command to copy".into();
                return;
            }
        };

        #[cfg(target_os = "linux")]
        self.handle_linux_clipboard(selected_cmd);

        #[cfg(target_os = "macos")]
        self.handle_macos_clipboard(selected_cmd);

        #[cfg(target_os = "windows")]
        self.handle_windows_clipboard(selected_cmd);

        self.message = format!("Copied: {}", selected_cmd);
    }

    #[cfg(target_os = "linux")]
    fn handle_linux_clipboard(&self, cmd: &str) {
        use std::io::Write;
        
        let wayland = env::var("WAYLAND_DISPLAY").is_ok();
        let x11 = env::var("DISPLAY").is_ok();

        if wayland {
            let _ = Command::new("wl-copy")
                .arg(cmd)
                .spawn();
        } else if x11 {
            let _ = Command::new("xclip")
                .args(&["-selection", "clipboard"])
                .stdin(Stdio::piped())
                .spawn()
                .and_then(|mut child| {
                    child.stdin.as_mut().unwrap().write_all(cmd.as_bytes())
                });
        }
    }

    #[cfg(target_os = "macos")]
    fn handle_macos_clipboard(&self, cmd: &str) {
        use std::io::Write;
        
        let _ = Command::new("pbcopy")
            .stdin(Stdio::piped())
            .spawn()
            .and_then(|mut child| {
                child.stdin.as_mut().unwrap().write_all(cmd.as_bytes())
            });
    }

    #[cfg(target_os = "windows")]
    fn handle_windows_clipboard(&self, cmd: &str) {
        let _ = Command::new("powershell")
            .args(&[
                "-Command",
                &format!("Set-Clipboard -Value '{}'", cmd.replace("'", "''")),
            ])
            .spawn();
    }

    pub fn current_list(&self) -> &Vec<String> {
        if self.bookmark_mode {
            &self.bookmarks
        } else {
            &self.queried_history
        }
    }

    fn get_bookmark_path() -> PathBuf {
        directories::BaseDirs::new()
            .unwrap()
            .home_dir()
            .join(".term_kit_bookmarks")
    }

    fn load_bookmarks(&mut self) {
        if let Ok(content) = fs::read_to_string(&self.bookmark_path) {
            self.bookmarks = serde_json::from_str(&content).unwrap_or_default();
        }
    }

    fn save_bookmarks(&self) {
        let _ = fs::write(
            &self.bookmark_path,
            serde_json::to_string_pretty(&self.bookmarks).unwrap(),
        );
    }

    pub fn toggle_bookmark_mode(&mut self) {
        self.bookmark_mode = !self.bookmark_mode;
        self.selected = 0;
        self.skipped_items = 0;
        self.queried_history = if self.bookmark_mode {
            self.bookmarks.clone()
        } else {
            self.history.clone()
        };
    }

    pub fn toggle_bookmark(&mut self) {
        if let Some(cmd) = self.current_list().get(self.selected) {
            if let Some(pos) = self.bookmarks.iter().position(|b| b == cmd) {
                self.bookmarks.remove(pos);
                self.message = "Bookmark removed!".to_string();
            } else {
                self.bookmarks.push(cmd.clone());
                self.message = "Bookmark added!".to_string();
            }
            self.save_bookmarks();
        }
    }

    pub fn delete_bookmark(&mut self) {
        if !self.bookmarks.is_empty() {
            self.bookmarks.remove(self.selected);
            self.selected = self.selected.min(self.bookmarks.len().saturating_sub(1));
            self.save_bookmarks();
            self.message = "Bookmark deleted!".to_string();
        }
    }

    pub fn get_help_text(&self) -> &'static str {
        HELP_TEXT
    }

    pub fn set_size(&self, size: usize) {
        self.size.set(size);
    }
}
