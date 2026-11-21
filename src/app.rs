// app.rs
use copypasta::ClipboardProvider;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;

const MAX_HISTORY: usize = 5000;

const PREVIEW_CHARS: usize = 60;

pub const HELP_TEXT: &str = r#"
Navigation
  Up/Down, j/k   Move selection
  PgUp/PgDn      Move one screen
  Home/End       Jump to first / last
  Enter          Copy selected command
  b              Toggle bookmark
  B              Switch history / bookmark view
  d              Delete bookmark (bookmark view)
  h, F1          Show this help
  Esc            Close help / leave bookmark view
  q, Ctrl+C      Quit

Search
  Type to filter (case-insensitive)
  Backspace      Delete one character
  Enter          Copy and leave search
  Esc            Leave search, cursor stays on
                 the same command

Bookmarks live in ~/.term_kit_bookmarks
"#;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveDirection {
    Up,
    Down,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShellType {
    PowerShell,
    Zsh,
    Bash,
    Fish,
}

impl ShellType {
    const ALL: [ShellType; 4] = [
        ShellType::PowerShell,
        ShellType::Zsh,
        ShellType::Bash,
        ShellType::Fish,
    ];

    pub fn label(self) -> &'static str {
        match self {
            ShellType::PowerShell => "PowerShell",
            ShellType::Zsh => "Zsh",
            ShellType::Bash => "Bash",
            ShellType::Fish => "Fish",
        }
    }

    fn from_shell_path(path: &str) -> Option<Self> {
        let path = path.to_ascii_lowercase();
        if path.contains("pwsh") || path.contains("powershell") {
            Some(ShellType::PowerShell)
        } else if path.contains("zsh") {
            Some(ShellType::Zsh)
        } else if path.contains("fish") {
            Some(ShellType::Fish)
        } else if path.contains("bash")
            || path.ends_with("/sh")
            || path.ends_with("\\sh")
            || path.ends_with("sh.exe")
        {
            Some(ShellType::Bash)
        } else {
            None
        }
    }

    pub fn detect() -> Self {
        if let Some(shell) = env::var_os("SHELL")
            .map(|value| value.to_string_lossy().into_owned())
            .and_then(|value| Self::from_shell_path(&value))
        {
            return shell;
        }
        #[cfg(target_os = "windows")]
        {
            ShellType::PowerShell
        }

        #[cfg(not(target_os = "windows"))]
        {
            ShellType::Bash
        }
    }

    pub fn history_path(self) -> Option<PathBuf> {
        let bases = directories::BaseDirs::new()?;
        let home = bases.home_dir();

        let path = match self {
            ShellType::Bash => home.join(".bash_history"),
            ShellType::Zsh => home.join(".zsh_history"),
            ShellType::Fish => home.join(".local/share/fish/fish_history"),
            ShellType::PowerShell => {
                #[cfg(target_os = "windows")]
                {
                    bases
                        .data_dir()
                        .join("Microsoft")
                        .join("Windows")
                        .join("PowerShell")
                        .join("PSReadLine")
                        .join("ConsoleHost_history.txt")
                }

                #[cfg(not(target_os = "windows"))]
                {
                    home.join(".local/share/powershell/PSReadLine/ConsoleHost_history.txt")
                }
            }
        };

        Some(path)
    }

    pub fn parse_history(self, content: Vec<u8>) -> Vec<String> {
        let mut parsed = match self {
            ShellType::PowerShell => Self::parse_powershell(&content),
            ShellType::Zsh => Self::parse_zsh(&content),
            ShellType::Bash => Self::parse_bash(&content),
            ShellType::Fish => Self::parse_fish(&content),
        };
        parsed.truncate(MAX_HISTORY);
        parsed
    }

    // -- History Parsers -- //

    fn parse_powershell(content: &[u8]) -> Vec<String> {
        String::from_utf8_lossy(content)
            .lines()
            .rev()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(String::from)
            .collect()
    }

    fn parse_zsh(content: &[u8]) -> Vec<String> {
        String::from_utf8_lossy(content)
            .lines()
            .filter_map(|line| {
                let command = if line.starts_with(':') {
                    match line.split_once(';') {
                        Some((_, command)) => command,
                        None => line,
                    }
                } else {
                    line
                };

                let command = command.trim();
                if command.is_empty() {
                    None
                } else {
                    Some(command.to_string())
                }
            })
            .rev()
            .collect()
    }

    fn parse_bash(content: &[u8]) -> Vec<String> {
        String::from_utf8_lossy(content)
            .lines()
            .rev()
            .filter(|line| !line.trim().is_empty())
            .filter(|line| !is_timestamp_marker(line))
            .map(String::from)
            .collect()
    }

    fn parse_fish(content: &[u8]) -> Vec<String> {
        String::from_utf8_lossy(content)
            .lines()
            .filter_map(|line| line.strip_prefix("- cmd: "))
            .map(unescape_fish)
            .filter(|command| !command.trim().is_empty())
            .rev()
            .collect()
    }
}

