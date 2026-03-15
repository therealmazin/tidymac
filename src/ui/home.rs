use bytesize::ByteSize;
use ratatui::{
    prelude::*,
    widgets::{Block, BorderType, Borders, List, ListItem, Paragraph, Sparkline},
};

use crate::app::App;
use crate::system::SystemStats;
use super::theme;

pub fn draw(frame: &mut Frame, area: Rect, app: &mut App, stats: &SystemStats) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(50), // CPU area
            Constraint::Percentage(50), // Bottom: left (mem+net) | right (ports)
        ])
        .split(area);

    draw_cpu_area(frame, chunks[0], stats);

    let bottom = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
        .split(chunks[1]);

    // Left: Memory+Disks on top, Network at bottom
    let left = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(9)])
        .split(bottom[0]);

    draw_mem_disk(frame, left[0], stats);
    draw_network(frame, left[1], stats);

    // Right: Listening Ports (full height)
    draw_ports(frame, bottom[1], app, stats);
}

fn draw_cpu_area(frame: &mut Frame, area: Rect, stats: &SystemStats) {
    let cores = stats.per_core();
    let core_panel_width = if !cores.is_empty() { 24u16 } else { 0u16 };

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(core_panel_width)])
        .split(area);

    let cpu_block = Block::default()
        .title(format!(" 󰻠 CPU {:.0}% · {} cores ", stats.cpu_usage(), stats.cpu_count()))
        .title_style(Style::default().fg(theme::CPU_GREEN).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme::BORDER_NORMAL));

    let data = stats.cpu_history_u64();
    let sparkline = Sparkline::default()
        .block(cpu_block)
        .data(&data)
        .max(10000)
        .style(Style::default().fg(theme::CPU_GREEN));
    frame.render_widget(sparkline, chunks[0]);

    if core_panel_width > 0 {
        draw_core_panel(frame, chunks[1], cores);
    }
}

fn draw_core_panel(frame: &mut Frame, area: Rect, cores: &[f32]) {
    let block = Block::default()
        .title(" Cores ")
        .title_style(Style::default().fg(theme::CPU_GREEN).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme::BORDER_NORMAL));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let max_rows = inner.height as usize;
    let cores_to_show = cores.len().min(max_rows);
    if cores_to_show == 0 {
        return;
    }

    let constraints: Vec<Constraint> = (0..cores_to_show)
        .map(|_| Constraint::Length(1))
        .collect();
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(inner);

    for (i, &usage) in cores.iter().take(cores_to_show).enumerate() {
        let color = if usage > 80.0 {
            theme::CRIT_RED
        } else if usage > 50.0 {
            theme::WARN_YELLOW
        } else {
            theme::CPU_GREEN
        };

        let label = format!("C{:<2}", i);
        let pct_str = format!("{:>3.0}%", usage);
        let bar_width = rows[i].width.saturating_sub(label.len() as u16 + pct_str.len() as u16 + 1) as usize;
        let filled = ((usage / 100.0) * bar_width as f32) as usize;
        let empty = bar_width.saturating_sub(filled);

        let line = Line::from(vec![
            Span::styled(label, Style::default().fg(theme::TEXT_SECONDARY)),
            Span::styled("█".repeat(filled), Style::default().fg(color)),
            Span::styled("░".repeat(empty), Style::default().fg(theme::BG_BAR)),
            Span::styled(pct_str, Style::default().fg(color)),
        ]);
        frame.render_widget(Paragraph::new(line), rows[i]);
    }
}

/// Render a single-line bar: " Label  ████░░░░  63% "
fn render_bar(frame: &mut Frame, area: Rect, label: &str, pct: f64, color: Color) {
    let label_width = 7u16;
    let pct_width = 5u16;
    let bar_width = area.width.saturating_sub(label_width + pct_width + 2) as usize;
    let filled = ((pct / 100.0) * bar_width as f64) as usize;
    let empty = bar_width.saturating_sub(filled);

    let line = Line::from(vec![
        Span::styled(format!(" {:<6}", label), Style::default().fg(theme::TEXT_PRIMARY)),
        Span::styled("█".repeat(filled), Style::default().fg(color)),
        Span::styled("░".repeat(empty), Style::default().fg(theme::BG_BAR)),
        Span::styled(format!(" {:>3.0}%", pct), Style::default().fg(color)),
    ]);
    frame.render_widget(Paragraph::new(line), area);
}

