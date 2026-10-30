//! Explicit, serialized lifecycle coordination for registered providers.
//!
//! The manager never starts work in the background. Each operation is invoked
//! only through an explicit method call and is preceded by inspection and a
//! provider-generated change preview.

use std::collections::BTreeMap;

use super::{
    ConnectionProvider, ProviderChangePlan, ProviderError, ProviderErrorCategory, ProviderMetadata,
    ProviderOperation, ProviderSnapshot, ProviderStatus,
};

/// Errors detected by the manager before an operation can be safely invoked.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ManagerError {
    /// No provider is registered under this identifier.
    UnknownProvider,
    /// A provider with this identifier is already registered.
    DuplicateProvider,
    /// Provider identifiers must not be empty or whitespace.
    InvalidProviderId,
    /// The provider cannot inspect its current state.
    StatusInspectionUnsupported,
    /// The requested operation is not implemented by the provider.
    UnsupportedOperation,
    /// The provider is not currently available.
    Unavailable,
    /// The provider is not in the state required for this operation.
    InvalidState {
        /// State required before the operation.
        expected: ProviderStatus,
        /// State observed before the operation.
        actual: ProviderStatus,
    },
    /// The provider returned a plan for a different operation.
    InvalidPlan,
    /// Provider inspection or planning failed before execution.
    Provider(ProviderError),
}

impl std::fmt::Display for ManagerError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownProvider => formatter.write_str("provider is not registered"),
            Self::DuplicateProvider => {
                formatter.write_str("provider identifier is already registered")
            }
            Self::InvalidProviderId => formatter.write_str("provider identifier must not be empty"),
            Self::StatusInspectionUnsupported => {
                formatter.write_str("provider does not support status inspection")
            }
            Self::UnsupportedOperation => {
                formatter.write_str("provider does not support this operation")
            }
            Self::Unavailable => formatter.write_str("provider is currently unavailable"),
            Self::InvalidState { expected, actual } => {
                write!(
                    formatter,
                    "provider state is {actual:?}; expected {expected:?}"
                )
            }
            Self::InvalidPlan => {
                formatter.write_str("provider returned a plan for the wrong operation")
            }
            Self::Provider(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for ManagerError {}

/// Outcome after an operation was invoked.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OperationOutcome {
    /// The provider confirmed the requested target state.
    Succeeded {
        /// State observed after provider confirmation.
        after: ProviderSnapshot,
    },
    /// The operation failed or the provider did not confirm the target state.
    Failed {
        /// Sanitized failure category.
        error: ProviderError,
        /// Best-effort state observed after the operation; absent if inspection failed.
        after: Option<ProviderSnapshot>,
    },
}

/// Auditable result retaining the state observed before and after execution.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OperationReport {
    /// Stable identifier of the provider that was invoked.
    pub provider_id: String,
    /// Explicitly requested operation.
    pub operation: ProviderOperation,
    /// State observed before execution.
    pub before: ProviderSnapshot,
    /// Preview returned before execution.
    pub plan: ProviderChangePlan,
    /// Provider-confirmed outcome and best-effort resulting state.
    pub outcome: OperationOutcome,
}

/// Coordinates registered providers without performing OS network changes itself.
///
/// Mutable access serializes operations through this manager: callers cannot
/// concurrently invoke two operations through the same manager instance.
#[derive(Default)]
pub struct ProviderManager {
    providers: BTreeMap<String, Box<dyn ConnectionProvider>>,
}

impl ProviderManager {
    /// Creates an empty manager.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a provider under its stable metadata identifier.
    pub fn register<P>(&mut self, provider: P) -> Result<(), ManagerError>
    where
        P: ConnectionProvider + 'static,
    {
        let id = provider.metadata().id;
        if id.trim().is_empty() {
            return Err(ManagerError::InvalidProviderId);
        }
        match self.providers.entry(id) {
            std::collections::btree_map::Entry::Vacant(entry) => {
                entry.insert(Box::new(provider));
                Ok(())
            }
            std::collections::btree_map::Entry::Occupied(_) => Err(ManagerError::DuplicateProvider),
        }
    }

    /// Returns provider metadata in stable identifier order.
    pub fn providers(&self) -> Vec<ProviderMetadata> {
        self.providers
            .values()
            .map(|provider| provider.metadata())
            .collect()
    }

