/// Abstractions for retrieving exchange-rate data from upstream providers.
use chrono::{DateTime, Utc};
use domain::types::currency_pair::CurrencyPair;
use domain::types::exchange_rate::ExchangeRate;
use thiserror::Error;

/// Port implemented by services capable of supplying exchange-rate data.
///
/// The application layer depends on this trait instead of concrete API clients
/// so that rate retrieval can be tested, mocked, and swapped independently from
/// business logic.
#[async_trait::async_trait]
pub trait RateProvider: Send + Sync {
    /// Retrieves the exchange rate for `pair` at a specific `timestamp`.
    ///
    /// Implementations may satisfy this request from a historical endpoint,
    /// a cache, or another upstream data source.
    ///
    /// # Errors
    ///
    /// Returns a [`RateProviderError`] when the pair is unsupported, the
    /// upstream request times out, the remote API fails, or the response
    /// cannot be parsed into a valid [`ExchangeRate`].
    async fn get_rate(
        &self,
        pair: &CurrencyPair,
        timestamp: DateTime<Utc>,
    ) -> Result<ExchangeRate, RateProviderError>;

    /// Fetches the most recent available exchange rate for `pair`.
    ///
    /// # Errors
    ///
    /// Returns a [`RateProviderError`] when the pair is unsupported, the
    /// upstream request fails, or the provider response cannot be parsed.
    async fn fetch_latest(&self, pair: &CurrencyPair) -> Result<ExchangeRate, RateProviderError>;
}

/// Errors produced by [`RateProvider`] implementations.
#[derive(Error, Debug, Clone)]
pub enum RateProviderError {
    /// The requested currency pair is not supported by the upstream provider.
    #[error("pair not supported: {0}")]
    PairNotSupported(String),

    /// The upstream request exceeded the allowed response time.
    #[error("network timeout")]
    Timeout,

    /// The upstream provider returned an application or transport error.
    #[error("api error: {0}")]
    ApiError(String),

    /// The provider response could not be parsed into a valid domain value.
    #[error("parse error: {0}")]
    ParseError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pair_not_supported_display() {
        let err = RateProviderError::PairNotSupported("EUR-USD".into());
        assert_eq!(err.to_string(), "pair not supported: EUR-USD");
    }

    #[test]
    fn timeout_display() {
        let err = RateProviderError::Timeout;
        assert_eq!(err.to_string(), "network timeout");
    }

    #[test]
    fn api_error_display() {
        let err = RateProviderError::ApiError("503 Service Unavailable".into());
        assert_eq!(err.to_string(), "api error: 503 Service Unavailable");
    }

    #[test]
    fn parse_error_display() {
        let err = RateProviderError::ParseError("invalid JSON".into());
        assert_eq!(err.to_string(), "parse error: invalid JSON");
    }

    #[test]
    fn error_is_debug() {
        let err = RateProviderError::Timeout;
        let debug = format!("{:?}", err);
        assert!(debug.contains("Timeout"));
    }

    #[test]
    fn pair_not_supported_debug_contains_value() {
        let err = RateProviderError::PairNotSupported("GBP-JPY".into());
        let debug = format!("{:?}", err);
        assert!(debug.contains("PairNotSupported"));
        assert!(debug.contains("GBP-JPY"));
    }

    #[test]
    fn api_error_debug_contains_value() {
        let err = RateProviderError::ApiError("rate limit exceeded".into());
        let debug = format!("{:?}", err);
        assert!(debug.contains("ApiError"));
        assert!(debug.contains("rate limit exceeded"));
    }

    #[test]
    fn parse_error_debug_contains_value() {
        let err = RateProviderError::ParseError("unexpected EOF".into());
        let debug = format!("{:?}", err);
        assert!(debug.contains("ParseError"));
        assert!(debug.contains("unexpected EOF"));
    }
}