fn draw_mem_disk(frame: &mut Frame, area: Rect, stats: &SystemStats) {
    let mem_pct = stats.memory_percent();
    let block = Block::default()
        .title(format!(" 󰍛 Memory {:.0}% & Disks ", mem_pct))
        .title_style(Style::default().fg(theme::MEM_BLUE).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme::BORDER_NORMAL));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let disks = stats.disk_usage();

    // Use root disk only (/) for the Disk bar
    let root_disk = disks.iter().find(|d| d.mount_point == "/");

    let constraints = vec![
        Constraint::Length(1), // Used bar
        Constraint::Length(1), // Used detail
        Constraint::Length(1), // Avail bar
        Constraint::Length(1), // Avail detail
        Constraint::Length(1), // Swap bar
        Constraint::Length(1), // Swap detail
        Constraint::Length(1), // separator
        Constraint::Length(1), // Disk bar
        Constraint::Length(1), // Disk detail
        Constraint::Min(0),
    ];

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(inner);

    let total = stats.memory_total();
    let used = stats.memory_used();
    let used_pct = if total > 0 { (used as f64 / total as f64) * 100.0 } else { 0.0 };
    let used_color = if used_pct > 80.0 { theme::CRIT_RED } else if used_pct > 60.0 { theme::WARN_YELLOW } else { theme::MEM_BLUE };

    render_bar(frame, rows[0], "Used", used_pct, used_color);
    let used_detail = Paragraph::new(format!("        {} / {}", ByteSize(used), ByteSize(total)))
        .style(Style::default().fg(theme::TEXT_SECONDARY));
    frame.render_widget(used_detail, rows[1]);

    let avail = stats.memory_available();
    let avail_pct = if total > 0 { (avail as f64 / total as f64) * 100.0 } else { 0.0 };
    render_bar(frame, rows[2], "Avail", avail_pct, Color::Rgb(148, 226, 213));
    let avail_detail = Paragraph::new(format!("        {}", ByteSize(avail)))
        .style(Style::default().fg(theme::TEXT_SECONDARY));
    frame.render_widget(avail_detail, rows[3]);

    let swap_total = stats.swap_total();
    let swap_used = stats.swap_used();
    let swap_pct = if swap_total > 0 { (swap_used as f64 / swap_total as f64) * 100.0 } else { 0.0 };
    render_bar(frame, rows[4], "Swap", swap_pct, theme::WARN_YELLOW);
    let swap_detail = if swap_total > 0 {
        format!("        {} / {}", ByteSize(swap_used), ByteSize(swap_total))
    } else {
        "        --".to_string()
    };
    frame.render_widget(Paragraph::new(swap_detail).style(Style::default().fg(theme::TEXT_SECONDARY)), rows[5]);

    // Disk bar — show root disk
    if let Some(disk) = root_disk {
        let pct = disk.percent();
        let color = if pct > 90.0 { theme::CRIT_RED } else if pct > 70.0 { theme::WARN_YELLOW } else { theme::DISK_MAGENTA };

        render_bar(frame, rows[7], "Disk", pct as f64, color);
        let detail = Paragraph::new(format!(
            "        {} / {} ({} free)",
            ByteSize(disk.used()),
            ByteSize(disk.total),
            ByteSize(disk.available)
        ))
        .style(Style::default().fg(theme::TEXT_SECONDARY));
        frame.render_widget(detail, rows[8]);
    }
}

