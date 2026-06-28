//! memory check module

use anyhow::Result;

use crate::utils::format::human_bytes;
use crate::utils::system::host::get_hugepages_info;
use crate::utils::system::sysctl;

use super::{Check, Status};

pub fn check_memory() -> Result<Vec<Check>> {
    let mut checks = Vec::new();

    // bpf_jit_enable
    let bpf_jit_enable = sysctl::read("net.core.bpf_jit_enable").ok();

    checks.push(Check {
        label: "bpf_jit_enable".into(),
        value: bpf_jit_enable.clone().unwrap_or_else(|| "0".into()),
        status: match bpf_jit_enable.as_deref() {
            Some("1") | Some("2") => Status::Ok,
            _ => Status::Warn,
        },
        note: match bpf_jit_enable.as_deref() {
            Some("1") | Some("2") => None,
            _ => Some("eBPF JIT disabled".into()),
        },
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