fn is_timestamp_marker(line: &str) -> bool {
    match line.strip_prefix('#') {
        Some(rest) => !rest.is_empty() && rest.bytes().all(|byte| byte.is_ascii_digit()),
        None => false,
    }
}

fn unescape_fish(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut chars = line.chars();

    while let Some(ch) = chars.next() {
        if ch != '\\' {
            out.push(ch);
            continue;
        }
        match chars.next() {
            Some('n') => out.push('\n'),
            Some('\\') => out.push('\\'),
            Some(other) => {
                out.push('\\');
                out.push(other);
            }
            None => out.push('\\'),
        }
    }

    out
}

fn filter_indices(list: &[String], query: &str) -> Vec<usize> {
    if query.is_empty() {
        return (0..list.len()).collect();
    }

    let needle = query.to_lowercase();
    list.iter()
        .enumerate()
        .filter(|(_, command)| command.to_lowercase().contains(&needle))
        .map(|(index, _)| index)
        .collect()
}

pub struct App {
    bookmark_path: Option<PathBuf>,
    bookmark_candidates: Vec<PathBuf>,
    history: Vec<String>,
    bookmarks: Vec<String>,
    queried: Vec<usize>,
    pub selected: usize,
    pub skipped_items: usize,
    pub search_mode: bool,
    search_query: String,
    pub show_help: bool,
    pub should_quit: bool,
    pub message: String,
    pub bookmark_mode: bool,
    size: usize,
    shell: ShellType,
}

impl App {
    pub fn new() -> Self {
        let preferred = ShellType::detect();
        let (history, shell) = Self::load_history(preferred);

        let mut app = Self {
            bookmark_path: None,
            bookmark_candidates: Self::bookmark_candidates(),
            history,
            bookmarks: Vec::new(),
            queried: Vec::new(),
            selected: 0,
            skipped_items: 0,
            search_mode: false,
            search_query: String::new(),
            show_help: false,
            should_quit: false,
            message: String::new(),
            bookmark_mode: false,
            size: 0,
            shell,
        };

        app.load_bookmarks();
        app.refresh_list(true);

        app.message = if app.history.is_empty() {
            format!("No {} history file found", app.shell.label())
        } else if shell != preferred {
            format!(
                "{} commands read from {} history",
                app.history.len(),
                app.shell.label()
            )
        } else {
            String::new()
        };

        app
    }

    fn load_history(preferred: ShellType) -> (Vec<String>, ShellType) {
        if let Some((_, parsed)) = read_history(preferred) {
            if !parsed.is_empty() {
                return (parsed, preferred);
            }
        }

        let mut newest: Option<(SystemTime, ShellType, Vec<String>)> = None;
        for shell in ShellType::ALL {
            if shell == preferred {
                continue;
            }
            let Some((modified, parsed)) = read_history(shell) else {
                continue;
            };
            if parsed.is_empty() {
                continue;
            }
            let is_newer = newest.as_ref().is_none_or(|(best, _, _)| modified > *best);
            if is_newer {
                newest = Some((modified, shell, parsed));
            }
        }

        match newest {
            Some((_, shell, parsed)) => (parsed, shell),
            None => (Vec::new(), preferred),
        }
    }

    fn bookmark_candidates() -> Vec<PathBuf> {
        let mut candidates = Vec::new();

        if let Some(bases) = directories::BaseDirs::new() {
            candidates.push(bases.home_dir().join(".term_kit_bookmarks"));
            candidates.push(
                bases
                    .data_local_dir()
                    .join("term-kit")
                    .join("bookmarks.json"),
            );
        }
        candidates.push(env::temp_dir().join("term-kit-bookmarks.json"));

        candidates
    }

    fn source_list(&self) -> &[String] {
        if self.bookmark_mode {
            &self.bookmarks
        } else {
            &self.history
        }
    }

