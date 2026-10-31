//! Cross-module safety tests for network inspection and provider lifecycle execution.
//!
//! All fixtures are in-memory. These tests never alter the host network, routes,
//! DNS configuration, adapters, firewall, or router settings.

use std::cell::Cell;
use std::net::{IpAddr, Ipv4Addr};
use std::rc::Rc;

use netshift::app::inspect_network;
use netshift::core::network::{NetworkInterface, NetworkSnapshot};
use netshift::network::{NetworkInspectionError, NetworkInspector};
use netshift::providers::manager::{ManagerError, OperationOutcome, ProviderManager};
use netshift::providers::{
    ChangeScope, ConnectionProvider, PlannedChange, ProviderAvailability, ProviderCapabilities,
    ProviderChangePlan, ProviderError, ProviderErrorCategory, ProviderMetadata, ProviderOperation,
    ProviderSnapshot, ProviderStatus,
};

struct FixtureInspector {
    calls: Rc<Cell<usize>>,
    result: Result<NetworkSnapshot, NetworkInspectionError>,
}

impl NetworkInspector for FixtureInspector {
    fn inspect(&self) -> Result<NetworkSnapshot, NetworkInspectionError> {
        self.calls.set(self.calls.get() + 1);
        self.result.clone()
    }
}

struct FixtureProvider {
    status: ProviderStatus,
    scope: ChangeScope,
    calls: Rc<Cell<usize>>,
    fail_connect: bool,
}

impl FixtureProvider {
    fn new(scope: ChangeScope, calls: Rc<Cell<usize>>) -> Self {
        Self {
            status: ProviderStatus::Disconnected,
            scope,
            calls,
            fail_connect: false,
        }
    }

    fn snapshot(&self) -> ProviderSnapshot {
        ProviderSnapshot {
            status: self.status,
            availability: ProviderAvailability {
                available: true,
                prerequisites: Vec::new(),
            },
        }
    }
}

impl ConnectionProvider for FixtureProvider {
    fn metadata(&self) -> ProviderMetadata {
        ProviderMetadata {
            id: "integration.fixture".to_owned(),
            display_name: "Integration Fixture".to_owned(),
            description: "In-memory integration-test provider".to_owned(),
        }
    }

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            status_inspection: true,
            connect: true,
            disconnect: true,
            device_scoped_routing: self.scope == ChangeScope::ThisDevice,
            ..ProviderCapabilities::default()
        }
    }

    fn inspect(&self) -> Result<ProviderSnapshot, ProviderError> {
        Ok(self.snapshot())
    }

    fn plan_connect(&self) -> Result<ProviderChangePlan, ProviderError> {
        Ok(ProviderChangePlan {
            operation: ProviderOperation::Connect,
            scope: self.scope,
            changes: vec![PlannedChange {
                category: "tunnel".to_owned(),
                description: "Enable a device-local test tunnel".to_owned(),
            }],
        })
    }

    fn plan_disconnect(&self) -> Result<ProviderChangePlan, ProviderError> {
        Ok(ProviderChangePlan {
            operation: ProviderOperation::Disconnect,
            scope: self.scope,
            changes: vec![PlannedChange {
                category: "tunnel".to_owned(),
                description: "Disable a device-local test tunnel".to_owned(),
            }],
        })
    }

    fn connect(&mut self) -> Result<(), ProviderError> {
        self.calls.set(self.calls.get() + 1);
        if self.fail_connect {
            self.status = ProviderStatus::Error;
            return Err(ProviderError::new(ProviderErrorCategory::OperationFailed));
        }
        self.status = ProviderStatus::Connected;
        Ok(())
    }

    fn disconnect(&mut self) -> Result<(), ProviderError> {
        self.calls.set(self.calls.get() + 1);
        self.status = ProviderStatus::Disconnected;
        Ok(())
    }
}

fn interface_fixture() -> NetworkSnapshot {
    NetworkSnapshot {
        interfaces: vec![NetworkInterface {
            id: "fixture-adapter".to_owned(),
            name: "Fixture Ethernet".to_owned(),
            ipv4_index: Some(4),
            ipv6_index: Some(4),
            is_up: true,
            addresses: vec![IpAddr::V4(Ipv4Addr::new(192, 0, 2, 10))],
            dns_servers: vec![IpAddr::V4(Ipv4Addr::new(192, 0, 2, 53))],
        }],
        ..NetworkSnapshot::default()
    }
}

