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
        let adapters = get_adapters()
            .map_err(|error| NetworkInspectionError::QueryFailed(error.to_string()))?;
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
    let result;

    loop {
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

        if current_result == ERROR_BUFFER_OVERFLOW {
            continue;
        }
        result = current_result;
        break;
    }

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
                let id = CStr::from_ptr(current.AdapterName)
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

    let routes = unsafe {
        let rows =
            std::slice::from_raw_parts((*table).Table.as_ptr(), (*table).NumEntries as usize);
        rows.iter()
            .filter_map(|row| {
                let destination = sockaddr_to_ip(row.DestinationPrefix.Prefix)?;
                let next_hop =
                    sockaddr_to_ip(row.NextHop).filter(|address| !address.is_unspecified());
                let interface_id = interfaces
                    .iter()
                    .find(|interface| {
                        if destination.is_ipv4() {
                            interface.ipv4_index == Some(row.InterfaceIndex)
                        } else {
                            interface.ipv6_index == Some(row.InterfaceIndex)
                        }
                    })
                    .map(|interface| interface.id.clone());

                Some(NetworkRoute {
                    destination: IpPrefix {
                        address: destination,
                        prefix_length: row.DestinationPrefix.PrefixLength,
                    },
                    next_hop,
                    interface_index: row.InterfaceIndex,
                    interface_id,
                    metric: row.Metric,
                })
            })
            .collect::<Vec<_>>()
    };

    unsafe { FreeMibTable(table.cast()) };
    Ok(routes)
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
    use super::sockaddr_to_ip;
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
