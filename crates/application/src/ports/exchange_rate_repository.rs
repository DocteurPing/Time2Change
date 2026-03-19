use std::ops::RangeInclusive;

use chrono::{DateTime, Utc};
use domain::types::currency_info::CurrencyInfo;
use domain::types::currency_pair::CurrencyPair;
use domain::types::exchange_rate::ExchangeRate;
use domain::types::time_series::TimeSeries;
use thiserror::Error;

/// Repository port for persisting and retrieving exchange rates.
///
/// This trait defines the persistence capabilities required by the application
/// layer. Implementations are expected to live in infrastructure adapters and
/// can be backed by databases, files, caches, or external services.
#[async_trait::async_trait]
pub trait ExchangeRateRepository: Send + Sync {
    /// Persists a batch of exchange rates for the given currency pair.
    ///
    /// Implementations may reject duplicate or conflicting records depending on
    /// the storage model.
    ///
    /// # Errors
    ///
    /// Returns [`RepositoryError::Storage`] if the batch insert fails.
    async fn save_rates(
        &self,
        pair: &CurrencyPair,
        rates: &[ExchangeRate],
    ) -> Result<(), RepositoryError>;

    /// Loads all stored exchange rates for the given pair within the provided
    /// inclusive time range.
    ///
    /// Returns a `TimeSeries` containing the matching rates when successful.
    /// Rows are returned in ascending timestamp order.
    ///
    /// # Errors
    ///
    /// Returns [`RepositoryError::Storage`] if the query fails.
    async fn load_rates(
        &self,
        pair: &CurrencyPair,
        range: &RangeInclusive<DateTime<Utc>>,
    ) -> Result<TimeSeries, RepositoryError>;

    /// Returns whether data already exists for the given pair inside the
    /// provided inclusive time range.
    ///
    /// # Errors
    ///
    /// Returns `RepositoryError::Storage` if the underlying storage fails.
    async fn exists(
        &self,
        pair: &CurrencyPair,
        range: &RangeInclusive<DateTime<Utc>>,
    ) -> Result<bool, RepositoryError>;

    /// Persists the list of currencies available in the API.
    ///
    /// # Errors
    ///
    /// Returns `RepositoryError::Storage` if the batch insert fails.
    async fn save_currencies(&self, currencies: &[CurrencyInfo]) -> Result<(), RepositoryError>;

    /// Returns the list of currencies available in the API.
    ///
    /// # Errors
    ///
    /// Returns `RepositoryError::Storage` if the underlying storage fails.
    async fn list_currencies(&self) -> Result<Vec<CurrencyInfo>, RepositoryError>;
}

/// Errors produced by `ExchangeRateRepository` implementations.
#[derive(Error, Debug, Clone)]
pub enum RepositoryError {
    /// The requested currency pair or data slice could not be found.
    #[error("pair {0} not found")]
    NotFound(String),

    /// The requested write operation conflicts with already stored data.
    #[error("conflict: rates already stored: {0}")]
    Conflict(String),

    /// The underlying storage system failed while processing the request.
    #[error("storage failure: {0}")]
    Storage(String),

    /// The caller supplied invalid input for the repository operation.
    #[error("invalid input: {0}")]
    Invalid(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn not_found_display() {
        let err = RepositoryError::NotFound("EUR-USD".into());
        assert_eq!(err.to_string(), "pair EUR-USD not found");
    }

    #[test]
    fn conflict_display() {
        let err = RepositoryError::Conflict("2024-01-01".into());
        assert_eq!(
            err.to_string(),
            "conflict: rates already stored: 2024-01-01"
        );
    }

    #[test]
    fn storage_display() {
        let err = RepositoryError::Storage("connection refused".into());
        assert_eq!(err.to_string(), "storage failure: connection refused");
    }

    #[test]
    fn invalid_display() {
        let err = RepositoryError::Invalid("empty range".into());
        assert_eq!(err.to_string(), "invalid input: empty range");
    }

    #[test]
    fn error_is_debug() {
        let err = RepositoryError::NotFound("XYZ".into());
        let debug = format!("{err:?}");
        assert!(debug.contains("NotFound"));
        assert!(debug.contains("XYZ"));
    }
}
