//! TUI top level drawing

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style, Stylize},
    symbols,
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Padding, Paragraph},
};

use crate::tui::app::{App, MENU_ITEMS};
use crate::utils::format::colors::{R_BLUE, R_DARK_GREY, R_GREY, R_LAVENDER, R_LIGHT_GREY};

/// draws top level TUI layout
pub fn draw(frame: &mut Frame, app: &App) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(1),
        ])
        .split(frame.area());

    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(19), Constraint::Min(10)])
        .split(layout[1]);

    draw_header(frame, layout[0]);
    draw_sidebar(frame, body[0], app);
    draw_content(frame, body[1], app);
    draw_footer(frame, layout[2]);
}

/// header
fn draw_header(frame: &mut Frame, area: Rect) {
    let chunks = Layout::vertical([
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

/// navigation sidebar
fn draw_sidebar(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::vertical([
        Constraint::Length(1), // bordure haute
        Constraint::Length(1), // ligne vide
        Constraint::Length(MENU_ITEMS.len() as u16),
        Constraint::Length(1), // bordure basse
        Constraint::Min(0),    // reste vide
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

    let bottom_border = Span::styled(
        format!("╰{}╯", "─".repeat(area.width.saturating_sub(2) as usize)),
        Style::default().fg(R_DARK_GREY),
    );

    frame.render_widget(Paragraph::new(bottom_border), chunks[4]);
}

/// dispatches content rendering to the active panel
fn draw_content(frame: &mut Frame, area: Rect, app: &App) {
    let area = Rect {
        x: area.x + 3,
        y: area.y + 1,
        width: area.width.saturating_sub(2),
        height: area.height.saturating_sub(2),
    };

    app.panels.draw_active(frame, app.selected_menu, area);
}

/// TUI footer
fn draw_footer(frame: &mut Frame, area: Rect) {
    let footer = Paragraph::new(Line::from(vec![
        Span::styled("↑↓", Style::default().fg(R_LIGHT_GREY)),
        Span::styled(" menu • ", Style::default().fg(R_GREY)),
        Span::styled("q", Style::default().fg(R_LIGHT_GREY)),
        Span::styled(" quit", Style::default().fg(R_GREY)),
    ]))
    .alignment(Alignment::Center);

    frame.render_widget(footer, area);
}