    pub fn visible_len(&self) -> usize {
        self.queried.len()
    }

    pub fn visible_item(&self, row: usize) -> Option<&str> {
        let index = *self.queried.get(row)?;
        self.source_list().get(index).map(String::as_str)
    }

    pub fn current_command(&self) -> Option<&str> {
        self.visible_item(self.selected)
    }

    pub fn search_query(&self) -> &str {
        &self.search_query
    }

    fn refresh_list(&mut self, reset_selection: bool) {
        let queried = filter_indices(self.source_list(), &self.search_query);
        self.queried = queried;

        if self.queried.is_empty() {
            self.selected = 0;
            self.skipped_items = 0;
            return;
        }

        if reset_selection {
            self.selected = 0;
            self.skipped_items = 0;
        } else {
            self.selected = self.selected.min(self.queried.len() - 1);
        }
        self.ensure_visible();
    }

    fn ensure_visible(&mut self) {
        let visible = self.size.max(1);
        let total = self.queried.len();

        if total == 0 {
            self.skipped_items = 0;
            return;
        }

        if self.selected < self.skipped_items {
            self.skipped_items = self.selected;
        }
        if self.selected >= self.skipped_items + visible {
            self.skipped_items = (self.selected + 1).saturating_sub(visible);
        }

        let max_skip = total.saturating_sub(visible);
        self.skipped_items = self.skipped_items.min(max_skip).min(self.selected);
    }

    // -- Search -- //

    pub fn begin_search(&mut self) {
        self.search_mode = true;
        self.search_query.clear();
        self.refresh_list(false);
    }

    pub fn end_search(&mut self) {
        let keep = self.queried.get(self.selected).copied();
        self.search_mode = false;
        self.search_query.clear();
        self.refresh_list(false);

        if let Some(index) = keep {
            if let Some(position) = self.queried.iter().position(|&item| item == index) {
                self.selected = position;
            }
        }
        self.ensure_visible();
    }

    pub fn push_query(&mut self, ch: char) {
        if ch.is_control() {
            return;
        }
        self.search_query.push(ch);
        self.refresh_list(true);
    }

    pub fn pop_query(&mut self) {
        if self.search_query.pop().is_some() {
            self.refresh_list(true);
        }
    }

    // -- Selection -- //

    pub fn move_selection(&mut self, direction: MoveDirection) {
        let Some(last) = self.queried.len().checked_sub(1) else {
            return;
        };

        self.selected = match direction {
            MoveDirection::Up => self.selected.saturating_sub(1),
            MoveDirection::Down => (self.selected + 1).min(last),
        };
        self.ensure_visible();
    }

    pub fn move_page(&mut self, direction: MoveDirection) {
        let Some(last) = self.queried.len().checked_sub(1) else {
            return;
        };

        let page = self.size.max(1);
        self.selected = match direction {
            MoveDirection::Up => self.selected.saturating_sub(page),
            MoveDirection::Down => (self.selected + page).min(last),
        };
        self.ensure_visible();
    }

    pub fn go_home(&mut self) {
        self.selected = 0;
        self.skipped_items = 0;
    }

    pub fn go_end(&mut self) {
        let Some(last) = self.queried.len().checked_sub(1) else {
            return;
        };
        self.selected = last;
        self.ensure_visible();
    }

    // -- Clipboard -- //

