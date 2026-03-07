use chrono::{DateTime, Utc};
use domain::types::{currency_pair::CurrencyPair, exchange_rate::ExchangeRate};

use crate::ports::rate_provider::{RateProvider, RateProviderError};

pub(crate) struct MockProvider {
    result: Result<ExchangeRate, RateProviderError>,
}

impl MockProvider {
    pub(crate) fn ok(rate: ExchangeRate) -> Self {
        Self { result: Ok(rate) }
    }

    pub(crate) fn err(e: RateProviderError) -> Self {
        Self { result: Err(e) }
    }
}

#[async_trait::async_trait]
impl RateProvider for MockProvider {
    async fn get_rate(
        &self,
        _pair: &CurrencyPair,
        _timestamp: DateTime<Utc>,
    ) -> Result<ExchangeRate, RateProviderError> {
        match &self.result {
            Ok(r) => Ok(r.clone()),
            Err(e) => Err(e.clone()),
        }
    }

    async fn fetch_latest(&self, _pair: &CurrencyPair) -> Result<ExchangeRate, RateProviderError> {
        match &self.result {
            Ok(r) => Ok(r.clone()),
            Err(e) => Err(e.clone()),
        }
    }
}
