//! aether-wire check entrypoint

pub mod interfaces;
pub mod kernel;
pub mod memory;
pub mod print;
pub mod privileges;

use anyhow::Result;

use crate::cli::commands::check::CheckConfig;
use crate::utils::network::interfaces::interface_exists;

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
pub fn run(config: CheckConfig) -> Result<()> {
    // check if interface exists
    if let Some(name) = &config.iface
        && !interface_exists(name)
    {
        anyhow::bail!("interface '{name}' not found");
    }

    println!("system compatibility check\n");

    print::print_section("kernel", &kernel::check_kernel()?);
    print::print_section("privileges", &privileges::check_privileges()?);
    print::print_section("memory", &memory::check_memory()?);
    print::print_section_interfaces(&interfaces::check_interfaces(config.iface.as_deref())?);

    Ok(())
}
