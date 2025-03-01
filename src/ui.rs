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
    // 计算整体布局
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // 顶部信息框
            Constraint::Min(1),     // 历史记录列表
            Constraint::Length(3),  // 底部搜索框
            Constraint::Length(1),  // 消息提示
        ])
        .split(f.area());

    // ========== 顶部框 ==========
    let header = Paragraph::new(Line::from(vec![
        Span::styled("History Finder ", Style::default().fg(Color::Yellow)),
        Span::styled("v0.1", Style::default().fg(Color::LightBlue)),
        Span::raw(" | [↑/↓] 选择 | [Enter] 复制 | [/] 搜索 | [q] 退出 | [h] 帮助"),
    ]))
    .block(Block::default().title(" Info ").borders(Borders::ALL))
    .alignment(Alignment::Center);

    f.render_widget(header, layout[0]);

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

    let list_area = list_block.inner(layout[1]);
    app.set_size(list_area.height.into());
    f.render_widget(list_block, layout[1]);
    f.render_widget(Paragraph::new(items), list_area);

    let search_text = if app.get_query().is_empty() && !app.search_mode {
        "输入 '/' 开始搜索...".into()
    } else {
        format!("/{}", app.get_query())
    };

    let search_bar = Paragraph::new(Text::raw(search_text))
        .block(Block::default().title(" Search ").borders(Borders::ALL))
        .alignment(Alignment::Left);

    f.render_widget(search_bar, layout[2]);

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

    f.render_widget(Paragraph::new(Text::raw(app.message.clone())), layout[3]);
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
