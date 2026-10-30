# NetShift

NetShift is a Windows-focused network route management utility. The project is being built in small, verifiable stages.

## Project status

**Stage 0 — Foundation**

- Rust stable toolchain
- Minimal Cargo package with library and executable targets
- GitHub Actions CI for formatting, linting, tests, and release builds

No network configuration, VPN, WARP, proxy, or IP-switching functionality is implemented yet.

## Development

Requirements:
- Rust stable (the repository toolchain file selects it)
- Cargo

Run locally:

```powershell
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo build --release
```

## Planned direction

NetShift is intended to support multiple connection methods over time, rather than locking the project to a single provider. Each method will be designed and validated independently before integration. The initial foundation deliberately avoids choosing or implementing those methods.

## License

No license has been selected yet. Until one is added, the repository is not granted an open-source license by default.
