use std::ops::RangeInclusive;
use std::sync::{Arc, Mutex};

use chrono::{DateTime, Utc};
use domain::types::currency_info::CurrencyInfo;
use domain::types::currency_pair::CurrencyPair;
use domain::types::exchange_rate::ExchangeRate;
use domain::types::time_series::TimeSeries;

use crate::ports::exchange_rate_repository::{ExchangeRateRepository, RepositoryError};
use crate::tests::helpers::make_pair;

type SavedCurrenciesCall = Vec<CurrencyInfo>;

pub(crate) struct MockRepository {
    load_error: Option<RepositoryError>,
    save_result: Result<(), RepositoryError>,
    saved_rates: Arc<Mutex<Vec<TimeSeries>>>,
    saved_currencies: Arc<Mutex<SavedCurrenciesCall>>,
}

impl MockRepository {
    pub(crate) fn with_rates(pair: CurrencyPair, rates: Vec<ExchangeRate>) -> Self {
        Self {
            load_error: None,
            save_result: Ok(()),
            saved_rates: Arc::new(Mutex::new(vec![TimeSeries::new(pair, rates)])),
            saved_currencies: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub(crate) fn with_error(e: RepositoryError) -> Self {
        Self {
            load_error: Some(e),
            save_result: Ok(()),
            saved_rates: Arc::new(Mutex::new(Vec::new())),
            saved_currencies: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub(crate) fn empty() -> Self {
        Self {
            load_error: None,
            save_result: Ok(()),
            saved_rates: Arc::new(Mutex::new(Vec::new())),
            saved_currencies: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub(crate) fn err(e: RepositoryError) -> Self {
        Self {
            load_error: None,
            save_result: Err(e),
            saved_rates: Arc::new(Mutex::new(Vec::new())),
            saved_currencies: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub(crate) fn saved_rates(&self) -> Vec<TimeSeries> {
        self.saved_rates.lock().unwrap().clone()
    }

    pub(crate) fn saved_currencies(&self) -> Vec<CurrencyInfo> {
        self.saved_currencies.lock().unwrap().clone()
    }

    pub(crate) fn get_arc_saved_rates(&self) -> Arc<Mutex<Vec<TimeSeries>>> {
        self.saved_rates.clone()
    }
}

#[async_trait::async_trait]
impl ExchangeRateRepository for MockRepository {
    async fn save_rates(
        &self,
        pair: &CurrencyPair,
        rates: &[ExchangeRate],
    ) -> Result<(), RepositoryError> {
        self.saved_rates
            .lock()
            .unwrap()
            .push(TimeSeries::new(pair.clone(), rates.to_vec()));
        match &self.save_result {
            Ok(()) => Ok(()),
            Err(e) => Err(e.clone()),
        }
    }

    async fn load_rates(
        &self,
        pair: &CurrencyPair,
        _range: &RangeInclusive<DateTime<Utc>>,
    ) -> Result<TimeSeries, RepositoryError> {
        if let Some(ref e) = self.load_error {
            return Err(e.clone());
        }
        let time_series = self
            .saved_rates
            .lock()
            .unwrap()
            .iter()
            .find(|time_series| time_series.pair() == pair)
            .unwrap_or(&TimeSeries::new(make_pair(), vec![]))
            .clone();
        Ok(time_series)
    }

    async fn exists(
        &self,
        pair: &CurrencyPair,
        range: &RangeInclusive<DateTime<Utc>>,
    ) -> Result<bool, RepositoryError> {
        if let Some(ref e) = self.load_error {
            return Err(e.clone());
        }
        Ok(self.saved_rates().iter().any(|time_series| {
            time_series.pair() == pair
                && time_series
                    .rates()
                    .iter()
                    .any(|r| range.contains(r.timestamp()))
        }))
    }

    async fn save_currencies(&self, currencies: &[CurrencyInfo]) -> Result<(), RepositoryError> {
        self.saved_currencies
            .lock()
            .unwrap()
            .extend_from_slice(currencies);
        match &self.save_result {
            Ok(()) => Ok(()),
            Err(e) => Err(e.clone()),
        }
    }

    async fn list_currencies(&self) -> Result<Vec<CurrencyInfo>, RepositoryError> {
        if let Some(ref e) = self.load_error {
            return Err(e.clone());
        }
        Ok(self.saved_currencies())
    }
}
