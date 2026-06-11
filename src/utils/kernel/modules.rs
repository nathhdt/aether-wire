//! kernel modules utilities module

use std::fs::File;
use std::io::{BufRead, BufReader};

#[allow(unused)]
#[derive(Debug)]
pub enum KernelModuleError {
    Io(std::io::Error),
}

impl From<std::io::Error> for KernelModuleError {
    fn from(err: std::io::Error) -> Self {
        KernelModuleError::Io(err)
    }
}

/// checks whether a module is currently listed in /proc/modules
fn is_in_proc_modules(module: &str) -> Result<bool, KernelModuleError> {
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

/// checks whether a module is loaded
pub fn is_module_loaded(module: &str) -> Result<bool, KernelModuleError> {
    if is_in_proc_modules(module)? {
        return Ok(true);
    }

    Ok(false)
}
