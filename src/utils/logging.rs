//! logging utilities module

use libc::{localtime_r, time_t, tm};
use rustix::time::{ClockId, clock_gettime};
use std::fmt;

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

pub enum LogLevel {
    Info,
    Warn,
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Info => "INFO",
            Self::Warn => "WARN",
        })
    }
}

pub fn _log(level: LogLevel, args: fmt::Arguments<'_>) {
    let ts = local_timestamp();
    println!("{ts} - {level} - {args}");
}

#[macro_export]
macro_rules! __log_info {
    ($($arg:tt)*) => { $crate::utils::logging::_log($crate::utils::logging::LogLevel::Info,  format_args!($($arg)*)) };
}

#[macro_export]
macro_rules! __log_warn {
    ($($arg:tt)*) => { $crate::utils::logging::_log($crate::utils::logging::LogLevel::Warn,  format_args!($($arg)*)) };
}

pub use crate::__log_info as info;
pub use crate::__log_warn as warn;
