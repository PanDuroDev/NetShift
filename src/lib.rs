//! NetShift application foundation.
//!
//! Network providers and route-management capabilities will be introduced in
//! later stages behind explicit, independently testable modules.

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
