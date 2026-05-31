//! sidebar rendering

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{List, ListItem, Paragraph},
};

use crate::tui::app::{App, MENU_ITEMS};
use crate::utils::format::colors::{R_BLUE, R_DARK_GREY, R_LAVENDER};

/// sidebar
pub fn draw_sidebar(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(MENU_ITEMS.len() as u16),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .split(area);

    let top_border = Line::from(vec![
        Span::styled("╭─ ", Style::default().fg(R_DARK_GREY)),
        Span::styled("menu", Style::default().fg(R_LAVENDER)),
        Span::styled(
            format!(" {}╮", "─".repeat(area.width.saturating_sub(9) as usize)),
            Style::default().fg(R_DARK_GREY),
        ),
    ]);

    frame.render_widget(Paragraph::new(top_border), chunks[0]);

    let items: Vec<ListItem> = MENU_ITEMS
        .iter()
        .enumerate()
        .map(|(index, item)| {
            let selected = index == app.selected_menu;

            let style = if selected {
                Style::default().fg(R_LAVENDER).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(R_BLUE)
            };

            let prefix = if selected { " > " } else { "   " };

            ListItem::new(Line::from(format!("{prefix}{item}"))).style(style)
        })
        .collect();

    frame.render_widget(List::new(items), chunks[2]);

    frame.render_widget(
        Paragraph::new(Span::styled(
            format!("╰{}╯", "─".repeat(area.width.saturating_sub(2) as usize)),
            Style::default().fg(R_DARK_GREY),
        )),
        chunks[4],
    );
}
