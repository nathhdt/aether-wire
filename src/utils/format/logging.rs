//! logging utilities

use chrono::Local;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::utils::format::colors::*;

pub enum LogLevel {
    Info,
    Warn,
    Error,
}

/// set to true while the TUI is active to suppress stdout logging
static TUI_MODE: AtomicBool = AtomicBool::new(false);

pub fn set_tui_mode(enabled: bool) {
    TUI_MODE.store(enabled, Ordering::Relaxed);
}

pub fn is_tui_mode() -> bool {
    TUI_MODE.load(Ordering::Relaxed)
}

/// internal method to generate formatted log
pub fn log_message(level: LogLevel, prefix: Option<&str>, message: String) {
    if TUI_MODE.load(Ordering::Relaxed) {
        return;
    }

    let color = match level {
        LogLevel::Info => T_BLUE,
        LogLevel::Warn => T_PINK,
        LogLevel::Error => T_RED,
    };

    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string();

    match prefix {
        Some(prefix) => {
            println!("{color}{timestamp} [{prefix}] {message}{T_RESET}");
        }
        None => {
            println!("{color}{timestamp} {message}{T_RESET}");
        }
    }
}

#[macro_export]
macro_rules! info {
    ($prefix:expr, $fmt:expr $(, $arg:expr)* $(,)?) => {
        $crate::utils::format::logging::log_message(
            $crate::utils::format::logging::LogLevel::Info,
            Some($prefix),
            format!($fmt $(, $arg)*)
        )
    };
}

#[macro_export]
macro_rules! info_noprefix {
    ($($arg:tt)*) => {
        $crate::utils::format::logging::log_message(
            $crate::utils::format::logging::LogLevel::Info,
            None,
            format!($($arg)*)
        )
    };
}

#[macro_export]
macro_rules! info_noprefix_notimestamp {
    ($($arg:tt)*) => {
        if !$crate::utils::format::logging::is_tui_mode() {
            println!("{}", format!($($arg)*))
        }
    };
}

#[macro_export]
macro_rules! warn {
    ($prefix:expr, $fmt:expr $(, $arg:expr)* $(,)?) => {
        $crate::utils::format::logging::log_message(
            $crate::utils::format::logging::LogLevel::Warn,
            Some($prefix),
            format!($fmt $(, $arg)*)
        )
    };
}

#[macro_export]
macro_rules! warn_noprefix {
    ($($arg:tt)*) => {
        $crate::utils::format::logging::log_message(
            $crate::utils::format::logging::LogLevel::Warn,
            None,
            format!($($arg)*)
        )
    };
}

#[macro_export]
macro_rules! error {
    ($prefix:expr, $fmt:expr $(, $arg:expr)* $(,)?) => {
        $crate::utils::format::logging::log_message(
            $crate::utils::format::logging::LogLevel::Error,
            Some($prefix),
            format!($fmt $(, $arg)*)
        )
    };
}

#[macro_export]
macro_rules! error_noprefix {
    ($($arg:tt)*) => {
        $crate::utils::format::logging::log_message(
            $crate::utils::format::logging::LogLevel::Error,
            None,
            format!($($arg)*)
        )
    };
}

#[macro_export]
macro_rules! bail_error {
    ($prefix:expr, $fmt:expr $(, $arg:expr)* $(,)?) => {{
        let msg = format!($fmt $(, $arg)*);

        $crate::utils::format::logging::log_message(
            $crate::utils::format::logging::LogLevel::Error,
            Some($prefix),
            msg.clone()
        );

        return Err(anyhow::anyhow!(msg));
    }};
}

#[macro_export]
macro_rules! bail_error_noprefix {
    ($($arg:tt)*) => {{
        let msg = format!($($arg)*);

        $crate::utils::format::logging::log_message(
            $crate::utils::format::logging::LogLevel::Error,
            None,
            msg.clone()
        );

        return Err(anyhow::anyhow!(msg));
    }};
}
