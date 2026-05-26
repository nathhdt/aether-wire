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
use crate::utils::format::colors::{R_BLUE, R_GREY, R_LAVENDER, R_LIGHT_GREY};

/// draws top level TUI layout
pub fn draw(frame: &mut Frame, app: &App) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(6),
            Constraint::Min(10),
            Constraint::Length(1),
        ])
        .split(frame.area());

    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(20), Constraint::Min(10)])
        .split(layout[1]);

    draw_header(frame, layout[0]);
    draw_sidebar(frame, body[0], app);
    draw_content(frame, body[1], app);
    draw_footer(frame, layout[2]);
}

/// ASCII logo
fn draw_header(frame: &mut Frame, area: Rect) {
    let logo = vec![
        Line::from(Span::styled(
            r"             _   _                            _          ",
            Style::default().fg(R_BLUE).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            r"   __ _  ___| |_| |__   ___ _ __    __      _(_)_ __ ___ ",
            Style::default().fg(R_BLUE).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            r"  / _` |/ _ \ __| '_ \ / _ \ '__|___\ \ /\ / / | '__/ _ \",
            Style::default().fg(R_BLUE).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            r" | (_| |  __/ |_| | | |  __/ | |_____\ V  V /| | | |  __/",
            Style::default().fg(R_BLUE).add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![
            Span::styled(
                r"  \__,_|\___|\__|_| |_|\___|_|        \_/\_/ |_|_|  \___|",
                Style::default().fg(R_BLUE).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("  v{}", env!("CARGO_PKG_VERSION")),
                Style::default().fg(R_GREY),
            ),
        ]),
    ];

    frame.render_widget(Paragraph::new(logo), area);
}

/// navigation sidebar
fn draw_sidebar(frame: &mut Frame, area: Rect, app: &App) {
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

    let sidebar = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_set(symbols::border::ROUNDED)
            .title(" menu ".fg(R_LAVENDER))
            .border_style(Style::default().fg(R_BLUE))
            .padding(Padding::top(1)),
    );

    frame.render_widget(sidebar, area);
}

/// dispatches content rendering to the active panel
fn draw_content(frame: &mut Frame, area: Rect, app: &App) {
    app.panels.draw_active(frame, app.selected_menu, area);
}

/// TUI footer
fn draw_footer(frame: &mut Frame, area: Rect) {
    let footer = Paragraph::new(Line::from(vec![
        Span::styled("↑↓", Style::default().fg(R_LIGHT_GREY)),
        Span::styled(" navigate • ", Style::default().fg(R_GREY)),
        Span::styled("q", Style::default().fg(R_LIGHT_GREY)),
        Span::styled(" quit", Style::default().fg(R_GREY)),
    ]))
    .alignment(Alignment::Center);

    frame.render_widget(footer, area);
}
