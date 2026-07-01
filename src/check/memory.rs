//! memory check module

use anyhow::Result;

use crate::utils::format::human_bytes;
use crate::utils::system::host::get_hugepages_info;
use crate::utils::system::sysctl;

use super::types::{Check, Status};

/// eBPF JIT compiler state (net.core.bpf_jit_enable)
enum BpfJit {
    Disabled,
    Enabled,
    EnabledDebug,
    Unknown(String),
    Unreadable,
}

impl BpfJit {
    fn read() -> Self {
        match sysctl::read("net.core.bpf_jit_enable") {
            Ok(value) => match value.as_str() {
                "0" => Self::Disabled,
                "1" => Self::Enabled,
                "2" => Self::EnabledDebug,
                other => Self::Unknown(other.to_owned()),
            },
            Err(_) => Self::Unreadable,
        }
    }

    fn value(&self) -> String {
        match self {
            Self::Disabled => "0".into(),
            Self::Enabled => "1".into(),
            Self::EnabledDebug => "2".into(),
            Self::Unknown(value) => value.clone(),
            Self::Unreadable => "unknown".into(),
        }
    }

    fn status(&self) -> Status {
        match self {
            Self::Enabled | Self::EnabledDebug => Status::Ok,
            _ => Status::Warn,
        }
    }

    fn note(&self) -> Option<String> {
        match self {
            Self::Enabled | Self::EnabledDebug => None,
            Self::Disabled => Some("eBPF JIT disabled".into()),
            Self::Unknown(_) => Some("unexpected sysctl value".into()),
            Self::Unreadable => Some("unable to read sysctl".into()),
        }
    }
}

pub fn check_memory() -> Result<Vec<Check>> {
    let mut checks = Vec::new();

    // bpf_jit_enable
    let bpf_jit = BpfJit::read();

    checks.push(Check {
        label: "bpf_jit_enable".into(),
        value: bpf_jit.value(),
        status: bpf_jit.status(),
        note: bpf_jit.note(),
    });

    // hugepages
    let hugepages = get_hugepages_info();

    checks.push(Check {
        label: "hugepages".into(),
        value: format!(
            "{} × {}",
            hugepages.total_pages,
            human_bytes(hugepages.page_size_bytes)
        ),
        status: if hugepages.total_pages > 0 {
            Status::Ok
        } else {
            Status::Warn
        },
        note: if hugepages.total_pages > 0 {
            None
        } else {
            Some("higher TLB pressure".into())
        },
    });

    Ok(checks)
}
