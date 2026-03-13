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
            "  \u{f0032} Applications",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            "  Manage installed apps and find orphaned files.",
            Style::default().fg(Color::DarkGray),
        )),
    ]);
    frame.render_widget(title, chunks[0]);

    if app.show_orphans {
        draw_orphans(frame, chunks[1], app);
    } else {
        draw_app_list(frame, chunks[1], app);
    }

    let footer = Paragraph::new(
        "  [s] Scan Apps  [o] Scan Orphans  [Tab] Switch view"
    )
    .style(Style::default().fg(Color::DarkGray));
    frame.render_widget(footer, chunks[2]);
}

fn draw_app_list(frame: &mut Frame, area: Rect, app: &App) {
    if app.app_list.is_empty() {
        let empty = Paragraph::new("  Press 's' to scan installed applications.")
            .style(Style::default().fg(Color::DarkGray))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::DarkGray))
                    .title(" Installed Apps "),
            );
        frame.render_widget(empty, area);
        return;
    }

    let items: Vec<ListItem> = app
        .app_list
        .iter()
        .enumerate()
        .map(|(i, app_info)| {
            let size_str = ByteSize(app_info.size).to_string();
            let related_count = app_info.related_files.len();
            let text = format!(
                "  \u{f0032} {} {:>10}  ({} related files)",
                app_info.name, size_str, related_count
            );

            let style = if i == app.app_list_index {
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            ListItem::new(text).style(style)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray))
            .title(format!(" Installed Apps ({}) ", app.app_list.len())),
    );
    frame.render_widget(list, area);
}

fn draw_orphans(frame: &mut Frame, area: Rect, app: &App) {
    if app.orphan_results.is_empty() {
        let empty = Paragraph::new("  No orphaned files found. Your system is clean!")
            .style(Style::default().fg(Color::Green))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::DarkGray))
                    .title(" Orphaned Files "),
            );
        frame.render_widget(empty, area);
        return;
    }

    let items: Vec<ListItem> = app
        .orphan_results
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let size_str = ByteSize(entry.size).to_string();
            let text = format!(
                "  {} {} {:>10}\n      {}",
                entry.icon,
                entry.name,
                size_str,
                entry.path.to_string_lossy()
            );

            let style = if i == app.app_list_index {
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            ListItem::new(text).style(style)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray))
            .title(format!(" Orphaned Files ({}) ", app.orphan_results.len())),
    );
    frame.render_widget(list, area);
}
