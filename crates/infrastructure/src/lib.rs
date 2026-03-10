//! Infrastructure adapters for the `Time2Change` workspace.
//!
//! This crate is intended to contain concrete implementations of the
//! application-layer ports, such as:
//!
//! - exchange rate repositories
//! - external rate provider clients
//! - persistence and transport integrations
//! - configuration-backed adapter wiring
//!
//! At the moment this crate is a placeholder and does not yet expose any
//! production infrastructure components. As the project evolves, this crate
//! will host the boundary code that connects the pure domain and application
//! logic to databases, APIs, and other external systems.
