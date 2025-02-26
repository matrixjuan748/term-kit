use std::process::{Command, Stdio};
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
    pub search_mode: bool,
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
            search_mode: false,
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
        if direction == MoveDirection::Up && self.selected > 0 {
            self.selected -= 1;
        } else if direction == MoveDirection::Down && self.selected < self.history.len() - 1 {
            self.selected += 1;
        }
        if self.selected < self.skipped_items {
            self.skipped_items = self.selected;
        } else if self.selected >= self.skipped_items + self.size.get() {
            self.skipped_items += 1;
        }
    }

    pub fn copy_selected(&self) {
        if self.history.is_empty() {
            // eprintln!("⚠️ 没有可复制的历史记录");
            return;
        }

        let selected_cmd = &self.history[self.selected];
        copypasta::ClipboardContext::new().unwrap().set_contents(selected_cmd.clone()).expect("Failed to copy to clipboard");

    
        // let output = Command::new("wl-copy")
        // .arg(selected_cmd)
        // .stdin(Stdio::null())
        // .stdout(Stdio::null())
        // .stderr(Stdio::null())
        // .output();

        // match output {
        //     Ok(_) => println!("✅ 已复制: {}", selected_cmd),
        //     Err(err) => eprintln!("❌ 复制失败: {:?}", err),
        // }
    }

    

    pub fn get_help_text(&self) -> &'static str {
        HELP_TEXT
    }

    pub fn set_size(&self, size: usize) {
        self.size.set(size);
    }

    pub fn get_history(&self) -> Vec<String> {
    self.history.clone().into_iter().filter(|cmd| {
        cmd.contains(&self.search_query)
    }).collect()
}
    
}
