use std::collections::HashSet;

use chrono::NaiveDate;
use domain::types::currency::Currency;
use thiserror::Error;

use crate::ports::exchange_rate_repository::ExchangeRateRepository;
use crate::ports::rate_provider::{RateProvider, RateProviderError};
use crate::ports::repository_errors::RepositoryError;

/// Use case that fetches the exchange rates for a currency and a list of pair, then
/// persists it through the configured repository.
///
/// This workflow coordinates two application ports:
/// - a [`RateProvider`] that supplies the rate data for a given currency pair at a given date range
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

    /// Fetches all exchange rates for `currency` between `start` and `end`
    /// (inclusive), for all the given `list_currencies` persists the entire batch,
    /// and returns the number of rates ingested.
    ///
    /// # Errors
    ///
    /// Returns [`IngestError::Provider`] when the upstream provider cannot
    /// supply the rates, or [`IngestError::Repository`] when persistence
    /// fails.
    pub async fn fetch_rates_for_range(
        &self,
        list_currencies: &HashSet<Currency>,
        start: NaiveDate,
        end: NaiveDate,
        currency: &Currency,
    ) -> Result<usize, IngestError> {
        let rates = self
            .provider
            .get_rates_for_range(list_currencies, start, end, currency)
            .await?;
        let count = rates.values().map(Vec::len).sum();

        self.repository.save_rates(rates).await?;

        Ok(count)
    }
}

/// Errors that can occur while ingesting the exchange rate.
#[derive(Error, Debug)]
pub enum IngestError {
    /// The upstream rate provider failed to fetch the latest exchange rate.
    #[error("provider error: {0}")]
    Provider(#[from] RateProviderError),

    /// The repository failed to persist the fetched exchange rate.
    #[error("repository error: {0}")]
    Repository(#[from] RepositoryError),
}
