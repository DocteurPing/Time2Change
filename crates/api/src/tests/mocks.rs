use std::collections::{BTreeMap, HashMap};
use std::ops::RangeInclusive;

use application::ports::currency_repository::CurrencyRepository;
use application::ports::exchange_rate_repository::ExchangeRateRepository;
use application::ports::repository_errors::RepositoryError;
use chrono::{DateTime, Duration, Utc};
use domain::types::currency::Currency;
use domain::types::currency_info::CurrencyInfo;
use domain::types::currency_pair::CurrencyPair;
use domain::types::exchange_rate::ExchangeRate;
use domain::types::time_series::TimeSeries;
use rust_decimal::{Decimal, dec};

/// Exchange-rate repository returning a fixed series, or a fixed failure.
#[derive(Debug, Clone)]
pub(crate) struct MockRateRepository {
    rates: BTreeMap<DateTime<Utc>, Decimal>,
    error: Option<RepositoryError>,
}

impl MockRateRepository {
    /// A repository holding `values` as one rate per day, ending now.
    pub(crate) fn with_values(values: &[Decimal]) -> Self {
        let now = Utc::now();
        let count = i64::try_from(values.len()).unwrap_or(0);

        let rates = values
            .iter()
            .enumerate()
            .map(|(i, value)| {
                let offset = count - 1 - i64::try_from(i).unwrap_or(0);
                (now - Duration::days(offset), *value)
            })
            .collect();

        Self { rates, error: None }
    }

    /// A repository that holds nothing, so analysis has no data to work with.
    pub(crate) fn empty() -> Self {
        Self {
            rates: BTreeMap::new(),
            error: None,
        }
    }

    /// A repository whose reads always fail.
    pub(crate) fn failing() -> Self {
        Self {
            rates: BTreeMap::new(),
            error: Some(RepositoryError::Storage("connection lost".to_owned())),
        }
    }

    /// A rising series, so the newest rate sits at the top of its range.
    pub(crate) fn rising() -> Self {
        Self::with_values(&[
            dec!(1.00),
            dec!(1.02),
            dec!(1.04),
            dec!(1.06),
            dec!(1.08),
            dec!(1.10),
        ])
    }

    /// A falling series, so the newest rate sits at the bottom of its range.
    pub(crate) fn falling() -> Self {
        Self::with_values(&[
            dec!(1.10),
            dec!(1.08),
            dec!(1.06),
            dec!(1.04),
            dec!(1.02),
            dec!(1.00),
        ])
    }
}

#[async_trait::async_trait]
impl ExchangeRateRepository for MockRateRepository {
    async fn save_rates(
        &self,
        _rates: HashMap<CurrencyPair, Vec<ExchangeRate>>,
    ) -> Result<(), RepositoryError> {
        Ok(())
    }

    async fn load_rates(
        &self,
        pair: &CurrencyPair,
        _range: &RangeInclusive<DateTime<Utc>>,
    ) -> Result<TimeSeries, RepositoryError> {
        if let Some(error) = &self.error {
            return Err(error.clone());
        }
        Ok(TimeSeries::new(pair.clone(), self.rates.clone()))
    }

    async fn exists(
        &self,
        _pair: &CurrencyPair,
        _range: &RangeInclusive<DateTime<Utc>>,
    ) -> Result<bool, RepositoryError> {
        Ok(!self.rates.is_empty())
    }
}

/// Currency repository returning a fixed catalog, or a fixed failure.
#[derive(Debug, Clone)]
pub(crate) struct MockCurrencyRepository {
    currencies: Vec<CurrencyInfo>,
    error: Option<RepositoryError>,
}

impl MockCurrencyRepository {
    pub(crate) fn with_currencies(entries: &[(&str, &str)]) -> Self {
        Self {
            currencies: entries
                .iter()
                .map(|(code, name)| {
                    CurrencyInfo::new(Currency::new(code).unwrap(), (*name).to_owned())
                })
                .collect(),
            error: None,
        }
    }

    pub(crate) fn failing() -> Self {
        Self {
            currencies: Vec::new(),
            error: Some(RepositoryError::Storage("connection lost".to_owned())),
        }
    }
}

#[async_trait::async_trait]
impl CurrencyRepository for MockCurrencyRepository {
    async fn save_currencies(&self, _currencies: &[CurrencyInfo]) -> Result<(), RepositoryError> {
        Ok(())
    }

    async fn list_currencies(&self) -> Result<Vec<CurrencyInfo>, RepositoryError> {
        if let Some(error) = &self.error {
            return Err(error.clone());
        }
        Ok(self.currencies.clone())
    }
}
