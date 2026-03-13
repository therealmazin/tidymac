use bytesize::ByteSize;
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
            Constraint::Length(3), // Title + description
            Constraint::Min(0),   // List
            Constraint::Length(3), // Footer with totals + actions
        ])
        .split(area);

    // Title
    let title = Paragraph::new(vec![
        Line::from(Span::styled(
            "  󰃢 System Junk",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            "  Clean caches, logs, and brew leftovers to reclaim space.",
            Style::default().fg(Color::DarkGray),
        )),
    ]);
    frame.render_widget(title, chunks[0]);

    // Scan results list
    if app.scan_results.is_empty() {
        let empty = Paragraph::new("  Press 's' to scan for system junk.")
            .style(Style::default().fg(Color::DarkGray))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::DarkGray))
                    .title(" Scan Results "),
            );
        frame.render_widget(empty, chunks[1]);
    } else {
        let items: Vec<ListItem> = app
            .scan_results
            .iter()
            .enumerate()
            .map(|(i, entry)| {
                let checkbox = if entry.selected { "[✓]" } else { "[ ]" };
                let size_str = ByteSize(entry.size).to_string();
                let text = format!(
                    "  {} {} {} {:>10}",
                    checkbox, entry.icon, entry.name, size_str
                );

                let style = if i == app.scan_list_index {
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
                } else if entry.selected {
                    Style::default().fg(Color::White)
                } else {
                    Style::default().fg(Color::DarkGray)
                };

                ListItem::new(text).style(style)
            })
            .collect();

        let list = List::new(items).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray))
                .title(" Scan Results "),
        );
        frame.render_widget(list, chunks[1]);
    }

    // Footer
    let selected_size = ByteSize(app.selected_size());
    let footer = Paragraph::new(format!(
        "  Total selected: {}    [s] Scan  [Space] Toggle  [c] Clean Selected",
        selected_size
    ))
    .style(Style::default().fg(Color::DarkGray));
    frame.render_widget(footer, chunks[2]);
}
