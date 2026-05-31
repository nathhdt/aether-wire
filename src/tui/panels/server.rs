//! server panel

use std::net::Ipv4Addr;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, mpsc};
use std::thread;
use std::time::Instant;

use crossterm::event::KeyCode;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Padding, Paragraph},
};

use crate::server::{self, ServerParameters, ServerTuiEvent};
use crate::tui::components::footer::FooterItem;
use crate::tui::components::spinner::get_spinner_char;
use crate::tui::format::tcp_benchmark_result::format_tcp_result;
use crate::tui::format::udp_benchmark_result::format_udp_result;
use crate::tui::input::{InputList, separator, text, toggle};
use crate::tui::panels::PanelFooter;
use crate::tui::task::TaskHandle;
use crate::utils::format::colors::{R_BLUE, R_GREY, R_LAVENDER, R_PINK, R_RED};
use crate::utils::parser;

pub enum ServerPanelState {
    Idle,
    Running { addr: String },
    Error(String),
}

pub struct ServerPanel {
    pub state: ServerPanelState,
    pub inputs: InputList,
    pub task: Option<TaskHandle<ServerTuiEvent>>,
    pub log: Vec<String>,
    pub session_in_progress: bool,
    pub session_start: Option<Instant>,
}

impl PanelFooter for ServerPanel {
    fn footer_items(&self) -> Vec<FooterItem> {
        match &self.state {
            ServerPanelState::Idle => vec![
                FooterItem::new("enter", "start"),
                FooterItem::new("tab", "switch field"),
                FooterItem::new("space", "toggle"),
            ],
            ServerPanelState::Running { .. } => vec![FooterItem::new("esc", "stop server")],
            ServerPanelState::Error(_) => vec![FooterItem::new("esc", "back to idle")],
        }
    }
}

impl ServerPanel {
    pub fn new() -> Self {
        Self {
            state: ServerPanelState::Idle,
            inputs: InputList::new(vec![
                text("bind", "", "0.0.0.0", true),
                text("port", "", "9000", true),
                separator(),
                text("UDP recv. buffer", "16M", "16M", true),
                separator(),
                toggle("once", "once", false, false),
            ]),
            task: None,
            log: Vec::new(),
            session_in_progress: false,
            session_start: None,
        }
    }

    pub fn is_busy(&self) -> bool {
        matches!(self.state, ServerPanelState::Running { .. })
    }

    pub fn on_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Enter => {
                if let ServerPanelState::Idle = self.state {
                    self.start();
                }
            }

            KeyCode::Esc => {
                if matches!(self.state, ServerPanelState::Running { .. }) {
                    self.stop();
                } else if matches!(self.state, ServerPanelState::Error(_)) {
                    self.state = ServerPanelState::Idle;
                    self.log.clear();
                }
            }

