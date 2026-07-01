//! aether-wire check entrypoint

mod config;
mod interfaces;
mod kernel;
mod memory;
mod print;
mod privileges;
mod types;

use anyhow::Result;

use crate::cli::commands::check::CheckCliArgs;

use config::CheckConfig;

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
