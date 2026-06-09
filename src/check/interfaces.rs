//! network interfaces check module

use anyhow::Result;

use crate::utils::network::interfaces::{
    InterfaceClass, InterfaceError, InterfaceKind, get_interface_driver, get_interfaces,
};

use super::{Check, InterfaceChecks, Status};

pub fn check_interfaces() -> Result<Vec<InterfaceChecks>> {
    let mut interfaces_checks = Vec::new();

    let interfaces = get_interfaces()?;

    for interface in interfaces {
        let type_check = Check {
            label: "type".into(),
            value: match interface.kind {
                InterfaceKind::Ethernet => "ethernet".into(),
                InterfaceKind::Loopback => "loopback".into(),
                InterfaceKind::Ppp => "ppp".into(),
                InterfaceKind::Tunnel => "tunnel".into(),
                InterfaceKind::Other(kind) => format!("other ({kind})"),
            },
            status: Status::Info,
            note: None,
        };

        let class_check = Check {
            label: "class".into(),
            value: match interface.class {
                InterfaceClass::Device => "device".into(),
                InterfaceClass::Virtual => "virtual".into(),
            },
            status: Status::Info,
            note: None,
        };

        let driver_check = match interface.kind {
            InterfaceKind::Loopback => Check {
                label: "driver".into(),
                value: "n/a".into(),
                status: Status::Info,
                note: None,
            },

            _ => match get_interface_driver(&interface.name) {
                Ok(Some(driver)) => Check {
                    label: "driver".into(),
                    value: driver,
                    status: Status::Info,
                    note: None,
                },

                Ok(None) => Check {
                    label: "driver".into(),
                    value: "unknown".into(),
                    status: Status::Info,
                    note: Some("no driver associated".into()),
                },

                Err(InterfaceError::Io(err)) => Check {
                    label: "driver".into(),
                    value: "unknown".into(),
                    status: Status::Warn,
                    note: Some(format!("unable to determine driver: {err}")),
                },

                Err(_) => Check {
                    label: "driver".into(),
                    value: "unknown".into(),
                    status: Status::Warn,
                    note: Some("unable to determine driver".into()),
                },
            },
        };

        interfaces_checks.push(InterfaceChecks {
            interface: interface.name,
            checks: vec![type_check, class_check, driver_check],
        });
    }

    Ok(interfaces_checks)
}
