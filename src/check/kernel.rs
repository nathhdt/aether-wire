//! kernel check module

use anyhow::Result;

use crate::utils::constants::system::{KERNEL_MIN_MAJOR, KERNEL_MIN_MINOR};
use crate::utils::system::kernel::KernelVersion;

use super::{Check, Status};

pub fn check_kernel() -> Result<Vec<Check>> {
    let mut checks = Vec::new();

    let check = match KernelVersion::current() {
        Some(version) => {
            let supported = version.major > KERNEL_MIN_MAJOR
                || (version.major == KERNEL_MIN_MAJOR && version.minor >= KERNEL_MIN_MINOR);

            Check {
                label: "kernel version".into(),
                value: format!("{}.{}", version.major, version.minor),
                status: if supported { Status::Ok } else { Status::Fail },
                note: if supported {
                    None
                } else {
                    Some(format!(
                        "minimum required kernel version is {}.{}",
                        KERNEL_MIN_MAJOR, KERNEL_MIN_MINOR
                    ))
                },
            }
        }

        None => Check {
            label: "kernel version".into(),
            value: "unknown".into(),
            status: Status::Warn,
            note: Some("unable to determine kernel version".into()),
        },
    };

    checks.push(check);

    Ok(checks)
}
