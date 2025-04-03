// app.rs
use copypasta::ClipboardProvider;
use std::cell::Cell;
use std::fs;
use std::path::PathBuf;

#[cfg(not(target_os = "windows"))]
use std::env;

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

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ShellType {
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
    current_shell: ShellType,
}

impl ShellType {
    /// Detect the current shell based on platform and environment
    pub fn detect() -> Self {
        #[cfg(target_os = "windows")]
        {
            // Windows defaults to PowerShell
            ShellType::PowerShell
        }

        #[cfg(not(target_os = "windows"))]
        {
            let shell_path = env::var("SHELL").unwrap_or_else(|_| "/bin/bash".into());
            let shell_name = shell_path.to_lowercase();

            if shell_name.contains("pwsh") || shell_name.contains("powershell") {
                ShellType::PowerShell
            } else if shell_name.contains("zsh") {
                ShellType::Zsh
            } else if shell_name.contains("fish") {
                ShellType::Fish
            } else if shell_name.contains("bash") {
                ShellType::Bash
            } else {
                ShellType::Unknown(shell_path)
            }
        }
    }

    // Get history file path for the shell
    pub fn history_path(&self) -> PathBuf {
        let base_dirs = directories::BaseDirs::new().expect("Failed to determine system directories");
        let mut path = base_dirs.home_dir().to_path_buf();

        match self {
            ShellType::PowerShell => {
                #[cfg(target_os = "windows")]
                path.push("AppData\\Roaming\\Microsoft\\Windows\\PowerShell\\PSReadLine\\ConsoleHost_history.txt");

                #[cfg(not(target_os = "windows"))]
                path.push(".local/share/powershell/PSReadLine/ConsoleHost_history.txt");
            }
            ShellType::Zsh => path.push(".zsh_history"),
            ShellType::Bash => path.push(".bash_history"),
            ShellType::Fish => path.push(".local/share/fish/fish_history"),
            ShellType::Unknown(_) => path.push(".bash_history"), // Fallback
        }
        path
    }

    /// Parse shell-specific history format
    pub fn parse_history(&self, content: Vec<u8>) -> Vec<String> {
        match self {
            ShellType::PowerShell => Self::parse_powershell(content),
            ShellType::Zsh => Self::parse_zsh(content),
            ShellType::Bash => Self::parse_bash(content),
            ShellType::Fish => Self::parse_fish(content),
            ShellType::Unknown(_) => Self::parse_bash(content), // Fallback to bash parsing
        }
    }

    // -- History Parsers -- //

    fn parse_powershell(content: Vec<u8>) -> Vec<String> {
        String::from_utf8(content)
            .expect("Failed to decode PowerShell history")
            .lines()
            .rev()
            .map(|line| line.trim().to_string())
            .filter(|line| !line.is_empty())
            .take(1000)
            .collect()
    }

    fn parse_zsh(content: Vec<u8>) -> Vec<String> {
        let mut decoded = Vec::new();
        let mut p = 0;

        // Handle zsh's metacharacter encoding
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
            .filter_map(|line| line.split_once(';').map(|x| x.1))
            .map(String::from)
            .rev()
            .take(1000)
            .collect()
    }

    fn parse_bash(content: Vec<u8>) -> Vec<String> {
        String::from_utf8(content)
            .expect("Failed to decode Bash history")
            .lines()
            .rev()
            .take(1000)
            .map(String::from)
            .collect()
    }

    fn parse_fish(content: Vec<u8>) -> Vec<String> {
        String::from_utf8(content)
            .expect("Failed to decode Fish history")
            .lines()
            .filter_map(|line| line.strip_prefix("- cmd: "))
            .map(String::from)
            .rev()
            .take(1000)
            .collect()
    }
}

