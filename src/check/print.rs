//! print utilities for the check module

use crate::check::{Check, InterfaceChecks};

/// section print
pub fn print_section(title: &str, checks: &[Check]) {
    println!("{}", title);
    for check in checks {
        print_check(check, 2);
    }
    println!();
}

/// interfaces section print
pub fn print_section_interfaces(interfaces: &[InterfaceChecks]) {
    println!("interfaces");

    for (i, interface) in interfaces.iter().enumerate() {
        println!("  {}", interface.interface);

        for check in &interface.checks {
            print_check(check, 4);
        }

        if i + 1 < interfaces.len() {
            println!();
        }
    }
}

/// full check print
pub fn print_check(check: &Check, indent: usize) {
    let label = format!("{}{}", " ".repeat(indent), check.label);

    match &check.note {
        Some(note) => println!(
            "{:<25} {:<14} {}  {}",
            label,
            check.value,
            check.status.symbol(),
            note
        ),
        None => println!(
            "{:<25} {:<14} {}",
            label,
            check.value,
            check.status.symbol()
        ),
    }
}
