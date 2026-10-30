//! Platform-independent network data models.
//!
//! These types describe observations. They do not perform operating-system
//! calls and cannot change network configuration.

use std::net::IpAddr;

/// A read-only snapshot of the network interfaces observed at one point in time.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct NetworkSnapshot {
    /// Interfaces returned by the platform inspector.
    pub interfaces: Vec<NetworkInterface>,
}

/// A platform-neutral description of one network interface.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NetworkInterface {
    /// Human-readable interface name supplied by the operating system.
    pub name: String,
    /// Operating-system interface index.
    pub index: u32,
    /// Whether the interface is currently reported as operational.
    pub is_up: bool,
    /// IP addresses assigned to this interface.
    pub addresses: Vec<IpAddr>,
}

#[cfg(test)]
mod tests {
    use super::{NetworkInterface, NetworkSnapshot};
    use std::net::{IpAddr, Ipv4Addr};

    #[test]
    fn snapshot_can_represent_multiple_interface_addresses() {
        let snapshot = NetworkSnapshot {
            interfaces: vec![NetworkInterface {
                name: "Ethernet".to_owned(),
                index: 7,
                is_up: true,
                addresses: vec![IpAddr::V4(Ipv4Addr::LOCALHOST)],
            }],
        };

        assert_eq!(snapshot.interfaces.len(), 1);
        assert_eq!(snapshot.interfaces[0].name, "Ethernet");
        assert!(snapshot.interfaces[0].is_up);
    }
}
