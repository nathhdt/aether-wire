//! TUI terminal setup and main loop

use anyhow::Result;
use crossterm::{
    event::{self, Event},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io;
use std::time::Duration;

use crate::tui::app::App;
use crate::tui::ui::draw;
use crate::utils::format::logging::set_tui_mode;

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
