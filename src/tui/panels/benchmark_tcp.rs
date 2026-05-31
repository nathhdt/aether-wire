//! TCP benchmark panel

use crossterm::event::KeyCode;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Padding, Paragraph, Wrap},
};

use crate::protocol::stats::TcpStreamStats;
use crate::tui::components::footer::FooterItem;
use crate::tui::input::InputField;
use crate::tui::panels::PanelFooter;
use crate::tui::task::TaskHandle;
use crate::utils::format::colors::{R_GREY, R_LAVENDER, R_TEXT};

#[allow(dead_code)]
pub enum BenchmarkTcpEvent {
    Done {
        client: Vec<TcpStreamStats>,
        server: Vec<TcpStreamStats>,
    },
    Error(String),
}

#[allow(dead_code)]
pub enum BenchmarkTcpPanelState {
    Idle,
    Running,
    Done,
    Error(String),
}

#[allow(dead_code)]
pub struct BenchmarkTcpInputs {
    pub server: InputField,
    pub port: InputField,
    pub time: InputField,
    pub n_streams: InputField,
    pub focused: usize,
}

#[allow(dead_code)]
pub struct BenchmarkTcpPanel {
    pub state: BenchmarkTcpPanelState,
    pub inputs: BenchmarkTcpInputs,
    pub task: Option<TaskHandle<BenchmarkTcpEvent>>,
    pub results: Option<(Vec<TcpStreamStats>, Vec<TcpStreamStats>)>,
}

impl BenchmarkTcpPanel {
    pub fn new() -> Self {
        Self {
            state: BenchmarkTcpPanelState::Idle,
            inputs: BenchmarkTcpInputs {
                server: InputField::new("server", "", "192.168.1.11"),
                port: InputField::new("port", "", "9000"),
                time: InputField::new("time", "", "10s"),
                n_streams: InputField::new("streams", "1", "1"),
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
        let text = vec![Line::from(Span::styled(
            "not implemented yet",
            Style::default().fg(R_GREY),
        ))];

        let widget = Paragraph::new(text)
            .wrap(Wrap { trim: true })
            .style(Style::default().fg(R_TEXT))
            .block(
                Block::default()
                    .title(Line::from(" TCP benchmark ").fg(R_LAVENDER).bold())
                    .padding(Padding::new(1, 2, 1, 0)),
            );

        frame.render_widget(widget, area);
    }
}

impl PanelFooter for BenchmarkTcpPanel {
    fn footer_items(&self) -> Vec<FooterItem> {
        vec![]
    }
}
