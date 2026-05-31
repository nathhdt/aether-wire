//! about panel

use ratatui::{
    Frame,
    layout::Rect,
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Padding, Paragraph, Wrap},
};

use crate::utils::format::colors::{R_GREY, R_LAVENDER, R_TEXT};

pub struct AboutPanel;

impl AboutPanel {
    pub fn new() -> Self {
        Self
    }

    pub fn draw(&self, frame: &mut Frame, area: Rect) {
        let text = vec![
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
                    .title(Line::from(" about ").fg(R_LAVENDER).bold())
                    .padding(Padding::new(1, 3, 1, 0)),
            );

        frame.render_widget(widget, area);
    }
}
