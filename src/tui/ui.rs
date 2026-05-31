//! TUI top level drawing

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
};

use crate::tui::app::App;
use crate::tui::components::footer::{FooterItem, draw_footer};
use crate::tui::components::header::draw_header;
use crate::tui::components::sidebar::draw_sidebar;

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
    draw_footer_bar(frame, layout[2], app);
}

/// dispatches content rendering to the active panel
fn draw_content(frame: &mut Frame, area: Rect, app: &App) {
    let area = Rect {
        x: area.x + 3,
        y: area.y + 1,
        width: area.width.saturating_sub(2),
        height: area.height.saturating_sub(3),
    };

    app.panels.draw_active(frame, app.selected_menu, area);
}

/// renders footer bar
fn draw_footer_bar(frame: &mut Frame, area: Rect, app: &App) {
    let nav_enabled = !app.panels.active_is_busy(app.selected_menu);

    let mut items = vec![
        FooterItem {
            key: "↑↓",
            label: "menu",
            enabled: nav_enabled,
        },
        FooterItem::new("q", "quit"),
    ];

    items.extend(app.panels.active_footer_items(app.selected_menu));

    draw_footer(frame, area, &items);
}
