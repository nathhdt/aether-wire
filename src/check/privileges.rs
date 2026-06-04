//! privileges check module

use anyhow::Result;

use crate::utils::format::human_bytes;
use crate::utils::system::host::{MemlockLimitValue, get_memlock_limit};

use super::{Check, Status};

pub fn check_privileges() -> Result<Vec<Check>> {
    let mut checks = Vec::new();

    let memlock = match get_memlock_limit() {
        MemlockLimitValue::Unlimited => Check {
            label: "RLIMIT_MEMLOCK".into(),
            value: "unlimited".into(),
            status: Status::Ok,
            note: Some("no lockable-memory restriction".into()),
        },

        MemlockLimitValue::Bytes(bytes) => Check {
            label: "RLIMIT_MEMLOCK".into(),
            value: human_bytes(bytes),
            status: Status::Warn,
            note: Some("lockable memory is capped".into()),
        },
    };

    checks.push(memlock);

    Ok(checks)
}
