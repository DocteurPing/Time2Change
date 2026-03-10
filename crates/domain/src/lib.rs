//! Core domain model for the `Time2Change` application.
//!
//! This crate contains the pure business concepts and calculations used across
//! the system. It is intentionally focused on domain rules and value types so
//! that higher-level crates can depend on it without pulling in infrastructure
//! or delivery concerns.

/// Mathematical indicators and helper functions used by domain services.
pub mod indicators;

/// Core domain types such as currencies, pairs, rates, and time series.
pub mod types;

#[cfg(test)]
pub(crate) mod tests;
