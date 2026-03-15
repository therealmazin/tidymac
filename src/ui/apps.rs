use bytesize::ByteSize;
use ratatui::{
    prelude::*,
    widgets::{Block, BorderType, Borders, List, ListItem, Paragraph},
};

use crate::app::{App, AppView};
use super::theme;

pub fn draw(frame: &mut Frame, area: Rect, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(2), // Title
            Constraint::Length(1), // Tab bar
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
            "  Manage installed apps, find unused and leftover files.",
            Style::default().fg(theme::TEXT_SECONDARY),
        )),
    ]);
    frame.render_widget(title, chunks[0]);

    // Tab bar
    let tabs = vec![
        ("All", AppView::All),
        ("Unused", AppView::Unused),
        ("Leftovers", AppView::Leftovers),
    ];
    let tab_spans: Vec<Span> = tabs
        .iter()
        .flat_map(|(name, view)| {
            let style = if *view == app.app_view {
                Style::default().fg(theme::ACCENT).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme::TEXT_SECONDARY)
            };
            vec![
                Span::styled(format!("  {} ", name), style),
                Span::styled("│", Style::default().fg(theme::BORDER_NORMAL)),
            ]
        })
        .collect();
    frame.render_widget(Paragraph::new(Line::from(tab_spans)), chunks[1]);

    if app.scanning {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(theme::BORDER_NORMAL))
            .title(" Scanning... ");
        let inner = block.inner(chunks[2]);
        frame.render_widget(block, chunks[2]);

        let step_constraints: Vec<Constraint> = app.scan_steps
            .iter()
            .map(|_| Constraint::Length(1))
            .chain(std::iter::once(Constraint::Min(0)))
            .collect();
        let step_rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints(step_constraints)
            .split(inner);

        for (i, step) in app.scan_steps.iter().enumerate() {
            if i >= step_rows.len() - 1 { break; }
            let line = if step.done {
                Line::from(vec![
                    Span::styled("  ✓ ", Style::default().fg(theme::CPU_GREEN)),
                    Span::styled(&step.name, Style::default().fg(theme::CPU_GREEN)),
                ])
            } else {
                let is_current = (i == 0 || app.scan_steps[i - 1].done) && !step.done;
                if is_current {
                    Line::from(vec![
                        Span::styled(format!("  {} ", app.spinner_char()), Style::default().fg(theme::SPINNER_COLOR)),
                        Span::styled(&step.name, Style::default().fg(theme::SPINNER_COLOR)),
                    ])
                } else {
                    Line::from(vec![
                        Span::styled("    ", Style::default().fg(theme::TEXT_SECONDARY)),
                        Span::styled(&step.name, Style::default().fg(theme::TEXT_SECONDARY)),
                    ])
                }
            };
            frame.render_widget(Paragraph::new(line), step_rows[i]);
        }
        return;
    }

    match app.app_view {
        AppView::All => draw_app_list(frame, chunks[2], app),
        AppView::Unused => draw_unused(frame, chunks[2], app),
        AppView::Leftovers => draw_orphans(frame, chunks[2], app),
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
            .block(block.title(" All Apps "));
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
        .block(block.title(format!(" All Apps ({}) ", app.app_list.len())))
        .highlight_style(
            Style::default()
                .fg(theme::TEXT_PRIMARY)
                .bg(theme::SELECTED_BG)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▸");
    frame.render_stateful_widget(list, area, &mut app.app_list_state);
}

fn draw_unused(frame: &mut Frame, area: Rect, app: &mut App) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme::BORDER_NORMAL));

    if app.app_list.is_empty() {
        let empty = Paragraph::new("  Press 's' to scan apps first, then switch to Unused.")
            .style(Style::default().fg(theme::TEXT_SECONDARY))
            .block(block.title(" Unused Apps "));
        frame.render_widget(empty, area);
        return;
    }

    // TODO: Filter by last used date via mdls — for now show placeholder
    let empty = Paragraph::new("  Unused app detection coming soon.\n  Apps not opened in 6+ months will appear here.")
        .style(Style::default().fg(theme::TEXT_SECONDARY))
        .block(block.title(" Unused Apps "));
    frame.render_widget(empty, area);
}

fn draw_orphans(frame: &mut Frame, area: Rect, app: &mut App) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme::BORDER_NORMAL));

    if app.orphan_results.is_empty() {
        let empty = Paragraph::new("  No orphaned files found. Your system is clean!")
            .style(Style::default().fg(theme::CPU_GREEN))
            .block(block.title(" Leftovers "));
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
        .block(block.title(format!(" Leftovers ({}) ", app.orphan_results.len())))
        .highlight_style(
            Style::default()
                .fg(theme::TEXT_PRIMARY)
                .bg(theme::SELECTED_BG)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▸");
    frame.render_stateful_widget(list, area, &mut app.orphan_list_state);
}
