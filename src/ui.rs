// ui.rs
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};
use crate::app::App;

pub fn draw_ui(f: &mut Frame, app: &mut App) {
    // Main layout structure
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Min(1),     // Main content
            Constraint::Length(3),  // Search bar
            Constraint::Length(1),  // Status bar
        ])
        .split(f.area());

    // Render header with mode indication
    let header = Paragraph::new(Line::from(vec![
        Span::styled("History Finder ", Style::default().fg(Color::Yellow)),
        Span::styled("v0.1", Style::default().fg(Color::LightBlue)),
        Span::raw(" | Mode: "),
        Span::styled(
            if app.bookmark_mode { "BOOKMARKS" } else { "HISTORY" },
            Style::default().fg(Color::Cyan)
        ),
        Span::raw(" | [B]Toggle | [/]Search | [h]Help | [q]Quit"),
    ]))
    .block(Block::default().borders(Borders::ALL))
    .alignment(Alignment::Center);

    f.render_widget(header, layout[0]);

    // Main content area with dynamic title
    let content_title = if app.bookmark_mode {
        " Bookmarks (Press B to switch) "
    } else {
        " Command History (Press B to switch) "
    };

    let content_block = Block::default()
        .title(content_title)
        .borders(Borders::ALL)
        .style(if app.bookmark_mode {
            Style::default().fg(Color::Yellow) // Yellow border for bookmark mode
        } else {
            Style::default()
        });

    // Prepare list items with bookmark indicators
    let items = app.current_list()
        .iter()
        .enumerate()
        .skip(app.skipped_items)
        .map(|(i, cmd)| {
            // Add star prefix for bookmarks
            let prefix = if app.bookmark_mode {
                Span::styled("* ", Style::default().fg(Color::Yellow))
            } else {
                Span::raw("")
            };

            // Highlight selected item
            let line_style = if i == app.selected {
                Style::default()
                    .bg(Color::Rgb(30, 30, 30))  // Dark background
                    .fg(Color::Cyan)
            } else {
                Style::default()
            };

            Line::from(vec![
                Span::styled(
                    format!("{:3} ", i + 1),
                    Style::default().fg(Color::DarkGray)
                ),
                prefix,
                Span::raw(cmd.as_str())
            ]).style(line_style)
        })
        .collect::<Vec<_>>();

    let inner_area = content_block.inner(layout[1]);
    app.set_size(inner_area.height.into());
    
    f.render_widget(content_block, layout[1]);
    f.render_widget(Paragraph::new(items), inner_area);

    // Search bar implementation
    let search_text = if app.search_mode {
        format!("/{}", app.search_query)
    } else {
        "Press / to start searching".into()
    };

    let search_bar = Paragraph::new(Text::raw(search_text))
        .block(Block::default().title(" Search ").borders(Borders::ALL))
        .alignment(Alignment::Left);

    f.render_widget(search_bar, layout[2]);

    // Status bar with mode-specific actions
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
            format!(" {} ", if app.bookmark_mode { "BOOKMARK" } else { "HISTORY" }),
            Style::default()
                .fg(Color::Black)
                .bg(if app.bookmark_mode { Color::Yellow } else { Color::Blue })
        ),
        Span::raw(" "),
    ];
    status_line.extend(status_actions);
    status_line.push(Span::raw(&app.message));

    f.render_widget(
        Paragraph::new(Line::from(status_line))
            .block(Block::default()),
        layout[3]
    );

    // Help window overlay
    if app.show_help {
        let help_block = Block::default()
            .title(" Help (ESC to close) ")
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::DarkGray));

        let help_text = Text::from(app.get_help_text());
        let help_para = Paragraph::new(help_text)
            .block(help_block)
            .wrap(Wrap { trim: true });

        let help_area = centered_rect(60, 30, f.area());
        f.render_widget(help_para, help_area);
    }
}

/// Helper function to create centered rectangular area
fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical_chunks[1])[1]
}