#[test]
fn inspection_and_provider_registration_do_not_start_operations() {
    let inspection_calls = Rc::new(Cell::new(0));
    let inspector = FixtureInspector {
        calls: inspection_calls.clone(),
        result: Ok(interface_fixture()),
    };
    let snapshot = inspect_network(&inspector).expect("fixture inspection succeeds");
    assert_eq!(snapshot.interfaces.len(), 1);
    assert_eq!(inspection_calls.get(), 1);

    let provider_calls = Rc::new(Cell::new(0));
    let mut manager = ProviderManager::new();
    manager
        .register(FixtureProvider::new(
            ChangeScope::ThisDevice,
            provider_calls.clone(),
        ))
        .expect("fixture registration succeeds");
    assert_eq!(
        manager
            .inspect("integration.fixture")
            .expect("provider inspection succeeds")
            .status,
        ProviderStatus::Disconnected
    );
    assert_eq!(provider_calls.get(), 0);
}

#[test]
fn shared_or_unknown_scope_is_rejected_before_provider_execution() {
    for scope in [ChangeScope::SharedNetwork, ChangeScope::Unknown] {
        let calls = Rc::new(Cell::new(0));
        let mut manager = ProviderManager::new();
        manager
            .register(FixtureProvider::new(scope, calls.clone()))
            .expect("fixture registration succeeds");

        assert_eq!(
            manager.connect("integration.fixture"),
            Err(ManagerError::UnsafeChangeScope { actual: scope })
        );
        assert_eq!(calls.get(), 0);
        assert_eq!(
            manager
                .inspect("integration.fixture")
                .expect("provider remains inspectable")
                .status,
            ProviderStatus::Disconnected
        );
    }
}

#[test]
fn shared_or_unknown_disconnect_is_rejected_before_provider_execution() {
    for scope in [ChangeScope::SharedNetwork, ChangeScope::Unknown] {
        let calls = Rc::new(Cell::new(0));
        let mut provider = FixtureProvider::new(scope, calls.clone());
        provider.status = ProviderStatus::Connected;
        let mut manager = ProviderManager::new();
        manager
            .register(provider)
            .expect("fixture registration succeeds");

        assert_eq!(
            manager.disconnect("integration.fixture"),
            Err(ManagerError::UnsafeChangeScope { actual: scope })
        );
        assert_eq!(calls.get(), 0);
        assert_eq!(
            manager
                .inspect("integration.fixture")
                .expect("provider remains inspectable")
                .status,
            ProviderStatus::Connected
        );
    }
}

#[test]
fn only_an_explicit_device_scoped_request_can_connect_and_report_success() {
    let calls = Rc::new(Cell::new(0));
    let mut manager = ProviderManager::new();
    manager
        .register(FixtureProvider::new(ChangeScope::ThisDevice, calls.clone()))
        .expect("fixture registration succeeds");
    assert_eq!(calls.get(), 0);

    let report = manager
        .connect("integration.fixture")
        .expect("explicit connect request is accepted");
    assert_eq!(calls.get(), 1);
    assert_eq!(report.plan.scope, ChangeScope::ThisDevice);
    assert_eq!(report.before.status, ProviderStatus::Disconnected);
    assert!(matches!(
        report.outcome,
        OperationOutcome::Succeeded { after }
            if after.status == ProviderStatus::Connected
    ));
}

#[test]
fn failed_provider_operation_is_reported_as_failed_with_observed_state() {
    let calls = Rc::new(Cell::new(0));
    let mut provider = FixtureProvider::new(ChangeScope::ThisDevice, calls.clone());
    provider.fail_connect = true;
    let mut manager = ProviderManager::new();
    manager
        .register(provider)
        .expect("fixture registration succeeds");

    let report = manager
        .connect("integration.fixture")
        .expect("operation report is returned");
    assert_eq!(calls.get(), 1);
    assert!(matches!(
        report.outcome,
        OperationOutcome::Failed { error, after: Some(after) }
            if error.category == ProviderErrorCategory::OperationFailed
                && after.status == ProviderStatus::Error
    ));
}

#[test]
fn app_inspection_flow_propagates_a_platform_query_failure() {
    let calls = Rc::new(Cell::new(0));
    let inspector = FixtureInspector {
        calls: calls.clone(),
        result: Err(NetworkInspectionError::QueryFailed(
            "fixture failure".to_owned(),
        )),
    };

    let result = inspect_network(&inspector);

    assert_eq!(
        result,
        Err(NetworkInspectionError::QueryFailed(
            "fixture failure".to_owned()
        ))
    );
    assert_eq!(calls.get(), 1);
}
