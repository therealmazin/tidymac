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
        ])
        .split(area);

    let title = Paragraph::new(vec![
        Line::from(Span::styled(
            "  \u{f0032} Applications",
            Style::default()
                .fg(theme::ACCENT)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            "  Manage installed apps and find orphaned files.",
            Style::default().fg(theme::TEXT_SECONDARY),
        )),
    ]);
    frame.render_widget(title, chunks[0]);

    if app.scanning {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(theme::BORDER_NORMAL))
            .title(if app.show_orphans { " Orphaned Files " } else { " Installed Apps " });
        let spinner = Paragraph::new(format!(
            "  {} {}",
            app.spinner_char(),
            app.scan_status
        ))
        .style(Style::default().fg(theme::SPINNER_COLOR))
        .block(block);
        frame.render_widget(spinner, chunks[1]);
    } else if app.show_orphans {
        draw_orphans(frame, chunks[1], app);
    } else {
        draw_app_list(frame, chunks[1], app);
    }
}

fn draw_app_list(frame: &mut Frame, area: Rect, app: &mut App) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme::BORDER_NORMAL));

    if app.app_list.is_empty() {
        let empty = Paragraph::new("  Press 's' to scan installed applications.")
            .style(Style::default().fg(theme::TEXT_SECONDARY))
            .block(block.title(" Installed Apps "));
        frame.render_widget(empty, area);
        return;
    }

    let items: Vec<ListItem> = app
        .app_list
        .iter()
        .map(|app_info| {
            let size_str = ByteSize(app_info.size).to_string();
            let related_count = app_info.related_files.len();
            let text = format!(
                " \u{f0032} {} {:>10}  ({} related)",
                app_info.name, size_str, related_count
            );
            ListItem::new(text).style(Style::default().fg(theme::TEXT_PRIMARY))
        })
        .collect();

    let list = List::new(items)
        .block(block.title(format!(" Installed Apps ({}) ", app.app_list.len())))
        .highlight_style(
            Style::default()
                .fg(theme::TEXT_PRIMARY)
                .bg(theme::SELECTED_BG)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▸");
    frame.render_stateful_widget(list, area, &mut app.app_list_state);
}

fn draw_orphans(frame: &mut Frame, area: Rect, app: &mut App) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme::BORDER_NORMAL));

    if app.orphan_results.is_empty() {
        let empty = Paragraph::new("  No orphaned files found. Your system is clean!")
            .style(Style::default().fg(theme::CPU_GREEN))
            .block(block.title(" Orphaned Files "));
        frame.render_widget(empty, area);
        return;
    }

    let items: Vec<ListItem> = app
        .orphan_results
        .iter()
        .map(|entry| {
            let size_str = ByteSize(entry.size).to_string();
            let text = format!(
                " {} {} {:>10}  {}",
                entry.icon,
                entry.name,
                size_str,
                entry.path.to_string_lossy()
            );
            ListItem::new(text).style(Style::default().fg(theme::TEXT_PRIMARY))
        })
        .collect();

    let list = List::new(items)
        .block(block.title(format!(" Orphaned Files ({}) ", app.orphan_results.len())))
        .highlight_style(
            Style::default()
                .fg(theme::TEXT_PRIMARY)
                .bg(theme::SELECTED_BG)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▸");
    frame.render_stateful_widget(list, area, &mut app.orphan_list_state);
}
