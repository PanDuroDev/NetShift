# NetShift Architecture

## Current stage

NetShift is a small Rust 2024 application. The current implementation establishes portable domain models and a read-only inspection boundary. It does not query or modify the host network.

## Dependency direction

Dependencies point inward:

```text
app
 ├── core
 └── network (contracts)
       └── core (portable data models)

platform adapters (future)
 └── implement network contracts

providers (future)
 └── provider contracts and lifecycle, introduced in later stages
```

Rules:

1. `core` contains platform-independent domain data. It must not import Windows APIs, UI frameworks, provider SDKs, or operating-system adapters.
2. `network` defines portable read-only inspection contracts and their errors. It does not call operating-system APIs directly.
3. `platform/windows` will contain Windows-specific implementations in the network-discovery stage. Platform code must implement portable contracts rather than leaking Windows types into core.
4. `providers` will contain provider identity, capabilities, and lifecycle contracts in the provider stage. Provider implementations must not be embedded in UI or core domain models.
5. `app` is the composition root. It wires concrete adapters and services together; domain modules must not depend on the application entry point.
6. UI code, when introduced, will call application services. It must not call Windows networking APIs or provider implementations directly.

## Current module responsibilities

| Module | Responsibility | Current contents |
| --- | --- | --- |
| `app` | Startup and composition root | Minimal runnable entry point |
| `core::network` | Portable network observation models | Snapshot and interface data types |
| `network` | Read-only inspection port | Inspector trait and typed errors |
| `platform::windows` | Windows API adapters | Deferred until Stage 2 |
| `providers` | Provider contracts and capabilities | Deferred until Stage 3 |

Modules are introduced when they contain useful compiling code; empty placeholder modules are intentionally avoided.

## Error handling

- Fallible infrastructure operations return `Result<T, E>`.
- Errors are typed at module boundaries and implement `std::error::Error` when they represent reportable failures.
- Platform-specific error types must be translated into portable module errors before crossing into application/core code.
- Do not silently convert failures into empty or successful network state.

## Safety boundary

The current `NetworkInspector` contract is read-only. No API in this stage can change DNS, routes, adapter state, MAC addresses, VPN/WARP state, or IP assignments. Any future mutation capability must be a separate explicit contract and must require a user-initiated operation.

## Testing strategy

- Core models are tested without operating-system access.
- Infrastructure contracts are tested with fake implementations.
- Windows adapters will receive platform-specific tests in Stage 2.
- Integration tests must not modify the machine's actual network configuration.
