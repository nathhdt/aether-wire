//! ordered, visibility-aware list of panel input fields

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

/// input kinds
pub enum InputKind {
    Text(InputField),
    Toggle {
        label: &'static str,
        value: bool,
        focused: bool,
    },
    Separator,
}

/// panel entry
pub struct PanelInputEntry {
    pub key: &'static str,
    pub kind: InputKind,
    pub tui_visible: bool,
}

impl PanelInputEntry {
    pub fn draw(&self, frame: &mut Frame, area: Rect, label_width: usize) {
        match &self.kind {
            InputKind::Text(field) => field.draw(frame, area, label_width),
            InputKind::Toggle {
                label,
                value,
                focused,
            } => {
                let color = if *focused { R_PINK } else { R_MAROON };
                let bullet = if *value { "●" } else { "○" };
                frame.render_widget(
                    Paragraph::new(format!("{bullet} {label}")).style(Style::default().fg(color)),
                    area,
                );
            }
            InputKind::Separator => {}
        }
    }

    pub fn on_key(&mut self, key: KeyCode) {
        match &mut self.kind {
            InputKind::Text(field) => field.on_key(key),
            InputKind::Toggle { value, .. } => {
                if key == KeyCode::Char(' ') {
                    *value = !*value;
                }
            }
            InputKind::Separator => {}
        }
    }

    fn is_focusable(&self) -> bool {
        self.tui_visible && !matches!(self.kind, InputKind::Separator)
    }
}

/// input list
pub struct InputList {
    pub entries: Vec<PanelInputEntry>,
    focused: usize,
}

impl InputList {
    pub fn new(entries: Vec<PanelInputEntry>) -> Self {
        let mut list = Self {
            entries,
            focused: 0,
        };
        list.sync_focus();
        list
    }

    /// returns current text value for the given key
    pub fn get_text(&self, key: &str) -> &str {
        self.entries
            .iter()
            .find(|e| e.key == key)
            .and_then(|e| match &e.kind {
                InputKind::Text(f) => Some(f.value.as_str()),
                _ => None,
            })
            .unwrap_or("")
    }

    /// returns current toggle value
    pub fn get_toggle(&self, key: &str) -> bool {
        self.entries
            .iter()
            .find(|e| e.key == key)
            .and_then(|e| match &e.kind {
                InputKind::Toggle { value, .. } => Some(*value),
                _ => None,
            })
            .unwrap_or(false)
    }

    #[allow(dead_code)]
    pub fn focused_key(&self) -> Option<&'static str> {
        self.entries
            .iter()
            .filter(|e| e.is_focusable())
            .nth(self.focused)
            .map(|e| e.key)
    }

    pub fn focused_index(&self) -> usize {
        self.focused
    }

    pub fn visible_entries(&self) -> Vec<&PanelInputEntry> {
        self.entries.iter().filter(|e| e.tui_visible).collect()
    }

    pub fn visible_count(&self) -> usize {
        self.entries.iter().filter(|e| e.tui_visible).count()
    }

    pub fn focusable_count(&self) -> usize {
        self.entries.iter().filter(|e| e.is_focusable()).count()
    }

    pub fn draw(&self, frame: &mut Frame, areas: &[Rect], label_width: usize) {
        for (entry, area) in self.visible_entries().iter().zip(areas.iter()) {
            entry.draw(frame, *area, label_width);
        }
    }

    /// keyboard navigation
    pub fn on_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Tab => self.tab_next(),
            key => {
                let focused = self.focused;
                let mut count = 0;
                for entry in &mut self.entries {
                    if !entry.is_focusable() {
                        continue;
                    }
                    if count == focused {
                        entry.on_key(key);
                        break;
                    }
                    count += 1;
                }
            }
        }
    }

    fn tab_next(&mut self) {
        let count = self.focusable_count();
        if count == 0 {
            return;
        }
        self.focused = (self.focused + 1) % count;
        self.sync_focus();
    }

    /// propagates focused flag to each entry
    fn sync_focus(&mut self) {
        // clamp in case focusable count shrank (e.g. after a visibility change)
        let count = self.focusable_count();
        if count > 0 && self.focused >= count {
            self.focused = count - 1;
        }

        let focused = self.focused;
        let mut focusable_idx = 0;

        for entry in &mut self.entries {
            let is_focused = entry.is_focusable() && focusable_idx == focused;

            match &mut entry.kind {
                InputKind::Text(f) => {
                    f.focused = is_focused;
                }
                InputKind::Toggle { focused: f, .. } => {
                    *f = is_focused;
                }
                InputKind::Separator => {}
            }

            if entry.is_focusable() {
                focusable_idx += 1;
            }
        }
    }
}

/// text input entry constructor
pub fn text(
    key: &'static str,
    default: &str,
    placeholder: &'static str,
    visible: bool,
) -> PanelInputEntry {
    PanelInputEntry {
        key,
        kind: InputKind::Text(InputField::new(key, default, placeholder)),
        tui_visible: visible,
    }
}

/// text input entry constructor with display label distinct from key
#[allow(dead_code)]
pub fn text_labeled(
    key: &'static str,
    label: &'static str,
    default: &str,
    placeholder: &'static str,
    visible: bool,
) -> PanelInputEntry {
    PanelInputEntry {
        key,
        kind: InputKind::Text(InputField::new(label, default, placeholder)),
        tui_visible: visible,
    }
}

/// boolean toggle entry constructor
pub fn toggle(
    key: &'static str,
    label: &'static str,
    default: bool,
    visible: bool,
) -> PanelInputEntry {
    PanelInputEntry {
        key,
        kind: InputKind::Toggle {
            label,
            value: default,
            focused: false,
        },
        tui_visible: visible,
    }
}

/// empty row
pub fn separator() -> PanelInputEntry {
    PanelInputEntry {
        key: "",
        kind: InputKind::Separator,
        tui_visible: true,
    }
}
