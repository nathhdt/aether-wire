//! network interfaces error utilities module

use std::fmt;

#[derive(Debug)]
pub enum InterfaceError {
    InterfaceNotFound,
    Io(std::io::Error),
}

impl fmt::Display for InterfaceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InterfaceNotFound => write!(f, "interface not found"),
            Self::Io(err) => write!(f, "{err}"),
        }
    }
}

impl std::error::Error for InterfaceError {}

impl From<std::io::Error> for InterfaceError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}
