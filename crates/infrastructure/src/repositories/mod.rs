/// Postgres implementation of [`application::ports::exchange_rate_repository::ExchangeRateRepository`].
pub mod exchange_rate;

/// Interface and implementation for fetching exchange rate data from an upstream provider.
pub mod rate_provider;
