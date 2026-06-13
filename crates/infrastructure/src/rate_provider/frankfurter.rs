use std::collections::{HashMap, HashSet};

use application::ports::rate_provider::{RateProvider, RateProviderError};
use async_trait::async_trait;
use chrono::{NaiveDate, NaiveTime, TimeZone, Utc};
use domain::types::currency::Currency;
use domain::types::currency_info::CurrencyInfo;
use domain::types::currency_pair::CurrencyPair;
use domain::types::exchange_rate::ExchangeRate;
use reqwest::{Client, Response};

use crate::rate_provider::dto::{FrankfurterCurrenciesResponse, FrankfurterRangeResponse};

const BASE_URL: &str = "https://api.frankfurter.dev/v2";
const TIMEOUT_MILLIS: u64 = 5000;

/// HTTP adapter for the Frankfurter public exchange-rate API.
///
/// Wraps a [`Client`] and translates HTTP responses into domain
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

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(RateProviderError::PairNotSupported(
                "404 Not Found".to_owned(),
            ));
        }

        if !response.status().is_success() {
            return Err(RateProviderError::ApiError(format!(
                "HTTP {}",
                response.status()
            )));
        }
        Ok(response)
    }

    /// Fetches a date-range of rates for `pair` from `url` and returns them
    /// sorted in ascending chronological order.
    async fn fetch_pair_range(
        &self,
        url: &str,
    ) -> Result<HashMap<CurrencyPair, Vec<ExchangeRate>>, RateProviderError> {
        let mut rates: HashMap<CurrencyPair, Vec<ExchangeRate>> = HashMap::new();
        let response = self.fetch(url).await?;
        let list_rate: Vec<FrankfurterRangeResponse> = response
            .json()
            .await
            .map_err(|e| RateProviderError::ParseError(e.to_string()))?;

        for rate in list_rate {
            let pair = CurrencyPair::new(rate.base().to_owned(), rate.quote().to_owned())
                .map_err(|e| RateProviderError::ParseError(e.to_string()))?;
            let timestamp = Utc.from_utc_datetime(&rate.date().and_time(NaiveTime::MIN));
            rates
                .entry(pair)
                .or_default()
                .push(ExchangeRate::new(timestamp, rate.rate()));
        }

        Ok(rates)
    }
}

#[async_trait]
impl RateProvider for FrankfurterClient {
    async fn get_rates_for_range(
        &self,
        list_currencies: &HashSet<Currency>,
        start: NaiveDate,
        end: NaiveDate,
        currency: &Currency,
    ) -> Result<HashMap<CurrencyPair, Vec<ExchangeRate>>, RateProviderError> {
        let quote = list_currencies
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<String>>()
            .join(",");
        let url = format!(
            "{}/rates?from={}&to={}&base={}&quotes={}",
            self.base_url, start, end, currency, quote
        );
        self.fetch_pair_range(&url).await
    }

    async fn fetch_currencies(&self) -> Result<Vec<CurrencyInfo>, RateProviderError> {
        let url = format!("{}/currencies", self.base_url);
        let response: Response = self.fetch(&url).await?;
        let dto: Vec<FrankfurterCurrenciesResponse> = response
            .json()
            .await
            .map_err(|e| RateProviderError::ParseError(e.to_string()))?;
        dto.into_iter()
            .map(|currency| {
                let iso_code = Currency::new(currency.iso_code())
                    .map_err(|e| RateProviderError::ParseError(e.to_string()))?;
                Ok(CurrencyInfo::new(iso_code, currency.name().to_owned()))
            })
            .collect()
    }
}
