//! shared input field widget used by all panels

use crossterm::event::KeyCode;
use ratatui::{
    Frame,
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::utils::format::colors::{R_DARK_GREY, R_LIGHT_GREY, R_PINK};

/// single-line text input field
pub struct InputField {
    pub label: &'static str,
    pub value: String,
    pub placeholder: &'static str,
    pub focused: bool,
}

impl InputField {
    pub fn new(label: &'static str, default: &str, placeholder: &'static str) -> Self {
        Self {
            label,
            value: default.to_string(),
            placeholder,
            focused: false,
        }
    }

    pub fn on_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char(c) => {
                self.value.push(c);
            }
            KeyCode::Backspace => {
                self.value.pop();
            }
            _ => {}
        }
    }

    pub fn draw(&self, frame: &mut Frame, area: Rect) {
        let active_color = if self.focused { R_PINK } else { R_LIGHT_GREY };

        let label = Span::styled(
            format!("{}: ", self.label),
            Style::default().fg(active_color),
        );

        let content = if self.value.is_empty() {
            let mut chars = self.placeholder.chars();

            let first = chars.next().unwrap_or(' ');
            let rest: String = chars.collect();

            if self.focused {
                vec![
                    label,
                    Span::styled(
                        first.to_string(),
                        Style::default().fg(R_DARK_GREY).bg(R_PINK),
                    ),
                    Span::styled(rest, Style::default().fg(R_DARK_GREY)),
                ]
            } else {
                vec![
                    label,
                    Span::styled(self.placeholder, Style::default().fg(R_DARK_GREY)),
                ]
            }
        } else {
            vec![
                label,
                Span::styled(
                    if self.focused {
                        format!("{}█", self.value)
                    } else {
                        self.value.clone()
                    },
                    Style::default().fg(active_color),
                ),
            ]
        };

        let widget = Paragraph::new(Line::from(content));

        frame.render_widget(widget, area);
    }
}
