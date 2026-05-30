//! UDP benchmark panel

use crossterm::event::KeyCode;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Style, Stylize},
    symbols,
    text::{Line, Span},
    widgets::{Block, Borders, Padding, Paragraph, Wrap},
};

use crate::protocol::stats::UdpStreamStats;
use crate::tui::input::InputField;
use crate::tui::task::TaskHandle;
use crate::utils::format::colors::{R_DARK_GREY, R_GREY, R_LAVENDER};

#[allow(dead_code)]
pub enum BenchmarkUdpEvent {
    Done {
        client: Vec<UdpStreamStats>,
        server: Vec<UdpStreamStats>,
    },
    Error(String),
}

#[allow(dead_code)]
pub enum BenchmarkUdpPanelState {
    Idle,
    Running,
    Done,
    Error(String),
}

#[allow(dead_code)]
pub struct BenchmarkUdpInputs {
    pub server: InputField,
    pub port: InputField,
    pub time: InputField,
    pub n_streams: InputField,
    pub bandwidth: InputField,
    pub length: InputField,
    pub focused: usize,
}

#[allow(dead_code)]
pub struct BenchmarkUdpPanel {
    pub state: BenchmarkUdpPanelState,
    pub inputs: BenchmarkUdpInputs,
    pub task: Option<TaskHandle<BenchmarkUdpEvent>>,
    pub results: Option<(Vec<UdpStreamStats>, Vec<UdpStreamStats>)>,
}

impl BenchmarkUdpPanel {
    pub fn new() -> Self {
        Self {
            state: BenchmarkUdpPanelState::Idle,
            inputs: BenchmarkUdpInputs {
                server: InputField::new("server", "", "192.168.1.11"),
                port: InputField::new("port", "", "9000"),
                time: InputField::new("time", "", "10s"),
                n_streams: InputField::new("streams", "1", "1"),
                bandwidth: InputField::new("bandwidth", "", "10M"),
                length: InputField::new("length", "1400", "1400"),
                focused: 0,
            },
            task: None,
            results: None,
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
                .title(" UDP benchmark ".fg(R_LAVENDER))
                .border_style(Style::default().fg(R_DARK_GREY))
                .padding(Padding::new(2, 2, 0, 0)),
        );

        frame.render_widget(widget, area);
    }
}
