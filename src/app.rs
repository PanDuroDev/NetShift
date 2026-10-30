//! Application startup and top-level composition.

/// Starts the current command-line shell.
///
/// The application deliberately performs no network inspection or mutation in
/// this stage. Later stages will compose platform adapters and services here.
pub fn run() {
    println!("NetShift {}", crate::VERSION);
}
