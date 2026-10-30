//! Provider-neutral contracts for optional connection methods.
//!
//! This module defines descriptions and requested-operation plans only. It does
//! not load third-party plugins, access credentials, or change network state.

/// Stable identity and user-facing metadata for a connection provider.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderMetadata {
    /// Stable, provider-neutral identifier (for example, "cloudflare-warp").
    pub id: String,
    /// Human-readable provider name.
    pub display_name: String,
    /// Short, non-sensitive description.
    pub description: String,
}

/// Explicit features implemented by a provider.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ProviderCapabilities {
    /// Provider can carry IPv4 traffic.
    pub ipv4: bool,
    /// Provider can carry IPv6 traffic.
    pub ipv6: bool,
    /// Provider can restrict routing changes to this device.
    pub device_scoped_routing: bool,
    /// Provider can report its current connection state.
    pub status_inspection: bool,
}

/// A prerequisite checked by a provider before an operation is offered.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderPrerequisite {
    /// Stable machine-readable prerequisite identifier.
    pub id: String,
    /// Safe, user-facing explanation of the prerequisite.
    pub description: String,
    /// Whether the prerequisite is currently satisfied.
    pub satisfied: bool,
}

/// Whether a provider can currently be used.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ProviderAvailability {
    /// False when the provider is absent, disabled, or otherwise unavailable.
    pub available: bool,
    /// Explicit prerequisite results; an empty list means no prerequisites were reported.
    pub prerequisites: Vec<ProviderPrerequisite>,
}

/// Current lifecycle state reported by a provider.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ProviderStatus {
    /// Provider is not installed or cannot currently be accessed.
    Unavailable,
    /// Provider is available but not connected.
    #[default]
    Disconnected,
    /// A user-requested connection operation is in progress.
    Connecting,
    /// Provider reports an active connection.
    Connected,
    /// A user-requested disconnection operation is in progress.
    Disconnecting,
    /// Provider reports an error state.
    Error,
}

/// A provider's current, sanitized state.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ProviderSnapshot {
    /// Current lifecycle state.
    pub status: ProviderStatus,
    /// Availability and prerequisite details.
    pub availability: ProviderAvailability,
}

/// Stable categories for provider failures.
/// Provider-specific error strings and diagnostic payloads must not cross this boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProviderErrorCategory {
    /// The operation requires elevated permissions.
    PermissionDenied,
    /// A required application, service, driver, or configuration is missing.
    MissingPrerequisite,
    /// The provider does not implement the requested capability or operation.
    Unsupported,
    /// The requested target state is already active.
    AlreadyInState,
    /// The provider is temporarily unavailable.
    TemporarilyUnavailable,
    /// The provider operation failed for another reason.
    OperationFailed,
}

/// Sanitized error exposed to the core and UI.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProviderError {
    /// Stable error classification; no provider-specific output is retained.
    pub category: ProviderErrorCategory,
}

impl ProviderError {
    /// Creates a sanitized provider error from its stable category.
    pub const fn new(category: ProviderErrorCategory) -> Self {
        Self { category }
    }
}

impl std::fmt::Display for ProviderError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self.category {
            ProviderErrorCategory::PermissionDenied => "provider operation requires permission",
            ProviderErrorCategory::MissingPrerequisite => "provider prerequisite is missing",
            ProviderErrorCategory::Unsupported => "provider does not support this operation",
            ProviderErrorCategory::AlreadyInState => "provider is already in the requested state",
            ProviderErrorCategory::TemporarilyUnavailable => "provider is temporarily unavailable",
            ProviderErrorCategory::OperationFailed => "provider operation failed",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for ProviderError {}

/// The kind of connection operation a plan describes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProviderOperation {
    /// User-requested connection.
    Connect,
    /// User-requested disconnection.
    Disconnect,
}

/// Scope of the network impact described by a plan.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ChangeScope {
    /// Changes are limited to this device.
    ThisDevice,
    /// Changes may affect the shared network or other devices.
    SharedNetwork,
    /// Provider cannot determine the impact scope before execution.
    Unknown,
}

/// A non-secret description of one intended network change.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlannedChange {
    /// Short category such as "routes", "DNS", or "tunnel".
    pub category: String,
    /// Human-readable description of the intended change.
    pub description: String,
}

/// A preview of intended effects. Creating a plan never applies it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderChangePlan {
    /// Operation this plan describes.
    pub operation: ProviderOperation,
    /// Expected scope of the network impact.
    pub scope: ChangeScope,
    /// Individual intended changes.
    pub changes: Vec<PlannedChange>,
}

/// Provider-neutral interface implemented by built-in or mock providers.
///
/// Methods are observational or produce previews only. Applying a plan is
/// deliberately outside this stage and must require a separate explicit user
/// action in a later lifecycle manager.
pub trait ConnectionProvider {
    /// Returns stable identity and display metadata.
    fn metadata(&self) -> ProviderMetadata;

