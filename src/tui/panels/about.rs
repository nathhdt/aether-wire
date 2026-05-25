//! about panel

use ratatui::{
    Frame,
    layout::Rect,
    style::{Style, Stylize},
    symbols,
    text::{Line, Span},
    widgets::{Block, Borders, Padding, Paragraph, Wrap},
};

use crate::utils::format::colors::{R_BLUE, R_GREY, R_LAVENDER, R_TEXT};

pub struct AboutPanel;

impl AboutPanel {
    pub fn new() -> Self {
        Self
    }

    pub fn draw(&self, frame: &mut Frame, area: Rect) {
        let text = vec![
            Line::from(""),
            Line::from(Span::styled(
                "aether-wire is a lightweight native \
cross-platform tool written in Rust for \
end-to-end (E2E) network performance \
measurement.",
                Style::default().fg(R_GREY),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "this project is currently under development.",
                Style::default().fg(R_GREY),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "copyright (c) nathhdt",
                Style::default().fg(R_GREY),
            )),
        ];

        let widget = Paragraph::new(text)
            .wrap(Wrap { trim: true })
            .style(Style::default().fg(R_TEXT))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_set(symbols::border::ROUNDED)
                    .title(" about ".fg(R_LAVENDER))
                    .border_style(Style::default().fg(R_BLUE))
                    .padding(Padding::new(2, 2, 0, 0)),
            );

        frame.render_widget(widget, area);
    }
}
