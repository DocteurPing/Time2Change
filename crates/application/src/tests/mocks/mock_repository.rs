use std::{
    ops::RangeInclusive,
    sync::{Arc, Mutex},
};

use crate::ports::exchange_rate_repository::{ExchangeRateRepository, RepositoryError};
use chrono::{DateTime, Utc};
use domain::types::{
    currency_pair::CurrencyPair, exchange_rate::ExchangeRate, time_series::TimeSeries,
};

pub(crate) struct MockRepository {
    rates: Vec<ExchangeRate>,
    load_error: Option<RepositoryError>,
    save_result: Result<(), RepositoryError>,
    saved: Arc<Mutex<Vec<(CurrencyPair, Vec<ExchangeRate>)>>>,
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
        _range: RangeInclusive<DateTime<Utc>>,
    ) -> Result<TimeSeries, RepositoryError> {
        if let Some(ref e) = self.load_error {
            return Err(e.clone());
        }
        Ok(TimeSeries::new(pair.clone(), self.rates.clone()))
    }

    async fn exists(
        &self,
        _pair: &CurrencyPair,
        _range: RangeInclusive<DateTime<Utc>>,
    ) -> Result<bool, RepositoryError> {
        unimplemented!("not needed for current tests")
    }
}
