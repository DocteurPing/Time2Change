use std::ops::RangeInclusive;

use chrono::{DateTime, Utc};

use domain::types::currency_pair::CurrencyPair;
use domain::types::exchange_rate::ExchangeRate;
use domain::types::time_series::TimeSeries;

use crate::ports::repository_errors::RepositoryError;

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
}
