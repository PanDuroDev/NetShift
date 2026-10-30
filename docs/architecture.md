# NetShift Architecture

## Current stage

Stage 2 implements a read-only Windows network inspector. Portable domain types remain independent of Windows APIs. The Windows adapter uses the ipconfig crate for adapter address/DNS metadata and Windows IP Helper APIs for interface indices and the routing table.

## Dependency direction

Dependencies point inward:

    app
     ├── core
     ├── network (contracts)
     └── platform::windows
           ├── ipconfig
           └── Windows IP Helper APIs

Rules:

1. core contains platform-independent domain data. It must not import Windows APIs, UI frameworks, provider SDKs, or operating-system adapters.
2. network defines portable read-only inspection contracts and their errors. It does not call operating-system APIs directly.
3. platform::windows implements discovery using read-only operating-system APIs and translates results into portable models.
4. providers will contain provider identity, capabilities, and lifecycle contracts in a later stage.
5. app is the composition root. It invokes application services and does not call Windows APIs directly.
6. UI code, when introduced, will call application services. It must not call Windows networking APIs or provider implementations directly.

## Current module responsibilities

| Module | Responsibility | Current contents |
| --- | --- | --- |
| app | Startup and composition root | CLI summary of the local snapshot |
| core::network | Portable network observation models | Interfaces, addresses, routes, and address-family status |
| network | Read-only inspection port | Inspector trait and typed errors |
| platform::windows | Windows discovery adapter | Adapters, addresses, DNS, interface indices, and route table |
| providers | Provider contracts and capabilities | Deferred until Stage 3 |

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
- Unmatched route interface indices are preserved rather than silently discarded.

## Safety boundary

Stage 2 only calls read/query APIs. It does not modify DNS, routes, adapter state, MAC addresses, VPN/WARP state, firewall rules, proxy settings, or IP assignments. No network information is collected or transmitted outside the local process.

## Testing strategy

- Core models and address-family summaries are tested without operating-system access.
- Windows sockaddr conversion is tested with IPv4 and IPv6 fixtures.
- Route-row conversion is tested with multiple IPv4/IPv6 rows, multiple interfaces, and an unmatched interface index; destination, next hop, metric, and interface correlation are asserted.
- Windows CI compiles and tests the platform adapter, including a live read-only snapshot test.
- Tests must not modify the machine's actual network configuration.
