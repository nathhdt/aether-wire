//! output logging utilities module

use libc::{localtime_r, time_t, tm};
use rustix::time::{ClockId, clock_gettime};
use std::fmt;

#[derive(Clone, Copy)]
pub enum LogLevel {
    Info,
    Warn,
    #[allow(dead_code)]
    Error,
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Info => "INFO",
            Self::Warn => "WARN",
            Self::Error => "ERROR",
        })
    }
}

fn local_timestamp() -> String {
    let ts = clock_gettime(ClockId::Realtime);
    let secs = ts.tv_sec as time_t;

    let mut tm = unsafe { std::mem::zeroed::<tm>() };

    if unsafe { localtime_r(&secs, &mut tm) }.is_null() {
        return "0000-00-00 00:00:00.000".into();
    }

    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}.{:03}",
        tm.tm_year + 1900,
        tm.tm_mon + 1,
        tm.tm_mday,
        tm.tm_hour,
        tm.tm_min,
        tm.tm_sec,
        ts.tv_nsec / 1_000_000,
    )
}

pub fn log(level: LogLevel, args: fmt::Arguments<'_>) {
    eprintln!("{} - {} - {}", local_timestamp(), level, args);
}

#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        $crate::utils::output::logging::log(
            $crate::utils::output::logging::LogLevel::Info,
            format_args!($($arg)*),
        )
    };
}

#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {
        $crate::utils::output::logging::log(
            $crate::utils::output::logging::LogLevel::Warn,
            format_args!($($arg)*),
        )
    };
}

#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        $crate::utils::output::logging::log(
            $crate::utils::output::logging::LogLevel::Error,
            format_args!($($arg)*),
        )
    };
}
