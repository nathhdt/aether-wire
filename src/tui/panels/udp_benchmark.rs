//! UDP benchmark panel

use crossterm::event::KeyCode;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Padding, Paragraph},
};
use std::net::Ipv4Addr;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, mpsc};
use std::thread;
use std::time::{Duration, Instant};

use crate::client::benchmark::client::{UdpBenchmarkParameters, run_udp_silent};
use crate::protocol::stats::UdpStreamStats;
use crate::tui::components::footer::FooterItem;
use crate::tui::components::progress_bar::ProgressBar;
use crate::tui::components::spinner::get_spinner_char;
use crate::tui::format::udp_benchmark_result::format_udp_benchmark_result;
use crate::tui::input::{InputList, separator, text};
use crate::tui::panels::PanelFooter;
use crate::tui::task::TaskHandle;
use crate::utils::format::colors::{R_BLUE, R_GREY, R_LAVENDER, R_PINK, R_RED};
use crate::utils::parser;

pub enum UdpBenchmarkEvent {
    Done {
        client: Vec<UdpStreamStats>,
        server: Vec<UdpStreamStats>,
    },
    Error(String),
}

pub enum UdpBenchmarkPanelState {
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

pub struct UdpBenchmarkPanel {
    pub state: UdpBenchmarkPanelState,
    pub inputs: InputList,
    pub task: Option<TaskHandle<UdpBenchmarkEvent>>,
    pub log: Vec<String>,
}

impl PanelFooter for UdpBenchmarkPanel {
    fn footer_items(&self) -> Vec<FooterItem> {
        match &self.state {
            UdpBenchmarkPanelState::Idle => vec![
                FooterItem::new("enter", "run"),
                FooterItem::new("tab", "switch field"),
            ],
            UdpBenchmarkPanelState::Running { .. } => vec![],
            UdpBenchmarkPanelState::Done { .. } => {
                vec![
                    FooterItem::new("esc", "back"),
                    FooterItem::new("r", "retry"),
                ]
            }
            UdpBenchmarkPanelState::Error(_) => {
                vec![FooterItem::new("esc", "back")]
            }
        }
    }
}

impl UdpBenchmarkPanel {
    pub fn new() -> Self {
        Self {
            state: UdpBenchmarkPanelState::Idle,
            inputs: InputList::new(vec![
                text("server", "", "192.168.1.11", true),
                text("port", "", "9000", true),
                separator(),
                text("time", "10s", "10s", true),
                text("stream count", "1", "1", true),
                text("bandwidth", "", "10M", true),
                text("payload length", "1400", "1400", true),
            ]),
            task: None,
            log: Vec::new(),
        }
    }

    pub fn is_busy(&self) -> bool {
        matches!(self.state, UdpBenchmarkPanelState::Running { .. })
    }

    pub fn on_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Enter => {
                if let UdpBenchmarkPanelState::Idle = self.state {
                    self.start();
                }
            }

            KeyCode::Esc => {
                if matches!(
                    self.state,
                    UdpBenchmarkPanelState::Done { .. } | UdpBenchmarkPanelState::Error(_)
                ) {
                    self.state = UdpBenchmarkPanelState::Idle;
                    self.log.clear();
                }
            }

            KeyCode::Char('r') => {
                if matches!(self.state, UdpBenchmarkPanelState::Done { .. }) {
                    self.start();
                }
            }

            key => {
                if matches!(self.state, UdpBenchmarkPanelState::Idle) {
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
                        ("payload length", KeyCode::Char(c)) => {
                            c.is_ascii_digit() && self.inputs.get_text("payload length").len() < 4
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
                UdpBenchmarkEvent::Done { client, server } => {
                    let (target_server, target_port) =
                        if let UdpBenchmarkPanelState::Running { server, port, .. } = self.state {
                            (server, port)
                        } else {
                            unreachable!()
                        };

                    // format results into log
                    self.log.push("> sender (client)".to_string());
                    for line in format_udp_benchmark_result(&client, true) {
                        self.log.push(line);
                    }
                    self.log.push(String::new());
                    self.log.push("> receiver (server)".to_string());
                    for line in format_udp_benchmark_result(&server, false) {
                        self.log.push(line);
                    }
                    self.state = UdpBenchmarkPanelState::Done {
                        server: target_server,
                        port: target_port,
                    };
                }
                UdpBenchmarkEvent::Error(e) => {
                    self.log.push(format!("! {e}"));
                    self.state = UdpBenchmarkPanelState::Error(e);
                }
            }
        }

        // clean up finished thread
        if task.is_finished() {
            self.task = None;
            if matches!(self.state, UdpBenchmarkPanelState::Running { .. }) {
                // thread finished without sending an event, should not happen
                self.state = UdpBenchmarkPanelState::Idle;
            }
        }
    }