    pub fn copy_selected(&mut self) {
        let Some(command) = self.current_command().map(str::to_string) else {
            self.message = "Nothing to copy".into();
            return;
        };

        let copied = copypasta::ClipboardContext::new()
            .and_then(|mut ctx| ctx.set_contents(command.clone()))
            .is_ok()
            || self.copy_via_platform(&command);

        self.message = if copied {
            format!("Copied: {}", preview(&command))
        } else {
            "Failed to copy: no clipboard backend available".into()
        };
    }

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
                }
                wrote && child.wait().map(|status| status.success()).unwrap_or(false)
            }
            Err(_) => false,
        }
    }

    #[cfg(target_os = "linux")]
    fn copy_via_platform(&self, cmd: &str) -> bool {
        if env::var_os("WAYLAND_DISPLAY").is_some() {
            Self::pipe_to_child(cmd, "wl-copy", &[])
        } else if env::var_os("DISPLAY").is_some() {
            Self::pipe_to_child(cmd, "xclip", &["-selection", "clipboard"])
        } else {
            false
        }
    }

    #[cfg(target_os = "macos")]
    fn copy_via_platform(&self, cmd: &str) -> bool {
        Self::pipe_to_child(cmd, "pbcopy", &[])
    }

    #[cfg(target_os = "windows")]
    fn copy_via_platform(&self, cmd: &str) -> bool {
        if !cmd.is_ascii() {
            return false;
        }
        Self::pipe_to_child(cmd, "clip", &[])
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    fn copy_via_platform(&self, _cmd: &str) -> bool {
        false
    }

    // -- Bookmarks -- //

    fn load_bookmarks(&mut self) {
        for path in self.bookmark_storage_order() {
            let Ok(text) = fs::read_to_string(&path) else {
                continue;
            };

            match serde_json::from_str::<Vec<String>>(&text) {
                Ok(bookmarks) => {
                    self.bookmarks = bookmarks;
                    self.bookmark_path = Some(path);
                }
                Err(_) => {
                    let _ = fs::rename(&path, path.with_extension("corrupt"));
                    self.bookmark_path = Some(path);
                    self.message = "Bookmarks file was corrupt — backed up".to_string();
                }
            }
            return;
        }
    }

    fn bookmark_storage_order(&self) -> Vec<PathBuf> {
        let mut order = Vec::new();
        if let Some(current) = &self.bookmark_path {
            order.push(current.clone());
        }
        for candidate in &self.bookmark_candidates {
            if !order.contains(candidate) {
                order.push(candidate.clone());
            }
        }
        order
    }

    fn save_bookmarks(&mut self) -> bool {
        let Ok(data) = serde_json::to_string_pretty(&self.bookmarks) else {
            return false;
        };

        for path in self.bookmark_storage_order() {
            if let Some(parent) = path.parent() {
                let _ = fs::create_dir_all(parent);
            }

            let temp = path.with_extension("tmp");
            let renamed = fs::write(&temp, &data).is_ok() && fs::rename(&temp, &path).is_ok();
            if !renamed {
                let _ = fs::remove_file(&temp);
            }

            if renamed || fs::write(&path, &data).is_ok() {
                self.bookmark_path = Some(path);
                return true;
            }
        }

        false
    }

    fn persist_bookmarks(&mut self, action: &str) {
        self.save_bookmarks();
        self.message = action.to_string();
    }

    pub fn toggle_bookmark_mode(&mut self) {
        self.bookmark_mode = !self.bookmark_mode;
        self.refresh_list(true);

        let count = self.queried.len();
        self.message = if self.bookmark_mode {
            format!("Bookmark view ({count} items)")
        } else {
            format!("History view ({count} items)")
        };
    }

    pub fn toggle_bookmark(&mut self) {
        let Some(command) = self.current_command().map(str::to_string) else {
            return;
        };

        let existing = self.bookmarks.iter().position(|item| item == &command);
        let message = match existing {
            Some(position) => {
                self.bookmarks.remove(position);
                "Bookmark removed"
            }
            None => {
                self.bookmarks.push(command);
                "Bookmark added"
            }
        };

        self.persist_bookmarks(message);

        if self.bookmark_mode {
            self.refresh_list(false);
        }
    }

    pub fn delete_bookmark(&mut self) {
        if !self.bookmark_mode {
            return;
        }

        let Some(index) = self.queried.get(self.selected).copied() else {
            return;
        };
        if index >= self.bookmarks.len() {
            return;
        }

        self.bookmarks.remove(index);
        self.persist_bookmarks("Bookmark deleted");
        self.refresh_list(false);
    }

    pub fn get_help_text(&self) -> &'static str {
        HELP_TEXT
    }

    pub fn set_size(&mut self, size: usize) {
        self.size = size;
        if size > 0 {
            self.ensure_visible();
        }
    }
}

fn read_history(shell: ShellType) -> Option<(SystemTime, Vec<String>)> {
    let path = shell.history_path()?;
    let metadata = fs::metadata(&path).ok()?;
    if !metadata.is_file() {
        return None;
    }

    let modified = metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH);
    let content = fs::read(&path).ok()?;
    Some((modified, shell.parse_history(content)))
}

fn preview(command: &str) -> String {
    let mut preview: String = command.chars().take(PREVIEW_CHARS).collect();
    if command.chars().count() > PREVIEW_CHARS {
        preview.push('…');
    }
    preview
}
