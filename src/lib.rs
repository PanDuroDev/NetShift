//! NetShift application library.
//!
//! The library separates portable domain models and infrastructure contracts
//! from application startup and future platform-specific implementations.

pub mod app;
pub mod core;
pub mod network;

/// Current package version, sourced from Cargo metadata.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::VERSION;

    #[test]
    fn package_version_is_available() {
        assert_eq!(VERSION, "0.1.0");
    }
}
