//! server panel

use std::net::Ipv4Addr;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, mpsc};
use std::thread;
use std::time::Instant;

use crossterm::event::KeyCode;
use ratatui::symbols;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Padding, Paragraph},
};

use crate::server::{self, ServerParameters, ServerTuiEvent};
use crate::tui::input_list::{InputList, separator, text, toggle};
use crate::tui::task::TaskHandle;
use crate::utils::format::colors::{R_BLUE, R_GREY, R_LAVENDER, R_LIGHT_GREY, R_PINK, R_RED};
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
                toggle("once", "once", false, true),
            ]),
            task: None,
            log: Vec::new(),
            session_in_progress: false,
            session_start: None,
        }
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
                        _ => true, // backspace, arrows, tab, space on toggle
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
                    for line in crate::tui::format::format_tcp_result_lines(&stats, is_sender) {
                        self.log.push(line);
                    }
                }
                ServerTuiEvent::UdpSessionResult(stats) => {
                    for line in crate::tui::format::format_udp_result_lines(&stats) {
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
            .borders(Borders::ALL)
            .border_set(symbols::border::ROUNDED)
            .title(" server ".fg(R_LAVENDER))
            .border_style(Style::default().fg(R_BLUE))
            .padding(Padding::new(2, 2, 1, 0));

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
        constraints.push(Constraint::Min(0)); // spacer
        constraints.push(Constraint::Length(1)); // hint

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(area);

        self.inputs.draw(frame, &chunks[..n], 18);

        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled("enter", Style::default().fg(R_LIGHT_GREY)),
                Span::styled(" start • ", Style::default().fg(R_GREY)),
                Span::styled("tab", Style::default().fg(R_LIGHT_GREY)),
                Span::styled(" switch field • ", Style::default().fg(R_GREY)),
                Span::styled("space", Style::default().fg(R_LIGHT_GREY)),
                Span::styled(" toggle selection", Style::default().fg(R_GREY)),
            ])),
            chunks[n + 1],
        );
    }

    fn draw_running(&self, frame: &mut Frame, area: Rect, addr: &str) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Min(1),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .split(area);

        frame.render_widget(
            Paragraph::new(format!("* listening on {addr}"))
                .style(Style::default().fg(R_PINK).add_modifier(Modifier::BOLD)),
            chunks[0],
        );

        // animated dots
        let session_line = if self.session_in_progress {
            let spinner = match self
                .session_start
                .map(|t| (t.elapsed().as_millis() / 80) % 10)
                .unwrap_or(0)
            {
                0 => "⠋",
                1 => "⠙",
                2 => "⠹",
                3 => "⠸",
                4 => "⠼",
                5 => "⠴",
                6 => "⠦",
                7 => "⠧",
                8 => "⠇",
                _ => "⠏",
            };

            Paragraph::new(format!("  {spinner} session in progress"))
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

        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled("esc", Style::default().fg(R_LIGHT_GREY)),
                Span::styled(" stop server", Style::default().fg(R_GREY)),
            ])),
            chunks[5],
        );
    }

    fn draw_error(&self, frame: &mut Frame, area: Rect, err: &str) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Min(1),
                Constraint::Length(1),
            ])
            .split(area);

        frame.render_widget(
            Paragraph::new("server error")
                .style(Style::default().fg(R_RED).add_modifier(Modifier::BOLD)),
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

        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled("esc", Style::default().fg(R_LIGHT_GREY)),
                Span::styled(" back to idle", Style::default().fg(R_GREY)),
            ])),
            chunks[4],
        );
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
        // state transitions when event arrives in poll_task
    }

    fn stop(&mut self) {
        if let Some(task) = &self.task {
            task.stop();
        }
        self.log
            .push("· stopping after current session...".to_string());
    }
}
