use ratatui::{
    prelude::*,
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, Paragraph},
};
use bytesize::ByteSize;

use crate::app::{App, ConfirmKind, Focus, Screen};
use crate::system::SystemStats;
use super::theme;

pub fn draw(frame: &mut Frame, app: &mut App, stats: &SystemStats) {
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // header
            Constraint::Min(0),   // body
            Constraint::Length(1), // footer
        ])
        .split(frame.area());

    draw_header(frame, outer[0], stats);

    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(14), Constraint::Min(0)])
        .split(outer[1]);

    draw_sidebar(frame, body[0], app);
    draw_main(frame, body[1], app, stats);
    draw_footer(frame, outer[2], app);

    if app.is_confirming() {
        draw_confirm_dialog(frame, app);
    }
}

fn draw_header(frame: &mut Frame, area: Rect, stats: &SystemStats) {
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

    let left = Span::styled(
        " tidymac ",
        Style::default().fg(theme::ACCENT).add_modifier(Modifier::BOLD),
    );
    let right = Span::styled(
        format!("{}  {:.0}% mem ", disk_str, stats.memory_percent()),
        Style::default().fg(theme::TEXT_SECONDARY),
    );

    let header = Paragraph::new(Line::from(vec![
        left,
        Span::styled(
            "─".repeat(area.width.saturating_sub(30) as usize),
            Style::default().fg(theme::BORDER_NORMAL),
        ),
        right,
    ]));
    frame.render_widget(header, area);
}

fn draw_sidebar(frame: &mut Frame, area: Rect, app: &App) {
    let items: Vec<ListItem> = Screen::all()
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let marker = if i == app.sidebar_index { "▸ " } else { "  " };
            let style = if i == app.sidebar_index {
                Style::default().fg(theme::ACCENT).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme::TEXT_SECONDARY)
            };
            ListItem::new(format!("{}{}", marker, s.label())).style(style)
        })
        .collect();

    let border_color = if app.focus == Focus::Sidebar {
        theme::BORDER_FOCUSED
    } else {
        theme::BORDER_NORMAL
    };

    let sidebar = List::new(items).block(
        Block::default()
            .title(" tidymac ")
            .title_style(Style::default().fg(theme::ACCENT).add_modifier(Modifier::BOLD))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(border_color)),
    );
    frame.render_widget(sidebar, area);
}

fn draw_main(frame: &mut Frame, area: Rect, app: &mut App, stats: &SystemStats) {
    match app.screen {
        Screen::Home => super::home::draw(frame, area, app, stats),
        Screen::Scan => super::scan::draw(frame, area, app),
        Screen::Dev => super::dev::draw(frame, area, app),
        Screen::Apps => super::apps::draw(frame, area, app),
        Screen::Config => super::config::draw(frame, area, app),
    }
}

fn draw_footer(frame: &mut Frame, area: Rect, app: &App) {
    let hints = match app.screen {
        Screen::Home => vec![
            ("q", "Quit"), ("Tab", "Focus"), ("j/k", "Navigate"), ("x", "Kill Port"),
        ],
        Screen::Scan | Screen::Dev => {
            if app.scanning {
                vec![("q", "Quit"), ("Esc", "Sidebar")]
            } else if app.scan_results.is_empty() {
                vec![("q", "Quit"), ("Tab", "Focus"), ("s", "Scan")]
            } else {
                vec![
                    ("q", "Quit"), ("Tab", "Focus"), ("s", "Scan"),
                    ("Space", "Toggle"), ("c", "Clean"),
                ]
            }
        }
        Screen::Apps => {
            if app.app_list.is_empty() && app.orphan_results.is_empty() {
                vec![("q", "Quit"), ("Tab", "Focus"), ("s", "Scan Apps"), ("o", "Orphans")]
            } else {
                vec![
                    ("q", "Quit"), ("Tab", "Focus"), ("s", "Scan Apps"),
                    ("o", "Orphans"), ("d", "Uninstall"),
                ]
            }
        }
        Screen::Config => vec![
            ("q", "Quit"), ("Tab", "Focus"), ("Space", "Toggle"),
        ],
    };

    let spans: Vec<Span> = hints
        .iter()
        .flat_map(|(key, desc)| {
            vec![
                Span::styled(
                    format!(" {} ", key),
                    Style::default().fg(theme::ACCENT).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("{} ", desc),
                    Style::default().fg(theme::TEXT_SECONDARY),
                ),
            ]
        })
        .collect();

    let footer = Paragraph::new(Line::from(spans));
    frame.render_widget(footer, area);
}

fn draw_confirm_dialog(frame: &mut Frame, app: &App) {
    match app.confirm_kind {
        ConfirmKind::CleanScan => draw_clean_confirm(frame, app),
        ConfirmKind::UninstallApp => draw_uninstall_confirm(frame, app),
        ConfirmKind::KillPort => draw_kill_port_confirm(frame, app),
        ConfirmKind::None => {}
    }
}

