//! kernel check module

use anyhow::Result;

use crate::utils::constants::system::{
    KERNEL_MIN_MAJOR, KERNEL_MIN_MINOR, KERNEL_RECOMMENDED_MAJOR, KERNEL_RECOMMENDED_MINOR,
};
use crate::utils::kernel::flags::{KernelConfigError, KernelFlagValue, get_kernel_flag};
use crate::utils::kernel::version::KernelVersion;

use super::{Check, Status};

struct FlagCheck {
    flag: &'static str,
    allow_module: bool,
    required: bool,
    note: &'static str,
}

const FLAGS: &[FlagCheck] = &[
    FlagCheck {
        flag: "CONFIG_BPF_SYSCALL",
        allow_module: false,
        required: true,
        note: "required for eBPF program loading",
    },
    FlagCheck {
        flag: "CONFIG_XDP_SOCKETS",
        allow_module: false,
        required: true,
        note: "required for AF_XDP support",
    },
    FlagCheck {
        flag: "CONFIG_BPF_JIT",
        allow_module: false,
        required: false,
        note: "recommended for optimal performance",
    },
    FlagCheck {
        flag: "CONFIG_XDP_SOCKETS_DIAG",
        allow_module: true,
        required: false,
        note: "optional AF_XDP diagnostics support",
    },
];

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

    // kernel flags
    for flag in FLAGS {
        let check = match get_kernel_flag(flag.flag) {
            Ok(value) => {
                let valid = matches!(value, KernelFlagValue::Yes)
                    || (flag.allow_module && matches!(value, KernelFlagValue::Module));

                let status = if valid {
                    Status::Ok
                } else if flag.required {
                    Status::Fail
                } else {
                    Status::Warn
                };

                let value = match value {
                    KernelFlagValue::Yes => "y".into(),
                    KernelFlagValue::Module => "m".into(),
                    KernelFlagValue::No => "n".into(),
                    KernelFlagValue::Value(v) => v,
                };

                Check {
                    label: flag.flag.into(),
                    value,
                    status,
                    note: Some(flag.note.into()),
                }
            }

            Err(KernelConfigError::ConfigNotFound) => Check {
                label: flag.flag.into(),
                value: "unknown".into(),
                status: Status::Warn,
                note: Some("kernel config not found".into()),
            },

            Err(KernelConfigError::Io(err)) => Check {
                label: flag.flag.into(),
                value: "unknown".into(),
                status: Status::Warn,
                note: Some(format!("unable to read kernel config: {err}")),
            },
        };

        checks.push(check);
    }

    Ok(checks)
}
