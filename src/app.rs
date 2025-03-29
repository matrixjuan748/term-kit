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
            message: String::new(),
        };
        app.load_bookmarks();
        app
    }

    fn detect_shell() -> String {
        env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string())
    }

    fn get_history_path(shell: &str) -> PathBuf {
        let mut path = directories::BaseDirs::new().unwrap().home_dir().to_path_buf();
        match shell {
            s if s.contains("zsh") => path.push(".zsh_history"),
            s if s.contains("fish") => path.push(".local/share/fish/fish_history"),
            _ => path.push(".bash_history"),
        }
        path
    }

    fn parse_bash_history(content: Vec<u8>) -> Vec<String> {
        String::from_utf8(content)
            .unwrap_or_default()
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

        while p < content.len() {
            if content[p] == 0x83 {
                p += 1;
                if p < content.len() {
                    decoded.push(content[p] ^ 32);
                }
            } else {
                decoded.push(content[p]);
            }
            p += 1;
        }

        String::from_utf8(decoded)
            .unwrap_or_default()
            .lines()
            .filter_map(|line| line.splitn(2, ';').nth(1))
            .map(|s| s.trim().to_string())
            .rev()
            .take(1000)
            .collect()
    }

    fn parse_fish_history(content: Vec<u8>) -> Vec<String> {
        String::from_utf8(content)
            .unwrap_or_default()
            .lines()
            .filter_map(|line| line.strip_prefix("- cmd: "))
            .map(|s| s.trim().to_string())
            .rev()
            .take(1000)
            .collect()
    }

    fn load_history() -> Vec<String> {
        let shell = Self::detect_shell();
        let history_path = Self::get_history_path(&shell);

        fs::read(&history_path)
            .map(|content| match {
                _ if shell.contains("zsh") => Self::parse_zsh_history(content),
                _ if shell.contains("fish") => Self::parse_fish_history(content),
                _ => Self::parse_bash_history(content),
            })
            .unwrap_or_else(|_| vec!["No history found".into()])
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
        let len = self.current_list().len();
        if len == 0 {
            return;
        }

        match direction {
            MoveDirection::Up if self.selected > 0 => self.selected -= 1,
            MoveDirection::Down if self.selected < len - 1 => self.selected += 1,
            _ => {}
        }

        let view_height = self.size.get();
        if view_height == 0 {
            return;
        }

        if self.selected < self.skipped_items {
            self.skipped_items = self.selected;
        } else if self.selected >= self.skipped_items + view_height {
            self.skipped_items = self.selected - view_height + 1;
        }
    }

    pub fn copy_selected(&mut self) {
        let current_list = self.current_list();
        if current_list.is_empty() || self.selected >= current_list.len() {
            self.message = "No command to copy".to_string();
            return;
        }

        let selected_cmd = current_list[self.selected].clone();
        let mut success = false;

        // Platform-specific implementations
        #[cfg(target_os = "linux")]
        {
            let wayland = env::var("WAYLAND_DISPLAY").is_ok();
            let x11 = env::var("DISPLAY").is_ok();

            if wayland {
                success = Command::new("wl-copy")
                    .arg(&selected_cmd)
                    .status()
                    .map(|s| s.success())
                    .unwrap_or(false);
            }
            if !success && x11 {
                success = Command::new("xclip")
                    .args(&["-selection", "clipboard"])
                    .stdin(Stdio::piped())
                    .spawn()
                    .and_then(|mut child| {
                        child.stdin.as_mut().unwrap().write_all(selected_cmd.as_bytes())
                    })
                    .is_ok();
            }
        }

        #[cfg(target_os = "macos")]
        {
            success = Command::new("pbcopy")
                .stdin(Stdio::piped())
                .spawn()
                .and_then(|mut child| {
                    child.stdin.as_mut().unwrap().write_all(selected_cmd.as_bytes())
                })
                .is_ok();
        }

        #[cfg(target_os = "windows")]
        {
            success = Command::new("powershell")
                .args(&[
                    "-Command",
                    &format!("Set-Clipboard -Value '{}'", selected_cmd.replace("'", "''")),
                ])
                .status()
                .map(|s| s.success())
                .unwrap_or(false);
        }

        // Fallback to clipboard crate
        if !success {
            let _ = ClipboardProvider::new()
                .and_then(|mut ctx| ctx.set_contents(selected_cmd.clone()));
        }

        self.message = format!("Copied: {}", selected_cmd);
    }

    // ... bookmark相关方法保持不变 ...
}

// events.rs
use crossterm::event::{self, KeyCode, KeyEvent};
use ratatui::backend::Backend;
use ratatui::Terminal;
use std::io::Result;

pub fn handle_events<B: Backend>(terminal: &mut Terminal<B>, app: &mut crate::app::App) -> Result<()> {
    loop {
        terminal.draw(|f| crate::ui::draw_ui(f, app))?;

        if app.should_quit {
            break;
        }

        if event::poll(std::time::Duration::from_millis(100))? {
            if let event::Event::Key(KeyEvent { code, .. }) = event::read()? {
                match code {
                    KeyCode::Char('h') => app.show_help = !app.show_help,
                    KeyCode::Char('q') => app.should_quit = true,
                    KeyCode::Enter => app.copy_selected(),
                    KeyCode::Char('b') if !app.search_mode => app.add_bookmark(),
                    KeyCode::Char('B') if !app.search_mode => app.toggle_bookmark_mode(),
                    KeyCode::Up | KeyCode::Char('k') => app.move_selection(MoveDirection::Up),
                    KeyCode::Down | KeyCode::Char('j') => app.move_selection(MoveDirection::Down),
                    KeyCode::Char('/') => {
                        app.search_mode = true;
                        app.clear_query();
                    }
                    KeyCode::Esc => handle_escape(app),
                    KeyCode::Char(c) if app.search_mode => app.push_query(c),
                    KeyCode::Backspace if app.search_mode => app.pop_query(),
                    _ => {}
                }
            }
        }
    }
    Ok(())
}

fn handle_escape(app: &mut crate::app::App) {
    if app.search_mode {
        app.search_mode = false;
        app.clear_query();
    } else if app.show_help {
        app.show_help = false;
    } else if app.bookmark_mode {
        app.toggle_bookmark_mode();
    }
}
