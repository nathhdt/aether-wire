//! shared input field widget used by all panels

use crossterm::event::KeyCode;
use ratatui::{
    Frame,
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::utils::format::colors::{R_MAROON, R_PINK, R_TEXT};

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

    pub fn draw(&self, frame: &mut Frame, area: Rect, label_width: usize) {
        let active_color = if self.focused { R_PINK } else { R_MAROON };

        let label = Span::styled(
            format!(
                "{:<width$}",
                format!("{}:", self.label),
                width = label_width
            ),
            Style::default().fg(active_color),
        );

        let content = if self.value.is_empty() {
            let mut chars = self.placeholder.chars();

            let first = chars.next().unwrap_or(' ');
            let rest: String = chars.collect();

            if self.focused {
                vec![
                    label,
                    Span::styled(first.to_string(), Style::default().fg(R_TEXT).bg(R_PINK)),
                    Span::styled(rest, Style::default().fg(R_TEXT)),
                ]
            } else {
                vec![
                    label,
                    Span::styled(self.placeholder, Style::default().fg(R_TEXT)),
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
