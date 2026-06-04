//! kernel check module

use anyhow::Result;

use crate::utils::constants::system::{
    KERNEL_MIN_MAJOR, KERNEL_MIN_MINOR, KERNEL_RECOMMENDED_MAJOR, KERNEL_RECOMMENDED_MINOR,
};
use crate::utils::system::kernel::KernelVersion;

use super::{Check, Status};

pub fn check_kernel() -> Result<Vec<Check>> {
    let mut checks = Vec::new();

    // kernel version
    let check = match KernelVersion::current() {
        Some(version) => {
            let status = if (version.major, version.minor)
                >= (KERNEL_RECOMMENDED_MAJOR, KERNEL_RECOMMENDED_MINOR)
            {
                Status::Ok
            } else if (version.major, version.minor) >= (KERNEL_MIN_MAJOR, KERNEL_MIN_MINOR) {
                Status::Warn
            } else {
                Status::Fail
            };

            let note = match status {
                Status::Ok => Some(format!(
                    "XDP metadata available (≥{}.{})",
                    KERNEL_RECOMMENDED_MAJOR, KERNEL_RECOMMENDED_MINOR
                )),
                Status::Warn => Some(format!(
                    "XDP multi-buffer available (≥{}.{})",
                    KERNEL_MIN_MAJOR, KERNEL_MIN_MINOR
                )),
                Status::Fail => Some(format!(
                    "minimum required kernel version is {}.{}",
                    KERNEL_MIN_MAJOR, KERNEL_MIN_MINOR
                )),
            };

            Check {
                label: "version".into(),
                value: format!("{}.{}", version.major, version.minor),
                status,
                note,
            }
        }

        None => Check {
            label: "version".into(),
            value: "unknown".into(),
            status: Status::Warn,
            note: Some(format!(
                "min {}.{}, recommended {}.{}",
                KERNEL_MIN_MAJOR,
                KERNEL_MIN_MINOR,
                KERNEL_RECOMMENDED_MAJOR,
                KERNEL_RECOMMENDED_MINOR
            )),
        },
    };

    checks.push(check);

    Ok(checks)
}
