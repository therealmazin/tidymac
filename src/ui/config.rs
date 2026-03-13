use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

use crate::app::App;

pub fn draw(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .split(area);

    let title = Paragraph::new(vec![
        Line::from(Span::styled(
            "   Settings",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            "  Configure tidymac behavior.",
            Style::default().fg(Color::DarkGray),
        )),
    ]);
    frame.render_widget(title, chunks[0]);

    let safe_mode_str = if app.safe_mode {
        "[ON]  Safe Mode — preview only, no deletions"
    } else {
        "[OFF] Safe Mode — deletions will move to Trash"
    };

    let items = vec![
        ListItem::new(format!("  {}", safe_mode_str)).style(
            if app.config_index == 0 {
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            },
        ),
        ListItem::new("  [i] About tidymac v0.1.0").style(
            if app.config_index == 1 {
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            },
        ),
    ];

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray))
            .title(" Configuration "),
    );
    frame.render_widget(list, chunks[1]);
}
