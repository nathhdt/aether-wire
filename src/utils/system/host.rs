//! system host utilities module

use rustix::process;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemlockLimitValue {
    Unlimited,
    Bytes(u64),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MemlockLimit {
    pub current: MemlockLimitValue,
    pub hard: MemlockLimitValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HugePagesInfo {
    pub page_size_bytes: u64,
    pub total_pages: u64,
}

/// checks if the process running with root privileges
pub fn is_root_process() -> bool {
    process::getuid().is_root()
}

/// retrieves memlock limits
pub fn get_memlock_limit() -> MemlockLimit {
    let limit = process::getrlimit(process::Resource::Memlock);

    MemlockLimit {
        current: match limit.current {
            None => MemlockLimitValue::Unlimited,
            Some(value) => MemlockLimitValue::Bytes(value),
        },
        hard: match limit.maximum {
            None => MemlockLimitValue::Unlimited,
            Some(value) => MemlockLimitValue::Bytes(value),
        },
    }
}

/// retrieves hugepages informations
pub fn get_hugepages_info() -> HugePagesInfo {
    let Ok(file) = File::open("/proc/meminfo") else {
        return HugePagesInfo {
            page_size_bytes: 0,
            total_pages: 0,
        };
    };

    let reader = BufReader::new(file);

    let mut page_size_bytes = 0;
    let mut total_pages = 0;
    let mut found_page_size = false;
    let mut found_total_pages = false;

    for line in reader.lines().map_while(Result::ok) {
        let mut parts = line.split_whitespace();

        match (parts.next(), parts.next()) {
            (Some("Hugepagesize:"), Some(v)) => {
                if let Ok(kb) = v.parse::<u64>() {
                    page_size_bytes = kb * 1024;
                    found_page_size = true;
                }
            }
            (Some("HugePages_Total:"), Some(v)) => {
                if let Ok(total) = v.parse::<u64>() {
                    total_pages = total;
                    found_total_pages = true;
                }
            }
            _ => {}
        }

        if found_page_size && found_total_pages {
            break;
        }
    }

    HugePagesInfo {
        page_size_bytes,
        total_pages,
    }
}
