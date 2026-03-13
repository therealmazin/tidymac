use ratatui::prelude::*;
use ratatui::widgets::Paragraph;
use crate::app::App;
use crate::system::SystemStats;

pub fn draw(frame: &mut Frame, area: Rect, _app: &App, _stats: &SystemStats) {
    let text = Paragraph::new("  Home screen — coming soon")
        .style(Style::default().fg(Color::White));
    frame.render_widget(text, area);
}
