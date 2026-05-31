//! TCP benchmark panel

use std::net::Ipv4Addr;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, mpsc};
use std::thread;
use std::time::{Duration, Instant};

use crossterm::event::KeyCode;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Padding, Paragraph},
};

use crate::client::benchmark::client::{TcpBenchmarkParameters, run_tcp_silent};
use crate::protocol::messages::Direction as BenchmarkDirection;
use crate::protocol::stats::TcpStreamStats;
use crate::tui::components::footer::FooterItem;
use crate::tui::components::progress_bar::ProgressBar;
use crate::tui::components::spinner::get_spinner_char;
use crate::tui::format::tcp_benchmark_result::format_tcp_result;
use crate::tui::input::{InputKind, InputList, PanelInputEntry, separator, text, toggle};
use crate::tui::panels::PanelFooter;
use crate::tui::task::TaskHandle;
use crate::utils::format::colors::{R_BLUE, R_GREY, R_LAVENDER, R_PINK, R_RED};
use crate::utils::parser;

pub enum BenchmarkTcpEvent {
    Done {
        client: Vec<TcpStreamStats>,
        server: Vec<TcpStreamStats>,
    },
    Error(String),
}

pub enum BenchmarkTcpPanelState {
    Idle,
    Running {
        start: Instant,
        duration: Duration,
        server: Ipv4Addr,
        port: u16,
    },
    Done {
        server: Ipv4Addr,
        port: u16,
    },
    Error(String),
}

pub struct BenchmarkTcpPanel {
    pub state: BenchmarkTcpPanelState,
    pub inputs: InputList,
    pub task: Option<TaskHandle<BenchmarkTcpEvent>>,
    pub log: Vec<String>,
}

impl PanelFooter for BenchmarkTcpPanel {
    fn footer_items(&self) -> Vec<FooterItem> {
        match &self.state {
            BenchmarkTcpPanelState::Idle => vec![
                FooterItem::new("enter", "run"),
                FooterItem::new("tab", "switch field"),
                FooterItem::new("space", "toggle"),
            ],
            BenchmarkTcpPanelState::Running { .. } => vec![],
            BenchmarkTcpPanelState::Done { .. } | BenchmarkTcpPanelState::Error(_) => {
                vec![
                    FooterItem::new("esc", "back"),
                    FooterItem::new("r", "retry"),
                ]
            }
        }
    }
}

impl BenchmarkTcpPanel {
    pub fn new() -> Self {
        Self {
            state: BenchmarkTcpPanelState::Idle,
            inputs: InputList::new(vec![
                text("server", "", "192.168.1.11", true),
                text("port", "", "9000", true),
                separator(),
                text("time", "10s", "10s", true),
                text("stream count", "1", "1", true),
                PanelInputEntry {
                    key: "sep_verify",
                    kind: InputKind::Separator,
                    tui_visible: true,
                },
                toggle("verify", "verify integrity", false, true),
            ]),
            task: None,
            log: Vec::new(),
        }
    }

    pub fn is_busy(&self) -> bool {
        matches!(self.state, BenchmarkTcpPanelState::Running { .. })
    }

    pub fn on_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Enter => {
                if let BenchmarkTcpPanelState::Idle = self.state {
                    self.start();
                }
            }

            KeyCode::Esc => {
                if matches!(
                    self.state,
                    BenchmarkTcpPanelState::Done { .. } | BenchmarkTcpPanelState::Error(_)
                ) {
                    self.state = BenchmarkTcpPanelState::Idle;
                    self.log.clear();
                }
            }

            KeyCode::Char('r') => {
                if matches!(
                    self.state,
                    BenchmarkTcpPanelState::Done { .. } | BenchmarkTcpPanelState::Error(_)
                ) {
                    self.start();
                }
            }

