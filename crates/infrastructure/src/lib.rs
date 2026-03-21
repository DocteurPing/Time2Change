//! Infrastructure adapters for the `Time2Change` workspace.
//!
//! This crate is intended to contain concrete implementations of the
//! application-layer ports, such as:
//!
//! - exchange rate repositories
//! - external rate provider clients
//! - persistence and transport integrations
//! - configuration-backed adapter wiring

/// Postgres implementation of [`application::ports::exchange_rate_repository::ExchangeRateRepository`].
pub mod exchange_rate;

/// Interface and implementation for fetching exchange rate data from an upstream provider.
pub mod rate_provider;

#[cfg(test)]
mod tests;
