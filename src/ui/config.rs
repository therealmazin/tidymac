use ratatui::{
    prelude::*,
    widgets::{Block, BorderType, Borders, Paragraph},
};

use crate::app::App;
use super::theme;

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn draw(frame: &mut Frame, area: Rect, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(2), // Title
            Constraint::Length(5), // Safe Mode box
            Constraint::Length(1), // spacer
            Constraint::Length(7), // About + Version + Update box
            Constraint::Min(0),   // remainder
        ])
        .split(area);

    let title = Paragraph::new(vec![
        Line::from(Span::styled(
            "   Settings",
            Style::default()
                .fg(theme::ACCENT)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            "  Configure tidymac behavior.",
            Style::default().fg(theme::TEXT_SECONDARY),
        )),
    ]);
    frame.render_widget(title, chunks[0]);

    // Safe Mode setting
    let safe_border = if app.config_index == 0 {
        theme::BORDER_FOCUSED
    } else {
        theme::BORDER_NORMAL
    };
    let safe_block = Block::default()
        .title(" Safe Mode ")
        .title_style(Style::default().fg(if app.config_index == 0 { theme::ACCENT } else { theme::TEXT_SECONDARY }))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(safe_border));

    let (indicator, desc) = if app.safe_mode {
        ("[ON] ", "Preview only — no deletions")
    } else {
        ("[OFF]", "Deletions will move to Trash")
    };

    let safe_text = Paragraph::new(vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(
                format!("  {} ", indicator),
                Style::default()
                    .fg(if app.safe_mode { theme::CPU_GREEN } else { theme::WARN_YELLOW })
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(desc, Style::default().fg(theme::TEXT_PRIMARY)),
        ]),
    ])
    .block(safe_block);
    frame.render_widget(safe_text, chunks[1]);

    // About + Version + Update
    let about_border = if app.config_index == 1 {
        theme::BORDER_FOCUSED
    } else {
        theme::BORDER_NORMAL
    };
    let about_block = Block::default()
        .title(" About ")
        .title_style(Style::default().fg(if app.config_index == 1 { theme::ACCENT } else { theme::TEXT_SECONDARY }))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(about_border));

    let about_text = Paragraph::new(vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  tidymac ", Style::default().fg(theme::TEXT_PRIMARY).add_modifier(Modifier::BOLD)),
            Span::styled(format!("v{}", VERSION), Style::default().fg(theme::ACCENT)),
        ]),
        Line::from(Span::styled(
            "  A lightweight TUI system cleaner for macOS",
            Style::default().fg(theme::TEXT_SECONDARY),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "  Update: brew upgrade therealmazin/tidymac/tidymac",
            Style::default().fg(theme::TEXT_SECONDARY),
        )),
    ])
    .block(about_block);
    frame.render_widget(about_text, chunks[3]);
}
