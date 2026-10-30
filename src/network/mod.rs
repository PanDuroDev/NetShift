//! Read-only network inspection boundary.
//!
//! Implementations belong in platform-specific adapters. This module contains
//! portable contracts and must never modify routes, DNS, adapters, or IPs.

use crate::core::network::NetworkSnapshot;

/// Reads the current network state without changing host configuration.
pub trait NetworkInspector {
    /// Collects a snapshot of the current network state.
    fn inspect(&self) -> Result<NetworkSnapshot, NetworkInspectionError>;
}

/// Errors that can occur while reading network state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NetworkInspectionError {
    /// The current process does not have permission to read the requested data.
    PermissionDenied,
    /// The current operating system or API does not support the requested query.
    Unsupported,
    /// The platform query failed for another reason.
    QueryFailed(String),
}

impl std::fmt::Display for NetworkInspectionError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PermissionDenied => {
                formatter.write_str("permission denied while inspecting network state")
            }
            Self::Unsupported => {
                formatter.write_str("network inspection is unsupported on this platform")
            }
            Self::QueryFailed(message) => write!(formatter, "network inspection failed: {message}"),
        }
    }
}

impl std::error::Error for NetworkInspectionError {}

#[cfg(test)]
mod tests {
    use super::{NetworkInspectionError, NetworkInspector};
    use crate::core::network::NetworkSnapshot;

    struct FakeInspector;

    impl NetworkInspector for FakeInspector {
        fn inspect(&self) -> Result<NetworkSnapshot, NetworkInspectionError> {
            Ok(NetworkSnapshot::default())
        }
    }

    #[test]
    fn inspector_contract_can_be_tested_without_host_network_access() {
        let snapshot = FakeInspector
            .inspect()
            .expect("fake inspection should succeed");
        assert!(snapshot.interfaces.is_empty());
    }

    #[test]
    fn inspection_errors_have_readable_messages() {
        let error = NetworkInspectionError::PermissionDenied;
        assert_eq!(
            error.to_string(),
            "permission denied while inspecting network state"
        );
    }
}
