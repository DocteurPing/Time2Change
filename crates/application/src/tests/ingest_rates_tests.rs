use chrono::Utc;
use rust_decimal::dec;

use crate::ports::exchange_rate_repository::RepositoryError;
use crate::ports::rate_provider::RateProviderError;
use crate::tests::helpers::{make_pair, make_rate};
use crate::tests::mocks::mock_provider::MockProvider;
use crate::tests::mocks::mock_repository::MockRepository;
use crate::use_cases::ingest_rates::{IngestError, IngestRatesUseCase};

#[tokio::test]
async fn execute_success_returns_correct_result() {
    let now = Utc::now();
    let pair = make_pair();
    let rate = make_rate(now, dec!(1.0850));

    let provider = MockProvider::ok(rate);
    let repo = MockRepository::empty();
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
    let repo = MockRepository::empty();
    let saved_rates = repo.get_arc_saved_rates();
    let uc = IngestRatesUseCase::new(repo, provider);

    uc.execute(pair.clone()).await.unwrap();

    let saved = saved_rates.lock().unwrap();
    assert_eq!(saved.len(), 1);
    assert_eq!(*saved[0].pair(), pair);
    assert_eq!(saved[0].rates().len(), 1);
    assert_eq!(saved[0].rates()[0].rate(), &dec!(1.12));
    assert_eq!(saved[0].rates()[0].timestamp(), &now);
    drop(saved);
}

#[tokio::test]
async fn execute_provider_timeout_returns_provider_error() {
    let pair = make_pair();
    let provider = MockProvider::err(RateProviderError::Timeout);
    let repo = MockRepository::empty();
    let uc = IngestRatesUseCase::new(repo, provider);

    let err = uc.execute(pair).await.unwrap_err();
    assert!(matches!(err, IngestError::Provider(_)));
    assert!(err.to_string().contains("timeout"));
}

#[tokio::test]
async fn execute_provider_pair_not_supported_returns_provider_error() {
    let pair = make_pair();
    let provider = MockProvider::err(RateProviderError::PairNotSupported("EUR/USD".into()));
    let repo = MockRepository::empty();
    let uc = IngestRatesUseCase::new(repo, provider);

    let err = uc.execute(pair).await.unwrap_err();
    assert!(matches!(err, IngestError::Provider(_)));
    assert!(err.to_string().contains("EUR/USD"));
}

#[tokio::test]
async fn execute_provider_api_error_returns_provider_error() {
    let pair = make_pair();
    let provider = MockProvider::err(RateProviderError::ApiError("503".into()));
    let repo = MockRepository::empty();
    let uc = IngestRatesUseCase::new(repo, provider);

    let err = uc.execute(pair).await.unwrap_err();
    assert!(matches!(err, IngestError::Provider(_)));
    assert!(err.to_string().contains("503"));
}

#[tokio::test]
async fn execute_provider_parse_error_returns_provider_error() {
    let pair = make_pair();
    let provider = MockProvider::err(RateProviderError::ParseError("bad json".into()));
    let repo = MockRepository::empty();
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
    let repo = MockRepository::empty();
    let saved_rates = repo.get_arc_saved_rates();
    let uc = IngestRatesUseCase::new(repo, provider);

    let _ = uc.execute(pair).await;
    assert!(saved_rates.lock().unwrap().is_empty());
}

#[tokio::test]
async fn execute_preserves_exact_rate_value() {
    let now = Utc::now();
    let pair = make_pair();
    let precise_rate = dec!(1.123456789);
    let rate = make_rate(now, precise_rate);

    let provider = MockProvider::ok(rate);
    let repo = MockRepository::empty();
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
    let repo = MockRepository::empty();
    let uc = IngestRatesUseCase::new(repo, provider);

    let result = uc.execute(pair).await.unwrap();
    let debug = format!("{result:?}");
    assert!(debug.contains("IngestResult"));
}
