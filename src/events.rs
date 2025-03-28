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
        terminal.draw(|f| draw_ui(f, app))?;  

        if app.should_quit {
            break;
        }

        if event::poll(std::time::Duration::from_millis(100))? {
            if let event::Event::Key(KeyEvent { code, .. }) = event::read()? {
                match code {
                    KeyCode::Char('h') => app.show_help = true,
                    KeyCode::Char('q') => {
                        app.should_quit = true;
                    }
                    
                    KeyCode::Enter => {
                        app.copy_selected();
                    }

                    KeyCode::Char('b') if !app.search_mode => {
                        if app.bookmark_mode {
                            app.delete_bookmark();
                        } else {
                            app.toggle_bookmark();
                        }
                    }

                    KeyCode::Char('B') if !app.search_mode => {
                        app.toggle_bookmark_mode();
                        app.message = if app.bookmark_mode { 
                            "Switched to bookmark mode".to_string()
                        } else {
                            "Switched to history mode".to_string()
                        };
                    }

                    KeyCode::Char('d') if app.bookmark_mode && !app.search_mode => {
                        app.delete_bookmark();
                    }
                    
                    KeyCode::Up | KeyCode::Char('k') => app.move_selection(MoveDirection::Up),
                    KeyCode::Down | KeyCode::Char('j') => app.move_selection(MoveDirection::Down),

                    KeyCode::Char('/') => {
                        app.search_mode = true;
                        app.clear_query();  // Use method clear_query
                    }

                    KeyCode::Esc => {
                        if app.search_mode {
                            app.search_mode = false;
                            app.clear_query();  // Use method clear_query
                        } else if app.show_help {
                            app.show_help = false;
                        } else if app.bookmark_mode {
                            app.toggle_bookmark_mode(); // Exit Bookmark mode
                        }
                    }

                    KeyCode::Char(c) if app.search_mode => {
                        app.push_query(c);  // Use method push_query
                    }

                    KeyCode::Backspace if app.search_mode => {
                        app.pop_query();  // Use method pop_query
                    }

                    _ => {}
                }
            }
        }
    }
    Ok(())
}
