// events.rs
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
                    // 基础导航
                    KeyCode::Up | KeyCode::Char('k') => {
                        if app.selected > 0 {
                            app.move_selection(MoveDirection::Up)
                        }
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        if app.selected < app.current_list().len().saturating_sub(1) {
                            app.move_selection(MoveDirection::Down)
                        }
                    }

                    // 核心功能
                    KeyCode::Enter => {
                        app.copy_selected();
                    }
                    KeyCode::Char('q') => {
                        app.should_quit = true;
                    }

                    // 搜索功能
                    KeyCode::Char('/') => {
                        app.search_mode = true;
                        app.clear_query();
                    }
                    KeyCode::Char(c) if app.search_mode => {
                        app.push_query(c);
                    }
                    KeyCode::Backspace if app.search_mode => {
                        app.pop_query();
                    }

                    // 帮助系统
                    KeyCode::Char('h') => {
                        app.show_help = !app.show_help;
                    }

                    // 书签功能
                    KeyCode::Char('b') if !app.search_mode => {
                        app.add_bookmark();
                    }
                    KeyCode::Char('B') if !app.search_mode => {
                        app.toggle_bookmark_mode();
                    }
                    KeyCode::Char('d') if app.bookmark_mode => {
                        if !app.bookmarks.is_empty() {
                            app.bookmarks.remove(app.selected);
                            app.save_bookmarks();
                            app.selected = app.selected.min(app.bookmarks.len().saturating_sub(1));
                        }
                    }

                    // 退出模式
                    KeyCode::Esc => {
                        app.search_mode = false;
                        app.show_help = false;
                        if app.bookmark_mode {
                            app.toggle_bookmark_mode();
                        }
                    }

                    _ => {}
                }

                // 自动清除消息
                if !matches!(code, KeyCode::Char(_)) || app.message.len() > 30 {
                    app.message.clear();
                }
            }
        }
    }
    Ok(())
}
