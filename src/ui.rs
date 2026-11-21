// ui.rs
use crate::app::App;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

pub fn draw_ui(frame: &mut Frame, app: &mut App) {
    let [header_area, content_area, search_area, status_area] = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(1),    // Main content
            Constraint::Length(3), // Search bar
            Constraint::Length(1), // Status bar
        ])
        .areas(frame.area());

    draw_header(frame, app, header_area);
    draw_list(frame, app, content_area);
    draw_search_bar(frame, app, search_area);
    draw_status_bar(frame, app, status_area);

    if app.show_help {
        draw_help(frame, app);
    }
}

fn draw_header(frame: &mut Frame, app: &App, area: Rect) {
    let header = Paragraph::new(Line::from(vec![
        Span::styled("History Finder ", Style::default().fg(Color::Yellow)),
        Span::styled(
            format!("v{}", env!("CARGO_PKG_VERSION")),
            Style::default().fg(Color::LightBlue),
        ),
        Span::raw(" | Mode: "),
        Span::styled(
            if app.bookmark_mode {
                "BOOKMARKS"
            } else {
                "HISTORY"
            },
            Style::default().fg(Color::Cyan),
        ),
        Span::raw(" | [B]Toggle | [/]Search | [h]Help | [q]Quit"),
    ]))
    .block(Block::default().borders(Borders::ALL))
    .alignment(Alignment::Center);

    frame.render_widget(header, area);
}

fn draw_list(frame: &mut Frame, app: &mut App, area: Rect) {
    let content_title = if app.bookmark_mode {
        " Bookmarks (Press B to switch) "
    } else {
        " Command History (Press B to switch) "
    };

    let content_block = Block::default()
        .title(content_title)
        .borders(Borders::ALL)
        .style(if app.bookmark_mode {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        });

    let inner_area = content_block.inner(area);
    frame.render_widget(content_block, area);

    app.set_size(inner_area.height as usize);

    let total = app.visible_len();
    if total == 0 {
        let empty = Paragraph::new(if app.bookmark_mode {
            "No bookmarks yet (press b in history mode to add)"
        } else {
            "No history found"
        })
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);

        frame.render_widget(empty, inner_area);
        return;
    }

    let first_row = app.skipped_items;
    let last_row = (first_row + inner_area.height as usize).min(total);

    let items: Vec<Line<'static>> = (first_row..last_row)
        .filter_map(|row| {
            app.visible_item(row).map(|command| {
                command_line(
                    row + 1,
                    command,
                    row == app.selected,
                    app.bookmark_mode,
                    inner_area.width,
                )
            })
        })
        .collect();

    frame.render_widget(Paragraph::new(items), inner_area);
}

fn command_line(
    number: usize,
    command: &str,
    selected: bool,
    bookmark_mode: bool,
    width: u16,
) -> Line<'static> {
    let prefix = if bookmark_mode {
        Span::styled("* ", Style::default().fg(Color::Yellow))
    } else {
        Span::raw("")
    };

    let line_style = if selected {
        Style::default().bg(Color::Rgb(30, 30, 30)).fg(Color::Cyan)
    } else {
        Style::default()
    };

    Line::from(vec![
        Span::styled(format!("{number:3} "), Style::default().fg(Color::DarkGray)),
        prefix,
        Span::raw(display_text(command, width)),
    ])
    .style(line_style)
}

fn display_text(command: &str, width: u16) -> String {
    let limit = (width as usize).saturating_sub(6).max(8);
    let mut text = String::with_capacity(command.len().min(limit));

    for (index, ch) in command.chars().enumerate() {
        if index >= limit {
            break;
        }
        text.push(match ch {
            '\n' | '\t' => ' ',
            ch if ch.is_control() => ' ',
            ch => ch,
        });
    }

    text
}

fn draw_search_bar(frame: &mut Frame, app: &App, area: Rect) {
    let search_text = if app.search_mode {
        format!("/{}", app.search_query())
    } else {
        String::from("Press / to start searching")
    };

    let search_bar = Paragraph::new(Text::raw(search_text))
        .block(Block::default().title(" Search ").borders(Borders::ALL))
        .alignment(Alignment::Left);

    frame.render_widget(search_bar, area);

    if app.search_mode {
        let offset = 1 + app.search_query().chars().count() as u16;
        let x = (area.x + 1 + offset).min(area.x + area.width.saturating_sub(1));
        let y = area.y + 1;
        frame.set_cursor_position((x, y));
    }
}

fn draw_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let status_actions = if app.bookmark_mode {
        vec![
            Span::styled(" B ", Style::default().bg(Color::Yellow).fg(Color::Black)),
            Span::raw("Switch "),
            Span::styled(" d ", Style::default().bg(Color::Red).fg(Color::Black)),
            Span::raw("Delete "),
        ]
    } else {
        vec![
            Span::styled(" B ", Style::default().bg(Color::Blue).fg(Color::Black)),
            Span::raw("Switch "),
            Span::styled(" b ", Style::default().bg(Color::Green).fg(Color::Black)),
            Span::raw("Bookmark "),
        ]
    };

    let mut status_line = vec![
        Span::styled(
            format!(
                " {} ",
                if app.bookmark_mode {
                    "BOOKMARK"
                } else {
                    "HISTORY"
                }
            ),
            Style::default().fg(Color::Black).bg(if app.bookmark_mode {
                Color::Yellow
            } else {
                Color::Blue
            }),
        ),
        Span::raw(" "),
    ];

    status_line.extend(status_actions);
    status_line.push(Span::raw(app.message.clone()));

    frame.render_widget(Paragraph::new(Line::from(status_line)), area);
}

fn draw_help(frame: &mut Frame, app: &App) {
    let text = app.get_help_text();
    let line_count = text.lines().count() as u16;
    let area = popup_rect(frame.area(), 60, line_count + 2);

    frame.render_widget(Clear, area);

    let help_block = Block::default()
        .title(" Help (ESC to close) ")
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::DarkGray));

    let help_para = Paragraph::new(Text::from(text))
        .block(help_block)
        .wrap(Wrap { trim: true });

    frame.render_widget(help_para, area);
}

fn popup_rect(area: Rect, width_percent: u16, height: u16) -> Rect {
    let width = ((area.width as u32 * width_percent as u32 / 100) as u16)
        .min(area.width.saturating_sub(2))
        .max(1);
    let height = height.min(area.height.saturating_sub(2)).max(1);

    Rect::new(
        area.x + area.width.saturating_sub(width) / 2,
        area.y + area.height.saturating_sub(height) / 2,
        width,
        height,
    )
}
