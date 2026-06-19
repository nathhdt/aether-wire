//! kernel modules utilities module

use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug)]
pub enum KernelModuleError {
    Io(std::io::Error),
}

impl fmt::Display for KernelModuleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(err) => write!(f, "{err}"),
        }
    }
}

impl std::error::Error for KernelModuleError {}

impl From<std::io::Error> for KernelModuleError {
    fn from(err: std::io::Error) -> Self {
        KernelModuleError::Io(err)
    }
}

/// checks whether a module is currently listed in /proc/modules
pub fn is_in_proc_modules(module: &str) -> Result<bool, KernelModuleError> {
    let file = File::open("/proc/modules")?;
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = line?;

        if line
            .split_whitespace()
            .next()
            .is_some_and(|name| name == module)
        {
            return Ok(true);
        }
    }

    Ok(false)
}
