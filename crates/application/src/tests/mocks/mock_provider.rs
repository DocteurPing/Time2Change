use std::collections::HashMap;

use chrono::NaiveDate;
use domain::types::currency::Currency;
use domain::types::currency_info::CurrencyInfo;
use domain::types::currency_pair::CurrencyPair;
use domain::types::exchange_rate::ExchangeRate;

use crate::ports::rate_provider::{RateProvider, RateProviderError};

pub(crate) struct MockProvider {
    rates_result: Result<ExchangeRate, RateProviderError>,
    currencies_result: Result<Vec<CurrencyInfo>, RateProviderError>,
}

impl MockProvider {
    pub(crate) fn ok(rate: ExchangeRate) -> Self {
        Self {
            rates_result: Ok(rate),
            currencies_result: Ok(vec![CurrencyInfo::new(
                Currency::try_from("EUR").unwrap(),
                "Euro".to_owned(),
            )]),
        }
    }

    pub(crate) fn with_currencies_ok(currencies: Vec<CurrencyInfo>) -> Self {
        let default_rate = ExchangeRate::new(chrono::Utc::now(), rust_decimal::Decimal::new(1, 0));

        Self {
            rates_result: Ok(default_rate),
            currencies_result: Ok(currencies),
        }
    }

    pub(crate) fn with_currencies_err(error: RateProviderError) -> Self {
        let default_rate = ExchangeRate::new(chrono::Utc::now(), rust_decimal::Decimal::new(1, 0));

        Self {
            rates_result: Ok(default_rate),
            currencies_result: Err(error),
        }
    }
}

#[async_trait::async_trait]
impl RateProvider for MockProvider {
    async fn get_rates_for_range(
        &self,
        pair: &CurrencyPair,
        _start: NaiveDate,
        _end: NaiveDate,
    ) -> Result<HashMap<CurrencyPair, Vec<ExchangeRate>>, RateProviderError> {
        match &self.rates_result {
            Ok(r) => {
                let mut m = HashMap::new();
                m.insert(pair.clone(), vec![r.clone()]);
                Ok(m)
            }
            Err(e) => Err(e.clone()),
        }
    }

    async fn fetch_currencies(&self) -> Result<Vec<CurrencyInfo>, RateProviderError> {
        self.currencies_result.clone()
    }
}
