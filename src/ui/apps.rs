use ratatui::prelude::*;
use ratatui::widgets::Paragraph;
use crate::app::App;

pub fn draw(frame: &mut Frame, area: Rect, _app: &App) {
    let text = Paragraph::new("  Apps screen — coming soon")
        .style(Style::default().fg(Color::White));
    frame.render_widget(text, area);
}
