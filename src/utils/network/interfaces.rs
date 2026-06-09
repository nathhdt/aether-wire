//! network interfaces utilities module

use std::{fs, path::Path};

#[derive(Debug)]
pub enum InterfaceError {
    InterfaceNotFound,
    Io(std::io::Error),
}

impl From<std::io::Error> for InterfaceError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Interface {
    pub index: i32,
    pub name: String,
    pub kind: InterfaceKind,
    pub class: InterfaceClass,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterfaceKind {
    Ethernet,
    Loopback,
    Ppp,
    Tunnel,
    Other(u32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterfaceClass {
    Device,
    Virtual,
}

/// returns all network interfaces
pub fn get_interfaces() -> std::io::Result<Vec<Interface>> {
    let mut interfaces = Vec::new();

    for entry in fs::read_dir("/sys/class/net")? {
        let entry = entry?;

        let name = entry.file_name().to_string_lossy().into_owned();
        let path = entry.path();

        let index = get_interface_index(&path)?;
        let kind = get_interface_kind(&path)?;
        let class = get_interface_class(&path);

        interfaces.push(Interface {
            index,
            name,
            kind,
            class,
        });
    }

    interfaces.sort_unstable_by(|a, b| a.name.cmp(&b.name));

    Ok(interfaces)
}

fn get_interface_index(path: &Path) -> std::io::Result<i32> {
    let index_str = fs::read_to_string(path.join("ifindex"))?;

    index_str
        .trim()
        .parse::<i32>()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
}

fn get_interface_kind(path: &Path) -> std::io::Result<InterfaceKind> {
    let ty = fs::read_to_string(path.join("type"))?;
    let ty = ty.trim().parse::<u32>().unwrap_or_default();

    Ok(match ty {
        1 => InterfaceKind::Ethernet,
        512 => InterfaceKind::Ppp,
        768 | 769 | 776 | 778 => InterfaceKind::Tunnel,
        772 => InterfaceKind::Loopback,
        other => InterfaceKind::Other(other),
    })
}

fn get_interface_class(path: &Path) -> InterfaceClass {
    if path.join("device").exists() {
        InterfaceClass::Device
    } else {
        InterfaceClass::Virtual
    }
}

/// returns the driver associated with a network interface
pub fn get_interface_driver(name: &str) -> Result<Option<String>, InterfaceError> {
    let path = Path::new("/sys/class/net")
        .join(name)
        .join("device")
        .join("driver");

    match fs::read_link(path) {
        Ok(target) => Ok(target
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())),

        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            let interface_path = Path::new("/sys/class/net").join(name);

            if !interface_path.exists() {
                return Err(InterfaceError::InterfaceNotFound);
            }

            Ok(None)
        }

        Err(err) => Err(err.into()),
    }
}
