use bytesize::ByteSize;
use ratatui::{
    prelude::*,
    widgets::{Block, BorderType, Borders, List, ListItem, Paragraph},
};

use crate::app::App;
use super::theme;

pub fn draw(frame: &mut Frame, area: Rect, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(2), // Title
            Constraint::Min(0),   // List
            Constraint::Length(1), // Detail path
        ])
        .split(area);

    let title = Paragraph::new(vec![
        Line::from(Span::styled(
            "  󰅐 Dev Tools",
            Style::default()
                .fg(theme::ACCENT)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            "  Clean Xcode, Docker, node_modules, and Cargo artifacts.",
            Style::default().fg(theme::TEXT_SECONDARY),
        )),
    ]);
    frame.render_widget(title, chunks[0]);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme::BORDER_NORMAL))
        .title(" Dev Scan Results ");

    if app.scanning {
        let spinner = Paragraph::new(format!(
            "  {} {}",
            app.spinner_char(),
            app.scan_status
        ))
        .style(Style::default().fg(theme::SPINNER_COLOR))
        .block(block);
        frame.render_widget(spinner, chunks[1]);
    } else if app.scan_results.is_empty() {
        let empty = Paragraph::new("  Press 's' to scan for dev tool junk.")
            .style(Style::default().fg(theme::TEXT_SECONDARY))
            .block(block);
        frame.render_widget(empty, chunks[1]);
    } else {
        let items: Vec<ListItem> = app
            .scan_results
            .iter()
            .map(|entry| {
                let checkbox = if entry.selected { "[✓]" } else { "[ ]" };
                let size_str = ByteSize(entry.size).to_string();
                let text = format!(
                    " {} {} {} {:>10}",
                    checkbox, entry.icon, entry.name, size_str
                );

                let style = if entry.selected {
                    Style::default().fg(theme::TEXT_PRIMARY)
                } else {
                    Style::default().fg(theme::TEXT_SECONDARY)
                };

                ListItem::new(text).style(style)
            })
            .collect();

        let selected_size = ByteSize(app.selected_size());
        let list = List::new(items)
            .block(
                block.title(format!(
                    " Dev Scan Results ({}) · {} selected ",
                    app.scan_results.len(),
                    selected_size
                )),
            )
            .highlight_style(
                Style::default()
                    .fg(theme::TEXT_PRIMARY)
                    .bg(theme::SELECTED_BG)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("▸");
        frame.render_stateful_widget(list, area, &mut app.scan_list_state);
    }

    // Detail: show path of currently selected item
    let detail = if let Some(entry) = app.scan_results.get(app.scan_list_index) {
        format!("  {}", entry.path.to_string_lossy())
    } else {
        String::new()
    };
    let detail_line = Paragraph::new(detail)
        .style(Style::default().fg(theme::TEXT_SECONDARY));
    frame.render_widget(detail_line, chunks[2]);
}