    /// Returns the provider's explicitly supported capabilities.
    fn capabilities(&self) -> ProviderCapabilities;

    /// Inspects current status and prerequisites without changing network state.
    fn inspect(&self) -> Result<ProviderSnapshot, ProviderError>;

    /// Previews the effects of a user-requested connection.
    fn plan_connect(&self) -> Result<ProviderChangePlan, ProviderError>;

    /// Previews the effects of a user-requested disconnection.
    fn plan_disconnect(&self) -> Result<ProviderChangePlan, ProviderError>;
}

#[cfg(test)]
mod tests {
    use super::{
        ChangeScope, ConnectionProvider, PlannedChange, ProviderAvailability,
        ProviderCapabilities, ProviderChangePlan, ProviderError, ProviderErrorCategory,
        ProviderMetadata, ProviderOperation, ProviderPrerequisite, ProviderSnapshot,
        ProviderStatus,
    };

    struct MockProvider {
        status: ProviderStatus,
    }

    impl ConnectionProvider for MockProvider {
        fn metadata(&self) -> ProviderMetadata {
            ProviderMetadata {
                id: "mock.test".to_owned(),
                display_name: "Mock Provider".to_owned(),
                description: "Test-only provider".to_owned(),
            }
        }

        fn capabilities(&self) -> ProviderCapabilities {
            ProviderCapabilities {
                ipv4: true,
                ipv6: false,
                device_scoped_routing: true,
                status_inspection: true,
            }
        }

        fn inspect(&self) -> Result<ProviderSnapshot, ProviderError> {
            Ok(ProviderSnapshot {
                status: self.status,
                availability: ProviderAvailability {
                    available: true,
                    prerequisites: vec![ProviderPrerequisite {
                        id: "mock-ready".to_owned(),
                        description: "Mock provider is ready".to_owned(),
                        satisfied: true,
                    }],
                },
            })
        }

        fn plan_connect(&self) -> Result<ProviderChangePlan, ProviderError> {
            Ok(ProviderChangePlan {
                operation: ProviderOperation::Connect,
                scope: ChangeScope::ThisDevice,
                changes: vec![PlannedChange {
                    category: "tunnel".to_owned(),
                    description: "Create a device-scoped test tunnel".to_owned(),
                }],
            })
        }

        fn plan_disconnect(&self) -> Result<ProviderChangePlan, ProviderError> {
            Ok(ProviderChangePlan {
                operation: ProviderOperation::Disconnect,
                scope: ChangeScope::ThisDevice,
                changes: vec![PlannedChange {
                    category: "tunnel".to_owned(),
                    description: "Remove the device-scoped test tunnel".to_owned(),
                }],
            })
        }
    }

    #[test]
    fn mock_provider_reports_only_explicit_capabilities() {
        let provider = MockProvider {
            status: ProviderStatus::Disconnected,
        };
        let capabilities = provider.capabilities();

        assert!(capabilities.ipv4);
        assert!(!capabilities.ipv6);
        assert!(capabilities.device_scoped_routing);
        assert!(capabilities.status_inspection);
    }

    #[test]
    fn mock_provider_reports_status_and_prerequisites() {
        let provider = MockProvider {
            status: ProviderStatus::Connected,
        };
        let snapshot = provider.inspect().expect("mock inspection should succeed");

        assert_eq!(snapshot.status, ProviderStatus::Connected);
        assert!(snapshot.availability.available);
        assert!(snapshot.availability.prerequisites[0].satisfied);
    }

    #[test]
    fn connect_and_disconnect_return_previews_without_applying_changes() {
        let provider = MockProvider {
            status: ProviderStatus::Disconnected,
        };
        let connect = provider.plan_connect().expect("connect plan should exist");
        let disconnect = provider.plan_disconnect().expect("disconnect plan should exist");

        assert_eq!(connect.operation, ProviderOperation::Connect);
        assert_eq!(disconnect.operation, ProviderOperation::Disconnect);
        assert_eq!(connect.scope, ChangeScope::ThisDevice);
        assert_eq!(connect.changes.len(), 1);
    }

    #[test]
    fn provider_errors_map_to_stable_sanitized_categories() {
        let error = ProviderError::new(ProviderErrorCategory::MissingPrerequisite);

        assert_eq!(error.category, ProviderErrorCategory::MissingPrerequisite);
        assert_eq!(error.to_string(), "provider prerequisite is missing");
        assert!(!error.to_string().contains("token"));
    }

    #[test]
    fn unsupported_capabilities_are_not_assumed() {
        let provider = MockProvider {
            status: ProviderStatus::Unavailable,
        };
        let snapshot = provider.inspect().expect("mock inspection should succeed");

        assert_eq!(snapshot.status, ProviderStatus::Unavailable);
        assert!(!provider.capabilities().ipv6);
    }
}
