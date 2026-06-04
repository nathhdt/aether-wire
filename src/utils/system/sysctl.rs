//! system host utilities module for sysctl interface

use std::fs;
use std::path::PathBuf;

/// reads a sysctl parameter
pub fn read(param: &str) -> Result<String, std::io::Error> {
    let path = PathBuf::from("/proc/sys").join(param.replace('.', "/"));

    Ok(fs::read_to_string(path)?.trim().to_owned())
}
