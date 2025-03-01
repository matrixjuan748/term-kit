// app.rs 
use copypasta::ClipboardProvider;
use serde::{Deserialize, Serialize};
use std::cell::Cell;
use std::fs;
use std::path::PathBuf;

#[cfg(all(
    unix,
    not(any(
        target_os = "macos",
        target_os = "android",
        target_os = "ios",
        target_os = "emscripten"
    ))
))]
use wl_clipboard_rs::copy::{MimeType, Options, Source};

const HELP_TEXT: &str = r#"
Navigation:
  Up/Down Arrow  - Move selection
  j/k            - Move selection up/down
  Enter          - Copy selected command
  i              - Enter search input mode
  /              - Start search (in input mode)
  h              - Toggle help
  q              - Quit

Search Mode:
  Type to filter history
  Press ESC to cancel search
"#;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveDirection {
    Up,
    Down,
}

#[derive(Serialize, Deserialize)]
pub struct App {
    history: Vec<String>,
    queryed_history: Vec<String>,
    pub selected: usize,
    pub search_mode: bool,
    search_query: String,
    pub skipped_items: usize,
    pub size: Cell<usize>,
    pub show_help: bool,
    pub should_quit: bool,
    pub message: String,
}

impl App {
    pub fn new() -> Self {
        let history = Self::load_history();
        Self {
            queryed_history: history.clone(),
            history,
            selected: 0,
            search_mode: false,
            search_query: String::new(),
            skipped_items: 0,
            size: Cell::new(0),
            show_help: false,
            should_quit: false,
            message: "".to_string()
        }
    }

    fn load_history() -> Vec<String> {
        let mut history_path = PathBuf::new();
        history_path.push(directories::BaseDirs::new().unwrap().home_dir());
        history_path.push(".bash_history");

        if let Ok(content) = fs::read_to_string(&history_path) {
            content.lines().rev().take(1000).map(String::from).collect()
        } else {
            vec!["No history found".into()]
        }
    }

    pub fn get_query(&self) -> String {
        self.search_query.clone()
    }

    pub fn push_query(&mut self, c: char) {
        self.search_query.push(c);
        self.queryed_history = self.queryed_history // The new one must be a subset of the old one.
            .clone()
            .into_iter()
            .filter(|cmd| cmd.contains(&self.search_query))
            .collect()
    }

    pub fn pop_query(&mut self) {
        self.search_query.pop();
        self.queryed_history = self.history
            .clone()
            .into_iter()
            .filter(|cmd| cmd.contains(&self.search_query))
            .collect()
    }

    pub fn clear_query(&mut self) {
        self.search_query.clear();
        self.queryed_history = self.history.clone();
    }

    pub fn move_selection(&mut self, direction: MoveDirection) {
        if direction == MoveDirection::Up && self.selected > 0 {
            self.selected -= 1;
        } else if direction == MoveDirection::Down && self.selected < self.queryed_history.len() - 1 {
            self.selected += 1;
        }
        if self.selected < self.skipped_items {
            self.skipped_items = self.selected;
        } else if self.selected >= self.skipped_items + self.size.get() {
            self.skipped_items += 1;
        }
    }

    pub fn copy_selected(&mut self) {
        if self.queryed_history.is_empty() {
            self.message = "没有可复制的历史记录".to_string();
            return;
        }

        let selected_cmd = &self.queryed_history[self.selected];
        let output;
        #[cfg(all(
            unix,
            not(any(
                target_os = "macos",
                target_os = "android",
                target_os = "ios",
                target_os = "emscripten"
            ))
        ))]
        if let Ok(_) = std::env::var("WAYLAND_DISPLAY") {
            let opts = Options::new();
            output = opts.copy(Source::Bytes(selected_cmd.clone().into_bytes().into()), MimeType::Autodetect)
                .map_err(|e|e.into());
        } else {
            output = copypasta::ClipboardContext::new()
                .unwrap()
                .set_contents(selected_cmd.clone());
        }

        #[cfg(not(all(
            unix,
            not(any(
                target_os = "macos",
                target_os = "android",
                target_os = "ios",
                target_os = "emscripten"
            ))
        )))]
        let output = copypasta::ClipboardContext::new()
            .unwrap()
            .set_contents(selected_cmd.clone());

        match output {
            Ok(_) => self.message = format!("已复制: {}", selected_cmd),
            Err(err) => self.message = format!("复制失败: {:?}", err),
        }
    }

    pub fn get_help_text(&self) -> &'static str {
        HELP_TEXT
    }

    pub fn set_size(&self, size: usize) {
        self.size.set(size);
    }

    pub fn get_history(&self) -> Vec<String> {
        self.queryed_history.clone()
    }
}
