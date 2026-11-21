// main.rs
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
    install_panic_hook();

    let mut terminal = setup_terminal()?;
    let mut app = app::App::new();

    let run_result = events::handle_events(&mut terminal, &mut app);
    let restore_result = restore_terminal(&mut terminal);

    run_result?;
    restore_result?;
    Ok(())
}

fn install_panic_hook() {
    std::panic::set_hook(Box::new(|info| {
        let _ = disable_raw_mode();
        let _ = stdout().execute(LeaveAlternateScreen);
        eprintln!("term-kit crashed: {info}");
    }));
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>> {
    enable_raw_mode()?;

    let mut out = stdout();
    if let Err(err) = out.execute(EnterAlternateScreen) {
        let _ = disable_raw_mode();
        return Err(err.into());
    }

    let mut terminal = Terminal::new(CrosstermBackend::new(out))?;
    terminal.clear()?;
    Ok(terminal)
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
    let _ = disable_raw_mode();
    stdout().execute(LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}
