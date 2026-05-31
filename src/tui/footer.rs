//! footer items and rendering

use ratatui::{
    Frame,
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::utils::format::colors::{R_DARK_GREY, R_DARK_LAVENDER, R_GREY, R_LIGHT_GREY};

/// single footer shortcut entry
#[derive(Clone)]
pub struct FooterItem {
    pub key: &'static str,
    pub label: &'static str,
    pub enabled: bool,
}

impl FooterItem {
    pub const fn new(key: &'static str, label: &'static str) -> Self {
        Self {
            key,
            label,
            enabled: true,
        }
    }
}

/// footer bar rendering
pub fn draw_footer(frame: &mut Frame, area: Rect, items: &[FooterItem]) {
    let mut spans = vec![Span::raw(" ")];

    for (i, item) in items.iter().enumerate() {
        if i > 0 {
            let sep_style = if item.enabled {
                Style::default().fg(R_GREY).bg(R_DARK_LAVENDER)
            } else {
                Style::default().fg(R_DARK_GREY).bg(R_DARK_LAVENDER)
            };
            spans.push(Span::styled(" • ", sep_style));
        }

        let (key_style, label_style) = if item.enabled {
            (
                Style::default()
                    .fg(ratatui::style::Color::White)
                    .bg(R_DARK_LAVENDER),
                Style::default().fg(R_LIGHT_GREY).bg(R_DARK_LAVENDER),
            )
        } else {
            (
                Style::default().fg(R_GREY).bg(R_DARK_LAVENDER),
                Style::default().fg(R_GREY).bg(R_DARK_LAVENDER),
            )
        };

        spans.push(Span::styled(item.key, key_style));
        spans.push(Span::styled(format!(" {}", item.label), label_style));
    }

    // ull width
    spans.push(Span::styled(
        " ".repeat(area.width as usize),
        Style::default().bg(R_DARK_LAVENDER),
    ));

    let footer = Paragraph::new(Line::from(spans)).style(Style::default().bg(R_DARK_LAVENDER));

    frame.render_widget(footer, area);
}
