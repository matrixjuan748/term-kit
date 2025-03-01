use crossterm::event::{self, KeyCode, KeyEvent};
use ratatui::Terminal;
use std::io::Result;
use crate::app::{App, MoveDirection};
use crate::ui::draw_ui;

pub fn handle_events<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> Result<()> {
    loop {
        terminal.draw(|f| draw_ui(f, app))?;  // ✅ 每次事件循环都刷新 UI

        if app.should_quit {
            break;
        }

        if event::poll(std::time::Duration::from_millis(100))? {
            if let event::Event::Key(KeyEvent { code, .. }) = event::read()? {
                app.message = "".to_string();
                match code {
                    KeyCode::Char('h') => app.show_help = true,
                    KeyCode::Char('q') => {
                        app.should_quit = true;
                    }
                    
                    KeyCode::Enter => {
                        app.copy_selected();
                    }

                    KeyCode::Up | KeyCode::Char('k') => app.move_selection(MoveDirection::Up),
                    KeyCode::Down | KeyCode::Char('j') => app.move_selection(MoveDirection::Down),
                    KeyCode::Char('/') => {
                        app.search_mode = true;
                        app.clear_query();
                    }
                    KeyCode::Esc => {
                        app.search_mode = false;
                        app.show_help = false;
                        app.clear_query();
                    }
                    KeyCode::Char(c) if app.search_mode => {
                        app.push_query(c);
                    }
                    KeyCode::Backspace if app.search_mode => {
                        app.pop_query();
                    }
                    _ => {}
                }
            }
        }
    }
    Ok(())
}
