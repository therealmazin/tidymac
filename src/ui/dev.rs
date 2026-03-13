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
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(area);

    let title = Paragraph::new(vec![
        Line::from(Span::styled(
            "  󰅐 Dev Tools",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            "  Clean Xcode, Docker, node_modules, and Cargo build artifacts.",
            Style::default().fg(Color::DarkGray),
        )),
    ]);
    frame.render_widget(title, chunks[0]);

    if app.scan_results.is_empty() {
        let empty = Paragraph::new("  Press 's' to scan for dev tool junk.")
            .style(Style::default().fg(Color::DarkGray))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::DarkGray))
                    .title(" Dev Scan Results "),
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
                let path_str = entry.path.to_string_lossy();
                let text = format!(
                    "  {} {} {} {:>10}\n      {}",
                    checkbox, entry.icon, entry.name, size_str, path_str
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
                .title(" Dev Scan Results "),
        );
        frame.render_widget(list, chunks[1]);
    }

    let selected_size = ByteSize(app.selected_size());
    let footer = Paragraph::new(format!(
        "  Total selected: {}    [s] Scan  [Space] Toggle  [c] Clean Selected",
        selected_size
    ))
    .style(Style::default().fg(Color::DarkGray));
    frame.render_widget(footer, chunks[2]);
}