impl App {
    pub fn new() -> Self {
        let current_shell = ShellType::detect();
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

    // -- History -- //
    fn load_history(shell: &ShellType) -> Vec<String> {
        let history_path = shell.history_path();

        fs::read(&history_path)
            .map(|content| shell.parse_history(content))
            .unwrap_or_else(|_| vec!["No history found".into()])
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
        self.queried_history = self
            .history
            .iter()
            .filter(|cmd| cmd.contains(&self.search_query))
            .cloned()
            .collect();
        self.selected = self
            .selected
            .min(self.queried_history.len().saturating_sub(1));
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

    // -- Selection -- //
    pub fn copy_selected(&mut self) {
        let Some(selected_cmd) = self.current_list().get(self.selected) else {
            self.message = "No command to copy".into();
            return;
        };

        // Platform-specific clipboard handling
        #[cfg(target_os = "linux")]
        self.handle_linux_clipboard(selected_cmd);

        #[cfg(target_os = "macos")]
        self.handle_macos_clipboard(selected_cmd);

        #[cfg(target_os = "windows")]
        self.handle_windows_clipboard(selected_cmd);

        // Universal fallback
        let _ = copypasta::ClipboardContext::new()
            .and_then(|mut ctx| ctx.set_contents(selected_cmd.to_owned()));
    }

    #[cfg(target_os = "linux")]
    fn handle_linux_clipboard(&self, cmd: &str) {
        use std::io::Write;
        use std::process::{Command, Stdio};
        use std::env;

        let wayland = env::var("WAYLAND_DISPLAY").is_ok();
        let x11 = env::var("DISPLAY").is_ok();

        if wayland {
            let _ = Command::new("wl-copy")
                .arg(cmd)
                .spawn()
                .map_err(|e| eprintln!("Wayland error: {e}"));
        } else if x11 {
            let _ = Command::new("xclip")
                .args(["-selection", "clipboard"])
                .stdin(Stdio::piped())
                .spawn()
                .and_then(|mut child| {
                    if let Some(stdin) = child.stdin.as_mut() {
                        stdin.write_all(cmd.as_bytes())
                    } else {
                        eprintln!("Failed to get xclip stdin");
                        Ok(())
                    }
                });
        }
    }

    #[cfg(target_os = "macos")]
    fn handle_macos_clipboard(&self, cmd: &str) {
        use std::process::{Command, Stdio};
        use std::io::Write;

        let _ = Command::new("pbcopy")
            .stdin(Stdio::piped())
            .spawn()
            .and_then(|mut child| {
                if let Some(stdin) = child.stdin.as_mut() {
                    stdin.write_all(cmd.as_bytes())
                } else {
                    eprintln!("Failed to get pbcopy stdin");
                    Ok(())
                }
            });
    }

    #[cfg(target_os = "windows")]
    fn handle_windows_clipboard(&self, cmd: &str) {
        use std::process::Command;
        let _ = Command::new("powershell")
            .args([
                "-Command",
                &format!("Set-Clipboard -Value '{}'", cmd.replace("'", "''")),
            ])
            .spawn();
    }


    // -- Bookmarks -- //
    pub fn current_list(&self) -> &Vec<String> {
        if self.bookmark_mode {
            &self.bookmarks
        } else {
            &self.queried_history
        }
    }

    fn get_bookmark_path() -> PathBuf {
        directories::BaseDirs::new()
            .expect("Failed to determine user home directory")
            .home_dir()
            .join(".term_kit_bookmarks")
    }

    fn load_bookmarks(&mut self) {
        if let Ok(content) = fs::read_to_string(&self.bookmark_path) {
            self.bookmarks = serde_json::from_str(&content).unwrap_or_default();
        }
    }

    fn save_bookmarks(&self) {
        match serde_json::to_string_pretty(&self.bookmarks) {
            Ok(data) => {
                if let Err(e) = fs::write(&self.bookmark_path, data) {
                    eprintln!("Failed to save bookmarks: {}", e);
                }
            }
            Err(e) => eprintln!("Failed to serialize bookmarks: {}", e),
        }
    }

    pub fn toggle_bookmark_mode(&mut self) {
        self.bookmark_mode = !self.bookmark_mode;
        self.selected = 0;
        self.skipped_items = 0;
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

            if self.bookmark_mode {
                self.queried_history = self.bookmarks.clone();
            }
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
