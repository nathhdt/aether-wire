//! logging utilities

use chrono::Local;

use crate::utils::colors::*;

pub enum LogLevel {
    Info,
    Warn,
    Error,
}

/// internal method to generate the formatted log
pub fn log_message(level: LogLevel, prefix: Option<&str>, message: String) {
    let color = match level {
        LogLevel::Info => BLUE,
        LogLevel::Warn => PINK,
        LogLevel::Error => RED,
    };

    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string();

    match prefix {
        Some(prefix) => {
            println!("{color}{timestamp} [{prefix}] {message}{RESET}");
        }
        None => {
            println!("{color}{timestamp} {message}{RESET}");
        }
    }
}

#[macro_export]
macro_rules! info {
    ($prefix:expr, $fmt:expr $(, $arg:expr)* $(,)?) => {
        $crate::utils::logging::log_message(
            $crate::utils::logging::LogLevel::Info,
            Some($prefix),
            format!($fmt $(, $arg)*)
        )
    };
}

#[macro_export]
macro_rules! info_noprefix {
    ($($arg:tt)*) => {
        $crate::utils::logging::log_message(
            $crate::utils::logging::LogLevel::Info,
            None,
            format!($($arg)*)
        )
    };
}

#[macro_export]
macro_rules! info_noprefix_notimestamp {
    ($($arg:tt)*) => {
        println!("{}", format!($($arg)*))
    };
}

#[macro_export]
macro_rules! warn {
    ($prefix:expr, $fmt:expr $(, $arg:expr)* $(,)?) => {
        $crate::utils::logging::log_message(
            $crate::utils::logging::LogLevel::Warn,
            Some($prefix),
            format!($fmt $(, $arg)*)
        )
    };
}

#[macro_export]
macro_rules! warn_noprefix {
    ($($arg:tt)*) => {
        $crate::utils::logging::log_message(
            $crate::utils::logging::LogLevel::Warn,
            None,
            format!($($arg)*)
        )
    };
}

#[macro_export]
macro_rules! error {
    ($prefix:expr, $fmt:expr $(, $arg:expr)* $(,)?) => {
        $crate::utils::logging::log_message(
            $crate::utils::logging::LogLevel::Error,
            Some($prefix),
            format!($fmt $(, $arg)*)
        )
    };
}

#[macro_export]
macro_rules! error_noprefix {
    ($($arg:tt)*) => {
        $crate::utils::logging::log_message(
            $crate::utils::logging::LogLevel::Error,
            None,
            format!($($arg)*)
        )
    };
}

#[macro_export]
macro_rules! bail_error {
    ($prefix:expr, $fmt:expr $(, $arg:expr)* $(,)?) => {{
        let msg = format!($fmt $(, $arg)*);

        $crate::utils::logging::log_message(
            $crate::utils::logging::LogLevel::Error,
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

        $crate::utils::logging::log_message(
            $crate::utils::logging::LogLevel::Error,
            None,
            msg.clone()
        );

        return Err(anyhow::anyhow!(msg));
    }};
}
