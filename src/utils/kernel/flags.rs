//! kernel flags utilities module

use rustix::system::uname;
use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug)]
pub enum KernelConfigError {
    ConfigNotFound,
    Io(std::io::Error),
}

impl fmt::Display for KernelConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ConfigNotFound => write!(f, "kernel config not found"),
            Self::Io(err) => write!(f, "{err}"),
        }
    }
}

impl std::error::Error for KernelConfigError {}

impl From<std::io::Error> for KernelConfigError {
    fn from(err: std::io::Error) -> Self {
        KernelConfigError::Io(err)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum KernelFlagValue {
    Yes,
    Module,
    No,
    Value(String),
}

fn find_kernel_config_path() -> Result<String, KernelConfigError> {
    let uts = uname();
    let release = uts.release().to_string_lossy();

    let boot_path = format!("/boot/config-{}", release);

    if std::fs::metadata(&boot_path).is_ok() {
        return Ok(boot_path);
    }

    let mod_path = format!("/lib/modules/{}/config", release);
    if std::fs::metadata(&mod_path).is_ok() {
        return Ok(mod_path);
    }

    Err(KernelConfigError::ConfigNotFound)
}

pub fn get_kernel_flag(flag: &str) -> Result<KernelFlagValue, KernelConfigError> {
    let config_path = find_kernel_config_path()?;

    let file = File::open(&config_path)?;
    let mut reader = BufReader::new(file);

    let target_equal = format!("{}=", flag);
    let target_not_set = format!("# {} is not set", flag);

    let mut line = String::new();

    while reader.read_line(&mut line)? > 0 {
        let trimmed = line.trim();

        if trimmed == target_not_set {
            return Ok(KernelFlagValue::No);
        }

        if trimmed.starts_with(&target_equal) {
            let value_part = &trimmed[target_equal.len()..];
            let value = match value_part {
                "y" => KernelFlagValue::Yes,
                "m" => KernelFlagValue::Module,
                "n" => KernelFlagValue::No,
                other => KernelFlagValue::Value(other.to_string()),
            };
            return Ok(value);
        }

        line.clear();
    }

    Ok(KernelFlagValue::No)
}
