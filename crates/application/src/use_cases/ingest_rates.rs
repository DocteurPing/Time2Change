use crate::ports::{
    exchange_rate_repository::{ExchangeRateRepository, RepositoryError},
    rate_provider::{RateProvider, RateProviderError},
};
use domain::types::currency_pair::CurrencyPair;
use thiserror::Error;

/// Ingest latest FX rate for a pair and persist.
pub struct IngestRatesUseCase<R, C>
where
    R: ExchangeRateRepository,
    C: RateProvider,
{
    repository: R,
    provider: C,
}

impl<R, C> IngestRatesUseCase<R, C>
where
    R: ExchangeRateRepository,
    C: RateProvider,
{
    pub fn new(repository: R, provider: C) -> Self {
        Self {
            repository,
            provider,
        }
    }

    pub async fn execute(&self, pair: CurrencyPair) -> Result<IngestResult, IngestError> {
        let fx_record = self.provider.fetch_latest(&pair).await?;

        let rate = domain::types::exchange_rate::ExchangeRate::new(
            *fx_record.timestamp(),
            *fx_record.rate(),
        );

        self.repository.save_rates(&pair, &[rate]).await?;

        Ok(IngestResult {
            pair,
            timestamp: *fx_record.timestamp(),
            rate: *fx_record.rate(),
        })
    }
}

#[derive(Debug)]
pub struct IngestResult {
    pair: CurrencyPair,
    timestamp: chrono::DateTime<chrono::Utc>,
    rate: rust_decimal::Decimal,
}

impl IngestResult {
    pub fn pair(&self) -> &CurrencyPair {
        &self.pair
    }

    pub fn timestamp(&self) -> &chrono::DateTime<chrono::Utc> {
        &self.timestamp
    }

    pub fn rate(&self) -> &rust_decimal::Decimal {
        &self.rate
    }
}

