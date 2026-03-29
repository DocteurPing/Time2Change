use application::ports::rate_provider::{RateProvider, RateProviderError};
use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, NaiveTime, TimeZone, Utc};
use domain::types::currency::Currency;
use domain::types::currency_info::CurrencyInfo;
use domain::types::currency_pair::CurrencyPair;
use domain::types::exchange_rate::ExchangeRate;
use reqwest::{Client, Response};
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;

use crate::rate_provider::dto::{
    FrankfurterCurrenciesResponse, FrankfurterRangeResponse, FrankfurterRateProviderResponse,
};

const BASE_URL: &str = "https://api.frankfurter.dev/v2";
const TIMEOUT_MILLIS: u64 = 5000;

/// HTTP adapter for the Frankfurter public exchange-rate API.
///
/// Wraps a [`reqwest::Client`] and translates HTTP responses into domain
/// [`ExchangeRate`] values, mapping all failure modes to [`RateProviderError`].
#[derive(Debug, Clone)]
pub struct FrankfurterClient {
    client: Client,
    base_url: String, // kept as a field for test
}

impl FrankfurterClient {
    /// Creates a client pointing at the default base URL.
    ///
    /// # Errors
    ///
    /// Returns an error if the underlying `reqwest::Client` cannot be built.
    pub fn with_default_url() -> Result<Self, reqwest::Error> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_millis(TIMEOUT_MILLIS))
            .build()?;
        Ok(Self {
            client,
            base_url: BASE_URL.to_owned(),
        })
    }

    /// Creates a client pointing at a custom base URL (useful for tests with
    /// a mock HTTP server such as `wiremock`).
    ///
    /// # Errors
    ///
    /// Returns an error if the underlying `reqwest::Client` cannot be built.
    pub fn with_base_url_and_timeout(
        base_url: impl Into<String>,
        timeout_millis: u64,
    ) -> Result<Self, reqwest::Error> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_millis(timeout_millis))
            .build()?;
        Ok(Self {
            client,
            base_url: base_url.into(),
        })
    }

    /// Returns the base URL used by this client.
    #[must_use]
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Performs a GET request and returns the raw [`Response`], mapping HTTP
    /// and transport errors to [`RateProviderError`].
    async fn fetch(&self, url: &str) -> Result<Response, RateProviderError> {
        let response = self.client.get(url).send().await.map_err(|e| {
            if e.is_timeout() {
                RateProviderError::Timeout
            } else {
                RateProviderError::ApiError(e.to_string())
            }
        })?;

        if !response.status().is_success() {
            return Err(RateProviderError::ApiError(format!(
                "HTTP {}",
                response.status()
            )));
        }
        Ok(response)
    }

    async fn fetch_pairs_and_validate(
        &self,
        url: &str,
        pair: &CurrencyPair,
    ) -> Result<Response, RateProviderError> {
        let response = self.client.get(url).send().await.map_err(|e| {
            if e.is_timeout() {
                RateProviderError::Timeout
            } else {
                RateProviderError::ApiError(e.to_string())
            }
        })?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(RateProviderError::PairNotSupported(pair.to_string()));
        }

        if !response.status().is_success() {
            return Err(RateProviderError::ApiError(format!(
                "HTTP {}",
                response.status()
            )));
        }
        Ok(response)
    }

    /// Fetches a single-date rate for `pair` from `url`.
    async fn fetch_pair(
        &self,
        url: &str,
        pair: &CurrencyPair,
    ) -> Result<ExchangeRate, RateProviderError> {
        let response = self.fetch_pairs_and_validate(url, pair).await?;
        let dto: FrankfurterRateProviderResponse = response
            .json()
            .await
            .map_err(|e| RateProviderError::ParseError(e.to_string()))?;

        let quote_str = pair.quote().to_string();
        let raw_rate = dto
            .rates()
            .get(&quote_str)
            .ok_or_else(|| RateProviderError::PairNotSupported(pair.to_string()))?;

        // Should never fail since the API returns a number.
        let rate = Decimal::from_f64(*raw_rate).unwrap_or_default();
        let timestamp: DateTime<Utc> = Utc.from_utc_datetime(&dto.date().and_time(NaiveTime::MIN));

        Ok(ExchangeRate::new(timestamp, rate))
    }

    /// Fetches a date-range of rates for `pair` from `url` and returns them
    /// sorted in ascending chronological order.
    async fn fetch_pair_range(
        &self,
        url: &str,
        pair: &CurrencyPair,
    ) -> Result<Vec<ExchangeRate>, RateProviderError> {
        let response = self.fetch_pairs_and_validate(url, pair).await?;
        let list_rate: Vec<FrankfurterRangeResponse> = response
            .json()
            .await
            .map_err(|e| RateProviderError::ParseError(e.to_string()))?;

        let rates = list_rate
            .iter()
            .map(|dto| {
                let date = dto.date();
                if dto.base() != pair.base().to_string() || dto.quote() != pair.quote().to_string()
                {
                    return Err(RateProviderError::PairNotSupported(pair.to_string()));
                }
                let rate = Decimal::from_f64(dto.rate()).unwrap_or_default();
                let timestamp = Utc.from_utc_datetime(&date.and_time(NaiveTime::MIN));
                Ok(ExchangeRate::new(timestamp, rate))
            })
            .collect::<Result<Vec<_>, RateProviderError>>()?;

        Ok(rates)
    }
}

#[async_trait]
impl RateProvider for FrankfurterClient {
    async fn fetch_latest(&self, pair: &CurrencyPair) -> Result<ExchangeRate, RateProviderError> {
        let url = format!(
            "{}/latest?base={}&symbols={}",
            self.base_url,
            pair.base(),
            pair.quote()
        );
        self.fetch_pair(&url, pair).await
    }

    async fn get_rate(
        &self,
        pair: &CurrencyPair,
        timestamp: DateTime<Utc>,
    ) -> Result<ExchangeRate, RateProviderError> {
        let date = timestamp.format("%Y-%m-%d");
        let url = format!(
            "{}/{date}?base={}&symbols={}",
            self.base_url,
            pair.base(),
            pair.quote()
        );
        self.fetch_pair(&url, pair).await
    }

    async fn get_rates_for_range(
        &self,
        pair: &CurrencyPair,
        start: NaiveDate,
        end: NaiveDate,
    ) -> Result<Vec<ExchangeRate>, RateProviderError> {
        let url = format!(
            "{}/rates?from={}&to={}&base={}&quotes={}",
            self.base_url,
            start,
            end,
            pair.base(),
            pair.quote()
        );
        self.fetch_pair_range(&url, pair).await
    }

    async fn fetch_currencies(&self) -> Result<Vec<CurrencyInfo>, RateProviderError> {
        let url = format!("{}/currencies", self.base_url);
        let response: Response = self.fetch(&url).await?;
        let dto: Vec<FrankfurterCurrenciesResponse> = response
            .json()
            .await
            .map_err(|e| RateProviderError::ParseError(e.to_string()))?;
        dto.iter()
            .map(|currency| {
                let iso_code = Currency::new(currency.iso_code())
                    .map_err(|e| RateProviderError::ParseError(e.to_string()))?;
                Ok(CurrencyInfo::new(iso_code, currency.name().to_owned()))
            })
            .collect()
    }
}
