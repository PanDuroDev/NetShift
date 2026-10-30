//! Platform-independent network observation models.
//!
//! These types contain observations only. They do not perform operating-system
//! calls, test internet reachability, or change network configuration.

use std::net::IpAddr;

/// A read-only snapshot of the network state observed at one point in time.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct NetworkSnapshot {
    /// Interfaces returned by the platform inspector.
    pub interfaces: Vec<NetworkInterface>,
    /// Routes currently present in the operating-system routing table.
    pub routes: Vec<NetworkRoute>,
    /// Local IPv4 configuration summary.
    pub ipv4: AddressFamilyStatus,
    /// Local IPv6 configuration summary.
    pub ipv6: AddressFamilyStatus,
}

/// A platform-neutral description of one network interface.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NetworkInterface {
    /// Stable operating-system identifier, such as the Windows adapter GUID.
    pub id: String,
    /// Human-readable interface name supplied by the operating system.
    pub name: String,
    /// IPv4 interface index, when the operating system reports one.
    pub ipv4_index: Option<u32>,
    /// IPv6 interface index, when the operating system reports one.
    pub ipv6_index: Option<u32>,
    /// Whether the interface is currently reported as operational.
    pub is_up: bool,
    /// IP addresses assigned to this interface.
    pub addresses: Vec<IpAddr>,
    /// DNS servers configured for this interface.
    pub dns_servers: Vec<IpAddr>,
}

/// An IP network prefix.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IpPrefix {
    /// Network address.
    pub address: IpAddr,
    /// Prefix length, from 0 through 32 for IPv4 or 0 through 128 for IPv6.
    pub prefix_length: u8,
}

/// A route reported by the operating system.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NetworkRoute {
    /// Destination network and prefix length.
    pub destination: IpPrefix,
    /// Next-hop address. None represents an on-link route.
    pub next_hop: Option<IpAddr>,
    /// Interface index reported by the operating system.
    pub interface_index: u32,
    /// Stable interface identifier, when it could be correlated.
    pub interface_id: Option<String>,
    /// Route metric reported by the operating system.
    pub metric: u32,
}

impl NetworkRoute {
    /// Returns true when this route is a default route (0.0.0.0/0 or ::/0).
    pub fn is_default(&self) -> bool {
        self.destination.prefix_length == 0 && self.destination.address.is_unspecified()
    }
}

/// Local configuration status for one IP address family.
///
/// usable_on_up_interface means an address is assigned to an operational
/// interface. It does not mean that the internet or a remote host is reachable.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct AddressFamilyStatus {
    /// At least one non-unspecified address is assigned to an interface.
    pub configured: bool,
    /// At least one non-loopback, non-unspecified address is assigned to an
    /// operational interface. Link-local addresses count as locally usable.
    pub usable_on_up_interface: bool,
}

impl NetworkSnapshot {
    /// Recomputes the local IPv4 and IPv6 summaries from the interface list.
    pub fn refresh_address_family_status(&mut self) {
        self.ipv4 = summarize_family(&self.interfaces, true);
        self.ipv6 = summarize_family(&self.interfaces, false);
    }
}

fn summarize_family(interfaces: &[NetworkInterface], ipv4: bool) -> AddressFamilyStatus {
    let mut status = AddressFamilyStatus::default();

    for interface in interfaces {
        for address in &interface.addresses {
            let matches_family = address.is_ipv4() == ipv4;
            if !matches_family || address.is_unspecified() {
                continue;
            }

            status.configured = true;
            if interface.is_up && !address.is_loopback() {
                status.usable_on_up_interface = true;
            }
        }
    }

    status
}

#[cfg(test)]
mod tests {
    use super::{IpPrefix, NetworkInterface, NetworkRoute, NetworkSnapshot};
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

    fn interface(name: &str, is_up: bool, addresses: Vec<IpAddr>) -> NetworkInterface {
        NetworkInterface {
            id: format!("adapter-{name}"),
            name: name.to_owned(),
            ipv4_index: None,
            ipv6_index: None,
            is_up,
            addresses,
            dns_servers: Vec::new(),
        }
    }

    #[test]
    fn snapshot_supports_multiple_interfaces_and_address_families() {
        let mut snapshot = NetworkSnapshot {
            interfaces: vec![
                interface(
                    "Ethernet",
                    true,
                    vec![
                        IpAddr::V4(Ipv4Addr::new(192, 168, 1, 20)),
                        IpAddr::V6(Ipv6Addr::LOCALHOST),
                    ],
                ),
                interface(
                    "Wi-Fi",
                    false,
                    vec![IpAddr::V6("2001:db8::10".parse().unwrap())],
                ),
            ],
            ..NetworkSnapshot::default()
        };

        snapshot.refresh_address_family_status();

        assert_eq!(snapshot.interfaces.len(), 2);
        assert!(snapshot.ipv4.configured);
        assert!(snapshot.ipv4.usable_on_up_interface);
        assert!(snapshot.ipv6.configured);
        assert!(!snapshot.ipv6.usable_on_up_interface);
    }

    #[test]
    fn loopback_and_unspecified_addresses_do_not_count_as_usable() {
        let mut snapshot = NetworkSnapshot {
            interfaces: vec![interface(
                "Loopback",
                true,
                vec![
                    IpAddr::V4(Ipv4Addr::LOCALHOST),
                    IpAddr::V6(Ipv6Addr::UNSPECIFIED),
                ],
            )],
            ..NetworkSnapshot::default()
        };

        snapshot.refresh_address_family_status();

        assert!(snapshot.ipv4.configured);
        assert!(!snapshot.ipv4.usable_on_up_interface);
        assert!(!snapshot.ipv6.configured);
    }

    #[test]
    fn default_route_detection_checks_prefix_and_unspecified_address() {
        let default_route = NetworkRoute {
            destination: IpPrefix {
                address: IpAddr::V4(Ipv4Addr::UNSPECIFIED),
                prefix_length: 0,
            },
            next_hop: Some(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1))),
            interface_index: 7,
            interface_id: Some("adapter-ethernet".to_owned()),
            metric: 25,
        };
        let specific_route = NetworkRoute {
            destination: IpPrefix {
                address: IpAddr::V4(Ipv4Addr::new(10, 0, 0, 0)),
                prefix_length: 8,
            },
            ..default_route.clone()
        };

        assert!(default_route.is_default());
        assert!(!specific_route.is_default());
    }

    #[test]
    fn ipv6_default_route_is_supported() {
        let route = NetworkRoute {
            destination: IpPrefix {
                address: IpAddr::V6(Ipv6Addr::UNSPECIFIED),
                prefix_length: 0,
            },
            next_hop: None,
            interface_index: 12,
            interface_id: None,
            metric: 5,
        };

        assert!(route.is_default());
    }
}
