//! Abstractions that define the application's external dependencies.
//!
//! Ports describe the capabilities the application layer needs from the
//! outside world, such as persistence and upstream rate retrieval. Concrete
//! adapters implementing these contracts live in other crates.

/// Repository abstraction for storing and loading exchange-rate data.
pub mod exchange_rate_repository;

/// Abstractions for fetching exchange-rate data from upstream providers.
pub mod rate_provider;
