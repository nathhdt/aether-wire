//! check types module

pub enum Status {
    Ok,
    Warn,
    Fail,
    Info,
}

impl Status {
    pub fn symbol(&self) -> &'static str {
        match self {
            Status::Ok => "✓",
            Status::Warn => "⚠",
            Status::Fail => "✗",
            Status::Info => " ",
        }
    }
}

pub struct Check {
    pub label: String,
    pub value: String,
    pub status: Status,
    pub note: Option<String>,
}

pub struct InterfaceChecks {
    pub interface: String,
    pub checks: Vec<Check>,
}
