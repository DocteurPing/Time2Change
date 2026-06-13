use std::collections::{BTreeMap, HashMap};
use std::ops::RangeInclusive;
use std::sync::{Arc, Mutex};

use chrono::{DateTime, Utc};
use domain::types::currency_info::CurrencyInfo;
use domain::types::currency_pair::CurrencyPair;
use domain::types::exchange_rate::ExchangeRate;
use domain::types::time_series::TimeSeries;

use crate::ports::currency_repository::CurrencyRepository;
use crate::ports::exchange_rate_repository::ExchangeRateRepository;
use crate::ports::repository_errors::RepositoryError;

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

    pub(crate) fn with_save_error(e: RepositoryError) -> Self {
        Self {
            load_error: None,
            save_result: Err(e),
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

    pub(crate) fn saved_currencies(&self) -> Vec<CurrencyInfo> {
        self.saved_currencies.lock().unwrap().clone()
    }
}

#[async_trait::async_trait]
impl ExchangeRateRepository for MockRepository {
    async fn save_rates(
        &self,
        rates: HashMap<CurrencyPair, Vec<ExchangeRate>>,
    ) -> Result<(), RepositoryError> {
        let mut saved_rates = self.saved_rates.lock().unwrap();

        for (p, p_rates) in rates {
            let mut handled = false;
            for ts in saved_rates.iter_mut() {
                if ts.pair() == &p {
                    for rate in &p_rates {
                        let ts_key = rate.timestamp();
                        if !ts.rates().contains_key(ts_key) {
                            ts.add_rate(*ts_key, *rate.rate());
                        }
                    }
                    handled = true;
                    break;
                }
            }
            if !handled {
                let mut map = BTreeMap::new();
                for r in &p_rates {
                    let key = r.timestamp();
                    if !map.contains_key(key) {
                        map.insert(*key, *r.rate());
                    }
                }
                saved_rates.push(TimeSeries::new(p, map));
            }
        }
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
}

#[async_trait::async_trait]
impl CurrencyRepository for MockRepository {
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
