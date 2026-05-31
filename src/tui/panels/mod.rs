//! TUI panels

pub mod about;
pub mod qualify;
pub mod server;
pub mod tcp_benchmark;
pub mod udp_benchmark;

use crossterm::event::KeyCode;
use ratatui::{Frame, layout::Rect};

use about::AboutPanel;
use qualify::QualifyPanel;
use server::ServerPanel;
use tcp_benchmark::BenchmarkTcpPanel;
use udp_benchmark::BenchmarkUdpPanel;

use crate::tui::components::footer::FooterItem;

/// panel instances
pub struct Panels {
    pub benchmark_tcp: BenchmarkTcpPanel,
    pub benchmark_udp: BenchmarkUdpPanel,
    pub qualify: QualifyPanel,
    pub server: ServerPanel,
    pub about: AboutPanel,
}

impl Panels {
    pub fn new() -> Self {
        Self {
            benchmark_tcp: BenchmarkTcpPanel::new(),
            benchmark_udp: BenchmarkUdpPanel::new(),
            qualify: QualifyPanel::new(),
            server: ServerPanel::new(),
            about: AboutPanel::new(),
        }
    }

    /// polls active panel's events
    pub fn poll_active(&mut self, selected: usize) {
        match selected {
            0 => self.benchmark_tcp.poll_task(),
            1 => self.benchmark_udp.poll_task(),
            2 => self.qualify.poll_task(),
            3 => self.server.poll_task(),
            4 => {}
            _ => {}
        }
    }

    pub fn on_key_active(&mut self, selected: usize, key: KeyCode) {
        match selected {
            0 => self.benchmark_tcp.on_key(key),
            1 => self.benchmark_udp.on_key(key),
            2 => self.qualify.on_key(key),
            3 => self.server.on_key(key),
            4 => {}
            _ => {}
        }
    }

    /// draws active panel
    pub fn draw_active(&self, frame: &mut Frame, selected: usize, area: Rect) {
        match selected {
            0 => self.benchmark_tcp.draw(frame, area),
            1 => self.benchmark_udp.draw(frame, area),
            2 => self.qualify.draw(frame, area),
            3 => self.server.draw(frame, area),
            4 => self.about.draw(frame, area),
            _ => {}
        }
    }

    /// true if the active panel has a running background task
    pub fn active_is_busy(&self, selected: usize) -> bool {
        match selected {
            0 => self.benchmark_tcp.is_busy(),
            3 => self.server.is_busy(),
            _ => false,
        }
    }

    /// returns the footer items for the active panel
    pub fn active_footer_items(&self, selected: usize) -> Vec<FooterItem> {
        match selected {
            0 => self.benchmark_tcp.footer_items(),
            1 => self.benchmark_udp.footer_items(),
            2 => self.qualify.footer_items(),
            3 => self.server.footer_items(),
            _ => vec![],
        }
    }
}

/// panels that contribute footer items implement this
pub trait PanelFooter {
    fn footer_items(&self) -> Vec<FooterItem>;
}