    pub fn draw(&self, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .title(Line::from(" UDP benchmark ").fg(R_LAVENDER).bold())
            .padding(Padding::new(1, 3, 1, 0));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        match &self.state {
            UdpBenchmarkPanelState::Idle => self.draw_idle(frame, inner),
            UdpBenchmarkPanelState::Running {
                start,
                duration,
                server,
                port,
            } => self.draw_running(frame, inner, *start, *duration, *server, *port),
            UdpBenchmarkPanelState::Done { server, port } => {
                self.draw_results(frame, inner, *server, *port)
            }
            UdpBenchmarkPanelState::Error(e) => self.draw_error(frame, inner, e),
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

        self.inputs.draw(frame, &chunks[..n], 17);
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

        let status_text = if elapsed >= duration {
            format!("{spinner} benchmark done, waiting for server answer")
        } else {
            format!("{spinner} session in progress")
        };

        frame.render_widget(
            Paragraph::new(status_text).style(Style::default().fg(R_BLUE)),
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
                        UdpBenchmarkPanelState::Error(format!("invalid server address: {e}"));
                    return;
                }
            };

        // port
        let port: u16 = match self.inputs.get_text("port").trim().parse() {
            Ok(p) => p,
            Err(_) => {
                self.state = UdpBenchmarkPanelState::Error(format!(
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
                self.state = UdpBenchmarkPanelState::Error(format!("invalid time: {e}"));
                return;
            }
        };

        // n_streams
        let n_streams: u16 = match self.inputs.get_text("stream count").trim().parse::<u16>() {
            Ok(n) if (1..=32).contains(&n) => n,
            Ok(_) => {
                self.state =
                    UdpBenchmarkPanelState::Error("streams must be between 1 and 32".into());
                return;
            }
            Err(_) => {
                self.state = UdpBenchmarkPanelState::Error(format!(
                    "invalid streams: {}",
                    self.inputs.get_text("stream count")
                ));
                return;
            }
        };

        // bandwidth
        let bandwidth = match parser::parse_bandwidth(self.inputs.get_text("bandwidth").trim()) {
            Ok(b) => b,
            Err(e) => {
                self.state = UdpBenchmarkPanelState::Error(format!("invalid bandwidth: {e}"));
                return;
            }
        };

        // payload length
        let payload_size: u16 = match self.inputs.get_text("payload length").trim().parse::<u16>() {
            Ok(l) if l > 0 => l,
            Ok(_) => {
                self.state =
                    UdpBenchmarkPanelState::Error("payload length must be at least 1".into());
                return;
            }
            Err(_) => {
                self.state = UdpBenchmarkPanelState::Error(format!(
                    "invalid payload length: {}",
                    self.inputs.get_text("payload length")
                ));
                return;
            }
        };

        let stop = Arc::new(AtomicBool::new(false));
        let (tx, rx) = mpsc::channel::<UdpBenchmarkEvent>();

        let params = UdpBenchmarkParameters {
            server,
            port,
            duration,
            n_streams,
            bandwidth,
            payload_size,
        };

        let tx_clone = tx.clone();

        let thread = thread::spawn(move || match run_udp_silent(params) {
            Ok((Some(client), Some(server))) => {
                let _ = tx_clone.send(UdpBenchmarkEvent::Done { client, server });
            }
            Ok(_) => {
                let _ = tx_clone.send(UdpBenchmarkEvent::Error(
                    "benchmark returned incomplete stats".into(),
                ));
            }
            Err(e) => {
                let _ = tx_clone.send(UdpBenchmarkEvent::Error(e.to_string()));
            }
        });

        self.state = UdpBenchmarkPanelState::Running {
            start: Instant::now(),
            duration,
            server,
            port,
        };
        self.task = Some(TaskHandle::new(stop, thread, rx));
    }
}
