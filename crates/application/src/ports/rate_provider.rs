use chrono::{DateTime, Utc};
use domain::types::currency_pair::CurrencyPair;
use domain::types::exchange_rate::ExchangeRate;

#[async_trait::async_trait]
pub trait RateProvider: Send + Sync {
    async fn get_rate(
        &self,
        pair: &CurrencyPair,
        timestamp: DateTime<Utc>,
    ) -> Result<ExchangeRate, RateProviderError>;

    async fn fetch_latest(&self, pair: &CurrencyPair) -> Result<ExchangeRate, RateProviderError>;
}

#[derive(thiserror::Error, Debug)]
pub enum RateProviderError {
    #[error("pair not supported: {0}")]
    PairNotSupported(String),
    #[error("network timeout")]
    Timeout,
    #[error("api error: {0}")]
    ApiError(String),
    #[error("parse error: {0}")]
    ParseError(String),
}
