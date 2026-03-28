use chrono::{DateTime, NaiveDate, Utc};
use domain::types::currency_pair::CurrencyPair;
use rust_decimal::Decimal;
use thiserror::Error;

use crate::ports::exchange_rate_repository::{ExchangeRateRepository, RepositoryError};
use crate::ports::rate_provider::{RateProvider, RateProviderError};

/// Use case that fetches the latest exchange rate for a currency pair and
/// persists it through the configured repository.
///
/// This workflow coordinates two application ports:
/// - a [`RateProvider`] that supplies the latest rate data
/// - an [`ExchangeRateRepository`] that stores the retrieved rate
///
/// It returns an [`IngestResult`] containing the pair, timestamp, and rate that
/// were successfully ingested.
#[derive(Debug)]
pub struct IngestRatesUseCase<R, C>
where
    R: ExchangeRateRepository,
    C: RateProvider,
{
    repository: R,
    provider: C,
}

impl<R, C> IngestRatesUseCase<R, C>
where
    R: ExchangeRateRepository,
    C: RateProvider,
{
    /// Creates a new ingest-rates use case from a repository and rate provider.
    #[must_use]
    pub const fn new(repository: R, provider: C) -> Self {
        Self {
            repository,
            provider,
        }
    }

    /// Fetches the latest exchange rate for `pair`, persists it, and returns
    /// the ingested values.
    ///
    /// # Errors
    ///
    /// Returns [`IngestError::Provider`] when the upstream provider cannot
    /// supply the latest rate, or [`IngestError::Repository`] when persistence
    /// fails.
    pub async fn execute(&self, pair: CurrencyPair) -> Result<IngestResult, IngestError> {
        let fx_record = self.provider.fetch_latest(&pair).await?;
        let timestamp = *fx_record.timestamp();
        let rate = *fx_record.rate();

        self.repository.save_rates(&pair, &[fx_record]).await?;

        Ok(IngestResult {
            pair,
            timestamp,
            rate,
        })
    }

    /// Fetches the specific exchange rate for `pair` at `timestamp`.
    ///
    /// # Errors
    ///
    /// Returns [`IngestError::Provider`] when the upstream provider cannot
    /// supply the rate, or [`IngestError::Repository`] when persistence fails.
    pub async fn fetch_rate(
        &self,
        pair: &CurrencyPair,
        timestamp: DateTime<Utc>,
    ) -> Result<IngestResult, IngestError> {
        let rate = self.provider.get_rate(pair, timestamp).await?;

        self.repository
            .save_rates(pair, std::slice::from_ref(&rate))
            .await?;

        Ok(IngestResult {
            pair: pair.clone(),
            timestamp,
            rate: *rate.rate(),
        })
    }

    /// Fetches all exchange rates for `pair` between `start` and `end`
    /// (inclusive), persists the entire batch, and returns the number of
    /// rates ingested.
    ///
    /// This is more efficient than calling [`Self::fetch_rate`] in a loop
    /// because it issues a single request to the upstream provider and a
    /// single bulk write to the repository.
    ///
    /// # Errors
    ///
    /// Returns [`IngestError::Provider`] when the upstream provider cannot
    /// supply the rates, or [`IngestError::Repository`] when persistence
    /// fails.
    pub async fn fetch_rates_for_range(
        &self,
        pair: &CurrencyPair,
        start: NaiveDate,
        end: NaiveDate,
    ) -> Result<usize, IngestError> {
        let rates = self.provider.get_rates_for_range(pair, start, end).await?;
        let count = rates.len();

        self.repository.save_rates(pair, &rates).await?;

        Ok(count)
    }
}

/// Result returned after a successful ingestion of the latest exchange rate.
#[derive(Debug)]
pub struct IngestResult {
    pair: CurrencyPair,
    timestamp: DateTime<Utc>,
    rate: Decimal,
}

impl IngestResult {
    /// Returns the currency pair associated with the ingested rate.
    #[must_use]
    pub const fn pair(&self) -> &CurrencyPair {
        &self.pair
    }

    /// Returns the timestamp attached to the ingested exchange-rate record.
    #[must_use]
    pub const fn timestamp(&self) -> &DateTime<Utc> {
        &self.timestamp
    }

    /// Returns the ingested exchange-rate value.
    #[must_use]
    pub const fn rate(&self) -> &Decimal {
        &self.rate
    }
}

/// Errors that can occur while ingesting the latest exchange rate.
#[derive(Error, Debug)]
pub enum IngestError {
    /// The upstream rate provider failed to fetch the latest exchange rate.
    #[error("provider error: {0}")]
    Provider(#[from] RateProviderError),

    /// The repository failed to persist the fetched exchange rate.
    #[error("repository error: {0}")]
    Repository(#[from] RepositoryError),
}
