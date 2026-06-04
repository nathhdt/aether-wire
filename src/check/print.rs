//! print utilities for the check module

use crate::check::Check;

/// section print
pub fn print_section(title: &str, checks: &[Check]) {
    println!("{}", title);
    for check in checks {
        print_check(check, 2);
    }
    println!();
}

/// full check print
pub fn print_check(check: &Check, indent: usize) {
    let pad = " ".repeat(indent);
    match &check.note {
        Some(note) => println!(
            "{}{:<22} {:<16} {}  {}",
            pad,
            check.label,
            check.value,
            check.status.symbol(),
            note
        ),
        None => println!(
            "{}{:<22} {:<16} {}",
            pad,
            check.label,
            check.value,
            check.status.symbol()
        ),
    }
}
