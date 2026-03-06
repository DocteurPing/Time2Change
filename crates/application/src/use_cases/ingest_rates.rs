use crate::ports::{
    exchange_rate_repository::{ExchangeRateRepository, RepositoryError},
    rate_provider::{RateProvider, RateProviderError},
};
use domain::types::currency_pair::CurrencyPair;
use thiserror::Error;

/// Ingest latest FX rate for a pair and persist.
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
    pub fn new(repository: R, provider: C) -> Self {
        Self {
            repository,
            provider,
        }
    }

    pub async fn execute(&self, pair: CurrencyPair) -> Result<IngestResult, IngestError> {
        let fx_record = self.provider.fetch_latest(&pair).await?;

        let rate = domain::types::exchange_rate::ExchangeRate::new(
            *fx_record.timestamp(),
            *fx_record.rate(),
        );

        self.repository.save_rates(&pair, &[rate]).await?;

        Ok(IngestResult {
            pair,
            timestamp: *fx_record.timestamp(),
            rate: *fx_record.rate(),
        })
    }
}

#[derive(Debug)]
pub struct IngestResult {
    pair: CurrencyPair,
    timestamp: chrono::DateTime<chrono::Utc>,
    rate: rust_decimal::Decimal,
}

#[derive(Error, Debug)]
pub enum IngestError {
    #[error("provider error: {0}")]
    Provider(#[from] RateProviderError),
    #[error("repository error: {0}")]
    Repository(#[from] RepositoryError),
}
