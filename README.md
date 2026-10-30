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
- Does not execute connection operations or modify network state.

**Stage 4 — Provider lifecycle manager: implemented**\n\n- Registers and enumerates providers by stable identifier.\n- Inspects availability and status before accepting an operation.\n- Rejects unsupported operations and invalid lifecycle states before execution.\n- Runs connect/disconnect only through explicit manager calls and verifies the resulting provider state.\n- Preserves the pre-operation snapshot and reports failures with the best-known resulting state.\n- Uses deterministic mock providers; no production VPN/WARP integration is included.\n\nThe current Windows network inspector remains read-only. The lifecycle manager does not directly modify Windows routes, DNS, adapters, or firewall rules. No automatic provider switching, background retries, IP rotation, or public egress IP lookup is implemented.

## Address terminology

Interface addresses are local addresses assigned to adapters. They are not necessarily the public address seen by websites. Public egress IP detection is a separate future capability and is not implemented.

The IPv4/IPv6 status fields describe local address configuration and interface operational state only. They do not prove that DNS, a gateway, or the internet is reachable.

## Architecture

See docs/architecture.md for module responsibilities, Windows API usage, provider contracts, address-family semantics, and safety boundaries.

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

NetShift is intended to support multiple connection methods over time, rather than locking the project to a single provider. Provider implementations and lifecycle execution will be introduced in later stages, after the provider-neutral contract is validated.

## License

No license has been selected yet. Until one is added, the repository is not granted an open-source license by default.
