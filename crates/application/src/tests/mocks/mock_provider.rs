use chrono::NaiveDate;
use domain::types::currency::Currency;
use domain::types::currency_info::CurrencyInfo;
use domain::types::currency_pair::CurrencyPair;
use domain::types::exchange_rate::ExchangeRate;

use crate::ports::rate_provider::{RateProvider, RateProviderError};

pub(crate) struct MockProvider {
    result: Result<ExchangeRate, RateProviderError>,
}

impl MockProvider {
    pub(crate) fn ok(rate: ExchangeRate) -> Self {
        Self { result: Ok(rate) }
    }
}

#[async_trait::async_trait]
impl RateProvider for MockProvider {
    async fn get_rates_for_range(
        &self,
        _pair: &CurrencyPair,
        _start: NaiveDate,
        _end: NaiveDate,
    ) -> Result<Vec<ExchangeRate>, RateProviderError> {
        match &self.result {
            Ok(r) => Ok(vec![r.clone()]),
            Err(e) => Err(e.clone()),
        }
    }

    async fn fetch_currencies(&self) -> Result<Vec<CurrencyInfo>, RateProviderError> {
        match &self.result {
            Ok(_) => Ok(vec![CurrencyInfo::new(
                Currency::try_from("EUR").unwrap(),
                "Euro".to_owned(),
            )]),
            Err(e) => Err(e.clone()),
        }
    }
}