    /// Inspects one registered provider without changing its state.
    pub fn inspect(&self, provider_id: &str) -> Result<ProviderSnapshot, ManagerError> {
        let provider = self
            .providers
            .get(provider_id)
            .ok_or(ManagerError::UnknownProvider)?;
        if !provider.capabilities().status_inspection {
            return Err(ManagerError::StatusInspectionUnsupported);
        }
        provider.inspect().map_err(ManagerError::Provider)
    }

    /// Executes a connection only when explicitly called by the application.
    pub fn connect(&mut self, provider_id: &str) -> Result<OperationReport, ManagerError> {
        self.execute(provider_id, ProviderOperation::Connect)
    }

    /// Executes a disconnection only when explicitly called by the application.
    pub fn disconnect(&mut self, provider_id: &str) -> Result<OperationReport, ManagerError> {
        self.execute(provider_id, ProviderOperation::Disconnect)
    }

    fn execute(
        &mut self,
        provider_id: &str,
        operation: ProviderOperation,
    ) -> Result<OperationReport, ManagerError> {
        let provider = self
            .providers
            .get_mut(provider_id)
            .ok_or(ManagerError::UnknownProvider)?;
        let capabilities = provider.capabilities();
        if !capabilities.status_inspection {
            return Err(ManagerError::StatusInspectionUnsupported);
        }
        let supported = match operation {
            ProviderOperation::Connect => capabilities.connect,
            ProviderOperation::Disconnect => capabilities.disconnect,
        };
        if !supported {
            return Err(ManagerError::UnsupportedOperation);
        }

        let before = provider.inspect().map_err(ManagerError::Provider)?;
        if !before.availability.available {
            return Err(ManagerError::Unavailable);
        }

        let (required_state, target_state) = match operation {
            ProviderOperation::Connect => (ProviderStatus::Disconnected, ProviderStatus::Connected),
            ProviderOperation::Disconnect => {
                (ProviderStatus::Connected, ProviderStatus::Disconnected)
            }
        };
        if before.status != required_state {
            return Err(ManagerError::InvalidState {
                expected: required_state,
                actual: before.status,
            });
        }

        let plan = match operation {
            ProviderOperation::Connect => provider.plan_connect(),
            ProviderOperation::Disconnect => provider.plan_disconnect(),
        }
        .map_err(ManagerError::Provider)?;
        if plan.operation != operation {
            return Err(ManagerError::InvalidPlan);
        }

        let execution = match operation {
            ProviderOperation::Connect => provider.connect(),
            ProviderOperation::Disconnect => provider.disconnect(),
        };

        let outcome = match execution {
            Err(error) => OperationOutcome::Failed {
                error,
                after: provider.inspect().ok(),
            },
            Ok(()) => match provider.inspect() {
                Ok(after) if after.status == target_state => OperationOutcome::Succeeded { after },
                Ok(after) => OperationOutcome::Failed {
                    error: ProviderError::new(ProviderErrorCategory::OperationFailed),
                    after: Some(after),
                },
                Err(error) => OperationOutcome::Failed {
                    error,
                    after: None,
                },
            },
        };

        Ok(OperationReport {
            provider_id: provider_id.to_owned(),
            operation,
            before,
            plan,
            outcome,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::rc::Rc;

    use super::{ManagerError, OperationOutcome, ProviderManager};
    use crate::providers::{
        ChangeScope, ConnectionProvider, PlannedChange, ProviderAvailability, ProviderCapabilities,
        ProviderChangePlan, ProviderError, ProviderErrorCategory, ProviderMetadata,
        ProviderOperation, ProviderSnapshot, ProviderStatus,
    };

    struct MockProvider {
        id: &'static str,
        status: ProviderStatus,
        connect_supported: bool,
        disconnect_supported: bool,
        fail_connect: bool,
        update_status_on_connect: bool,
        inspect_error_when_connected: bool,
        calls: Rc<Cell<usize>>,
    }

    impl MockProvider {
        fn new(id: &'static str, status: ProviderStatus, calls: Rc<Cell<usize>>) -> Self {
            Self {
                id,
                status,
                connect_supported: true,
                disconnect_supported: true,
                fail_connect: false,
                update_status_on_connect: true,
                inspect_error_when_connected: false,
                calls,
            }
        }
    }

    impl ConnectionProvider for MockProvider {
        fn metadata(&self) -> ProviderMetadata {
            ProviderMetadata {
                id: self.id.to_owned(),
                display_name: self.id.to_owned(),
                description: "Deterministic test provider".to_owned(),
            }
        }

        fn capabilities(&self) -> ProviderCapabilities {
            ProviderCapabilities {
                status_inspection: true,
                connect: self.connect_supported,
                disconnect: self.disconnect_supported,
                ..ProviderCapabilities::default()
            }
        }

        fn inspect(&self) -> Result<ProviderSnapshot, ProviderError> {
            if self.inspect_error_when_connected && self.status == ProviderStatus::Connected {
                return Err(ProviderError::new(ProviderErrorCategory::MissingPrerequisite));
            }
            Ok(ProviderSnapshot {
                status: self.status,
                availability: ProviderAvailability {
                    available: self.status != ProviderStatus::Unavailable,
                    prerequisites: Vec::new(),
                },
            })
        }

        fn plan_connect(&self) -> Result<ProviderChangePlan, ProviderError> {
            Ok(ProviderChangePlan {
                operation: ProviderOperation::Connect,
                scope: ChangeScope::ThisDevice,
                changes: vec![PlannedChange {
                    category: "mock".to_owned(),
                    description: "Mock connect".to_owned(),
                }],
            })
        }

        fn plan_disconnect(&self) -> Result<ProviderChangePlan, ProviderError> {
            Ok(ProviderChangePlan {
                operation: ProviderOperation::Disconnect,
                scope: ChangeScope::ThisDevice,
                changes: Vec::new(),
            })
        }

        fn connect(&mut self) -> Result<(), ProviderError> {
            self.calls.set(self.calls.get() + 1);
            if self.fail_connect {
                self.status = ProviderStatus::Error;
                return Err(ProviderError::new(ProviderErrorCategory::OperationFailed));
            }
            if self.update_status_on_connect {
                self.status = ProviderStatus::Connected;
            }
            Ok(())
        }

        fn disconnect(&mut self) -> Result<(), ProviderError> {
            self.calls.set(self.calls.get() + 1);
            self.status = ProviderStatus::Disconnected;
            Ok(())
        }
    }

    #[test]
    fn registers_enumerates_and_inspects_providers() {
        let calls = Rc::new(Cell::new(0));
        let mut manager = ProviderManager::new();
        manager
            .register(MockProvider::new(
                "mock.alpha",
                ProviderStatus::Disconnected,
                calls,
            ))
            .expect("registration should succeed");

        assert_eq!(manager.providers().len(), 1);
        assert_eq!(manager.providers()[0].id, "mock.alpha");
        assert_eq!(
            manager
                .inspect("mock.alpha")
                .expect("inspection should succeed")
                .status,
            ProviderStatus::Disconnected
        );
    }

    #[test]
    fn no_operation_runs_without_an_explicit_manager_request() {
        let calls = Rc::new(Cell::new(0));
        let mut manager = ProviderManager::new();
        manager
            .register(MockProvider::new(
                "mock.alpha",
                ProviderStatus::Disconnected,
                calls.clone(),
            ))
            .expect("registration should succeed");

        assert_eq!(calls.get(), 0);
    }

    #[test]
    fn explicit_connect_and_disconnect_are_confirmed_by_inspection() {
        let calls = Rc::new(Cell::new(0));
        let mut manager = ProviderManager::new();
        manager
            .register(MockProvider::new(
                "mock.alpha",
                ProviderStatus::Disconnected,
                calls.clone(),
            ))
            .expect("registration should succeed");

        let connected = manager
            .connect("mock.alpha")
            .expect("connect should be invoked");
        assert!(matches!(
            connected.outcome,
            OperationOutcome::Succeeded { .. }
        ));
        assert_eq!(connected.before.status, ProviderStatus::Disconnected);

        let disconnected = manager
            .disconnect("mock.alpha")
            .expect("disconnect should be invoked");
        assert!(matches!(
            disconnected.outcome,
            OperationOutcome::Succeeded { .. }
        ));
        assert_eq!(disconnected.before.status, ProviderStatus::Connected);
        assert_eq!(calls.get(), 2);
    }

    #[test]
    fn unsupported_operation_is_rejected_before_provider_execution() {
        let calls = Rc::new(Cell::new(0));
        let mut provider =
            MockProvider::new("mock.alpha", ProviderStatus::Disconnected, calls.clone());
        provider.connect_supported = false;
        let mut manager = ProviderManager::new();
        manager
            .register(provider)
            .expect("registration should succeed");

        assert_eq!(
            manager.connect("mock.alpha"),
            Err(ManagerError::UnsupportedOperation)
        );
        assert_eq!(calls.get(), 0);
    }

    #[test]
    fn unavailable_or_wrong_state_is_rejected_before_execution() {
        let calls = Rc::new(Cell::new(0));
        let mut manager = ProviderManager::new();
        manager
            .register(MockProvider::new(
                "mock.alpha",
                ProviderStatus::Unavailable,
                calls.clone(),
            ))
            .expect("registration should succeed");

        assert_eq!(
            manager.connect("mock.alpha"),
            Err(ManagerError::Unavailable)
        );
        assert_eq!(calls.get(), 0);
    }

    #[test]
    fn successful_operation_requires_confirmed_target_state() {
        let calls = Rc::new(Cell::new(0));
        let mut provider =
            MockProvider::new("mock.alpha", ProviderStatus::Disconnected, calls.clone());
        provider.update_status_on_connect = false;
        let mut manager = ProviderManager::new();
        manager
            .register(provider)
            .expect("registration should succeed");

        let report = manager
            .connect("mock.alpha")
            .expect("operation should be reported");
        match report.outcome {
            OperationOutcome::Failed { error, after } => {
                assert_eq!(error.category, ProviderErrorCategory::OperationFailed);
                assert_eq!(
                    after.expect("post-operation state should be observable").status,
                    ProviderStatus::Disconnected
                );
            }
            OperationOutcome::Succeeded { .. } => {
                panic!("unconfirmed target state must not be reported as success")
            }
        }
        assert_eq!(calls.get(), 1);
    }

    #[test]
    fn wrong_starting_state_is_rejected_before_execution() {
        let calls = Rc::new(Cell::new(0));
        let mut manager = ProviderManager::new();
        manager
            .register(MockProvider::new(
                "mock.connected",
                ProviderStatus::Connected,
                calls.clone(),
            ))
            .expect("registration should succeed");
        manager
            .register(MockProvider::new(
                "mock.disconnected",
                ProviderStatus::Disconnected,
                calls.clone(),
            ))
            .expect("registration should succeed");

        assert_eq!(
            manager.connect("mock.connected"),
            Err(ManagerError::InvalidState {
                expected: ProviderStatus::Disconnected,
                actual: ProviderStatus::Connected,
            })
        );
        assert_eq!(
            manager.disconnect("mock.disconnected"),
            Err(ManagerError::InvalidState {
                expected: ProviderStatus::Connected,
                actual: ProviderStatus::Disconnected,
            })
        );
        assert_eq!(calls.get(), 0);
    }

    #[test]
    fn post_operation_inspection_error_is_preserved() {
        let calls = Rc::new(Cell::new(0));
        let mut provider =
            MockProvider::new("mock.alpha", ProviderStatus::Disconnected, calls);
        provider.inspect_error_when_connected = true;
        let mut manager = ProviderManager::new();
        manager
            .register(provider)
            .expect("registration should succeed");

        let report = manager
            .connect("mock.alpha")
            .expect("operation should be reported");
        match report.outcome {
            OperationOutcome::Failed { error, after } => {
                assert_eq!(error.category, ProviderErrorCategory::MissingPrerequisite);
                assert!(after.is_none());
            }
            OperationOutcome::Succeeded { .. } => {
                panic!("inspection failure must not be reported as success")
            }
        }
    }

    #[test]
    fn failed_operation_reports_failure_and_observed_resulting_state() {
        let calls = Rc::new(Cell::new(0));
        let mut provider = MockProvider::new("mock.alpha", ProviderStatus::Disconnected, calls);
        provider.fail_connect = true;
        let mut manager = ProviderManager::new();
        manager
            .register(provider)
            .expect("registration should succeed");

        let report = manager
            .connect("mock.alpha")
            .expect("operation should be reported");
        match report.outcome {
            OperationOutcome::Failed { error, after } => {
                assert_eq!(error.category, ProviderErrorCategory::OperationFailed);
                assert_eq!(
                    after
                        .expect("post-failure state should be observable")
                        .status,
                    ProviderStatus::Error
                );
            }
            OperationOutcome::Succeeded { .. } => {
                panic!("failed operation must not be reported as success")
            }
        }
    }

    #[test]
    fn duplicate_provider_ids_are_rejected() {
        let calls = Rc::new(Cell::new(0));
        let mut manager = ProviderManager::new();
        manager
            .register(MockProvider::new(
                "mock.alpha",
                ProviderStatus::Disconnected,
                calls.clone(),
            ))
            .expect("first registration should succeed");

        assert_eq!(
            manager.register(MockProvider::new(
                "mock.alpha",
                ProviderStatus::Disconnected,
                calls
            )),
            Err(ManagerError::DuplicateProvider)
        );
    }
}
