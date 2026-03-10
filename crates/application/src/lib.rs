//! Application layer for orchestration of use cases, ports, and response models.
//!
//! This crate coordinates domain logic without owning infrastructure details.

/// Port traits that describe the external dependencies required by the
/// application layer.
pub mod ports;

/// Response DTOs returned by application use cases.
pub mod responses;

/// Use-case implementations that orchestrate domain operations.
pub mod use_cases;

#[cfg(test)]
mod tests;