fn draw_network(frame: &mut Frame, area: Rect, stats: &SystemStats) {
    let block = Block::default()
        .title(" 󰛳 Network ")
        .title_style(Style::default().fg(theme::CPU_GREEN).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme::BORDER_NORMAL));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let net = &stats.network_stats;

    let constraints = vec![
        Constraint::Length(1), // Down speed
        Constraint::Length(1), // Down top
        Constraint::Length(1), // Down total
        Constraint::Length(1), // Up speed
        Constraint::Length(1), // Up top
        Constraint::Length(1), // Up total
        Constraint::Min(0),
    ];
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(inner);

    let down_color = theme::CPU_GREEN;
    let up_color = theme::CRIT_RED;

    // Download
    let down_speed = Paragraph::new(Line::from(vec![
        Span::styled(" ▼ Down  ", Style::default().fg(down_color).add_modifier(Modifier::BOLD)),
        Span::styled(format!("{}/s", ByteSize(net.download_speed)), Style::default().fg(theme::TEXT_PRIMARY)),
    ]));
    frame.render_widget(down_speed, rows[0]);

    let down_top = Paragraph::new(format!("   Top:   {}/s", ByteSize(net.download_top)))
        .style(Style::default().fg(theme::TEXT_SECONDARY));
    frame.render_widget(down_top, rows[1]);

    let down_total = Paragraph::new(format!("   Total: {}", ByteSize(net.download_total)))
        .style(Style::default().fg(theme::TEXT_SECONDARY));
    frame.render_widget(down_total, rows[2]);

    // Upload
    let up_speed = Paragraph::new(Line::from(vec![
        Span::styled(" ▲ Up    ", Style::default().fg(up_color).add_modifier(Modifier::BOLD)),
        Span::styled(format!("{}/s", ByteSize(net.upload_speed)), Style::default().fg(theme::TEXT_PRIMARY)),
    ]));
    frame.render_widget(up_speed, rows[3]);

    let up_top = Paragraph::new(format!("   Top:   {}/s", ByteSize(net.upload_top)))
        .style(Style::default().fg(theme::TEXT_SECONDARY));
    frame.render_widget(up_top, rows[4]);

    let up_total = Paragraph::new(format!("   Total: {}", ByteSize(net.upload_total)))
        .style(Style::default().fg(theme::TEXT_SECONDARY));
    frame.render_widget(up_total, rows[5]);
}

fn draw_ports(frame: &mut Frame, area: Rect, app: &mut App, stats: &SystemStats) {
    let ports = &stats.listening_ports;

    let block = Block::default()
        .title(format!(" 󰒍 Listening Ports ({}) ", ports.len()))
        .title_style(Style::default().fg(theme::ACCENT).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme::BORDER_NORMAL));

    if ports.is_empty() {
        let empty = Paragraph::new(" No listening ports found")
            .style(Style::default().fg(theme::TEXT_SECONDARY))
            .block(block);
        frame.render_widget(empty, area);
        return;
    }

    // Split: header row + list
    let inner_block = block.inner(area);
    frame.render_widget(block, area);

    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(inner_block);

    // Header row
    let header = Paragraph::new(Line::from(vec![
        Span::styled(format!(" {:<7}", "Port"), Style::default().fg(theme::TEXT_SECONDARY).add_modifier(Modifier::BOLD)),
        Span::styled(format!("{:<14}", "Process"), Style::default().fg(theme::TEXT_SECONDARY).add_modifier(Modifier::BOLD)),
        Span::styled(format!("{:>8}", "Mem"), Style::default().fg(theme::TEXT_SECONDARY).add_modifier(Modifier::BOLD)),
        Span::styled(format!("{:>7}", "CPU%"), Style::default().fg(theme::TEXT_SECONDARY).add_modifier(Modifier::BOLD)),
    ]));
    frame.render_widget(header, sections[0]);

    // Port list items
    let items: Vec<ListItem> = ports
        .iter()
        .map(|p| {
            let mem_str = if p.memory > 0 {
                format!("{:>8}", ByteSize(p.memory))
            } else {
                "     --".to_string()
            };
            let cpu_str = format!("{:>6.1}", p.cpu_usage);
            let line = Line::from(vec![
                Span::styled(format!(" :{:<6}", p.port), Style::default().fg(theme::ACCENT)),
                Span::styled(format!("{:<14}", p.process_name), Style::default().fg(theme::TEXT_PRIMARY)),
                Span::styled(mem_str, Style::default().fg(theme::WARN_YELLOW)),
                Span::styled(cpu_str, Style::default().fg(theme::CPU_GREEN)),
            ]);
            ListItem::new(line)
        })
        .collect();

    let list = List::new(items)
        .highlight_style(
            Style::default()
                .fg(theme::TEXT_PRIMARY)
                .bg(theme::SELECTED_BG)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▸");
    frame.render_stateful_widget(list, sections[1], &mut app.port_list_state);
}
