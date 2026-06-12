//! kernel check module

use anyhow::Result;

use crate::utils::constants::system::{
    KERNEL_MIN_MAJOR, KERNEL_MIN_MINOR, KERNEL_RECOMMENDED_MAJOR, KERNEL_RECOMMENDED_MINOR,
};
use crate::utils::kernel::flags::{KernelConfigError, KernelFlagValue, get_kernel_flag};
use crate::utils::kernel::modules::is_in_proc_modules;
use crate::utils::kernel::version::KernelVersion;

use super::{Check, Status};

struct FlagCheck {
    flag: &'static str,
    module: Option<&'static str>,
    required: bool,
    note: &'static str,
}

const FLAGS: &[FlagCheck] = &[
    FlagCheck {
        flag: "CONFIG_BPF_SYSCALL",
        module: None,
        required: true,
        note: "required for eBPF program loading",
    },
    FlagCheck {
        flag: "CONFIG_XDP_SOCKETS",
        module: None,
        required: true,
        note: "required for AF_XDP support",
    },
    FlagCheck {
        flag: "CONFIG_BPF_JIT",
        module: None,
        required: false,
        note: "recommended for optimal performance",
    },
    FlagCheck {
        flag: "CONFIG_XDP_SOCKETS_DIAG",
        module: Some("xsk_diag"),
        required: true,
        note: "required for AF_XDP diagnostics",
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
                Status::Ok => None,
                Status::Warn => Some(format!(
                    "XDP metadata unavailable (needs ≥ {}.{})",
                    KERNEL_RECOMMENDED_MAJOR, KERNEL_RECOMMENDED_MINOR
                )),
                Status::Fail => Some(format!(
                    "minimum required kernel version is {}.{}",
                    KERNEL_MIN_MAJOR, KERNEL_MIN_MINOR
                )),
                Status::Info => None,
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
                let (valid, note) = match &value {
                    KernelFlagValue::Yes => (true, None),
                    KernelFlagValue::Module => {
                        if let Some(module) = flag.module {
                            check_module(module)
                        } else {
                            (true, None)
                        }
                    }
                    _ => (false, None),
                };

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
                    note: if valid {
                        note
                    } else {
                        note.or_else(|| Some(flag.note.into()))
                    },
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

/// checks whether a module is loaded, optionally attempting to load it first
fn check_module(module: &str) -> (bool, Option<String>) {
    match is_in_proc_modules(module) {
        Ok(true) => (true, Some(format!("module '{module}' loaded"))),
        Ok(false) => (false, Some(format!("module '{module}' not loaded"))),
        Err(_) => (false, Some(format!("module '{module}' not loaded"))),
    }
}
