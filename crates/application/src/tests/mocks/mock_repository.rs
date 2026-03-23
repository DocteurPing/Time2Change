use std::collections::BTreeMap;
use std::ops::RangeInclusive;
use std::sync::{Arc, Mutex};

use chrono::{DateTime, Utc};
use domain::types::currency_info::CurrencyInfo;
use domain::types::currency_pair::CurrencyPair;
use domain::types::exchange_rate::ExchangeRate;
use domain::types::time_series::TimeSeries;

use crate::ports::exchange_rate_repository::{ExchangeRateRepository, RepositoryError};

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
            saved_rates: Arc::new(Mutex::new(vec![TimeSeries::new(
                pair,
                rates.into_iter().map(ExchangeRate::into_parts).collect(),
            )])),
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

    pub(crate) fn saved_currencies(&self) -> Vec<CurrencyInfo> {
        self.saved_currencies.lock().unwrap().clone()
    }

    pub(crate) fn get_arc_saved_rates(&self) -> Arc<Mutex<Vec<TimeSeries>>> {
        Arc::<Mutex<Vec<TimeSeries>>>::clone(&self.saved_rates)
    }
}

#[async_trait::async_trait]
impl ExchangeRateRepository for MockRepository {
    async fn save_rates(
        &self,
        pair: &CurrencyPair,
        rates: &[ExchangeRate],
    ) -> Result<(), RepositoryError> {
        let mut saved_rates = self.saved_rates.lock().unwrap();
        saved_rates
            .iter_mut()
            .find(|ts| ts.pair() == pair)
            .map(|ts| {
                // Mimic Postgres `ON CONFLICT DO NOTHING`: preserve existing values on duplicate timestamps.
                for rate in rates {
                    let ts_key = rate.timestamp();
                    if !ts.rates().contains_key(ts_key) {
                        ts.add_rate(*ts_key, *rate.rate());
                    }
                }
            })
            .unwrap_or_else(|| {
                // When creating a new TimeSeries, also avoid overwriting duplicates within `rates`.
                let mut map = BTreeMap::new();
                for r in rates {
                    let key = r.timestamp();
                    if !map.contains_key(key) {
                        map.insert(*key, *r.rate());
                    }
                }
                saved_rates.push(TimeSeries::new(pair.clone(), map));
            });
        drop(saved_rates);

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
            .cloned()
            .unwrap_or_else(|| TimeSeries::new(pair.clone(), BTreeMap::new()));
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
        Ok(self.saved_rates.lock().unwrap().iter().any(|time_series| {
            time_series.pair() == pair && time_series.rates().keys().any(|ts| range.contains(ts))
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
