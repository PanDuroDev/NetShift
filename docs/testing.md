# Testing NetShift

All automated tests use deterministic in-memory fixtures except the Windows-only live inspector smoke test. The suite must not require administrator privileges or change the host network configuration.

## Run locally

From the repository root:

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo build --workspace --release
```

## Safety coverage

- Unit tests cover provider registration, explicit-operation gating, supported capabilities, availability, valid starting states, plan-operation matching, confirmed target states, and failure reporting.
- Integration tests combine the read-only network-inspector contract and provider lifecycle manager. They verify that inspection and registration do not start provider operations, and that only explicitly requested device-scoped plans can execute.
- Plans with `SharedNetwork` or `Unknown` scope must be rejected before the provider's connect/disconnect method is called.
- Inspection errors remain errors; they are not replaced with empty successful snapshots.
- Provider errors expose stable categories only. Tests must not capture credentials, tokens, or raw provider diagnostics.
- The Windows live-inspector test reads current adapter and route state only. It must not modify routes, DNS, adapters, firewall rules, proxy settings, or router configuration.

## Scope limitations

These tests do not prove internet reachability or public egress IP changes. Local adapter addresses are not public IP evidence. No automatic IP rotation, router configuration, production VPN/WARP integration, or service-limit circumvention is implemented.
