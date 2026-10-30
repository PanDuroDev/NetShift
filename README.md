# NetShift

NetShift is a Windows-focused network route management utility. The project is being built in small, verifiable stages.

## Project status

**Stage 1 — Core architecture**

- Portable network observation models
- Read-only network inspection contract
- Application startup separated from domain and infrastructure modules
- Architecture and dependency-direction documentation

No real network discovery, network configuration, VPN, WARP, proxy, or IP-switching functionality is implemented yet.

## Architecture

See [docs/architecture.md](docs/architecture.md) for module responsibilities, dependency direction, error handling, and safety boundaries.

## Development

Requirements:
- Rust stable (the repository toolchain file selects it)
- Cargo

Run locally:

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo build --workspace --release
```

## Planned direction

NetShift is intended to support multiple connection methods over time, rather than locking the project to a single provider. Each method will be designed and validated independently before integration. The current stage establishes architecture only; platform discovery and provider behavior are separate stages.

## License

No license has been selected yet. Until one is added, the repository is not granted an open-source license by default.
