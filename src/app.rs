// app.rs
use copypasta::ClipboardProvider;
use serde::{Deserialize, Serialize};
use std::cell::Cell;
use std::env;
use std::fs;
use std::io::Write;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveDirection {
    Up,
    Down,
}

#[derive(Serialize, Deserialize)]
pub struct App {
    bookmark_path: PathBuf,
    history: Vec<String>,
    queryed_history: Vec<String>,
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
}

impl App {
    pub fn new() -> Self {
        let history = Self::load_history();
        let mut app = Self {
            bookmarks: Vec::new(),
            bookmark_mode: false,
            bookmark_path: Self::get_bookmark_path(),
            queryed_history: history.clone(),
            history,
            selected: 0,
            search_mode: false,
            search_query: String::new(),
            skipped_items: 0,
            size: Cell::new(0),
            show_help: false,
            should_quit: false,
            message: "".to_string(),
        };

        app.load_bookmarks();
        app
    }

    fn detect_shell() -> String {
        env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string())
    }

    fn get_history_path(shell: &str) -> PathBuf {
        let mut path = PathBuf::new();
        path.push(directories::BaseDirs::new().unwrap().home_dir());

        match shell {
            s if s.contains("zsh") => path.push(".zsh_history"),
            s if s.contains("fish") => path.push(".local/share/fish/fish_history"),
            _ => path.push(".bash_history"),
        }

        path
    }

    fn parse_bash_history(content: Vec<u8>) -> Vec<String> {
        String::from_utf8(content)
            .expect("Can't decode")
            .lines()
            .rev()
            .take(1000)
            .map(String::from)
            .collect()
    }

    fn parse_zsh_history(content: Vec<u8>) -> Vec<String> {
        let mut decoded = Vec::new();
        let mut p = 0;

        // Skip metadata
        while p < content.len() && content[p] != 0x83 {
            decoded.push(content[p]);
            p += 1;
        }

        // Process encoded characters
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

        // Error handling for UTF-8 conversion
        let decoded_str = match String::from_utf8(decoded) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Error decoding zsh history: {}", e);
                return Vec::new();
            }
        };

        decoded_str
            .lines()
            .filter_map(|line| line.splitn(2, ';').nth(1))
            .map(|cmd| cmd.trim().to_string())
            .rev()
            .take(1000)
            .collect()
    }

    fn parse_fish_history(content: Vec<u8>) -> Vec<String> {
        String::from_utf8(content)
            .expect("Can't decode")
            .lines()
            .filter_map(|line| line.strip_prefix("- cmd: "))
            .map(|cmd| cmd.trim().to_string())
            .rev()
            .take(1000)
            .collect()
    }

    fn load_history() -> Vec<String> {
        let shell = Self::detect_shell();
        let history_path = Self::get_history_path(&shell);

        match fs::read(&history_path) {
            Ok(content) => {
                if shell.contains("zsh") {
                    Self::parse_zsh_history(content)
                } else if shell.contains("fish") {
                    Self::parse_fish_history(content)
                } else {
                    Self::parse_bash_history(content)
                }
            }
            Err(_) => vec!["No history found".into()],
        }
    }

    pub fn push_query(&mut self, c: char) {
        self.search_query.push(c);
        self.update_filtered_history();
    }

    pub fn pop_query(&mut self) {
        self.search_query.pop();
        self.update_filtered_history();
    }

    pub fn clear_query(&mut self) {
        self.search_query.clear();
        self.update_filtered_history();
    }

    fn update_filtered_history(&mut self) {
        let query = self.search_query.to_lowercase();
        self.queryed_history = self.history
            .iter()
            .filter(|cmd| cmd.to_lowercase().contains(&query))
            .cloned()
            .collect();
        self.selected = self.selected.min(self.queryed_history.len().saturating_sub(1));
    }

    pub fn move_selection(&mut self, direction: MoveDirection) {
        match direction {
            MoveDirection::Up if self.selected > 0 => self.selected -= 1,
            MoveDirection::Down if self.selected < self.queryed_history.len() - 1 => {
                self.selected += 1
            }
            _ => (),
        }

        // Handle scroll position
        if self.selected < self.skipped_items {
            self.skipped_items = self.selected;
        } else if self.selected >= self.skipped_items + self.size.get() {
            self.skipped_items += 1;
        }
    }

    pub fn copy_selected(&mut self) {
        let current_list = self.current_list();
        if current_list.is_empty() || self.selected >= current_list.len() {
            self.message = "No command to copy".to_string();
            return;
        }

        let selected_cmd = current_list[self.selected].clone();

        // Platform-specific clipboard handling
        #[cfg(target_os = "linux")]
        {
            let wayland = env::var("WAYLAND_DISPLAY").is_ok();
            let x11 = env::var("DISPLAY").is_ok();

            if wayland {
                if Command::new("wl-copy")
                    .arg(&selected_cmd)
                    .status()
                    .is_ok()
                {
                    return;
                }
            }

            if x11 {
                if let Ok(mut child) = Command::new("xclip")
                    .args(&["-selection", "clipboard"])
                    .stdin(Stdio::piped())
                    .spawn()
                {
                    let _ = child.stdin.as_mut().unwrap().write_all(selected_cmd.as_bytes());
                    return;
                }
            }
        }

        #[cfg(target_os = "macos")]
        {
            if let Ok(mut child) = Command::new("pbcopy")
                .stdin(Stdio::piped())
                .spawn()
            {
                let _ = child.stdin.as_mut().unwrap().write_all(selected_cmd.as_bytes());
                return;
            }
        }

        #[cfg(target_os = "windows")]
        {
            let _ = Command::new("powershell")
                .args(&[
                    "-Command",
                    &format!("Set-Clipboard -Value '{}'", selected_cmd.replace("'", "''")),
                ])
                .spawn();
        }

        // Universal fallback
        if let Ok(mut ctx) = copypasta::ClipboardContext::new() {
            let _ = ctx.set_contents(selected_cmd);
        }

        self.message = format!("Copied: {}", selected_cmd);
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
        self.message = format!(
            "Switched to {} mode",
            if self.bookmark_mode { "bookmark" } else { "history" }
        );
    }

    pub fn add_bookmark(&mut self) {
        let current_list = self.current_list();
        if self.selected >= current_list.len() {
            return;
        }

        let cmd = current_list[self.selected].trim().to_string();
        if cmd.is_empty() {
            return;
        }

        if self.bookmarks.iter().any(|b| b.trim() == cmd) {
            self.message = format!("Already bookmarked: {}", cmd);
        } else {
            self.bookmarks.push(cmd.clone());
            self.save_bookmarks();
            self.message = format!("Bookmarked: {}", cmd);
        }
    }

    pub fn current_list(&self) -> &Vec<String> {
        if self.bookmark_mode {
            &self.bookmarks
        } else {
            &self.queryed_history
        }
    }

    pub fn get_help_text(&self) -> &'static str {
        HELP_TEXT
    }

    pub fn set_size(&self, size: usize) {
        self.size.set(size);
    }
}
