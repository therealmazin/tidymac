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
            Constraint::Min(0),   // Results
        ])
        .split(area);

    let title = Paragraph::new(vec![
        Line::from(Span::styled(
            "  󰃢 Smart Scan",
            Style::default()
                .fg(theme::ACCENT)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            "  Scan everything — system junk, build artifacts, and trash.",
            Style::default().fg(theme::TEXT_SECONDARY),
        )),
    ]);
    frame.render_widget(title, chunks[0]);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme::BORDER_NORMAL));

    if app.scanning {
        let inner_block = block.title(" Smart Scan ");
        let inner = inner_block.inner(chunks[1]);
        frame.render_widget(inner_block, chunks[1]);

        // Show step-by-step progress
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
                        Span::styled(
                            format!("  {} ", app.spinner_char()),
                            Style::default().fg(theme::SPINNER_COLOR),
                        ),
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

    if app.smart_scan_categories.is_empty() {
        let empty = Paragraph::new("  Press 's' to run a smart scan.")
            .style(Style::default().fg(theme::TEXT_SECONDARY))
            .block(block.title(" Smart Scan "));
        frame.render_widget(empty, chunks[1]);
        return;
    }

    // Build list items from categories
    let mut items: Vec<ListItem> = Vec::new();
    let mut total_size: u64 = 0;

    for cat in &app.smart_scan_categories {
        total_size += cat.total_size;
        let status = if cat.total_size > 100_000_000 { "⚠" } else { "✓" };
        let checkbox = if cat.selected { "[✓]" } else { "[ ]" };

        // Category header line
        let header = Line::from(vec![
            Span::styled(format!(" {} ", checkbox), Style::default().fg(theme::TEXT_PRIMARY)),
            Span::styled(format!("{} ", cat.icon), Style::default().fg(theme::ACCENT)),
            Span::styled(
                format!("{:<20}", cat.name),
                Style::default().fg(theme::TEXT_PRIMARY).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("{:>10}", ByteSize(cat.total_size)),
                Style::default().fg(theme::WARN_YELLOW),
            ),
            Span::styled(format!("  {}", status), Style::default().fg(
                if cat.total_size > 100_000_000 { theme::WARN_YELLOW } else { theme::CPU_GREEN }
            )),
        ]);
        items.push(ListItem::new(header));

        // Expanded: show individual entries
        if cat.expanded {
            for entry in &cat.entries {
                let entry_check = if entry.selected { " ✓" } else { "  " };
                let line = Line::from(vec![
                    Span::styled(format!("   {}", entry_check), Style::default().fg(theme::TEXT_SECONDARY)),
                    Span::styled(
                        format!("  {:<22}", entry.name),
                        Style::default().fg(theme::TEXT_SECONDARY),
                    ),
                    Span::styled(
                        format!("{:>10}", ByteSize(entry.size)),
                        Style::default().fg(theme::TEXT_SECONDARY),
                    ),
                ]);
                items.push(ListItem::new(line));
            }
        }
    }

    // Total line
    items.push(ListItem::new(Line::from("")));
    items.push(ListItem::new(Line::from(vec![
        Span::styled("  Total reclaimable: ", Style::default().fg(theme::TEXT_SECONDARY)),
        Span::styled(
            ByteSize(total_size).to_string(),
            Style::default().fg(theme::ACCENT).add_modifier(Modifier::BOLD),
        ),
    ])));

    let list = List::new(items)
        .block(block.title(" Smart Scan Results "))
        .highlight_style(
            Style::default()
                .fg(theme::TEXT_PRIMARY)
                .bg(theme::SELECTED_BG)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▸");
    frame.render_stateful_widget(list, chunks[1], &mut app.scan_list_state);
}
