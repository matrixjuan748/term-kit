use copypasta::ClipboardProvider;
// app.rs
use serde::{Deserialize, Serialize};
use std::cell::Cell;
use std::fs;
use std::path::PathBuf;

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

#[derive(Debug, Serialize, Deserialize)]
pub struct App {
    pub history: Vec<String>,
    pub selected: usize,
    pub input_mode: bool,
    pub search_query: String,
    pub skipped_items: usize,
    pub size: Cell<usize>,
    pub show_help: bool,
    pub should_quit: bool,
}

impl App {
    pub fn new() -> Self {
        let history = Self::load_history();
        Self {
            history,
            selected: 0,
            input_mode: false,
            search_query: String::new(),
            skipped_items: 0,
            size: Cell::new(0),
            show_help: false,
            should_quit: false,
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

    pub fn move_selection(&mut self, direction: MoveDirection) {
        let len = self.history.len();
        if len == 0 { return; }

        match direction {
            MoveDirection::Up if self.selected > 0 => self.selected -= 1,
            MoveDirection::Down if self.selected < len - 1 => self.selected += 1,
            _ => {}
        }

        if self.selected < self.skipped_items {
            self.skipped_items = self.selected;
        } else if self.selected >= self.skipped_items + self.size.get() {
            self.skipped_items += 1;
        }
    }

     pub fn copy_selected(&self) {
        copypasta::ClipboardContext::new()
           .unwrap()
           .set_contents(self.history[self.selected].clone()).unwrap();
     }

    pub fn get_help_text(&self) -> &'static str {
        HELP_TEXT
    }
}
