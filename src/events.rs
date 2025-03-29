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
