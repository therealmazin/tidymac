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
            Constraint::Min(0),   // Content
        ])
        .split(area);

    let title = Paragraph::new(vec![
        Line::from(Span::styled(
            "  󰋊 Space Lens",
            Style::default()
                .fg(theme::DISK_MAGENTA)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            "  Explore disk usage by folder. Enter to expand/collapse.",
            Style::default().fg(theme::TEXT_SECONDARY),
        )),
    ]);
    frame.render_widget(title, chunks[0]);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme::BORDER_NORMAL));

    if app.scanning {
        let scan_block = block.title(" Space Lens ");
        let inner = scan_block.inner(chunks[1]);
        frame.render_widget(scan_block, chunks[1]);

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

    if app.space_visible.is_empty() {
        let empty = Paragraph::new("  Press 's' to scan disk usage.")
            .style(Style::default().fg(theme::TEXT_SECONDARY))
            .block(block.title(" Space Lens "));
        frame.render_widget(empty, chunks[1]);
        return;
    }

    // Show expanding indicator in title
    let title_suffix = if app.space_expanding {
        format!(" {} loading... ", app.spinner_char())
    } else {
        String::new()
    };

    let max_size = app.space_visible.first().map(|e| e.size).unwrap_or(1);

    let items: Vec<ListItem> = app
        .space_visible
        .iter()
        .map(|item| {
            let indent = "  ".repeat(item.depth);
            let icon = if !item.is_dir {
                " "
            } else if item.expanded {
                "▼"
            } else {
                "▸"
            };

            let bar_width = 16usize;
            let ratio = item.size as f64 / max_size as f64;
            let filled = (ratio * bar_width as f64) as usize;
            let empty = bar_width.saturating_sub(filled);

            let color = if ratio > 0.7 {
                theme::CRIT_RED
            } else if ratio > 0.4 {
                theme::WARN_YELLOW
            } else {
                theme::DISK_MAGENTA
            };

            // Truncate name to fit
            let max_name_len = 20usize.saturating_sub(item.depth * 2);
            let display_name = if item.name.len() > max_name_len {
                format!("{}…", &item.name[..max_name_len.saturating_sub(1)])
            } else {
                item.name.clone()
            };

            let line = Line::from(vec![
                Span::styled(
                    format!(" {}{} ", indent, icon),
                    Style::default().fg(if item.is_dir { theme::ACCENT } else { theme::TEXT_SECONDARY }),
                ),
                Span::styled(
                    format!("{:<width$}", display_name, width = max_name_len),
                    Style::default().fg(theme::TEXT_PRIMARY),
                ),
                Span::styled("█".repeat(filled), Style::default().fg(color)),
                Span::styled("░".repeat(empty), Style::default().fg(theme::BG_BAR)),
                Span::styled(
                    format!(" {:>10}", ByteSize(item.size)),
                    Style::default().fg(color),
                ),
            ]);
            ListItem::new(line)
        })
        .collect();

    let total: u64 = app.space_tree.iter().map(|n| n.size).sum();
    let list = List::new(items)
        .block(block.title(format!(" Space Lens — {} total{}", ByteSize(total), title_suffix)))
        .highlight_style(
            Style::default()
                .fg(theme::TEXT_PRIMARY)
                .bg(theme::SELECTED_BG)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▸");
    frame.render_stateful_widget(list, chunks[1], &mut app.space_list_state);
}
