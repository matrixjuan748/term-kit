// ui.rs
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame
};
use crate::app::App;

pub fn draw_ui(f: &mut Frame, app: &mut App) {
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1)])
        .split(f.area());

    // 绘制主列表
    let list_block = Block::default()
        .borders(Borders::ALL)
        .title(" Command History ");
    
    let items = app.get_history().iter().enumerate()
        .skip(app.skipped_items)
        .map(|(i, cmd)| {
            let style = if i == app.selected {
                Style::default().bg(Color::Blue)
            } else {
                Style::default()
            };
            Line::from(format!("{:3}. {}", i + 1, cmd)).style(style)
        })
        .collect::<Vec<_>>();

    let list_area = list_block.inner(main_layout[0]);
    app.set_size(list_area.height.into());
    f.render_widget(list_block, main_layout[0]);
    f.render_widget(Paragraph::new(items), list_area);

    // 绘制帮助窗口
    if app.show_help {
        let help_block = Block::default()
            .title(" Help (ESC to close) ")
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::DarkGray));

        let text = Text::from(app.get_help_text());

        let help_paragraph = Paragraph::new(text)
            .block(help_block)
            .alignment(ratatui::layout::Alignment::Left)
            .wrap(Wrap { trim: true });

        let area = centered_rect(60, 30, f.area());
        f.render_widget(help_paragraph, area);
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical[1])[1]
}