            key => {
                if matches!(self.state, BenchmarkTcpPanelState::Idle) {
                    // focused_key via focusable_key() to avoid separator index mismatch
                    let focused_key = self.inputs.focused_key().unwrap_or("");

                    let valid = match (focused_key, key) {
                        ("server", KeyCode::Char(c)) => {
                            (c.is_ascii_digit() || c == '.')
                                && self.inputs.get_text("server").len() < 15
                        }
                        ("port", KeyCode::Char(c)) => {
                            c.is_ascii_digit() && self.inputs.get_text("port").len() < 5
                        }
                        ("stream count", KeyCode::Char(c)) => {
                            c.is_ascii_digit() && self.inputs.get_text("stream count").len() < 2
                        }
                        _ => true,
                    };

                    if valid {
                        self.inputs.on_key(key);
                    }

                    // hide and uncheck verify when streams > 1
                    let multi_stream = self
                        .inputs
                        .get_text("stream count")
                        .trim()
                        .parse::<u16>()
                        .map(|n| n > 1)
                        .unwrap_or(false);

                    for entry in &mut self.inputs.entries {
                        if entry.key == "verify" || entry.key == "sep_verify" {
                            entry.tui_visible = !multi_stream;
                        }
                        if entry.key == "verify"
                            && multi_stream
                            && let InputKind::Toggle { value, .. } = &mut entry.kind
                        {
                            *value = false;
                        }
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
                BenchmarkTcpEvent::Done { client, server } => {
                    let (target_server, target_port) =
                        if let BenchmarkTcpPanelState::Running { server, port, .. } = self.state {
                            (server, port)
                        } else {
                            unreachable!()
                        };

                    // format results into log
                    self.log.push("> sender (client)".to_string());
                    for line in format_tcp_result(&client, true) {
                        self.log.push(line);
                    }
                    self.log.push(String::new());
                    self.log.push("> receiver (server)".to_string());
                    for line in format_tcp_result(&server, false) {
                        self.log.push(line);
                    }
                    self.state = BenchmarkTcpPanelState::Done {
                        server: target_server,
                        port: target_port,
                    };
                }
                BenchmarkTcpEvent::Error(e) => {
                    self.log.push(format!("! {e}"));
                    self.state = BenchmarkTcpPanelState::Error(e);
                }
            }
        }

        // clean up finished thread
        if task.is_finished() {
            self.task = None;
            if matches!(self.state, BenchmarkTcpPanelState::Running { .. }) {
                // thread finished without sending an event, should not happen
                self.state = BenchmarkTcpPanelState::Idle;
            }
        }
    }

