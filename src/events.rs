// events.rs
use crate::app::{App, MoveDirection};
use crate::ui::draw_ui;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{backend::Backend, Terminal};
use std::io::Result;

pub fn handle_events<B>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()>
where
    B: Backend,
    std::io::Error: From<B::Error>,
{
    draw(terminal, app)?;

    while !app.should_quit {
        let redraw = match event::read()? {
            Event::Key(key) if key.kind == KeyEventKind::Press => {
                handle_key(app, key);
                true
            }
            Event::Resize(_, _) => true,
            _ => false,
        };

        if redraw && !app.should_quit {
            draw(terminal, app)?;
        }
    }

    Ok(())
}

fn draw<B>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()>
where
    B: Backend,
    std::io::Error: From<B::Error>,
{
    terminal.draw(|frame| draw_ui(frame, app))?;
    Ok(())
}

fn handle_key(app: &mut App, key: KeyEvent) {
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        if matches!(key.code, KeyCode::Char('c') | KeyCode::Char('C')) {
            app.should_quit = true;
        }
        return;
    }

    if app.show_help {
        if matches!(
            key.code,
            KeyCode::Esc | KeyCode::Char('h') | KeyCode::Char('q')
        ) {
            app.show_help = false;
        }
        return;
    }

    if app.search_mode {
        match key.code {
            KeyCode::Esc => app.end_search(),
            KeyCode::Enter => {
                app.copy_selected();
                app.end_search();
            }
            KeyCode::Backspace => app.pop_query(),
            KeyCode::Char(ch) => app.push_query(ch),
            KeyCode::Up => app.move_selection(MoveDirection::Up),
            KeyCode::Down => app.move_selection(MoveDirection::Down),
            KeyCode::PageUp => app.move_page(MoveDirection::Up),
            KeyCode::PageDown => app.move_page(MoveDirection::Down),
            KeyCode::Home => app.go_home(),
            KeyCode::End => app.go_end(),
            _ => {}
        }
        return;
    }

    match key.code {
        KeyCode::Char('q') => app.should_quit = true,
        KeyCode::Char('h') | KeyCode::F(1) => app.show_help = true,
        KeyCode::Enter => app.copy_selected(),

        KeyCode::Char('b') => app.toggle_bookmark(),
        KeyCode::Char('B') => app.toggle_bookmark_mode(),
        KeyCode::Char('d') if app.bookmark_mode => app.delete_bookmark(),

        KeyCode::Up | KeyCode::Char('k') => app.move_selection(MoveDirection::Up),
        KeyCode::Down | KeyCode::Char('j') => app.move_selection(MoveDirection::Down),
        KeyCode::PageUp => app.move_page(MoveDirection::Up),
        KeyCode::PageDown => app.move_page(MoveDirection::Down),
        KeyCode::Home => app.go_home(),
        KeyCode::End => app.go_end(),

        KeyCode::Char('/') => app.begin_search(),
        KeyCode::Esc if app.bookmark_mode => app.toggle_bookmark_mode(),
        _ => {}
    }
}
