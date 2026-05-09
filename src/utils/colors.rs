//! ANSI terminal colors and style attributes

pub const GREEN: &str = "\x1b[38;2;0;240;150m";
pub const CYAN: &str = "\x1b[96m";
pub const YELLOW: &str = "\x1b[38;2;255;235;140m";
pub const RED: &str = "\x1b[91m";

pub const BOLD: &str = "\x1b[1m";
pub const NO_BOLD: &str = "\x1b[22m";
pub const RESET: &str = "\x1b[0m";
