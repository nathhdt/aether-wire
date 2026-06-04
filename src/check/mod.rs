//! aether-wire check entrypoint

pub mod interfaces;
pub mod kernel;
pub mod memory;
pub mod print;
pub mod privileges;

use anyhow::Result;

use crate::cli::commands::check::CheckConfig;

pub enum Status {
    Ok,
    Warn,
    Fail,
    None,
}

impl Status {
    fn symbol(&self) -> &'static str {
        match self {
            Status::Ok => "✓",
            Status::Warn => "⚠",
            Status::Fail => "✗",
            Status::None => " ",
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

/// run system compatibility checks
pub fn run(_config: CheckConfig) -> Result<()> {
    println!("system compatibility check for aether-wire\n");

    print::print_section("kernel", &kernel::check_kernel()?);
    print::print_section("privileges", &privileges::check_privileges()?);
    print::print_section("memory", &memory::check_memory()?);
    print::print_section_interfaces(&interfaces::check_interfaces()?);

    Ok(())
}
