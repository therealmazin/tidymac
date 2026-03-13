use bytesize::ByteSize;
use ratatui::{
    prelude::*,
    widgets::{Bar, BarChart, BarGroup, Block, Borders, Gauge, Paragraph},
};

use crate::app::App;
use crate::system::SystemStats;

pub fn draw(frame: &mut Frame, area: Rect, _app: &App, stats: &SystemStats) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(5), // CPU + Memory gauges
            Constraint::Length(1), // spacer
            Constraint::Min(0),   // Disk usage
        ])
        .split(area);

    draw_cpu_mem(frame, chunks[0], stats);
    draw_disks(frame, chunks[2], stats);
}

fn draw_cpu_mem(frame: &mut Frame, area: Rect, stats: &SystemStats) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // CPU block
    let cpu_block = Block::default()
        .title(" 󰻠 CPU ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let sparkline_str = stats.cpu_sparkline();
    let cpu_text = format!(
        " {} {:.0}% · {} cores",
        sparkline_str,
        stats.cpu_usage(),
        stats.cpu_count()
    );
    let cpu = Paragraph::new(cpu_text)
        .style(Style::default().fg(Color::Green))
        .block(cpu_block);
    frame.render_widget(cpu, chunks[0]);

    // Memory block
    let mem_block = Block::default()
        .title(" 󰍛 Memory ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let mem_pct = stats.memory_percent();
    let mem_color = if mem_pct > 80.0 {
        Color::Red
    } else if mem_pct > 60.0 {
        Color::Yellow
    } else {
        Color::Blue
    };

    let mem_text = format!(
        " {} / {}  {:.0}%",
        ByteSize(stats.memory_used()),
        ByteSize(stats.memory_total()),
        mem_pct,
    );
    let mem = Paragraph::new(mem_text)
        .style(Style::default().fg(mem_color))
        .block(mem_block);
    frame.render_widget(mem, chunks[1]);
}

fn draw_disks(frame: &mut Frame, area: Rect, stats: &SystemStats) {
    let disk_block = Block::default()
        .title(" 󰋊 Disk Usage ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let inner = disk_block.inner(area);
    frame.render_widget(disk_block, area);

    let disks = stats.disk_usage();
    if disks.is_empty() {
        let no_disk = Paragraph::new("  No disks found")
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(no_disk, inner);
        return;
    }

    let constraints: Vec<Constraint> = disks.iter().map(|_| Constraint::Length(2)).collect();
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(inner);

    for (i, disk) in disks.iter().enumerate() {
        if i >= rows.len() {
            break;
        }

        let cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(20), Constraint::Min(0), Constraint::Length(14)])
            .split(rows[i]);

        let label = Paragraph::new(format!("  {}", disk.mount_point))
            .style(Style::default().fg(Color::White));
        frame.render_widget(label, cols[0]);

        let pct = disk.percent();
        let bar_color = if pct > 90.0 {
            Color::Red
        } else if pct > 70.0 {
            Color::Yellow
        } else {
            Color::Cyan
        };

        let gauge = Gauge::default()
            .gauge_style(Style::default().fg(bar_color).bg(Color::DarkGray))
            .ratio((pct as f64 / 100.0).min(1.0))
            .label(format!("{:.0}%", pct));
        frame.render_widget(gauge, cols[1]);

        let size_label = Paragraph::new(format!(
            " {} free",
            ByteSize(disk.available)
        ))
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Right);
        frame.render_widget(size_label, cols[2]);
    }
}
