//! Windows network discovery using read-only IP Helper APIs.

use std::collections::HashMap;
use std::ffi::CStr;
use std::mem::transmute_copy;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::ptr::null_mut;

use ipconfig::{OperStatus, get_adapters};
use windows_sys::Win32::Foundation::{ERROR_BUFFER_OVERFLOW, ERROR_NOT_FOUND, ERROR_SUCCESS};
use windows_sys::Win32::NetworkManagement::IpHelper::{
    FreeMibTable, GetAdaptersAddresses, GetIpForwardTable2, IP_ADAPTER_ADDRESSES_LH,
    MIB_IPFORWARD_TABLE2,
};
use windows_sys::Win32::Networking::WinSock::{AF_INET, AF_INET6, AF_UNSPEC, SOCKADDR_INET};

use crate::core::network::{
    AddressFamilyStatus, IpPrefix, NetworkInterface, NetworkRoute, NetworkSnapshot,
};
use crate::network::{NetworkInspectionError, NetworkInspector};

/// Windows implementation of the read-only network inspector.
#[derive(Clone, Copy, Debug, Default)]
pub struct WindowsNetworkInspector;

impl NetworkInspector for WindowsNetworkInspector {
    fn inspect(&self) -> Result<NetworkSnapshot, NetworkInspectionError> {
        let adapters = get_adapters().map_err(|error| {
            NetworkInspectionError::QueryFailed(format!("GetAdaptersAddresses failed: {error}"))
        })?;
        let indices = enumerate_interface_indices()?;

        let interfaces = adapters
            .into_iter()
            .map(|adapter| {
                let ipv4_index = indices
                    .get(adapter.adapter_name())
                    .map(|value| value.0)
                    .filter(|index| *index != 0);
                let ipv6_index = indices
                    .get(adapter.adapter_name())
                    .map(|value| value.1)
                    .filter(|index| *index != 0)
                    .or_else(|| {
                        let index = adapter.ipv6_if_index();
                        (index != 0).then_some(index)
                    });

                NetworkInterface {
                    id: adapter.adapter_name().to_owned(),
                    name: adapter.friendly_name().to_owned(),
                    ipv4_index,
                    ipv6_index,
                    is_up: adapter.oper_status() == OperStatus::IfOperStatusUp,
                    addresses: adapter.ip_addresses().to_vec(),
                    dns_servers: adapter.dns_servers().to_vec(),
                }
            })
            .collect::<Vec<_>>();

        let routes = enumerate_routes(&interfaces)?;
        let mut snapshot = NetworkSnapshot {
            interfaces,
            routes,
            ipv4: AddressFamilyStatus::default(),
            ipv6: AddressFamilyStatus::default(),
        };
        snapshot.refresh_address_family_status();
        Ok(snapshot)
    }
}

/// Returns adapter GUID -> (IPv4 interface index, IPv6 interface index).
///
/// GetAdaptersAddresses is used here only to read interface metadata.
fn enumerate_interface_indices() -> Result<HashMap<String, (u32, u32)>, NetworkInspectionError> {
    let mut buffer_length = 16 * 1024_u32;
    let mut buffer = Vec::<u64>::new();
    let result = loop {
        let word_count = (buffer_length as usize).div_ceil(std::mem::size_of::<u64>());
        buffer.resize(word_count, 0);

        let current_result = unsafe {
            GetAdaptersAddresses(
                AF_UNSPEC as u32,
                0,
                null_mut(),
                buffer.as_mut_ptr().cast::<IP_ADAPTER_ADDRESSES_LH>(),
                &mut buffer_length,
            )
        };

        if current_result != ERROR_BUFFER_OVERFLOW {
            break current_result;
        }
    };

    if result != ERROR_SUCCESS {
        return Err(NetworkInspectionError::QueryFailed(format!(
            "GetAdaptersAddresses failed with Windows error {result}"
        )));
    }

    let mut indices = HashMap::new();
    let mut adapter = buffer.as_mut_ptr().cast::<IP_ADAPTER_ADDRESSES_LH>();

    unsafe {
        while !adapter.is_null() {
            let current = &*adapter;
            if !current.AdapterName.is_null() {
                let id = CStr::from_ptr(current.AdapterName.cast())
                    .to_string_lossy()
                    .into_owned();

                // The first two DWORDs of the SDK's first union are Length and
                // IfIndex. Reading the union as bytes avoids generated anonymous
                // union field names.
                let first_union: [u32; 2] = transmute_copy(&current.Anonymous1);
                indices.insert(id, (first_union[1], current.Ipv6IfIndex));
            }
            adapter = current.Next;
        }
    }

    Ok(indices)
}

