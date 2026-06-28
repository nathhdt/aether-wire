//! output utilities module

use libc::{localtime_r, time_t, tm};
use rustix::time::{ClockId, clock_gettime};
use std::{
    fmt,
    io::{self, Write},
};

#[derive(Clone, Copy)]
pub enum Stream {
    #[allow(dead_code)]
    Stdout,
    Stderr,
}

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

pub(crate) fn log(stream: Stream, level: LogLevel, args: fmt::Arguments<'_>) {
    let ts = local_timestamp();

    match stream {
        Stream::Stdout => {
            let mut stdout = io::stdout();
            let _ = writeln!(stdout, "{ts} - {level} - {args}");
        }
        Stream::Stderr => {
            let mut stderr = io::stderr();
            let _ = writeln!(stderr, "{ts} - {level} - {args}");
        }
    }
}

#[macro_export]
macro_rules! out_info {
    ($($arg:tt)*) => {
        $crate::utils::output::log(
            $crate::utils::output::Stream::Stdout,
            $crate::utils::output::LogLevel::Info,
            format_args!($($arg)*),
        )
    };
}

#[macro_export]
macro_rules! out_warn {
    ($($arg:tt)*) => {
        $crate::utils::output::log(
            $crate::utils::output::Stream::Stdout,
            $crate::utils::output::LogLevel::Warn,
            format_args!($($arg)*),
        )
    };
}

#[macro_export]
macro_rules! out_error {
    ($($arg:tt)*) => {
        $crate::utils::output::log(
            $crate::utils::output::Stream::Stdout,
            $crate::utils::output::LogLevel::Error,
            format_args!($($arg)*),
        )
    };
}

#[macro_export]
macro_rules! err_info {
    ($($arg:tt)*) => {
        $crate::utils::output::log(
            $crate::utils::output::Stream::Stderr,
            $crate::utils::output::LogLevel::Info,
            format_args!($($arg)*),
        )
    };
}

#[macro_export]
macro_rules! err_warn {
    ($($arg:tt)*) => {
        $crate::utils::output::log(
            $crate::utils::output::Stream::Stderr,
            $crate::utils::output::LogLevel::Warn,
            format_args!($($arg)*),
        )
    };
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        $crate::utils::output::log(
            $crate::utils::output::Stream::Stderr,
            $crate::utils::output::LogLevel::Error,
            format_args!($($arg)*),
        )
    };
}