    pub fn draw(&self, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .title(Line::from(" TCP benchmark ").fg(R_LAVENDER).bold())
            .padding(Padding::new(1, 3, 1, 0));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        match &self.state {
            BenchmarkTcpPanelState::Idle => self.draw_idle(frame, inner),
            BenchmarkTcpPanelState::Running {
                start,
                duration,
                server,
                port,
            } => self.draw_running(frame, inner, *start, *duration, *server, *port),
            BenchmarkTcpPanelState::Done { server, port } => {
                self.draw_results(frame, inner, *server, *port)
            }
            BenchmarkTcpPanelState::Error(e) => self.draw_error(frame, inner, e),
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

        self.inputs.draw(frame, &chunks[..n], 15);
    }

    fn draw_running(
        &self,
        frame: &mut Frame,
        area: Rect,
        start: Instant,
        duration: Duration,
        server: Ipv4Addr,
        port: u16,
    ) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Min(0),
            ])
            .split(area);

        let spinner = get_spinner_char(start);

        let elapsed = start.elapsed();
        let total = duration;
        let ratio = (elapsed.as_secs_f64() / total.as_secs_f64()).clamp(0.0, 1.0);

        frame.render_widget(
            Paragraph::new(format!("* sending to {server}:{port}"))
                .style(Style::default().fg(R_PINK).add_modifier(Modifier::BOLD)),
            chunks[0],
        );

        frame.render_widget(
            Paragraph::new(format!("{spinner} session in progress"))
                .style(Style::default().fg(R_BLUE)),
            chunks[1],
        );

        // unicode progress bar
        frame.render_widget(
            ProgressBar::new(ratio)
                .style(Style::default().fg(R_LAVENDER))
                .percent_style(Style::default().fg(R_BLUE)),
            chunks[3],
        );
    }

    fn draw_results(&self, frame: &mut Frame, area: Rect, server: Ipv4Addr, port: u16) {
        let log_height = area.height.saturating_sub(2) as usize;
        let skip = self.log.len().saturating_sub(log_height);

        let mut lines: Vec<Line> = Vec::new();

        lines.push(Line::from(Span::styled(
            format!("* benchmark done ({server}:{port})"),
            Style::default().fg(R_PINK).add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));

        // results log
        for s in self.log.iter().skip(skip) {
            let line = if s.is_empty() {
                Line::from("")
            } else if s.starts_with("  ") {
                // stream data row
                Line::from(Span::styled(s.as_str(), Style::default().fg(R_GREY)))
            } else {
                // section label
                Line::from(Span::styled(s.as_str(), Style::default().fg(R_BLUE)))
            };
            lines.push(line);
        }

        frame.render_widget(Paragraph::new(lines), area);
    }

    fn draw_error(&self, frame: &mut Frame, area: Rect, err: &str) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Min(0),
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
    }

    fn start(&mut self) {
        self.log.clear();

        // server address
        let server: Ipv4Addr =
            match parser::parse_server_ipv4(self.inputs.get_text("server").trim()) {
                Ok(a) => a,
                Err(e) => {
                    self.state =
                        BenchmarkTcpPanelState::Error(format!("invalid server address: {e}"));
                    return;
                }
            };

        // port
        let port: u16 = match self.inputs.get_text("port").trim().parse() {
            Ok(p) => p,
            Err(_) => {
                self.state = BenchmarkTcpPanelState::Error(format!(
                    "invalid port: {}",
                    self.inputs.get_text("port")
                ));
                return;
            }
        };

        // duration
        let duration = match parser::parse_duration_min_1s(self.inputs.get_text("time").trim()) {
            Ok(d) => d,
            Err(e) => {
                self.state = BenchmarkTcpPanelState::Error(format!("invalid time: {e}"));
                return;
            }
        };

        // n_streams
        let n_streams: u16 = match self.inputs.get_text("stream count").trim().parse::<u16>() {
            Ok(n) if (1..=32).contains(&n) => n,
            Ok(_) => {
                self.state =
                    BenchmarkTcpPanelState::Error("streams must be between 1 and 32".into());
                return;
            }
            Err(_) => {
                self.state = BenchmarkTcpPanelState::Error(format!(
                    "invalid streams: {}",
                    self.inputs.get_text("stream count")
                ));
                return;
            }
        };

        // verify integrity
        let verify_integrity = if self.inputs.get_toggle("verify") {
            if n_streams > 1 {
                self.state = BenchmarkTcpPanelState::Error(
                    "verify integrity requires a single stream".into(),
                );
                return;
            }
            Some(1024 * 1024 * 1024u64) // 1 GiB default
        } else {
            None
        };

        let stop = Arc::new(AtomicBool::new(false));
        let (tx, rx) = mpsc::channel::<BenchmarkTcpEvent>();

        let params = TcpBenchmarkParameters {
            server,
            port,
            duration,
            n_streams,
            verify_integrity,
            direction: BenchmarkDirection::Default,
        };

        let tx_clone = tx.clone();

        let thread = thread::spawn(move || match run_tcp_silent(params) {
            Ok((Some(client), Some(server))) => {
                let _ = tx_clone.send(BenchmarkTcpEvent::Done { client, server });
            }
            Ok(_) => {
                let _ = tx_clone.send(BenchmarkTcpEvent::Error(
                    "benchmark returned incomplete stats".into(),
                ));
            }
            Err(e) => {
                let _ = tx_clone.send(BenchmarkTcpEvent::Error(e.to_string()));
            }
        });

        self.state = BenchmarkTcpPanelState::Running {
            start: Instant::now(),
            duration,
            server,
            port,
        };
        self.task = Some(TaskHandle::new(stop, thread, rx));
    }
}