/// Reads the complete IPv4 + IPv6 route table. The returned table is released
/// with FreeMibTable after conversion.
fn enumerate_routes(
    interfaces: &[NetworkInterface],
) -> Result<Vec<NetworkRoute>, NetworkInspectionError> {
    let mut table: *mut MIB_IPFORWARD_TABLE2 = null_mut();
    let result = unsafe { GetIpForwardTable2(AF_UNSPEC, &mut table) };

    if result == ERROR_NOT_FOUND {
        return Ok(Vec::new());
    }
    if result != ERROR_SUCCESS {
        return Err(NetworkInspectionError::QueryFailed(format!(
            "GetIpForwardTable2 failed with Windows error {result}"
        )));
    }

    if table.is_null() {
        return Err(NetworkInspectionError::QueryFailed(
            "GetIpForwardTable2 returned a null table".to_owned(),
        ));
    }

    let rows = unsafe {
        let rows =
            std::slice::from_raw_parts((*table).Table.as_ptr(), (*table).NumEntries as usize);
        rows.iter()
            .filter_map(|row| {
                Some(RouteRow {
                    destination: sockaddr_to_ip(row.DestinationPrefix.Prefix)?,
                    prefix_length: row.DestinationPrefix.PrefixLength,
                    next_hop: sockaddr_to_ip(row.NextHop)
                        .filter(|address| !address.is_unspecified()),
                    interface_index: row.InterfaceIndex,
                    metric: row.Metric,
                })
            })
            .collect::<Vec<_>>()
    };

    unsafe { FreeMibTable(table.cast()) };
    Ok(convert_route_rows(rows, interfaces))
}

/// Platform-neutral values extracted from one Windows route-table row.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct RouteRow {
    destination: IpAddr,
    prefix_length: u8,
    next_hop: Option<IpAddr>,
    interface_index: u32,
    metric: u32,
}

/// Converts route-table rows into the portable model and correlates each row
/// with the correct address-family-specific interface index.
fn convert_route_rows(
    rows: impl IntoIterator<Item = RouteRow>,
    interfaces: &[NetworkInterface],
) -> Vec<NetworkRoute> {
    rows.into_iter()
        .map(|row| NetworkRoute {
            destination: IpPrefix {
                address: row.destination,
                prefix_length: row.prefix_length,
            },
            next_hop: row.next_hop,
            interface_index: row.interface_index,
            interface_id: crate::core::network::find_interface_id(
                interfaces,
                row.destination,
                row.interface_index,
            ),
            metric: row.metric,
        })
        .collect()
}

