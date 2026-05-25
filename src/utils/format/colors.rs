//! project colors

use ratatui::style::Color;

/// ANSI terminal colors and style attributes
pub const T_BLUE: &str = "\x1b[38;2;137;180;250m";
pub const T_MAROON: &str = "\x1b[38;2;235;160;172m";
pub const T_PINK: &str = "\x1b[38;2;245;194;231m";
pub const T_RED: &str = "\x1b[91m";

pub const T_BOLD: &str = "\x1b[1m";
pub const T_NO_BOLD: &str = "\x1b[22m";
pub const T_RESET: &str = "\x1b[0m";

/// ratatui terminal colors
pub const R_BLUE: Color = Color::Rgb(137, 180, 250);
pub const R_DARK_GREY: Color = Color::Rgb(88, 91, 112);
pub const R_GREY: Color = Color::Rgb(127, 132, 156);
pub const R_LAVENDER: Color = Color::Rgb(180, 190, 254);
pub const R_LIGHT_GREY: Color = Color::Rgb(166, 173, 200);
pub const R_PINK: Color = Color::Rgb(245, 194, 231);
pub const R_RED: Color = Color::Rgb(243, 139, 168);
pub const R_TEXT: Color = Color::Rgb(88, 91, 112);
