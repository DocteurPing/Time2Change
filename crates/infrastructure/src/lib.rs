//! Infrastructure adapters for the `Time2Change` workspace.
//!
//! This crate is intended to contain concrete implementations of the
//! application-layer ports, such as:
//!
//! - exchange rate repositories
//! - external rate provider clients
//! - persistence and transport integrations
//! - configuration-backed adapter wiring

/// Concrete repository adapters backed by a database.
pub mod repositories;

#[cfg(test)]
mod tests;
