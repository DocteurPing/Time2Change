use std::ops::RangeInclusive;
use std::sync::{Arc, Mutex};

use chrono::{DateTime, Utc};
use domain::types::currency_pair::CurrencyPair;
use domain::types::exchange_rate::ExchangeRate;
use domain::types::time_series::TimeSeries;

use crate::ports::exchange_rate_repository::{ExchangeRateRepository, RepositoryError};

type SavedCall = (CurrencyPair, Vec<ExchangeRate>);

pub(crate) struct MockRepository {
    rates: Vec<ExchangeRate>,
    load_error: Option<RepositoryError>,
    save_result: Result<(), RepositoryError>,
    saved: Arc<Mutex<Vec<SavedCall>>>,
}

impl MockRepository {
    pub(crate) fn with_rates(rates: Vec<ExchangeRate>) -> Self {
        Self {
            rates,
            load_error: None,
            save_result: Ok(()),
            saved: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub(crate) fn with_error(e: RepositoryError) -> Self {
        Self {
            rates: Vec::new(),
            load_error: Some(e),
            save_result: Ok(()),
            saved: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub(crate) fn empty() -> Self {
        Self {
            rates: Vec::new(),
            load_error: None,
            save_result: Ok(()),
            saved: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub(crate) fn err(e: RepositoryError) -> Self {
        Self {
            rates: Vec::new(),
            load_error: None,
            save_result: Err(e),
            saved: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub(crate) fn saved_calls(&self) -> Vec<(CurrencyPair, Vec<ExchangeRate>)> {
        self.saved.lock().unwrap().clone()
    }
}

#[async_trait::async_trait]
impl ExchangeRateRepository for MockRepository {
    async fn save_rates(
        &self,
        pair: &CurrencyPair,
        rates: &[ExchangeRate],
    ) -> Result<(), RepositoryError> {
        self.saved
            .lock()
            .unwrap()
            .push((pair.clone(), rates.to_vec()));
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
        Ok(TimeSeries::new(pair.clone(), self.rates.clone()))
    }

    async fn exists(
        &self,
        pair: &CurrencyPair,
        range: &RangeInclusive<DateTime<Utc>>,
    ) -> Result<bool, RepositoryError> {
        if let Some(ref e) = self.load_error {
            return Err(e.clone());
        }
        Ok(self
            .saved
            .lock()
            .unwrap()
            .iter()
            .any(|(p, rates)| p == pair && rates.iter().any(|r| range.contains(r.timestamp()))))
    }
}
