//! NetShift application library.

pub mod app;
pub mod core;
pub mod network;
pub mod platform;
pub mod providers;

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
