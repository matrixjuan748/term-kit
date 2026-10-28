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
  /              - Start search
  h              - Toggle help
  q              - Quit

Search Mode:
  Type to filter history
  ESC            - Cancel search

Bookmark Mode:
  b - Toggle bookmark on selected command
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

pub struct App {
    bookmark_path: PathBuf,
    history: Vec<String>,
    queried: Vec<String>, // 统一：当前模式 + 搜索过滤后的列表
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

impl ShellType {
    pub fn detect() -> Self {
        #[cfg(target_os = "windows")]
        {
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
            ShellType::Unknown(_) => path.push(".bash_history"),
        }
        path
    }

    pub fn parse_history(&self, content: Vec<u8>) -> Vec<String> {
        match self {
            ShellType::PowerShell => Self::parse_powershell(content),
            ShellType::Zsh => Self::parse_zsh(content),
            ShellType::Bash => Self::parse_bash(content),
            ShellType::Fish => Self::parse_fish(content),
            ShellType::Unknown(_) => Self::parse_bash(content),
        }
    }

    // -- History Parsers（全部 lossy，不再 panic）-- //

    fn parse_powershell(content: Vec<u8>) -> Vec<String> {
        String::from_utf8_lossy(&content)
            .lines()
            .rev()
            .map(|line| line.trim().to_string())
            .filter(|line| !line.is_empty())
            .take(1000)
            .collect()
    }

    // 修复 #11：没有 ';' 时保留整行（兼容未开 EXTENDED_HISTORY 的 zsh）
    fn parse_zsh(content: Vec<u8>) -> Vec<String> {
        String::from_utf8_lossy(&content)
            .lines()
            .map(|line| line.split_once(';').map(|(_, cmd)| cmd).unwrap_or(line))
            .filter(|cmd| !cmd.is_empty())
            .rev()
            .take(1000)
            .map(String::from)
            .collect()
    }

    // 修复 #1：lossy 替代 expect
    fn parse_bash(content: Vec<u8>) -> Vec<String> {
        String::from_utf8_lossy(&content)
            .lines()
            .rev()
            .filter(|line| !line.is_empty())
            .take(1000)
            .map(String::from)
            .collect()
    }

    fn parse_fish(content: Vec<u8>) -> Vec<String> {
        String::from_utf8_lossy(&content)
            .lines()
            .filter_map(|line| line.strip_prefix("- cmd: "))
            .rev()
            .take(1000)
            .map(String::from)
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
            queried: history.clone(),
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
        app.refresh_list();
        app
    }

    // 修复 #9：读不到历史返回空列表，不再塞占位符
    fn load_history(shell: &ShellType) -> Vec<String> {
        let history_path = shell.history_path();
        fs::read(&history_path)
            .map(|content| shell.parse_history(content))
            .unwrap_or_default()
    }

    fn source_list(&self) -> &Vec<String> {
        if self.bookmark_mode { &self.bookmarks } else { &self.history }
    }

    // 修复 #5：搜索对 history 和 bookmarks 统一生效
    fn refresh_list(&mut self) {
        self.queried = if self.search_query.is_empty() {
            self.source_list().clone()
        } else {
            self.source_list()
                .iter()
                .filter(|cmd| cmd.contains(&self.search_query))
                .cloned()
                .collect()
        };
        self.selected = self.selected.min(self.queried.len().saturating_sub(1));
        self.skipped_items = self.skipped_items.min(self.selected);
    }

    pub fn current_list(&self) -> &Vec<String> {
        &self.queried
    }

    pub fn search_query(&self) -> &str {
        &self.search_query
    }

    pub fn push_query(&mut self, c: char) {
        if c.is_ascii_graphic() || c == ' ' {
            self.search_query.push(c);
            self.refresh_list();
        }
    }

    pub fn pop_query(&mut self) {
        if self.search_query.pop().is_some() {
            self.refresh_list();
        }
    }

