// app.rs
use copypasta::ClipboardProvider;
use serde::{Deserialize, Serialize};
use std::cell::Cell;
use std::env;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

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
            message: "".to_string(),
        }
    }

    fn detect_shell() -> String {
        env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string()) // Default to bash
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

        while p < content.len() && content[p] != 0x83 {
            decoded.push(content[p]);
            p += 1;
        }

        // Process the string
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
            .expect("Can't decode")
            .lines()
            .filter_map(|line| line.splitn(2, ';').nth(1)) // Get everything after `;`
            .map(String::from)
            .rev()
            .take(1000)
            .collect()
    }

    fn parse_fish_history(content: Vec<u8>) -> Vec<String> {
        String::from_utf8(content)
            .expect("Can't decode")
            .lines()
            .filter_map(|line| line.strip_prefix("- cmd: ")) // Extract command part
            .map(String::from)
            .rev()
            .take(1000)
            .collect()
    }

    fn load_history() -> Vec<String> {
        let shell = Self::detect_shell();
        let history_path = Self::get_history_path(&shell);

        if let Ok(content) = fs::read(&history_path) {
            if shell.contains("zsh") {
                Self::parse_zsh_history(content)
            } else if shell.contains("fish") {
                Self::parse_fish_history(content)
            } else {
                Self::parse_bash_history(content)
            }
        } else {
            vec!["No history found".into()]
        }
    }

    pub fn get_query(&self) -> String {
        self.search_query.clone()
    }

    pub fn push_query(&mut self, c: char) {
        self.search_query.push(c);
        self.queryed_history = self
            .queryed_history // The new one must be a subset of the old one.
            .clone()
            .into_iter()
            .filter(|cmd| cmd.contains(&self.search_query))
            .collect()
    }

    pub fn pop_query(&mut self) {
        self.search_query.pop();
        self.queryed_history = self
            .history
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
        } else if direction == MoveDirection::Down && self.selected < self.queryed_history.len() - 1
        {
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
            return;
        }

        let selected_cmd = &self.history[self.selected];

        // 跨平台剪贴板支持
        #[cfg(target_os = "linux")]
        {
            // Wayland优先使用wl-copy
            let wayland_success = Command::new("wl-copy")
                .arg(selected_cmd)
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .is_ok();

            if !wayland_success {
                // 回退到X11的xclip
                let _ = Command::new("xclip")
                    .args(&["-selection", "clipboard"])
                    .stdin(Stdio::piped())
                    .spawn()
                    .and_then(|mut child| {
                        child
                            .stdin
                            .as_mut()
                            .unwrap()
                            .write_all(selected_cmd.as_bytes())
                    });
            }
        }

        #[cfg(target_os = "windows")]
        {
            // PowerShell剪贴板支持
            let _ = Command::new("powershell")
                .args(&[
                    "-Command",
                    &format!("Set-Clipboard -Value '{}'", selected_cmd.replace("'", "''")),
                ])
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn();
        }

        #[cfg(target_os = "macos")]
        {
            // macOS使用pbcopy命令
            let _ = Command::new("pbcopy")
                .stdin(Stdio::piped())
                .spawn()
                .and_then(|mut child| {
                    child
                        .stdin
                        .as_mut()
                        .unwrap()
                        .write_all(selected_cmd.as_bytes())
                });
        }

        // 所有平台的备用方案（使用copypasta库）
        let _ = copypasta::ClipboardContext::new()
            .and_then(|mut ctx| ctx.set_contents(selected_cmd.to_owned()));
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
