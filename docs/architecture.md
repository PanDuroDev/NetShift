# NetShift Architecture

## Current stage

Stage 2 provides read-only Windows network discovery. Stage 3 defines the provider-neutral contract. Stage 4 adds an explicit, serialized lifecycle manager with deterministic mock providers. No production provider is integrated.

## Dependency direction

Dependencies point inward:

    app
     ├── core
     ├── network (read-only inspection contract)
     ├── providers (provider-neutral contracts)
     └── platform::windows
           ├── ipconfig
           └── Windows IP Helper APIs

Rules:

1. core contains platform-independent domain data. It must not import Windows APIs, UI frameworks, provider SDKs, or operating-system adapters.
2. network defines portable read-only inspection contracts and their errors. It does not call operating-system APIs directly.
3. providers defines provider metadata, capabilities, availability, lifecycle status, sanitized errors, and change previews. It must not depend on a specific provider SDK or platform adapter.
4. platform::windows implements discovery using read-only operating-system APIs and translates results into portable models.
5. app is the composition root. It invokes application services and does not call Windows APIs directly.
6. UI code, when introduced, will call application services. It must not call Windows networking APIs or provider implementations directly.

## Current module responsibilities

| Module | Responsibility | Current contents |
| --- | --- | --- |
| app | Startup and composition root | CLI summary of the local snapshot |
| core::network | Portable network observation models | Interfaces, addresses, routes, and address-family status |
| network | Read-only inspection port | Inspector trait and typed errors |
| providers | Provider contracts and lifecycle coordination | Metadata, capabilities, prerequisites, status, errors, change plans, and explicit operation manager |
| platform::windows | Windows discovery adapter | Adapters, addresses, DNS, interface indices, and route table |

## Provider contract

The `ConnectionProvider` trait is deliberately small and provider-neutral:

- `metadata` returns a stable identifier and safe display metadata.
- `capabilities` explicitly reports IPv4, IPv6, device-scoped routing, and status-inspection support. Unsupported capabilities are never inferred.
- `inspect` returns a sanitized lifecycle status and prerequisite checks.
- `plan_connect` and `plan_disconnect` return structured previews of intended effects. They do not execute those effects.

The lifecycle status distinguishes unavailable, disconnected, connecting, connected, disconnecting, and error. Availability is separate from connection status and includes explicit prerequisite results.

Provider failures cross the boundary only as stable categories: permission denied, missing prerequisite, unsupported, already in state, temporarily unavailable, or operation failed. Provider-specific diagnostic strings, credentials, tokens, and raw output are not retained in core-facing errors.

A change plan identifies the requested operation, expected impact scope (this device, shared network, or unknown), and a list of non-secret change descriptions. The plan is descriptive only. A later lifecycle manager must require a separate explicit user action before applying any change.

No dynamic third-party plugin loader is introduced. Provider implementations can be compiled into the application and tested through the trait.

## Windows discovery details

- Adapter enumeration and per-interface IPv4/IPv6 addresses, DNS servers, stable adapter IDs, and operational state are read through ipconfig, which wraps Windows IP Helper adapter enumeration.
- Interface indices are collected through GetAdaptersAddresses.
- IPv4 and IPv6 routes are collected through GetIpForwardTable2(AF_UNSPEC).
- The route table is released with FreeMibTable.
- Routes are correlated to adapters by address-family-specific interface index. If no adapter match exists, the raw route index is preserved and the stable ID remains unavailable.
- The default route is identified by an unspecified destination address and prefix length zero.

## Address-family semantics

- configured means at least one non-unspecified address of that family is assigned to an interface, including a loopback address.
- usable_on_up_interface means a non-loopback, non-unspecified address is assigned to an interface reported operational by Windows. Link-local addresses count as locally usable.
- These fields describe local configuration only. They do not test DNS resolution, gateway reachability, internet access, or public egress IP.
- Local interface addresses must never be presented as the public address observed by internet services.

## Error handling

- Fallible infrastructure operations return Result<T, E>.
- Windows API failures are surfaced as typed inspection errors with the originating API and error code.
- Adapter enumeration failures are not converted into an empty successful snapshot.
- Provider errors use stable categories and must not expose provider-specific diagnostic output.
- Unmatched route interface indices are preserved rather than silently discarded.

## Safety boundary

Windows discovery only calls read/query APIs. Provider contract inspection and previews are read-only; Stage 4 invokes provider operations only after explicit manager requests. The manager itself never directly modifies DNS, routes, adapter state, MAC addresses, firewall rules, proxy settings, or IP assignments. No network information is collected or transmitted outside the local process. No automatic connection, disconnection, provider switching, or address rotation is implemented.

## Testing strategy

- Core models and address-family summaries are tested without operating-system access.
- Provider contracts and lifecycle coordination are tested with deterministic mocks, including explicit-operation gating, capability rejection, state validation, confirmed transitions, duplicate registration, and failure-state reporting.
- Windows sockaddr conversion is tested with IPv4 and IPv6 fixtures.
- Route-row conversion is tested with multiple IPv4/IPv6 rows, multiple interfaces, and an unmatched interface index; destination, next hop, metric, and interface correlation are asserted.
- Windows CI compiles and tests the platform adapter, including a live read-only snapshot test.
- Tests must not modify the machine's actual network configuration.
