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

pub fn is_module_loaded(module: &str) -> Result<bool, KernelModuleError> {
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

pub fn ensure_module_loaded(module: &str) -> Result<bool, KernelModuleError> {
    if is_module_loaded(module)? {
        return Ok(true);
    }

    let loaded = Command::new("modprobe")
        .arg(module)
        .status()
        .is_ok_and(|status| status.success());

    if !loaded {
        return Ok(false);
    }

    is_module_loaded(module)
}
