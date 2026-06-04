//! kernel modules utilities module

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::process::Command;

#[allow(dead_code)]
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

/// checks whether a module is loaded, optionally attempting to load it via modprobe first
pub fn is_module_loaded(module: &str, load: bool) -> Result<bool, KernelModuleError> {
    if is_in_proc_modules(module)? {
        return Ok(true);
    }

    if !load {
        return Ok(false);
    }

    let _ = Command::new("modprobe")
        .arg(module)
        .status()
        .is_ok_and(|status| status.success());

    is_in_proc_modules(module)
}
