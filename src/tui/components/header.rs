//! header rendering

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::utils::format::colors::{R_BLUE, R_DARK_GREY, R_GREY};

/// header
pub fn draw_header(frame: &mut Frame, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(area);

    let border_style = Style::default().fg(R_DARK_GREY);

    frame.render_widget(
        Paragraph::new(Span::styled(
            format!("╭{}╮", "─".repeat(area.width.saturating_sub(2) as usize)),
            border_style,
        )),
        chunks[0],
    );

    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::raw("  "),
            Span::styled(
                "aether-wire",
                Style::default().fg(R_BLUE).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(" v{}", env!("CARGO_PKG_VERSION")),
                Style::default().fg(R_GREY),
            ),
        ])),
        chunks[1],
    );

    frame.render_widget(
        Paragraph::new(Span::styled(
            format!("╰{}╯", "─".repeat(area.width.saturating_sub(2) as usize)),
            border_style,
        )),
        chunks[2],
    );
}
