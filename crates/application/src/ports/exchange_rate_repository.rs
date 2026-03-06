use chrono::{DateTime, Utc};
use domain::types::{
    currency_pair::CurrencyPair, exchange_rate::ExchangeRate, time_series::TimeSeries,
};
use std::ops::RangeInclusive;
use thiserror::Error;

/// Repository port for persisting and retrieving exchange rates.
/// Implementation lives in infrastructure crate.
#[async_trait::async_trait]
pub trait ExchangeRateRepository: Send + Sync {
    async fn save_rates(
        &self,
        pair: &CurrencyPair,
        rates: &[ExchangeRate],
    ) -> Result<(), RepositoryError>;

    async fn load_rates(
        &self,
        pair: &CurrencyPair,
        range: RangeInclusive<DateTime<Utc>>,
    ) -> Result<TimeSeries, RepositoryError>;

    async fn exists(
        &self,
        pair: &CurrencyPair,
        range: RangeInclusive<DateTime<Utc>>,
    ) -> Result<bool, RepositoryError>;
}

#[derive(Error, Debug)]
pub enum RepositoryError {
    #[error("pair {0} not found")]
    NotFound(String),
    #[error("conflict: rates already stored: {0}")]
    Conflict(String),
    #[error("storage failure: {0}")]
    Storage(String),
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
        let debug = format!("{:?}", err);
        assert!(debug.contains("NotFound"));
        assert!(debug.contains("XYZ"));
    }
}
