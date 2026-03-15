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
        Screen::SmartScan => super::smart_scan::draw(frame, area, app),
        Screen::Apps => super::apps::draw(frame, area, app),
        Screen::SpaceLens => super::space_lens::draw(frame, area, app),
        Screen::LargeOld => super::large_old::draw(frame, area, app),
        Screen::Config => super::config::draw(frame, area, app),
    }
}

fn draw_footer(frame: &mut Frame, area: Rect, app: &App) {
    let hints = match app.screen {
        Screen::Home => vec![
            ("q", "Quit"), ("Tab", "Focus"), ("j/k", "Navigate"), ("x", "Kill Port"),
        ],
        Screen::SmartScan => {
            if app.scanning {
                vec![("q", "Quit"), ("Esc", "Sidebar")]
            } else if app.smart_scan_categories.is_empty() {
                vec![("q", "Quit"), ("Tab", "Focus"), ("s", "Scan")]
            } else {
                vec![
                    ("q", "Quit"), ("Tab", "Focus"), ("s", "Scan"),
                    ("Space", "Toggle"), ("Enter", "Expand"), ("c", "Clean"),
                ]
            }
        }
        Screen::Apps => {
            let view_name = match app.app_view {
                crate::app::AppView::All => "All",
                crate::app::AppView::Unused => "Unused",
                crate::app::AppView::Leftovers => "Leftovers",
            };
            vec![
                ("q", "Quit"), ("Tab", view_name), ("s", "Scan"),
                ("d", "Uninstall"),
            ]
        }
        Screen::SpaceLens => {
            if app.space_visible.is_empty() {
                vec![("q", "Quit"), ("Tab", "Focus"), ("s", "Scan")]
            } else {
                vec![
                    ("q", "Quit"), ("Tab", "Focus"), ("s", "Scan"),
                    ("Enter", "Expand"),
                ]
            }
        }
        Screen::LargeOld => {
            if app.scan_results.is_empty() {
                vec![("q", "Quit"), ("Tab", "Focus"), ("s", "Scan")]
            } else {
                vec![
                    ("q", "Quit"), ("Tab", "Focus"), ("s", "Scan"),
                    ("Space", "Toggle"), ("c", "Clean"),
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
        ConfirmKind::Cleaning => draw_cleaning_progress(frame, app),
        ConfirmKind::CleanDone => draw_clean_done(frame, app),
        ConfirmKind::UninstallApp => draw_uninstall_confirm(frame, app),
        ConfirmKind::KillPort => draw_kill_port_confirm(frame, app),
        ConfirmKind::None => {}
    }
}

fn draw_clean_confirm(frame: &mut Frame, app: &App) {
    let area = frame.area();

    // Build content lines based on screen type
    let mut lines = vec![Line::from("")];

    if app.screen == Screen::SmartScan {
        // Smart Scan: show category breakdown
        let mut total_items = 0u32;
        let mut total_size = 0u64;

        for cat in &app.smart_scan_categories {
            let selected_entries: Vec<_> = cat.entries.iter().filter(|e| e.selected).collect();
            if selected_entries.is_empty() { continue; }
            let cat_size: u64 = selected_entries.iter().map(|e| e.size).sum();
            total_items += selected_entries.len() as u32;
            total_size += cat_size;

            lines.push(Line::from(vec![
                Span::styled(format!("  {} ", cat.icon), Style::default().fg(theme::ACCENT)),
                Span::styled(
                    format!("{:<18}", cat.name),
                    Style::default().fg(theme::TEXT_PRIMARY),
                ),
                Span::styled(
                    format!("{:>10}  ({} items)", ByteSize(cat_size), selected_entries.len()),
                    Style::default().fg(theme::TEXT_SECONDARY),
                ),
            ]));
        }

        lines.insert(1, Line::from(Span::styled(
            format!("  Move {} ({} items) to Trash?", ByteSize(total_size), total_items),
            Style::default().fg(theme::WARN_YELLOW).add_modifier(Modifier::BOLD),
        )));
        lines.insert(2, Line::from(""));
    } else {
        // LargeOld or other: show selected entries
        let selected: Vec<_> = app.scan_results.iter().filter(|e| e.selected).collect();
        let total = ByteSize(app.selected_size());

        lines.push(Line::from(Span::styled(
            format!("  Move {} ({} items) to Trash?", total, selected.len()),
            Style::default().fg(theme::WARN_YELLOW).add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));

        for entry in selected.iter().take(8) {
            lines.push(Line::from(Span::styled(
                format!("   {} {} ({})", entry.icon, entry.name, ByteSize(entry.size)),
                Style::default().fg(theme::TEXT_SECONDARY),
            )));
        }
        if selected.len() > 8 {
            lines.push(Line::from(Span::styled(
                format!("   ... and {} more", selected.len() - 8),
                Style::default().fg(theme::TEXT_SECONDARY),
            )));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "  Items will be moved to Trash (recoverable).",
        Style::default().fg(theme::TEXT_SECONDARY),
    )));
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("  Enter ", Style::default().fg(theme::ACCENT).add_modifier(Modifier::BOLD)),
        Span::styled("Clean  ", Style::default().fg(theme::TEXT_SECONDARY)),
        Span::styled("Esc ", Style::default().fg(theme::ACCENT).add_modifier(Modifier::BOLD)),
        Span::styled("Cancel", Style::default().fg(theme::TEXT_SECONDARY)),
    ]));

    let popup_width = 56.min(area.width - 4);
    let popup_height = (lines.len() as u16 + 2).min(area.height - 4);
    let popup_area = Rect::new(
        (area.width - popup_width) / 2,
        (area.height - popup_height) / 2,
        popup_width,
        popup_height,
    );

    frame.render_widget(Clear, popup_area);

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

fn draw_cleaning_progress(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let popup_width = 56.min(area.width - 4);
    let popup_height = 12u16.min(area.height - 4);

    let popup_area = Rect::new(
        (area.width - popup_width) / 2,
        (area.height - popup_height) / 2,
        popup_width,
        popup_height,
    );

    frame.render_widget(Clear, popup_area);

    let pct = if app.clean_total > 0 {
        (app.clean_progress as f64 / app.clean_total as f64) * 100.0
    } else {
        0.0
    };

    // Progress bar
    let bar_width = (popup_width as usize).saturating_sub(12);
    let filled = ((pct / 100.0) * bar_width as f64) as usize;
    let empty = bar_width.saturating_sub(filled);

    let mut lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            "  Cleaning...",
            Style::default().fg(theme::ACCENT).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled("█".repeat(filled), Style::default().fg(theme::CPU_GREEN)),
            Span::styled("░".repeat(empty), Style::default().fg(theme::BG_BAR)),
            Span::styled(format!(" {:>3.0}%", pct), Style::default().fg(theme::CPU_GREEN)),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            format!("  {}/{} items  ·  {} freed", app.clean_progress, app.clean_total, ByteSize(app.clean_size_freed)),
            Style::default().fg(theme::TEXT_SECONDARY),
        )),
        Line::from(""),
    ];

    // Show last few completed items
    let show_count = 3.min(app.last_clean_results.len());
    for msg in app.last_clean_results.iter().rev().take(show_count) {
        let color = if msg.starts_with('✓') { theme::CPU_GREEN } else { theme::CRIT_RED };
        lines.push(Line::from(Span::styled(
            format!("  {}", msg),
            Style::default().fg(color),
        )));
    }

    let dialog = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(theme::ACCENT))
            .title(" Cleaning ")
            .title_style(Style::default().fg(theme::ACCENT).add_modifier(Modifier::BOLD))
            .style(Style::default().bg(Color::Rgb(30, 30, 35))),
    );

    frame.render_widget(dialog, popup_area);
}

