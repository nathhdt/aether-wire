//! qualify pipeline panel

use crossterm::event::KeyCode;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Style, Stylize},
    symbols,
    text::{Line, Span},
    widgets::{Block, Borders, Padding, Paragraph, Wrap},
};

use crate::tui::input::InputField;
use crate::tui::task::TaskHandle;
use crate::utils::format::colors::{R_DARK_GREY, R_GREY, R_LAVENDER};

#[allow(dead_code)]
pub enum QualifyEvent {
    StepStarted { step: u8, name: String },
    StepDone { step: u8, result: String },
    Done { tref_bps: f64 },
    Error(String),
}

#[allow(dead_code)]
pub enum QualifyPanelState {
    Idle,
    Running { step: u8 },
    Done,
    Error(String),
}

#[allow(dead_code)]
pub struct QualifyInputs {
    pub server: InputField,
    pub port: InputField,
    pub focused: usize,
}

#[allow(dead_code)]
pub struct QualifyPanel {
    pub state: QualifyPanelState,
    pub inputs: QualifyInputs,
    pub task: Option<TaskHandle<QualifyEvent>>,
    pub log: Vec<String>,
}

impl QualifyPanel {
    pub fn new() -> Self {
        Self {
            state: QualifyPanelState::Idle,
            inputs: QualifyInputs {
                server: InputField::new("server", "", "192.168.1.11"),
                port: InputField::new("port", "", "9000"),
                focused: 0,
            },
            task: None,
            log: Vec::new(),
        }
    }

    pub fn on_key(&mut self, _key: KeyCode) {
        // to-do
    }

    pub fn poll_task(&mut self) {
        // to-do
    }

    pub fn draw(&self, frame: &mut Frame, area: Rect) {
        let text = vec![
            Line::from(""),
            Line::from(Span::styled(
                "not implemented yet",
                Style::default().fg(R_GREY),
            )),
        ];

        let widget = Paragraph::new(text).wrap(Wrap { trim: true }).block(
            Block::default()
                .borders(Borders::ALL)
                .border_set(symbols::border::ROUNDED)
                .title(" qualify ".fg(R_LAVENDER))
                .border_style(Style::default().fg(R_DARK_GREY))
                .padding(Padding::new(2, 2, 0, 0)),
        );

        frame.render_widget(widget, area);
    }
}
