use domain::types::currency_info::CurrencyInfo;
use thiserror::Error;

use crate::ports::currency_repository::CurrencyRepository;
use crate::ports::rate_provider::{RateProvider, RateProviderError};
use crate::ports::repository_errors::RepositoryError;

/// Use case that synchronizes the upstream currency catalog into local storage.
///
/// This workflow coordinates two application ports:
/// - a [`RateProvider`] that returns the current list of available currencies
/// - an [`ExchangeRateRepository`] that persists that list
///
/// It returns a [`SyncCurrenciesResult`] with simple counters useful for
/// logging/telemetry.
#[derive(Debug)]
pub struct SyncCurrenciesUseCase<R, C>
where
    R: CurrencyRepository,
    C: RateProvider,
{
    repository: R,
    provider: C,
}

impl<R, C> SyncCurrenciesUseCase<R, C>
where
    R: CurrencyRepository,
    C: RateProvider,
{
    /// Creates a new sync-currencies use case from a repository and provider.
    #[must_use]
    pub const fn new(repository: R, provider: C) -> Self {
        Self {
            repository,
            provider,
        }
    }

    /// Fetches the available currencies from the provider and persists them.
    ///
    /// # Errors
    ///
    /// Returns [`SyncCurrenciesError::Provider`] when upstream retrieval fails,
    /// or [`SyncCurrenciesError::Repository`] when persistence fails.
    pub async fn execute(&self) -> Result<usize, SyncCurrenciesError> {
        let currencies = self.provider.fetch_currencies().await?;
        let fetched = currencies.len();

        self.repository.save_currencies(&currencies).await?;

        Ok(fetched)
    }

    /// Returns the list of currencies currently persisted in the repository.
    ///
    /// # Errors
    ///
    /// Returns [`RepositoryError`] when the list could not be retrieved.
    pub async fn list_currencies(&self) -> Result<Vec<CurrencyInfo>, RepositoryError> {
        self.repository.list_currencies().await
    }
}

/// Errors that can occur while synchronizing currencies.
#[derive(Error, Debug)]
pub enum SyncCurrenciesError {
    /// The upstream provider failed to return currencies.
    #[error("provider error: {0}")]
    Provider(#[from] RateProviderError),

    /// The repository failed to persist currencies.
    #[error("repository error: {0}")]
    Repository(#[from] RepositoryError),
}
