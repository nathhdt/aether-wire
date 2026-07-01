//! check configuration module

use anyhow::Result;

use crate::cli::commands::check::CheckCliArgs;
use crate::utils::network::interfaces::get_interface;
use crate::utils::network::interfaces::types::Interface;

#[derive(Debug, Clone)]
pub struct CheckConfig {
    pub iface: Option<Interface>,
}

impl TryFrom<CheckCliArgs> for CheckConfig {
    type Error = anyhow::Error;

    fn try_from(args: CheckCliArgs) -> Result<Self> {
        let iface = args.iface.map(|iface| get_interface(&iface)).transpose()?;

        Ok(Self { iface })
    }
}
