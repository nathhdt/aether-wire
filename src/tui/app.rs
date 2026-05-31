//! TUI application state

use crossterm::event::KeyCode;

use crate::tui::panels::Panels;

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
