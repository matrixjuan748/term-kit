mod app;
mod events;
mod ui;

use anyhow::Result;
use crossterm::{
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::prelude::*;
use std::io::{stdout, Stdout};

fn main() -> Result<()> {
    // panic 时也要恢复终端
    std::panic::set_hook(Box::new(|info| {
        let _ = disable_raw_mode();
        let _ = stdout().execute(LeaveAlternateScreen);
        eprintln!("{info}");
    }));

    let mut terminal = setup_terminal()?;
    let mut app = app::App::new();

    // 修复 #4：先保存结果，恢复终端后再返回
    let result = events::handle_events(&mut terminal, &mut app);

    restore_terminal(&mut terminal)?;
    Ok(result?)
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    Ok(Terminal::new(CrosstermBackend::new(stdout()))?)
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}
