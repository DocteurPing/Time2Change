use application::ports::rate_provider::{RateProvider, RateProviderError};
use async_trait::async_trait;
use chrono::{DateTime, NaiveTime, TimeZone, Utc};
use domain::types::{currency_pair::CurrencyPair, exchange_rate::ExchangeRate};
use reqwest::Client;
use rust_decimal::{Decimal, prelude::FromPrimitive};

use crate::repositories::rate_provider::dto::FrankfurterRateProviderResponse;

const BASE_URL: &str = "https://api.frankfurter.app";
const TIMEOUT_SECONDS: u64 = 5;

/// HTTP adapter for the Frankfurter public exchange-rate API.
///
/// Wraps a [`reqwest::Client`] and translates HTTP responses into domain
/// [`ExchangeRate`] values, mapping all failure modes to [`RateProviderError`].
#[derive(Debug, Clone)]
pub struct FrankfurterClient {
    client: Client,
    base_url: String, // kept as a field for test
}

impl Default for FrankfurterClient {
    fn default() -> Self {
        Self {
            client: Client::new(),
            base_url: BASE_URL.to_owned(),
        }
    }
}

impl FrankfurterClient {
    /// Creates a client pointing at a custom base URL (useful for tests with
    /// a mock HTTP server such as `wiremock`).
    ///
    /// # Errors
    ///
    /// Returns an error if the underlying `reqwest::Client` cannot be built
    pub fn with_base_url(base_url: impl Into<String>) -> Result<Self, reqwest::Error> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(TIMEOUT_SECONDS))
            .build()?;
        Ok(Self {
            client,
            base_url: base_url.into(),
        })
    }

    async fn fetch(
        &self,
        url: &str,
        pair: &CurrencyPair,
    ) -> Result<ExchangeRate, RateProviderError> {
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

        let dto: FrankfurterRateProviderResponse = response
            .json()
            .await
            .map_err(|e| RateProviderError::ParseError(e.to_string()))?;

        let quote_str = pair.quote().to_string();
        let raw_rate = dto
            .rates()
            .get(&quote_str)
            .ok_or_else(|| RateProviderError::PairNotSupported(pair.to_string()))?;

        // Should never fail since the API returns a number
        let rate = Decimal::from_f64(*raw_rate).unwrap_or_default();

        let timestamp: DateTime<Utc> = Utc.from_utc_datetime(&dto.date().and_time(NaiveTime::MIN));

        Ok(ExchangeRate::new(timestamp, rate))
    }

    /// Returns the base URL used by this client
    #[must_use]
    pub fn base_url(&self) -> &str {
        &self.base_url
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
        self.fetch(&url, pair).await
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
        self.fetch(&url, pair).await
    }
}
