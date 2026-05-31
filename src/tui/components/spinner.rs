//! TUI spinner

use std::time::Instant;

pub fn get_spinner_char(start: Instant) -> &'static str {
    match (start.elapsed().as_millis() / 80) % 10 {
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
    }
}
