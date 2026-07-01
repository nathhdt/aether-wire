//! aether-wire check entrypoint

pub mod config;
pub mod interfaces;
pub mod kernel;
pub mod memory;
pub mod print;
pub mod privileges;

use anyhow::Result;

use crate::check::config::CheckConfig;
use crate::cli::commands::check::CheckCliArgs;

pub enum Status {
    Ok,
    Warn,
    Fail,
    Info,
}

impl Status {
    fn symbol(&self) -> &'static str {
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

/// run system compatibility check
pub fn run(args: CheckCliArgs) -> Result<()> {
    let config = CheckConfig::try_from(args)?;

    let interfaces = interfaces::check_interfaces(config.iface.as_ref())?;
    let kernel = kernel::check_kernel()?;
    let privileges = privileges::check_privileges()?;
    let memory = memory::check_memory()?;

    println!("system compatibility check\n");

    print::print_section("kernel", &kernel);
    print::print_section("privileges", &privileges);
    print::print_section("memory", &memory);
    print::print_section_interfaces(&interfaces);

    Ok(())
}