    pub fn clear_query(&mut self) {
        self.search_query.clear();
        self.selected = 0;
        self.skipped_items = 0;
        self.refresh_list();
    }

    pub fn move_selection(&mut self, direction: MoveDirection) {
        let max_index = self.current_list().len().saturating_sub(1);

        match direction {
            MoveDirection::Up if self.selected > 0 => self.selected -= 1,
            MoveDirection::Down if self.selected < max_index => self.selected += 1,
            _ => (),
        }

        let size = self.size.get();
        if size == 0 {
            return;
        }

        if self.selected < self.skipped_items {
            self.skipped_items = self.selected;
        } else if self.selected >= self.skipped_items + size {
            self.skipped_items = self.selected.saturating_sub(size - 1);
        }
    }

    // -- Clipboard（修复 #2 #6）-- //

    pub fn copy_selected(&mut self) {
        let Some(selected_cmd) = self.current_list().get(self.selected).cloned() else {
            self.message = "No command to copy".into();
            return;
        };

        // 先平台工具，失败才 fallback 到 copypasta，不再重复复制
        let copied = self.copy_via_platform(&selected_cmd)
            || copypasta::ClipboardContext::new()
                .and_then(|mut ctx| ctx.set_contents(selected_cmd.clone()))
                .is_ok();

        self.message = if copied {
            format!("Copied: {selected_cmd}")
        } else {
            "Failed to copy: no clipboard backend available".into()
        };
    }

    // 修复 #2：stdin 必须 take + drop 发送 EOF，否则子进程挂起、wait 死锁
    fn pipe_to_child(cmd: &str, program: &str, extra_args: &[&str]) -> bool {
        use std::io::Write;
        use std::process::{Command, Stdio};

        match Command::new(program)
            .args(extra_args)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
        {
            Ok(mut child) => {
                let mut wrote = true;
                if let Some(mut stdin) = child.stdin.take() {
                    wrote = stdin.write_all(cmd.as_bytes()).is_ok();
                } // ← stdin 在这里 drop，子进程收到 EOF
                wrote && child.wait().map(|s| s.success()).unwrap_or(false)
            }
            Err(_) => false,
        }
    }

    #[cfg(target_os = "linux")]
    fn copy_via_platform(&self, cmd: &str) -> bool {
        if env::var("WAYLAND_DISPLAY").is_ok() {
            Self::pipe_to_child(cmd, "wl-copy", &[])
        } else if env::var("DISPLAY").is_ok() {
            Self::pipe_to_child(cmd, "xclip", &["-selection", "clipboard"])
        } else {
            false
        }
    }

    #[cfg(target_os = "macos")]
    fn copy_via_platform(&self, cmd: &str) -> bool {
        Self::pipe_to_child(cmd, "pbcopy", &[])
    }

    // 修复：用 clip.exe 走 stdin，替代 PowerShell 拼字符串（无注入风险、更快）
    #[cfg(target_os = "windows")]
    fn copy_via_platform(&self, cmd: &str) -> bool {
        Self::pipe_to_child(cmd, "clip", &[])
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    fn copy_via_platform(&self, _cmd: &str) -> bool {
        false
    }

    // -- Bookmarks -- //

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
        self.refresh_list();
    }

    pub fn toggle_bookmark(&mut self) {
        let Some(cmd) = self.current_list().get(self.selected).cloned() else {
            return;
        };

        if let Some(pos) = self.bookmarks.iter().position(|b| b == &cmd) {
            self.bookmarks.remove(pos);
            self.message = "Bookmark removed!".to_string();
        } else {
            self.bookmarks.push(cmd);
            self.message = "Bookmark added!".to_string();
        }
        self.save_bookmarks();

        if self.bookmark_mode {
            self.refresh_list();
        }
    }

    pub fn delete_bookmark(&mut self) {
        if self.selected < self.bookmarks.len() {
            self.bookmarks.remove(self.selected);
            self.save_bookmarks();
            self.refresh_list();
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
