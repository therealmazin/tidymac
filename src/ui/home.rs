use bytesize::ByteSize;
use ratatui::{
    prelude::*,
    widgets::{Block, BorderType, Borders, LineGauge, List, ListItem, Paragraph, Sparkline},
};

use crate::app::App;
use crate::system::SystemStats;
use super::theme;

pub fn draw(frame: &mut Frame, area: Rect, app: &mut App, stats: &SystemStats) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(55), // CPU area
            Constraint::Percentage(45), // Memory+Disks | Ports
        ])
        .split(area);

    draw_cpu_area(frame, chunks[0], stats);

    let bottom = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[1]);

    draw_mem_disk(frame, bottom[0], stats);
    draw_ports(frame, bottom[1], app, stats);
}

fn draw_cpu_area(frame: &mut Frame, area: Rect, stats: &SystemStats) {
    let cores = stats.per_core();
    let core_panel_width = if !cores.is_empty() { 24u16 } else { 0u16 };

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(core_panel_width)])
        .split(area);

    // CPU sparkline graph
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

        let p = Paragraph::new(line);
        frame.render_widget(p, rows[i]);
    }
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
    let disk_rows = disks.len() * 2; // label + gauge per disk

    let mut constraints = vec![
        Constraint::Length(1), // Used
        Constraint::Length(1), // Available
        Constraint::Length(1), // Swap
        Constraint::Length(1), // separator
    ];
    for _ in 0..disk_rows {
        constraints.push(Constraint::Length(1));
    }
    constraints.push(Constraint::Min(0));

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(inner);

    let total = stats.memory_total();

    // Used
    let used = stats.memory_used();
    let used_ratio = if total > 0 { used as f64 / total as f64 } else { 0.0 };
    let used_color = if mem_pct > 80.0 {
        theme::CRIT_RED
    } else if mem_pct > 60.0 {
        theme::WARN_YELLOW
    } else {
        theme::MEM_BLUE
    };

    let used_gauge = LineGauge::default()
        .label(format!(" Used:  {} / {}", ByteSize(used), ByteSize(total)))
        .ratio(used_ratio.min(1.0))
        .filled_style(Style::default().fg(used_color))
        .unfilled_style(Style::default().fg(theme::BG_BAR))
        .line_set(symbols::line::THICK);
    frame.render_widget(used_gauge, rows[0]);

    // Available
    let avail = stats.memory_available();
    let avail_ratio = if total > 0 { avail as f64 / total as f64 } else { 0.0 };

    let avail_gauge = LineGauge::default()
        .label(format!(" Avail: {}", ByteSize(avail)))
        .ratio(avail_ratio.min(1.0))
        .filled_style(Style::default().fg(Color::Rgb(80, 180, 140)))
        .unfilled_style(Style::default().fg(theme::BG_BAR))
        .line_set(symbols::line::THICK);
    frame.render_widget(avail_gauge, rows[1]);

    // Swap
    let swap_total = stats.swap_total();
    let swap_used = stats.swap_used();
    let swap_ratio = if swap_total > 0 { swap_used as f64 / swap_total as f64 } else { 0.0 };

    let swap_label = if swap_total > 0 {
        format!(" Swap:  {} / {}", ByteSize(swap_used), ByteSize(swap_total))
    } else {
        " Swap:  --".to_string()
    };

    let swap_gauge = LineGauge::default()
        .label(swap_label)
        .ratio(swap_ratio.min(1.0))
        .filled_style(Style::default().fg(theme::WARN_YELLOW))
        .unfilled_style(Style::default().fg(theme::BG_BAR))
        .line_set(symbols::line::THICK);
    frame.render_widget(swap_gauge, rows[2]);

    // Disk section
    for (i, disk) in disks.iter().enumerate() {
        let label_idx = 4 + i * 2;
        let gauge_idx = 4 + i * 2 + 1;
        if gauge_idx >= rows.len() {
            break;
        }

        let pct = disk.percent();
        let bar_color = if pct > 90.0 {
            theme::CRIT_RED
        } else if pct > 70.0 {
            theme::WARN_YELLOW
        } else {
            theme::DISK_MAGENTA
        };

        let label = Paragraph::new(format!(
            " {}  {} free",
            disk.mount_point,
            ByteSize(disk.available)
        ))
        .style(Style::default().fg(theme::TEXT_SECONDARY));
        frame.render_widget(label, rows[label_idx]);

        let gauge = LineGauge::default()
            .label(format!(" {:.0}%", pct))
            .ratio((pct as f64 / 100.0).min(1.0))
            .filled_style(Style::default().fg(bar_color))
            .unfilled_style(Style::default().fg(theme::BG_BAR))
            .line_set(symbols::line::THICK);
        frame.render_widget(gauge, rows[gauge_idx]);
    }
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

    let items: Vec<ListItem> = ports
        .iter()
        .map(|p| {
            let mem_str = if p.memory > 0 {
                format!("{:>8}", ByteSize(p.memory))
            } else {
                "     --".to_string()
            };
            let line = Line::from(vec![
                Span::styled(format!(" :{:<6}", p.port), Style::default().fg(theme::ACCENT).add_modifier(Modifier::BOLD)),
                Span::styled(format!("{:<14}", p.process_name), Style::default().fg(theme::TEXT_PRIMARY)),
                Span::styled(mem_str, Style::default().fg(theme::WARN_YELLOW)),
            ]);
            ListItem::new(line)
        })
        .collect();

    let list = List::new(items)
        .block(block)
        .highlight_style(
            Style::default()
                .fg(theme::TEXT_PRIMARY)
                .bg(theme::SELECTED_BG)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▸");
    frame.render_stateful_widget(list, area, &mut app.port_list_state);
}
