//! TUI application base

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io;
use std::time::Duration;

use crate::tui::panels::Panels;
use crate::tui::ui::draw;
use crate::utils::format::logging::set_tui_mode;

/// menu items
pub const MENU_ITEMS: [&str; 5] = [
    "TCP benchmark",
    "UDP benchmark",
    "qualify",
    "server",
    "about",
];

/// app state
pub struct App {
    pub should_quit: bool,
    pub selected_menu: usize,
    pub panels: Panels,
}

impl App {
    pub fn new() -> Self {
        Self {
            should_quit: false,
            selected_menu: 0,
            panels: Panels::new(),
        }
    }

    // handles keyboard input
    pub fn on_key(&mut self, key: KeyCode) {
        match key {
            // global navigation
            KeyCode::Char('q') => self.should_quit = true,

            KeyCode::Down => {
                if !self.panels.active_is_busy(self.selected_menu) {
                    self.selected_menu = (self.selected_menu + 1) % MENU_ITEMS.len();
                }
            }

            KeyCode::Up => {
                if !self.panels.active_is_busy(self.selected_menu) {
                    if self.selected_menu == 0 {
                        self.selected_menu = MENU_ITEMS.len() - 1;
                    } else {
                        self.selected_menu -= 1;
                    }
                }
            }

            // active panel
            key => self.panels.on_key_active(self.selected_menu, key),
        }
    }
}

/// main TUI loop
pub fn run() -> Result<()> {
    // terminal setup
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let mut app = App::new();

    set_tui_mode(true);

    let result = tui_loop(&mut terminal, &mut app);

    set_tui_mode(false);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn tui_loop(
    terminal: &mut ratatui::Terminal<ratatui::backend::CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> Result<()> {
    loop {
        // poll background task events for active panel
        app.panels.poll_active(app.selected_menu);

        terminal.draw(|frame| {
            draw(frame, app);
        })?;

        // non-blocking poll
        if event::poll(Duration::from_millis(100))?
            && let Event::Key(key) = event::read()?
        {
            app.on_key(key.code);
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}
