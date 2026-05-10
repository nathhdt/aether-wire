//! user interfaces for TUI app

use ratatui::widgets::Padding;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

use crate::tui::app::{App, MENU_ITEMS};
use crate::tui::colors::{CYAN, GREEN, MUTED, TEXT};

/// main drawing function
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
        .constraints([Constraint::Length(24), Constraint::Min(10)])
        .split(layout[1]);

    draw_header(frame, layout[0]);
    draw_sidebar(frame, body[0], app);
    draw_content(frame, body[1], app);
    draw_footer(frame, layout[2]);
}

/// ASCII logo
fn draw_header(frame: &mut Frame, area: ratatui::layout::Rect) {
    let logo = vec![
        Line::from(Span::styled(
            r"             _   _                            _          ",
            Style::default().fg(CYAN).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            r"   __ _  ___| |_| |__   ___ _ __    __      _(_)_ __ ___ ",
            Style::default().fg(CYAN).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            r"  / _` |/ _ \ __| '_ \ / _ \ '__|___\ \ /\ / / | '__/ _ \",
            Style::default().fg(CYAN).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            r" | (_| |  __/ |_| | | |  __/ | |_____\ V  V /| | | |  __/",
            Style::default().fg(CYAN).add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![
            Span::styled(
                r"  \__,_|\___|\__|_| |_|\___|_|        \_/\_/ |_|_|  \___|",
                Style::default().fg(CYAN).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("  v{}", env!("CARGO_PKG_VERSION")),
                Style::default().fg(MUTED),
            ),
        ]),
    ];

    let widget = Paragraph::new(logo);

    frame.render_widget(widget, area);
}

/// menu navigation sidebar
fn draw_sidebar(frame: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    let items: Vec<ListItem> = MENU_ITEMS
        .iter()
        .enumerate()
        .map(|(index, item)| {
            let style = if index == app.selected_menu {
                Style::default().fg(CYAN).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(MUTED)
            };

            ListItem::new(Line::from(format!("  {}", item))).style(style)
        })
        .collect();

    let sidebar = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" menu ")
            .border_style(Style::default().fg(CYAN))
            .padding(Padding::top(1)),
    );

    frame.render_widget(sidebar, area);
}

/// main content area
fn draw_content(frame: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    let text = match app.selected_menu {
        // benchmark view
        0 => {
            vec![
                Line::from(""),
                Line::from(Span::styled(
                    "benchmark mode",
                    Style::default().fg(GREEN).add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    "not implemented yet",
                    Style::default().fg(MUTED),
                )),
            ]
        }

        // qualify view
        1 => {
            vec![
                Line::from(""),
                Line::from(Span::styled(
                    "qualify mode",
                    Style::default().fg(GREEN).add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    "not implemented yet",
                    Style::default().fg(MUTED),
                )),
            ]
        }

        // server view
        2 => {
            vec![
                Line::from(""),
                Line::from(Span::styled(
                    "server mode",
                    Style::default().fg(GREEN).add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    "not implemented yet",
                    Style::default().fg(MUTED),
                )),
            ]
        }

        // about view
        _ => {
            vec![
                Line::from(""),
                Line::from(Span::styled(
                    "about",
                    Style::default().fg(CYAN).add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    "aether-wire is a lightweight, native cross-platform \
                    tool built in Rust for measuring end-to-end (E2E) network \
                    performance. it provides two modes: a raw TCP/UDP benchmark \
                    for quick throughput measurement, and a full link qualification \
                    pipeline that automatically profiles a network path \
                    (throughput, MTU, jitter, bufferbloat, packet loss).",
                    Style::default().fg(MUTED),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    "this project is under development.",
                    Style::default().fg(MUTED),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    "copyright (c) nathhdt",
                    Style::default().fg(MUTED),
                )),
            ]
        }
    };

    use ratatui::widgets::Wrap;

    let content = Paragraph::new(text)
        .wrap(Wrap { trim: true })
        .style(Style::default().fg(TEXT))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" view ")
                .border_style(Style::default().fg(CYAN))
                .padding(Padding::new(2, 2, 0, 0)),
        );

    frame.render_widget(content, area);
}

// keyboard shortcuts help
fn draw_footer(frame: &mut Frame, area: ratatui::layout::Rect) {
    let footer = Paragraph::new("↑↓ navigate • q quit")
        .style(Style::default().fg(MUTED))
        .alignment(Alignment::Center);

    frame.render_widget(footer, area);
}
