// ui.rs
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};
use crate::app::App;

pub fn draw_ui(f: &mut Frame, app: &mut App) {
    // Main layout structure
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(1),    // Main content
            Constraint::Length(3), // Search bar
            Constraint::Length(1), // Status bar
        ])
        .split(f.area());

    // Render header
    let header = Paragraph::new(Line::from(vec![
        Span::styled("History Finder ", Style::default().fg(Color::Yellow)),
        Span::styled("v0.1", Style::default().fg(Color::LightBlue)),
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

    f.render_widget(header, main_layout[0]);

    // Main content area
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

    // Prepare list items
    let items = app
        .current_list()
        .iter()
        .enumerate()
        .skip(app.skipped_items)
        .map(|(i, cmd)| {
            let prefix = if app.bookmark_mode {
                Span::styled("* ", Style::default().fg(Color::Yellow))
            } else {
                Span::raw("")
            };

            let line_style = if i == app.selected {
                Style::default()
                    .bg(Color::Rgb(30, 30, 30)).fg(Color::Cyan)
            } else {
                Style::default()
            };

            Line::from(vec![
                Span::styled(
                    format!("{:3} ", i + 1),
                    Style::default().fg(Color::DarkGray),
                ),
                prefix,
                Span::raw(cmd.as_str())
            ]).style(line_style)
        })
        .collect::<Vec<_>>();

    let inner_area = content_block.inner(main_layout[1]);
    app.set_size(inner_area.height.into());
    
    f.render_widget(content_block, main_layout[1]);
    f.render_widget(Paragraph::new(items), inner_area);

    // Search bar
    let search_text = if app.search_mode {
        format!("/{}", app.search_query())
    } else {
        "Press / to start searching".into()
    };

    let search_bar = Paragraph::new(Text::raw(search_text))
        .block(Block::default().title(" Search ").borders(Borders::ALL))
        .alignment(Alignment::Left);

    f.render_widget(search_bar, main_layout[2]);

    // Status bar
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
                    "HISTORY" }
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
    status_line.push(Span::raw(&app.message));

    f.render_widget(Paragraph::new(Line::from(status_line)), main_layout[3]);

    // Help window (rendered last to overlay other components)
    if app.show_help {
        // Create transparent overlay
        f.render_widget(Clear, f.area());
        
        // Calculate help window position
        let help_area = centered_rect(60, 60, f.area());
        let vertical_offset = (f.area().height.saturating_sub(help_area.height)) / 4;
        let adjusted_rect = Rect::new(
            help_area.x,
            vertical_offset,
            help_area.width,
            help_area.height.min(f.area().height - vertical_offset - 2),
        );

        // Create help content
        let help_block = Block::default()
            .title(" Help (ESC to close) ")
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::DarkGray));

        let help_text = Text::from(app.get_help_text());
        let help_para = Paragraph::new(help_text)
            .block(help_block)
            .wrap(Wrap { trim: true });

        f.render_widget(help_para, adjusted_rect);
    }
}

/// Create centered rectangle with size constraints
fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_width = (area.width * percent_x / 100).min(area.width - 4);
    let popup_height = (area.height * percent_y / 100).min(area.height - 4);
    
    Rect::new(
        (area.width - popup_width) / 2,
        (area.height - popup_height) / 3, // Adjusted vertical centering
        popup_width,
        popup_height,
    )
}