            key => {
                if matches!(self.state, ServerPanelState::Idle) {
                    // validate input before delegating
                    let focused_key = self
                        .inputs
                        .visible_entries()
                        .get(self.inputs.focused_index())
                        .map(|e| e.key)
                        .unwrap_or("");

                    let valid = match (focused_key, key) {
                        ("bind", KeyCode::Char(c)) => {
                            (c.is_ascii_digit() || c == '.' || c == ':')
                                && self.inputs.get_text("bind").len() < 15
                        }
                        ("port", KeyCode::Char(c)) => {
                            c.is_ascii_digit() && self.inputs.get_text("port").len() < 5
                        }
                        _ => true,
                    };

                    if valid {
                        self.inputs.on_key(key);
                    }
                }
            }
        }
    }

    pub fn poll_task(&mut self) {
        let Some(task) = &self.task else {
            return;
        };

        // drain all pending events
        while let Some(event) = task.try_recv() {
            match event {
                ServerTuiEvent::Listening { addr } => {
                    self.state = ServerPanelState::Running { addr };
                }
                ServerTuiEvent::SessionStarted { peer, session_type } => {
                    // insert empty separator if log already contains lines
                    if !self.log.is_empty() {
                        self.log.push(String::new());
                    }
                    self.log.push(format!("+ {peer}  [{session_type}]"));
                    self.session_in_progress = true;
                    self.session_start = Some(Instant::now());
                }
                ServerTuiEvent::TcpSessionResult { stats, is_sender } => {
                    for line in format_tcp_result(&stats, is_sender) {
                        self.log.push(line);
                    }
                }
                ServerTuiEvent::UdpSessionResult(stats) => {
                    for line in format_udp_result(&stats) {
                        self.log.push(line);
                    }
                }
                ServerTuiEvent::SessionEnded { peer } => {
                    self.log.push(format!("> {peer}  done"));
                    self.session_in_progress = false;
                    self.session_start = None;
                }
                ServerTuiEvent::Error(e) => {
                    self.log.push(format!("! {e}"));
                    self.session_in_progress = false;
                    self.session_start = None;
                    if matches!(self.state, ServerPanelState::Running { .. }) {
                        self.state = ServerPanelState::Error(e);
                    }
                }
            }
        }

        // clean up finished thread
        if task.is_finished() {
            self.task = None;
            if matches!(self.state, ServerPanelState::Running { .. }) {
                self.state = ServerPanelState::Idle;
            }
        }
    }

    pub fn draw(&self, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .title(Line::from(" server ").fg(R_LAVENDER).bold())
            .padding(Padding::new(1, 3, 1, 0));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        match &self.state {
            ServerPanelState::Idle => self.draw_idle(frame, inner),
            ServerPanelState::Running { addr } => self.draw_running(frame, inner, addr),
            ServerPanelState::Error(e) => self.draw_error(frame, inner, e),
        }
    }

    fn draw_idle(&self, frame: &mut Frame, area: Rect) {
        let n = self.inputs.visible_count();

        let mut constraints: Vec<Constraint> = (0..n).map(|_| Constraint::Length(1)).collect();
        constraints.push(Constraint::Min(0));

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(area);

        self.inputs.draw(frame, &chunks[..n], 18);
    }

    fn draw_running(&self, frame: &mut Frame, area: Rect, addr: &str) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Min(1),
            ])
            .split(area);

        frame.render_widget(
            Paragraph::new(format!("* listening on {addr}"))
                .style(Style::default().fg(R_PINK).add_modifier(Modifier::BOLD)),
            chunks[0],
        );

        // animated dots
        let session_line = if self.session_in_progress {
            let spinner = self.session_start.map(get_spinner_char).unwrap_or("⠋");

            Paragraph::new(format!("{spinner} session in progress"))
                .style(Style::default().fg(R_BLUE))
        } else {
            Paragraph::new("  waiting for connection").style(Style::default().fg(R_GREY))
        };
        frame.render_widget(session_line, chunks[1]);

        let log_height = chunks[3].height as usize;
        let skip = self.log.len().saturating_sub(log_height);
        let log_lines: Vec<Line> = self
            .log
            .iter()
            .skip(skip)
            .map(|s| {
                let color = if s.starts_with('!') { R_PINK } else { R_GREY };
                Line::from(Span::styled(s.as_str(), Style::default().fg(color)))
            })
            .collect();

        frame.render_widget(Paragraph::new(log_lines), chunks[3]);
    }

    fn draw_error(&self, frame: &mut Frame, area: Rect, err: &str) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Min(1),
            ])
            .split(area);

        frame.render_widget(
            Paragraph::new("error").style(Style::default().fg(R_RED).add_modifier(Modifier::BOLD)),
            chunks[0],
        );

        frame.render_widget(
            Paragraph::new(err).style(Style::default().fg(R_GREY)),
            chunks[1],
        );

        let log_height = chunks[3].height as usize;
        let skip = self.log.len().saturating_sub(log_height);
        let log_lines: Vec<Line> = self
            .log
            .iter()
            .skip(skip)
            .map(|s| Line::from(Span::styled(s.as_str(), Style::default().fg(R_GREY))))
            .collect();
        frame.render_widget(Paragraph::new(log_lines), chunks[3]);
    }

    fn start(&mut self) {
        // clear previous session history on clean start
        self.log.clear();

        let bind: Ipv4Addr = match self.inputs.get_text("bind").trim().parse() {
            Ok(a) => a,
            Err(_) => {
                self.state = ServerPanelState::Error(format!(
                    "invalid bind address: {}",
                    self.inputs.get_text("bind")
                ));
                return;
            }
        };

        let port: u16 = match self.inputs.get_text("port").trim().parse() {
            Ok(p) => p,
            Err(_) => {
                self.state = ServerPanelState::Error(format!(
                    "invalid port: {}",
                    self.inputs.get_text("port")
                ));
                return;
            }
        };

        let udp_recv_buffer =
            match parser::parse_udp_buf_mem_size(self.inputs.get_text("UDP recv. buffer")) {
                Ok(v) => v,
                Err(e) => {
                    self.state = ServerPanelState::Error(format!(
                        "invalid UDP recv. buffer value: {} ({e})",
                        self.inputs.get_text("UDP recv. buffer")
                    ));
                    return;
                }
            };

        let stop = Arc::new(AtomicBool::new(false));
        let (tx, rx) = mpsc::channel::<ServerTuiEvent>();

        let params = ServerParameters {
            bind,
            port,
            udp_recv_buffer,
            once: self.inputs.get_toggle("once"),
        };

        let stop_clone = Arc::clone(&stop);
        let tx_clone = tx.clone();

        let thread = thread::spawn(move || {
            if let Err(e) = server::run_tui(params, tx_clone, stop_clone) {
                let _ = tx.send(ServerTuiEvent::Error(e.to_string()));
            }
        });

        self.task = Some(TaskHandle::new(stop, thread, rx));
    }

    fn stop(&mut self) {
        if let Some(task) = &self.task {
            task.stop();
        }
        self.log
            .push("· stopping after current session...".to_string());
    }
}
