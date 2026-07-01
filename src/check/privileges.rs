//! privileges check module

use anyhow::Result;

use crate::utils::format::human_bytes;
use crate::utils::system::host::{MemlockLimitValue, get_memlock_limit};

use super::types::{Check, Status};

pub fn check_privileges() -> Result<Vec<Check>> {
    let mut checks = Vec::new();
    let memlock = get_memlock_limit();

    let (value, status) = match memlock.current {
        MemlockLimitValue::Unlimited => ("unlimited".into(), Status::Ok),
        MemlockLimitValue::Bytes(bytes) => (human_bytes(bytes), Status::Warn),
    };

    let note = match (memlock.current, memlock.hard) {
        (MemlockLimitValue::Unlimited, MemlockLimitValue::Unlimited) => None,

        (MemlockLimitValue::Bytes(_), MemlockLimitValue::Bytes(_))
            if memlock.current == memlock.hard =>
        {
            Some("memory locking is limited".into())
        }

        (_, MemlockLimitValue::Unlimited) => {
            Some("current limit can be increased up to unlimited".into())
        }

        (_, MemlockLimitValue::Bytes(bytes)) => Some(format!(
            "current limit can be increased up to {}",
            human_bytes(bytes)
        )),
    };

    checks.push(Check {
        label: "RLIMIT_MEMLOCK".into(),
        value,
        status,
        note,
    });

    Ok(checks)
}
