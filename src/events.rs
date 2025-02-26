// events.rs
use crossterm::event::{self, Event, KeyCode, KeyEvent};
use ratatui::backend::Backend;
use ratatui::Terminal;
use std::io::Result;
use crate::app::{App, MoveDirection};
use crate::ui::draw_ui;

pub fn handle_events<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> Result<()> {
    loop {
        terminal.draw(|f| draw_ui(f, app))?;
        
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(KeyEvent { code, .. }) = event::read()? {
                match code {
                    KeyCode::Char('q') => app.should_quit = true,
                    KeyCode::Up => app.move_selection(MoveDirection::Up),
                    KeyCode::Char('j') => app.move_selection(MoveDirection::Down),
                    KeyCode::Down => app.move_selection(MoveDirection::Down),
                    KeyCode::Char('k') => app.move_selection(MoveDirection::Up),
                    KeyCode::Enter => app.copy_selected(),
                    KeyCode::Char('i') => app.search_mode = true,
                    KeyCode::Char('h') => app.show_help = !app.show_help,
                    KeyCode::Esc => {
                        app.show_help = false;
                        app.search_mode = false;
                    }
                    _ => {}
                }
            }
        }
        if app.should_quit {
            break;
        }
    }
    Ok(())
}
