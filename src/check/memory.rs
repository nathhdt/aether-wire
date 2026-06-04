//! memory check module

use anyhow::Result;

use crate::utils::format::human_bytes;
use crate::utils::system::host::get_hugepages_info;

use super::{Check, Status};

pub fn check_memory() -> Result<Vec<Check>> {
    let mut checks = Vec::new();

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
