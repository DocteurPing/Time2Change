use std::collections::{HashMap, HashSet};
use std::ops::RangeInclusive;
use std::sync::{Arc, Mutex};

use application::ports::currency_repository::CurrencyRepository;
use application::ports::exchange_rate_repository::ExchangeRateRepository;
use application::ports::rate_provider::{RateProvider, RateProviderError};
use application::ports::repository_errors::RepositoryError;
use chrono::{DateTime, NaiveDate, Utc};
use domain::types::currency::Currency;
use domain::types::currency_info::CurrencyInfo;
use domain::types::currency_pair::CurrencyPair;
use domain::types::exchange_rate::ExchangeRate;
use domain::types::time_series::TimeSeries;
use rust_decimal::dec;

/// A single upstream fetch recorded by [`MockProvider`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FetchCall {
    pub start: NaiveDate,
    pub end: NaiveDate,
    pub base: Currency,
}

/// Shared, thread-safe log of what the runner asked its collaborators to do.
#[derive(Debug, Default)]
pub(crate) struct CallLog {
    fetches: Mutex<Vec<FetchCall>>,
    exists_checks: Mutex<usize>,
}

impl CallLog {
    pub(crate) fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    pub(crate) fn fetches(&self) -> Vec<FetchCall> {
        self.fetches.lock().unwrap().clone()
    }

    pub(crate) fn fetch_count(&self) -> usize {
        self.fetches.lock().unwrap().len()
    }

    pub(crate) fn exists_checks(&self) -> usize {
        *self.exists_checks.lock().unwrap()
    }

    /// Distinct `(start, end)` ranges requested, in first-seen order.
    pub(crate) fn ranges(&self) -> Vec<(NaiveDate, NaiveDate)> {
        let mut seen = Vec::new();
        for call in self.fetches() {
            let range = (call.start, call.end);
            if !seen.contains(&range) {
                seen.push(range);
            }
        }
        seen
    }
}

/// Rate provider that records every request and returns a canned outcome.
pub(crate) struct MockProvider {
    log: Arc<CallLog>,
    error: Option<RateProviderError>,
}

impl MockProvider {
    pub(crate) const fn ok(log: Arc<CallLog>) -> Self {
        Self { log, error: None }
    }

    pub(crate) const fn failing(log: Arc<CallLog>, error: RateProviderError) -> Self {
        Self {
            log,
            error: Some(error),
        }
    }
}

#[async_trait::async_trait]
impl RateProvider for MockProvider {
    async fn get_rates_for_range(
        &self,
        list_currencies: &HashSet<Currency>,
        start: NaiveDate,
        end: NaiveDate,
        currency: &Currency,
    ) -> Result<HashMap<CurrencyPair, Vec<ExchangeRate>>, RateProviderError> {
        self.log.fetches.lock().unwrap().push(FetchCall {
            start,
            end,
            base: currency.clone(),
        });

        if let Some(error) = &self.error {
            return Err(error.clone());
        }

        let mut rates = HashMap::new();
        for quote in list_currencies {
            if let Ok(pair) = CurrencyPair::new(currency.clone(), quote.clone()) {
                rates.insert(
                    pair,
                    vec![ExchangeRate::new(
                        start.and_hms_opt(0, 0, 0).unwrap_or_default().and_utc(),
                        dec!(1.1),
                    )],
                );
            }
        }
        Ok(rates)
    }

    async fn fetch_currencies(&self) -> Result<Vec<CurrencyInfo>, RateProviderError> {
        if let Some(error) = &self.error {
            return Err(error.clone());
        }
        Ok(Vec::new())
    }
}

/// How the repository should answer `exists`.
#[derive(Debug, Clone, Copy)]
pub(crate) enum Existing {
    /// Nothing is stored yet — nothing can be skipped.
    Nothing,
    /// Everything asked about is already stored.
    Everything,
    /// The existence check itself fails.
    Failing,
}

/// Exchange-rate repository with a scriptable `exists` answer.
pub(crate) struct MockRepository {
    log: Arc<CallLog>,
    existing: Existing,
    save_error: Option<RepositoryError>,
}

impl MockRepository {
    pub(crate) const fn new(log: Arc<CallLog>, existing: Existing) -> Self {
        Self {
            log,
            existing,
            save_error: None,
        }
    }

    pub(crate) const fn failing_saves(log: Arc<CallLog>, error: RepositoryError) -> Self {
        Self {
            log,
            existing: Existing::Nothing,
            save_error: Some(error),
        }
    }
}

#[async_trait::async_trait]
impl ExchangeRateRepository for MockRepository {
    async fn save_rates(
        &self,
        _rates: HashMap<CurrencyPair, Vec<ExchangeRate>>,
    ) -> Result<(), RepositoryError> {
        self.save_error.clone().map_or(Ok(()), Err)
    }

    async fn load_rates(
        &self,
        pair: &CurrencyPair,
        _range: &RangeInclusive<DateTime<Utc>>,
    ) -> Result<TimeSeries, RepositoryError> {
        Ok(TimeSeries::new(
            pair.clone(),
            std::collections::BTreeMap::new(),
        ))
    }

    async fn exists(
        &self,
        _pair: &CurrencyPair,
        _range: &RangeInclusive<DateTime<Utc>>,
    ) -> Result<bool, RepositoryError> {
        *self.log.exists_checks.lock().unwrap() += 1;

        match self.existing {
            Existing::Nothing => Ok(false),
            Existing::Everything => Ok(true),
            Existing::Failing => Err(RepositoryError::Storage("exists failed".to_owned())),
        }
    }
}

#[async_trait::async_trait]
impl CurrencyRepository for MockRepository {
    async fn save_currencies(&self, _currencies: &[CurrencyInfo]) -> Result<(), RepositoryError> {
        self.save_error.clone().map_or(Ok(()), Err)
    }

    async fn list_currencies(&self) -> Result<Vec<CurrencyInfo>, RepositoryError> {
        Ok(Vec::new())
    }
}