fn draw_clean_confirm(frame: &mut Frame, app: &App) {
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

    frame.render_widget(Clear, popup_area);

    let mut lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            format!("  Move {} to Trash:", total),
            Style::default().fg(theme::WARN_YELLOW),
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
    lines.push(Line::from(vec![
        Span::styled("  Enter ", Style::default().fg(theme::ACCENT).add_modifier(Modifier::BOLD)),
        Span::styled("Confirm  ", Style::default().fg(theme::TEXT_SECONDARY)),
        Span::styled("Esc ", Style::default().fg(theme::ACCENT).add_modifier(Modifier::BOLD)),
        Span::styled("Cancel", Style::default().fg(theme::TEXT_SECONDARY)),
    ]));

    let dialog = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(theme::WARN_YELLOW))
            .title(" Confirm Clean ")
            .title_style(Style::default().fg(theme::WARN_YELLOW).add_modifier(Modifier::BOLD))
            .style(Style::default().bg(Color::Rgb(30, 30, 35))),
    );

    frame.render_widget(dialog, popup_area);
}

fn draw_uninstall_confirm(frame: &mut Frame, app: &App) {
    let Some(app_info) = app.app_list.get(app.app_list_index) else {
        return;
    };

    let related_count = app_info.related_files.len();
    let total_size: u64 = app_info.size + app_info.related_files.iter().map(|r| r.size).sum::<u64>();

    let area = frame.area();
    let popup_width = 56.min(area.width - 4);
    let popup_height = (related_count as u16 + 8).min(area.height - 4).max(8);

    let popup_area = Rect::new(
        (area.width - popup_width) / 2,
        (area.height - popup_height) / 2,
        popup_width,
        popup_height,
    );

    frame.render_widget(Clear, popup_area);

    let mut lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            format!("  Uninstall {}?", app_info.name),
            Style::default().fg(theme::CRIT_RED).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            format!("  App: {} ({})", app_info.name, ByteSize(app_info.size)),
            Style::default().fg(theme::TEXT_PRIMARY),
        )),
    ];

    if !app_info.related_files.is_empty() {
        lines.push(Line::from(Span::styled(
            format!("  + {} related files:", related_count),
            Style::default().fg(theme::TEXT_SECONDARY),
        )));
        for related in &app_info.related_files {
            lines.push(Line::from(Span::styled(
                format!("    {} ({})", related.name, ByteSize(related.size)),
                Style::default().fg(theme::TEXT_SECONDARY),
            )));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        format!("  Total: {}", ByteSize(total_size)),
        Style::default().fg(theme::WARN_YELLOW),
    )));
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("  Enter ", Style::default().fg(theme::CRIT_RED).add_modifier(Modifier::BOLD)),
        Span::styled("Uninstall  ", Style::default().fg(theme::TEXT_SECONDARY)),
        Span::styled("Esc ", Style::default().fg(theme::ACCENT).add_modifier(Modifier::BOLD)),
        Span::styled("Cancel", Style::default().fg(theme::TEXT_SECONDARY)),
    ]));

    let dialog = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(theme::CRIT_RED))
            .title(" Uninstall App ")
            .title_style(Style::default().fg(theme::CRIT_RED).add_modifier(Modifier::BOLD))
            .style(Style::default().bg(Color::Rgb(30, 30, 35))),
    );

    frame.render_widget(dialog, popup_area);
}

fn draw_kill_port_confirm(frame: &mut Frame, app: &App) {
    let Some(ref info) = app.kill_port_info else {
        return;
    };

    let area = frame.area();
    let popup_width = 48.min(area.width - 4);
    let popup_height = 9u16.min(area.height - 4);

    let popup_area = Rect::new(
        (area.width - popup_width) / 2,
        (area.height - popup_height) / 2,
        popup_width,
        popup_height,
    );

    frame.render_widget(Clear, popup_area);

    let mem_str = if info.memory > 0 {
        ByteSize(info.memory).to_string()
    } else {
        "--".to_string()
    };

    let lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            format!("  Kill process on port :{}?", info.port),
            Style::default().fg(theme::CRIT_RED).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Process: ", Style::default().fg(theme::TEXT_SECONDARY)),
            Span::styled(&info.process_name, Style::default().fg(theme::TEXT_PRIMARY)),
        ]),
        Line::from(vec![
            Span::styled("  PID:     ", Style::default().fg(theme::TEXT_SECONDARY)),
            Span::styled(format!("{}", info.pid), Style::default().fg(theme::TEXT_PRIMARY)),
            Span::styled(format!("   Memory: {}", mem_str), Style::default().fg(theme::WARN_YELLOW)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Enter ", Style::default().fg(theme::CRIT_RED).add_modifier(Modifier::BOLD)),
            Span::styled("Kill  ", Style::default().fg(theme::TEXT_SECONDARY)),
            Span::styled("Esc ", Style::default().fg(theme::ACCENT).add_modifier(Modifier::BOLD)),
            Span::styled("Cancel", Style::default().fg(theme::TEXT_SECONDARY)),
        ]),
    ];

    let dialog = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(theme::CRIT_RED))
            .title(" Kill Port ")
            .title_style(Style::default().fg(theme::CRIT_RED).add_modifier(Modifier::BOLD))
            .style(Style::default().bg(Color::Rgb(30, 30, 35))),
    );

    frame.render_widget(dialog, popup_area);
}
