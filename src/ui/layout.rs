use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
};
use bytesize::ByteSize;

use crate::app::{App, Focus, Screen};
use crate::system::SystemStats;

pub fn draw(frame: &mut Frame, app: &App, stats: &SystemStats) {
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(frame.area());

    draw_header(frame, outer[0], stats);

    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(12), Constraint::Min(0)])
        .split(outer[1]);

    draw_sidebar(frame, body[0], app);
    draw_main(frame, body[1], app, stats);

    if app.show_confirm {
        draw_confirm_dialog(frame, app);
    }
}

fn draw_header(frame: &mut Frame, area: Rect, stats: &SystemStats) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(12), Constraint::Min(0)])
        .split(area);

    let title = Paragraph::new("  tidymac")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));
    frame.render_widget(title, chunks[0]);

    // Stats in header
    let disks = stats.disk_usage();
    let root_disk = disks.iter().find(|d| d.mount_point == "/");
    let disk_str = if let Some(d) = root_disk {
        format!(
            "{}/{} GB",
            d.used() / 1_000_000_000,
            d.total / 1_000_000_000
        )
    } else {
        "-- GB".to_string()
    };

    let header_stats = Paragraph::new(format!(
        "{}  · {:.0}% mem",
        disk_str,
        stats.memory_percent()
    ))
    .alignment(Alignment::Right)
    .style(Style::default().fg(Color::DarkGray));
    frame.render_widget(header_stats, chunks[1]);
}

fn draw_sidebar(frame: &mut Frame, area: Rect, app: &App) {
    let items: Vec<ListItem> = Screen::all()
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let style = if i == app.sidebar_index {
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            ListItem::new(format!(" {}", s.label())).style(style)
        })
        .collect();

    let border_style = if app.focus == Focus::Sidebar {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let sidebar = List::new(items)
        .block(Block::default().borders(Borders::RIGHT).border_style(border_style));
    frame.render_widget(sidebar, area);
}

fn draw_main(frame: &mut Frame, area: Rect, app: &App, stats: &SystemStats) {
    match app.screen {
        Screen::Home => super::home::draw(frame, area, app, stats),
        Screen::Scan => super::scan::draw(frame, area, app),
        Screen::Dev => super::dev::draw(frame, area, app),
        Screen::Apps => super::apps::draw(frame, area, app),
        Screen::Config => super::config::draw(frame, area, app),
    }
}

fn draw_confirm_dialog(frame: &mut Frame, app: &App) {
    let selected: Vec<_> = app.scan_results.iter().filter(|e| e.selected).collect();
    let total = ByteSize(app.selected_size());

    let area = frame.area();
    let popup_width = 50.min(area.width - 4);
    let popup_height = (selected.len() as u16 + 6).min(area.height - 4);

    let popup_area = Rect::new(
        (area.width - popup_width) / 2,
        (area.height - popup_height) / 2,
        popup_width,
        popup_height,
    );

    // Clear background
    frame.render_widget(Clear, popup_area);

    let mut lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            format!("  About to move {} to Trash:", total),
            Style::default().fg(Color::Yellow),
        )),
        Line::from(""),
    ];

    for entry in &selected {
        lines.push(Line::from(format!(
            "   {} {} ({})",
            entry.icon,
            entry.name,
            ByteSize(entry.size)
        )));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "  [Enter] Confirm    [Esc] Cancel",
        Style::default().fg(Color::DarkGray),
    )));

    let dialog = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow))
            .title(" 󰃢 Confirm Clean "),
    );

    frame.render_widget(dialog, popup_area);
}
