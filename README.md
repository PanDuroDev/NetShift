# NetShift

NetShift is a Windows-focused network route management utility. The project is being built in small, verifiable stages.

## Project status

**Stage 2 — Read-only network discovery: merged**

- Enumerates Windows network adapters with stable adapter identifiers and operational state.
- Reports assigned IPv4/IPv6 addresses and per-interface DNS servers.
- Reads IPv4 and IPv6 routing tables, including default routes and route metrics.
- Correlates routes with interfaces when Windows interface indices match.
- Summarizes local IPv4/IPv6 configuration without claiming internet reachability.

**Stage 3 — Provider-neutral contract: merged**

- Defines provider metadata, explicit capabilities, availability prerequisites, and lifecycle status.
- Defines sanitized provider error categories and structured connect/disconnect change previews.
- Provides a provider interface and a test-only mock implementation.

**Stage 4 — Provider lifecycle manager: implemented**

- Registers and enumerates providers by stable identifier.
- Inspects availability and status before accepting an operation.
- Rejects unsupported operations and invalid lifecycle states before execution.
- Runs connect/disconnect only through explicit manager calls, rejects shared-network or unknown-impact plans, and verifies the resulting provider state.
- Preserves the pre-operation snapshot and reports failures with the best-known resulting state.
- Uses deterministic mock providers; no production VPN/WARP integration is included.

**Stage 5 — Safety invariants and integration tests: merged**

- Enforces device-only change scope before provider execution.
- Adds cross-module tests for read-only inspection, explicit operation requests, and provider success/failure reporting.
- Documents safety guarantees and local test commands.
- CI passed on the merged pull request; shared-network and unknown-impact operations are rejected before provider execution.

**GUI Stage 1 — Framework evaluation and Tauri prototype: in progress**

- Compares Tauri, Slint, and egui for the Windows-first desktop UI, English-only LTR layout, accessibility, licensing, and Linux portability.
- The framework comparison is complete; Tauri 2 + React/TypeScript/Vite is the prototype candidate.
- A mock-only English LTR desktop shell is being built; it does not inspect or change network settings.
- See [docs/gui-framework-evaluation.md](docs/gui-framework-evaluation.md) and [docs/gui-poc.md](docs/gui-poc.md).

The current Windows network inspector remains read-only. The lifecycle manager does not directly modify Windows routes, DNS, adapters, or firewall rules. No automatic provider switching, background retries, IP rotation, or public egress IP lookup is implemented.

## Address terminology

Interface addresses are local addresses assigned to adapters. They are not necessarily the public address seen by websites. Public egress IP detection is a separate future capability and is not implemented.

The IPv4/IPv6 status fields describe local address configuration and interface operational state only. They do not prove that DNS, a gateway, or the internet is reachable.

## Architecture and testing

See [docs/architecture.md](docs/architecture.md) for module responsibilities, Windows API usage, provider contracts, address-family semantics, and safety boundaries. See [docs/testing.md](docs/testing.md) for the local test guide and safety-test coverage.

## Desktop prototype

The initial Tauri shell lives in `desktop/` and is intentionally isolated from the Rust core. See [desktop/README.md](desktop/README.md) for prerequisites and run commands. The shell uses static fixtures only and has no backend network commands.

## Development

Requirements:
- Rust stable (the repository toolchain file selects it)
- Cargo

Run locally:

    cargo fmt --all -- --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace --all-features
    cargo build --workspace --release

The live network inspector is currently Windows-only. Provider contracts and their mock tests are platform-independent.

## Planned direction

NetShift is intended to support multiple connection methods over time, rather than locking the project to a single provider. Production provider integrations remain planned; the provider lifecycle manager currently supports explicit operations through test providers only.

## License

No license has been selected yet. Until one is added, the repository is not granted an open-source license by default.