fn draw_clean_done(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let result_count = app.last_clean_results.len().min(10);
    let popup_width = 56.min(area.width - 4);
    let popup_height = (result_count as u16 + 10).min(area.height - 4);

    let popup_area = Rect::new(
        (area.width - popup_width) / 2,
        (area.height - popup_height) / 2,
        popup_width,
        popup_height,
    );

    frame.render_widget(Clear, popup_area);

    let success_count = app.last_clean_results.iter().filter(|m| m.starts_with('✓')).count();
    let error_count = app.last_clean_results.iter().filter(|m| m.starts_with('✗')).count();

    let mut lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            "  Cleaning Complete!",
            Style::default().fg(theme::CPU_GREEN).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Freed: ", Style::default().fg(theme::TEXT_SECONDARY)),
            Span::styled(
                ByteSize(app.clean_size_freed).to_string(),
                Style::default().fg(theme::ACCENT).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(Span::styled(
            format!("  {} items cleaned{}", success_count,
                if error_count > 0 { format!(", {} errors", error_count) } else { String::new() }),
            Style::default().fg(theme::TEXT_SECONDARY),
        )),
        Line::from(""),
    ];

    // Show results (last N items)
    for msg in app.last_clean_results.iter().take(result_count) {
        let color = if msg.starts_with('✓') { theme::CPU_GREEN } else { theme::CRIT_RED };
        lines.push(Line::from(Span::styled(
            format!("  {}", msg),
            Style::default().fg(color),
        )));
    }
    if app.last_clean_results.len() > result_count {
        lines.push(Line::from(Span::styled(
            format!("  ... and {} more", app.last_clean_results.len() - result_count),
            Style::default().fg(theme::TEXT_SECONDARY),
        )));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("  Enter ", Style::default().fg(theme::ACCENT).add_modifier(Modifier::BOLD)),
        Span::styled("Continue", Style::default().fg(theme::TEXT_SECONDARY)),
    ]));

    let dialog = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(theme::CPU_GREEN))
            .title(" Done ")
            .title_style(Style::default().fg(theme::CPU_GREEN).add_modifier(Modifier::BOLD))
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
