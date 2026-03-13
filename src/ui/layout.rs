use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

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
