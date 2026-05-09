//! TUI application base

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io;

use crate::tui::ui::draw;

/// menu structure
pub const MENU_ITEMS: [&str; 4] = ["benchmark", "qualify", "server", "about"];

/// application state
pub struct App {
    pub should_quit: bool,
    pub selected_menu: usize,
}

impl App {
    pub fn new() -> Self {
        Self {
            should_quit: false,
            selected_menu: 0,
        }
    }

    // handles keyboard input
    pub fn on_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('q') => {
                self.should_quit = true;
            }

            KeyCode::Down => {
                self.selected_menu = (self.selected_menu + 1) % MENU_ITEMS.len();
            }

            KeyCode::Up => {
                if self.selected_menu == 0 {
                    self.selected_menu = MENU_ITEMS.len() - 1;
                } else {
                    self.selected_menu -= 1;
                }
            }

            _ => {}
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

    loop {
        terminal.draw(|frame| {
            draw(frame, &app);
        })?;

        if let Event::Key(key) = event::read()? {
            app.on_key(key.code);
        }

        if app.should_quit {
            break;
        }
    }

    // terminal cleanup
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}