/// Converts the Windows SOCKADDR_INET union to a standard-library IP address.
unsafe fn sockaddr_to_ip(address: SOCKADDR_INET) -> Option<IpAddr> {
    let family = unsafe { address.si_family };
    let bytes = unsafe {
        std::slice::from_raw_parts(
            (&address as *const SOCKADDR_INET).cast::<u8>(),
            std::mem::size_of::<SOCKADDR_INET>(),
        )
    };

    match family {
        AF_INET => Some(IpAddr::V4(Ipv4Addr::new(
            bytes[4], bytes[5], bytes[6], bytes[7],
        ))),
        AF_INET6 => {
            let octets: [u8; 16] = bytes.get(8..24)?.try_into().ok()?;
            Some(IpAddr::V6(Ipv6Addr::from(octets)))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{RouteRow, convert_route_rows, sockaddr_to_ip};
    use crate::core::network::NetworkInterface;
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
    use windows_sys::Win32::Networking::WinSock::{AF_INET, AF_INET6, SOCKADDR_INET};

    #[test]
    fn converts_ipv4_sockaddr() {
        let mut bytes = [0_u8; 28];
        bytes[0..2].copy_from_slice(&AF_INET.to_ne_bytes());
        bytes[4..8].copy_from_slice(&[192, 168, 1, 1]);
        let address = unsafe { std::mem::transmute::<[u8; 28], SOCKADDR_INET>(bytes) };

        assert_eq!(
            unsafe { sockaddr_to_ip(address) },
            Some(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)))
        );
    }

    fn test_interface(
        id: &str,
        ipv4_index: Option<u32>,
        ipv6_index: Option<u32>,
    ) -> NetworkInterface {
        NetworkInterface {
            id: id.to_owned(),
            name: id.to_owned(),
            ipv4_index,
            ipv6_index,
            is_up: true,
            addresses: Vec::new(),
            dns_servers: Vec::new(),
        }
    }

    #[test]
    fn converts_multiple_ipv4_and_ipv6_route_rows_across_interfaces() {
        let interfaces = vec![
            test_interface("ethernet-guid", Some(7), Some(70)),
            test_interface("wifi-guid", Some(8), Some(80)),
        ];
        let rows = vec![
            RouteRow {
                destination: IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
                prefix_length: 0,
                next_hop: Some(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1))),
                interface_index: 7,
                metric: 25,
            },
            RouteRow {
                destination: IpAddr::V4(Ipv4Addr::new(10, 20, 0, 0)),
                prefix_length: 16,
                next_hop: None,
                interface_index: 8,
                metric: 40,
            },
            RouteRow {
                destination: IpAddr::V6(Ipv6Addr::UNSPECIFIED),
                prefix_length: 0,
                next_hop: Some(IpAddr::V6("fe80::1".parse().unwrap())),
                interface_index: 80,
                metric: 15,
            },
            RouteRow {
                destination: IpAddr::V6("2001:db8:1::".parse().unwrap()),
                prefix_length: 48,
                next_hop: None,
                interface_index: 70,
                metric: 60,
            },
        ];

        let routes = convert_route_rows(rows, &interfaces);

        assert_eq!(routes.len(), 4);
        assert_eq!(
            routes[0].destination,
            crate::core::network::IpPrefix {
                address: IpAddr::V4(Ipv4Addr::UNSPECIFIED),
                prefix_length: 0,
            }
        );
        assert_eq!(
            routes[0].next_hop,
            Some(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)))
        );
        assert_eq!(routes[0].interface_id.as_deref(), Some("ethernet-guid"));
        assert_eq!(routes[0].metric, 25);

        assert_eq!(routes[1].interface_id.as_deref(), Some("wifi-guid"));
        assert_eq!(routes[1].next_hop, None);
        assert_eq!(routes[1].metric, 40);

        assert_eq!(routes[2].interface_id.as_deref(), Some("wifi-guid"));
        assert_eq!(
            routes[2].next_hop,
            Some(IpAddr::V6("fe80::1".parse().unwrap()))
        );
        assert_eq!(routes[2].metric, 15);

        assert_eq!(routes[3].interface_id.as_deref(), Some("ethernet-guid"));
        assert_eq!(routes[3].next_hop, None);
        assert_eq!(routes[3].metric, 60);
    }

    #[test]
    fn route_with_unmatched_interface_index_keeps_route_and_index() {
        let interfaces = vec![test_interface("ethernet-guid", Some(7), Some(70))];
        let rows = vec![RouteRow {
            destination: IpAddr::V4(Ipv4Addr::new(172, 16, 0, 0)),
            prefix_length: 12,
            next_hop: Some(IpAddr::V4(Ipv4Addr::new(172, 16, 0, 1))),
            interface_index: 999,
            metric: 123,
        }];

        let routes = convert_route_rows(rows, &interfaces);

        assert_eq!(routes.len(), 1);
        assert_eq!(routes[0].interface_index, 999);
        assert_eq!(routes[0].interface_id, None);
        assert_eq!(routes[0].metric, 123);
        assert_eq!(
            routes[0].destination.address,
            IpAddr::V4(Ipv4Addr::new(172, 16, 0, 0))
        );
        assert_eq!(
            routes[0].next_hop,
            Some(IpAddr::V4(Ipv4Addr::new(172, 16, 0, 1)))
        );
    }

    #[test]
    fn live_inspector_returns_a_consistent_snapshot() {
        use super::WindowsNetworkInspector;
        use crate::network::NetworkInspector;

        let snapshot = WindowsNetworkInspector
            .inspect()
            .expect("Windows inspection should succeed");
        assert!(
            snapshot
                .interfaces
                .iter()
                .all(|interface| !interface.id.is_empty())
        );
        for route in &snapshot.routes {
            let maximum_prefix = if route.destination.address.is_ipv4() {
                32
            } else {
                128
            };
            assert!(route.destination.prefix_length <= maximum_prefix);
        }
    }

    #[test]
    fn converts_ipv6_sockaddr() {
        let mut bytes = [0_u8; 28];
        bytes[0..2].copy_from_slice(&AF_INET6.to_ne_bytes());
        bytes[8..24].copy_from_slice(&Ipv6Addr::LOCALHOST.octets());
        let address = unsafe { std::mem::transmute::<[u8; 28], SOCKADDR_INET>(bytes) };

        assert_eq!(
            unsafe { sockaddr_to_ip(address) },
            Some(IpAddr::V6(Ipv6Addr::LOCALHOST))
        );
    }
}