#[derive(Error, Debug)]
pub enum IngestError {
    #[error("provider error: {0}")]
    Provider(#[from] RateProviderError),
    #[error("repository error: {0}")]
    Repository(#[from] RepositoryError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, Utc};
    use domain::types::{currency::Currency, exchange_rate::ExchangeRate, time_series::TimeSeries};
    use rust_decimal::dec;
    use std::ops::RangeInclusive;
    use std::sync::{Arc, Mutex};

    // ── Helpers ─────────────────────────────────────────────────────

    fn make_pair() -> CurrencyPair {
        CurrencyPair::new(Currency::new("EUR").unwrap(), Currency::new("USD").unwrap()).unwrap()
    }

    fn make_rate(ts: DateTime<Utc>, rate: rust_decimal::Decimal) -> ExchangeRate {
        ExchangeRate::new(ts, rate)
    }

    // ── Mock RateProvider ───────────────────────────────────────────

    struct MockProvider {
        result: Result<ExchangeRate, RateProviderError>,
    }

    impl MockProvider {
        fn ok(rate: ExchangeRate) -> Self {
            Self { result: Ok(rate) }
        }

        fn err(e: RateProviderError) -> Self {
            Self { result: Err(e) }
        }
    }

    #[async_trait::async_trait]
    impl RateProvider for MockProvider {
        async fn get_rate(
            &self,
            _pair: &CurrencyPair,
            _timestamp: DateTime<Utc>,
        ) -> Result<ExchangeRate, RateProviderError> {
            match &self.result {
                Ok(r) => Ok(r.clone()),
                Err(_) => Err(RateProviderError::Timeout),
            }
        }

        async fn fetch_latest(
            &self,
            _pair: &CurrencyPair,
        ) -> Result<ExchangeRate, RateProviderError> {
            match &self.result {
                Ok(r) => Ok(r.clone()),
                Err(e) => Err(e.clone()),
            }
        }
    }

    // ── Mock Repository ─────────────────────────────────────────────

    struct MockRepository {
        save_result: Result<(), RepositoryError>,
        saved: Arc<Mutex<Vec<(CurrencyPair, Vec<ExchangeRate>)>>>,
    }

    impl MockRepository {
        fn ok() -> Self {
            Self {
                save_result: Ok(()),
                saved: Arc::new(Mutex::new(Vec::new())),
            }
        }

        fn err(e: RepositoryError) -> Self {
            Self {
                save_result: Err(e),
                saved: Arc::new(Mutex::new(Vec::new())),
            }
        }

        fn saved_calls(&self) -> Vec<(CurrencyPair, Vec<ExchangeRate>)> {
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
            _pair: &CurrencyPair,
            _range: RangeInclusive<DateTime<Utc>>,
        ) -> Result<TimeSeries, RepositoryError> {
            unimplemented!("not needed for ingest tests")
        }

        async fn exists(
            &self,
            _pair: &CurrencyPair,
            _range: RangeInclusive<DateTime<Utc>>,
        ) -> Result<bool, RepositoryError> {
            unimplemented!("not needed for ingest tests")
        }
    }

    // ── Tests ───────────────────────────────────────────────────────

    #[tokio::test]
    async fn execute_success_returns_correct_result() {
        let now = Utc::now();
        let pair = make_pair();
        let rate = make_rate(now, dec!(1.0850));

        let provider = MockProvider::ok(rate);
        let repo = MockRepository::ok();
        let uc = IngestRatesUseCase::new(repo, provider);

        let result = uc.execute(pair.clone()).await.unwrap();

        assert_eq!(result.pair(), &pair);
        assert_eq!(result.timestamp(), &now);
        assert_eq!(result.rate(), &dec!(1.0850));
    }

    #[tokio::test]
    async fn execute_success_persists_rate() {
        let now = Utc::now();
        let pair = make_pair();
        let rate = make_rate(now, dec!(1.12));

        let provider = MockProvider::ok(rate);
        let repo = MockRepository::ok();
        let uc = IngestRatesUseCase::new(repo, provider);

        uc.execute(pair.clone()).await.unwrap();

        let saved = uc.repository.saved_calls();
        assert_eq!(saved.len(), 1);
        assert_eq!(saved[0].0, pair);
        assert_eq!(saved[0].1.len(), 1);
        assert_eq!(saved[0].1[0].rate(), &dec!(1.12));
        assert_eq!(saved[0].1[0].timestamp(), &now);
    }

    #[tokio::test]
    async fn execute_provider_timeout_returns_provider_error() {
        let pair = make_pair();
        let provider = MockProvider::err(RateProviderError::Timeout);
        let repo = MockRepository::ok();
        let uc = IngestRatesUseCase::new(repo, provider);

        let err = uc.execute(pair).await.unwrap_err();
        assert!(matches!(err, IngestError::Provider(_)));
        assert!(err.to_string().contains("timeout"));
    }

    #[tokio::test]
    async fn execute_provider_pair_not_supported_returns_provider_error() {
        let pair = make_pair();
        let provider = MockProvider::err(RateProviderError::PairNotSupported("EUR/USD".into()));
        let repo = MockRepository::ok();
        let uc = IngestRatesUseCase::new(repo, provider);

        let err = uc.execute(pair).await.unwrap_err();
        assert!(matches!(err, IngestError::Provider(_)));
        assert!(err.to_string().contains("EUR/USD"));
    }

    #[tokio::test]
    async fn execute_provider_api_error_returns_provider_error() {
        let pair = make_pair();
        let provider = MockProvider::err(RateProviderError::ApiError("503".into()));
        let repo = MockRepository::ok();
        let uc = IngestRatesUseCase::new(repo, provider);

        let err = uc.execute(pair).await.unwrap_err();
        assert!(matches!(err, IngestError::Provider(_)));
        assert!(err.to_string().contains("503"));
    }

    #[tokio::test]
    async fn execute_provider_parse_error_returns_provider_error() {
        let pair = make_pair();
        let provider = MockProvider::err(RateProviderError::ParseError("bad json".into()));
        let repo = MockRepository::ok();
        let uc = IngestRatesUseCase::new(repo, provider);

        let err = uc.execute(pair).await.unwrap_err();
        assert!(matches!(err, IngestError::Provider(_)));
        assert!(err.to_string().contains("bad json"));
    }

    #[tokio::test]
    async fn execute_repository_storage_error_returns_repository_error() {
        let now = Utc::now();
        let pair = make_pair();
        let rate = make_rate(now, dec!(1.10));

        let provider = MockProvider::ok(rate);
        let repo = MockRepository::err(RepositoryError::Storage("disk full".into()));
        let uc = IngestRatesUseCase::new(repo, provider);

        let err = uc.execute(pair).await.unwrap_err();
        assert!(matches!(err, IngestError::Repository(_)));
        assert!(err.to_string().contains("disk full"));
    }

    #[tokio::test]
    async fn execute_repository_conflict_error_returns_repository_error() {
        let now = Utc::now();
        let pair = make_pair();
        let rate = make_rate(now, dec!(1.10));

        let provider = MockProvider::ok(rate);
        let repo = MockRepository::err(RepositoryError::Conflict("already exists".into()));
        let uc = IngestRatesUseCase::new(repo, provider);

        let err = uc.execute(pair).await.unwrap_err();
        assert!(matches!(err, IngestError::Repository(_)));
        assert!(err.to_string().contains("already exists"));
    }

    #[tokio::test]
    async fn execute_does_not_call_repo_when_provider_fails() {
        let pair = make_pair();
        let provider = MockProvider::err(RateProviderError::Timeout);
        let repo = MockRepository::ok();
        let uc = IngestRatesUseCase::new(repo, provider);

        let _ = uc.execute(pair).await;

        let saved = uc.repository.saved_calls();
        assert!(saved.is_empty());
    }

    #[tokio::test]
    async fn execute_preserves_exact_rate_value() {
        let now = Utc::now();
        let pair = make_pair();
        let precise_rate = dec!(1.123456789);
        let rate = make_rate(now, precise_rate);

        let provider = MockProvider::ok(rate);
        let repo = MockRepository::ok();
        let uc = IngestRatesUseCase::new(repo, provider);

        let result = uc.execute(pair).await.unwrap();

        assert_eq!(result.rate(), &precise_rate);
    }

    #[tokio::test]
    async fn ingest_error_display_provider() {
        let err = IngestError::Provider(RateProviderError::Timeout);
        assert_eq!(err.to_string(), "provider error: network timeout");
    }

    #[tokio::test]
    async fn ingest_error_display_repository() {
        let err = IngestError::Repository(RepositoryError::Storage("connection lost".into()));
        assert_eq!(
            err.to_string(),
            "repository error: storage failure: connection lost"
        );
    }

    #[tokio::test]
    async fn ingest_result_debug_impl() {
        let now = Utc::now();
        let pair = make_pair();
        let rate = make_rate(now, dec!(1.05));

        let provider = MockProvider::ok(rate);
        let repo = MockRepository::ok();
        let uc = IngestRatesUseCase::new(repo, provider);

        let result = uc.execute(pair).await.unwrap();
        let debug = format!("{:?}", result);
        assert!(debug.contains("IngestResult"));
    }
}
